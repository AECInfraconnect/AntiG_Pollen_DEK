import { mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { spawn } from "node:child_process";

const args = process.argv.slice(2);
const localHome = resolve(".storybook-home");
const localCache = resolve("node_modules", ".cache", "storybook");
mkdirSync(localHome, { recursive: true });
mkdirSync(localCache, { recursive: true });

const cli = resolve(
  "node_modules",
  "storybook",
  "dist",
  "bin",
  "dispatcher.js",
);

const child = spawn(process.execPath, [cli, ...args], {
  stdio: "inherit",
  env: {
    ...process.env,
    HOME: localHome,
    USERPROFILE: localHome,
    XDG_CONFIG_HOME: localHome,
    STORYBOOK_CACHE_DIR: localCache,
    STORYBOOK_DISABLE_TELEMETRY: "1",
  },
});

child.on("exit", (code, signal) => {
  process.exit(code ?? (signal ? 1 : 0));
});
