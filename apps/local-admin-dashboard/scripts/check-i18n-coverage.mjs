import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = process.cwd();
const enPath = join(root, "src", "i18n", "en.json");
const thPath = join(root, "src", "i18n", "th.json");
const srcDir = join(root, "src");
const e2eDir = join(root, "e2e");
const minCoverage = 0.95;

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function flattenKeys(value, prefix = "") {
  if (value && typeof value === "object" && !Array.isArray(value)) {
    return Object.entries(value).flatMap(([key, child]) =>
      flattenKeys(child, prefix ? `${prefix}.${key}` : key),
    );
  }
  return [prefix];
}

function valueAt(object, dottedKey) {
  if (Object.prototype.hasOwnProperty.call(object, dottedKey)) {
    return object[dottedKey];
  }
  return dottedKey.split(".").reduce((current, key) => current?.[key], object);
}

function walk(dir) {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    const stat = statSync(path);
    if (stat.isDirectory()) {
      if (
        ["node_modules", "dist", "playwright-report", "test-results"].includes(
          entry,
        )
      ) {
        return [];
      }
      return walk(path);
    }
    return path;
  });
}

const en = readJson(enPath);
const th = readJson(thPath);
const enKeys = flattenKeys(en).sort();
const thKeys = new Set(flattenKeys(th));
const missing = enKeys.filter((key) => !thKeys.has(key));
const untranslated = enKeys.filter((key) => {
  const value = valueAt(th, key);
  return typeof value !== "string" || value.trim().length === 0;
});
const translated = enKeys.length - untranslated.length;
const coverage = enKeys.length === 0 ? 1 : translated / enKeys.length;

const failures = [];
if (missing.length) {
  failures.push(
    `Missing Thai locale keys:\n${missing.map((key) => `  - ${key}`).join("\n")}`,
  );
}
if (coverage < minCoverage) {
  failures.push(
    `Thai locale coverage ${(coverage * 100).toFixed(1)}% is below ${(minCoverage * 100).toFixed(0)}%.`,
  );
}

const mojibake =
  /[\u0080-\u009f]|\uFFFD|Ã|Â|à¸|à¹|โ€|๏ฟฝ|เน€เธ|เน\u0081เธ|เน\u0082เธ|เธ[\u0080-\u00ff]|เน[\u0080-\u00ff]/;
const sourceFiles = [srcDir, e2eDir]
  .flatMap((dir) => walk(dir))
  .filter((path) => /\.(ts|tsx|json)$/.test(path));
const mojibakeHits = [];

for (const path of sourceFiles) {
  const relativePath = relative(root, path);
  const lines = readFileSync(path, "utf8").split(/\r?\n/);
  lines.forEach((line, index) => {
    const lower = line.toLowerCase();
    const allowedDetector =
      (relativePath ===
        join("src", "features", "guard", "GuardIncidentCard.tsx") &&
        line.includes(".test(value)")) ||
      (relativePath === join("e2e", "policy-first.spec.ts") &&
        (lower.includes("mojibake") || line.includes("\\u0080")));
    if (
      mojibake.test(line) &&
      !lower.includes("mojibake") &&
      !lower.includes("looksm") &&
      !lower.includes("lookscorrupted") &&
      !allowedDetector
    ) {
      mojibakeHits.push(`${relativePath}:${index + 1}: ${line.trim()}`);
    }
  });
}

if (mojibakeHits.length) {
  failures.push(
    `Potential mojibake in user-facing source:\n${mojibakeHits
      .slice(0, 25)
      .map((hit) => `  - ${hit}`)
      .join("\n")}`,
  );
}

if (failures.length) {
  console.error(failures.join("\n\n"));
  process.exit(1);
}

console.log(
  `i18n coverage ${(coverage * 100).toFixed(1)}% (${translated}/${enKeys.length}) with locale parity and no user-facing mojibake.`,
);
