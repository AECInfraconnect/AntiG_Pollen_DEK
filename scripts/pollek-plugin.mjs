#!/usr/bin/env node
import { createHash } from "node:crypto";
import { promises as fs } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function usage() {
  console.log(`Pollek plugin helper

Usage:
  node scripts/pollek-plugin.mjs new <dir> <plugin-id>
  node scripts/pollek-plugin.mjs checksum <wasm-file>
  node scripts/pollek-plugin.mjs pack <plugin-dir> [out-dir]
  node scripts/pollek-plugin.mjs publish-local <plugin-dir> [registry-dir]
  node scripts/pollek-plugin.mjs test-manifest <plugin-dir>
`);
}

async function exists(filePath) {
  try {
    await fs.access(filePath);
    return true;
  } catch {
    return false;
  }
}

async function readJson(filePath) {
  return JSON.parse(await fs.readFile(filePath, "utf8"));
}

async function writeJson(filePath, value) {
  await fs.writeFile(`${filePath}.tmp`, `${JSON.stringify(value, null, 2)}\n`);
  await fs.rename(`${filePath}.tmp`, filePath);
}

async function commandNew(dir, pluginId) {
  if (!dir || !pluginId) throw new Error("new requires <dir> and <plugin-id>");
  const target = path.resolve(dir);
  await fs.mkdir(path.join(target, "src"), { recursive: true });
  const manifest = {
    schema_version: "pollek.plugin.v1",
    id: pluginId,
    name: pluginId
      .split(".")
      .slice(-1)[0]
      .replaceAll("-", " ")
      .replace(/\b\w/g, (ch) => ch.toUpperCase()),
    version: "0.1.0",
    kind: "discovery.signature",
    wit_world: "pollek:discovery/discovery-plugin@0.1.0",
    abi: "component",
    min_engine_version: "1.0.0",
    os: ["windows", "linux", "macos"],
    entry: "plugin.wasm",
    capabilities: {
      host: ["logging", "clock"],
      http_out: [],
      kv: [],
      native: [],
      data_scope: [],
    },
    author: {
      name: "Your organization",
      verified: false,
    },
    signature: {
      type: "developer",
      status: "missing",
    },
    registry: {
      source: "sideload",
      update_channel: "local",
      rollback_versions: [],
    },
    governance: {
      review_required: true,
      public_marketplace_allowed: false,
      trust_labels: ["developer_preview"],
    },
  };
  await writeJson(path.join(target, "pollek-plugin.json"), manifest);
  await fs.writeFile(
    path.join(target, "src", "lib.rs"),
    `// Example Rust component. Generate WIT bindings with cargo-component/wit-bindgen.\n// See docs/PLUGIN_SDK.md for the current Pollek WIT worlds.\n\npub fn plugin_name() -> &'static str {\n    \"${pluginId}\"\n}\n`,
  );
  await fs.writeFile(
    path.join(target, "README.md"),
    `# ${manifest.name}\n\nLocal Pollek plugin scaffold. Build a wasm32-wasip2 component that implements ${manifest.wit_world}.\n`,
  );
  console.log(`Created ${path.relative(process.cwd(), target)}`);
}

async function commandChecksum(filePath) {
  if (!filePath) throw new Error("checksum requires <wasm-file>");
  const bytes = await fs.readFile(path.resolve(filePath));
  console.log(`sha256:${createHash("sha256").update(bytes).digest("hex")}`);
}

async function commandTestManifest(pluginDir) {
  const dir = path.resolve(pluginDir || ".");
  const manifestPath = path.join(dir, "pollek-plugin.json");
  const manifest = await readJson(manifestPath);
  const required = ["schema_version", "id", "name", "version", "kind", "entry", "capabilities"];
  const missing = required.filter((key) => manifest[key] === undefined);
  if (missing.length) throw new Error(`manifest missing: ${missing.join(", ")}`);
  if (manifest.schema_version !== "pollek.plugin.v1") {
    throw new Error("schema_version must be pollek.plugin.v1");
  }
  if (manifest.entry && !(await exists(path.join(dir, manifest.entry)))) {
    console.warn(`Warning: entry ${manifest.entry} does not exist yet`);
  }
  console.log(`Manifest ok: ${manifest.id}@${manifest.version}`);
}

async function copyDir(src, dest) {
  await fs.mkdir(dest, { recursive: true });
  for (const entry of await fs.readdir(src, { withFileTypes: true })) {
    const from = path.join(src, entry.name);
    const to = path.join(dest, entry.name);
    if (entry.isDirectory()) await copyDir(from, to);
    else if (entry.isFile()) await fs.copyFile(from, to);
  }
}

async function commandPack(pluginDir, outDir = "dist/plugins") {
  const dir = path.resolve(pluginDir);
  await commandTestManifest(dir);
  const manifest = await readJson(path.join(dir, "pollek-plugin.json"));
  const out = path.resolve(outDir, `${manifest.id}-${manifest.version}`);
  await fs.rm(out, { recursive: true, force: true });
  await fs.mkdir(out, { recursive: true });
  await copyDir(dir, out);
  if (manifest.entry && (await exists(path.join(dir, manifest.entry)))) {
    const bytes = await fs.readFile(path.join(dir, manifest.entry));
    manifest.checksum = `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
    await writeJson(path.join(out, "pollek-plugin.json"), manifest);
  }
  console.log(`Packed ${path.relative(process.cwd(), out)}`);
}

function defaultRegistryDir() {
  if (process.env.POLLEK_PLUGIN_REGISTRY_DIR) {
    return process.env.POLLEK_PLUGIN_REGISTRY_DIR;
  }
  return path.join(process.env.DEK_LCP_DATA ?? "pollek-local-data", "plugin-registry");
}

async function commandPublishLocal(pluginDir, registryDir = defaultRegistryDir()) {
  const dir = path.resolve(pluginDir);
  const manifest = await readJson(path.join(dir, "pollek-plugin.json"));
  const target = path.resolve(registryDir, manifest.id, manifest.version);
  await fs.rm(target, { recursive: true, force: true });
  await fs.mkdir(target, { recursive: true });
  await copyDir(dir, target);
  const indexPath = path.resolve(registryDir, "index.json");
  const index = (await exists(indexPath)) ? await readJson(indexPath) : { schema_version: "pollek.plugin_registry.v1", items: [] };
  index.items = [
    ...index.items.filter((item) => !(item.id === manifest.id && item.version === manifest.version)),
    {
      id: manifest.id,
      version: manifest.version,
      kind: manifest.kind,
      path: path.relative(path.resolve(registryDir), target).replaceAll(path.sep, "/"),
      signature_state: manifest.signature?.status ?? "unknown",
      checksum: manifest.checksum ?? null,
    },
  ];
  await fs.mkdir(path.dirname(indexPath), { recursive: true });
  await writeJson(indexPath, index);
  console.log(`Published local plugin ${manifest.id}@${manifest.version}`);
}

async function main() {
  const [command, ...args] = process.argv.slice(2);
  if (!command || command === "help" || command === "--help") {
    usage();
    return;
  }
  if (command === "new") return commandNew(args[0], args[1]);
  if (command === "checksum") return commandChecksum(args[0]);
  if (command === "pack") return commandPack(args[0], args[1]);
  if (command === "publish-local") return commandPublishLocal(args[0], args[1]);
  if (command === "test-manifest") return commandTestManifest(args[0]);
  throw new Error(`Unknown command: ${command}`);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
