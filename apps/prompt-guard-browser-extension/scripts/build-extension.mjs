import { deflateRawSync } from "node:zlib";
import { promises as fs } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const distRoot = path.join(root, "dist");
const packageRoot = path.join(root, "packages");
const targets = {
  chromium: {
    manifest: "targets/chromium/manifest.json",
    outDir: "chromium",
    zipName: "pollek-prompt-guard-chromium.zip",
  },
  edge: {
    manifest: "targets/edge/manifest.json",
    outDir: "edge",
    zipName: "pollek-prompt-guard-edge.zip",
  },
  safari: {
    manifest: "targets/safari/manifest.json",
    outDir: "safari-webextension",
    zipName: "pollek-prompt-guard-safari-webextension.zip",
  },
};

async function exists(filePath) {
  try {
    await fs.access(filePath);
    return true;
  } catch {
    return false;
  }
}

async function copyDir(src, dest) {
  await fs.mkdir(dest, { recursive: true });
  for (const entry of await fs.readdir(src, { withFileTypes: true })) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) await copyDir(srcPath, destPath);
    else if (entry.isFile()) await fs.copyFile(srcPath, destPath);
  }
}

async function removeOutputs() {
  await fs.rm(distRoot, { recursive: true, force: true });
  await fs.rm(packageRoot, { recursive: true, force: true });
}

async function buildTarget(name) {
  const target = targets[name];
  if (!target) throw new Error(`Unknown target: ${name}`);

  const outDir = path.join(distRoot, target.outDir);
  await fs.rm(outDir, { recursive: true, force: true });
  await fs.mkdir(outDir, { recursive: true });

  for (const folder of ["shared", "background", "content", "options"]) {
    await copyDir(path.join(root, "src", folder), path.join(outDir, folder));
  }
  await fs.copyFile(path.join(root, target.manifest), path.join(outDir, "manifest.json"));
  await fs.copyFile(path.join(root, "docs", "INSTALL.md"), path.join(outDir, "INSTALL.md"));

  await fs.mkdir(packageRoot, { recursive: true });
  await zipDir(outDir, path.join(packageRoot, target.zipName));
  console.log(`Built ${name}: ${path.relative(root, outDir)}`);
}

async function listFiles(dir, base = dir) {
  const entries = [];
  for (const entry of await fs.readdir(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) entries.push(...(await listFiles(fullPath, base)));
    else if (entry.isFile()) {
      entries.push({
        fullPath,
        relPath: path.relative(base, fullPath).replaceAll(path.sep, "/"),
      });
    }
  }
  return entries.sort((a, b) => a.relPath.localeCompare(b.relPath));
}

function crc32(buffer) {
  let crc = 0xffffffff;
  for (const byte of buffer) {
    crc = (crc >>> 8) ^ crcTable[(crc ^ byte) & 0xff];
  }
  return (crc ^ 0xffffffff) >>> 0;
}

const crcTable = Array.from({ length: 256 }, (_, index) => {
  let value = index;
  for (let bit = 0; bit < 8; bit += 1) {
    value = value & 1 ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
  }
  return value >>> 0;
});

function dosTimeDate(date = new Date()) {
  const time =
    (date.getHours() << 11) |
    (date.getMinutes() << 5) |
    Math.floor(date.getSeconds() / 2);
  const dosDate =
    ((date.getFullYear() - 1980) << 9) |
    ((date.getMonth() + 1) << 5) |
    date.getDate();
  return { time, dosDate };
}

function u16(value) {
  const buffer = Buffer.alloc(2);
  buffer.writeUInt16LE(value);
  return buffer;
}

function u32(value) {
  const buffer = Buffer.alloc(4);
  buffer.writeUInt32LE(value >>> 0);
  return buffer;
}

async function zipDir(sourceDir, zipPath) {
  const files = await listFiles(sourceDir);
  const localParts = [];
  const centralParts = [];
  let offset = 0;

  for (const file of files) {
    const data = await fs.readFile(file.fullPath);
    const compressed = deflateRawSync(data, { level: 9 });
    const name = Buffer.from(file.relPath, "utf8");
    const crc = crc32(data);
    const { time, dosDate } = dosTimeDate();
    const localHeader = Buffer.concat([
      u32(0x04034b50),
      u16(20),
      u16(0),
      u16(8),
      u16(time),
      u16(dosDate),
      u32(crc),
      u32(compressed.length),
      u32(data.length),
      u16(name.length),
      u16(0),
      name,
    ]);
    localParts.push(localHeader, compressed);

    const centralHeader = Buffer.concat([
      u32(0x02014b50),
      u16(20),
      u16(20),
      u16(0),
      u16(8),
      u16(time),
      u16(dosDate),
      u32(crc),
      u32(compressed.length),
      u32(data.length),
      u16(name.length),
      u16(0),
      u16(0),
      u16(0),
      u16(0),
      u32(0),
      u32(offset),
      name,
    ]);
    centralParts.push(centralHeader);
    offset += localHeader.length + compressed.length;
  }

  const central = Buffer.concat(centralParts);
  const local = Buffer.concat(localParts);
  const end = Buffer.concat([
    u32(0x06054b50),
    u16(0),
    u16(0),
    u16(files.length),
    u16(files.length),
    u32(central.length),
    u32(local.length),
    u16(0),
  ]);
  await fs.mkdir(path.dirname(zipPath), { recursive: true });
  await fs.writeFile(zipPath, Buffer.concat([local, central, end]));
}

async function checkTarget(name) {
  const target = targets[name];
  const manifestPath = path.join(root, target.manifest);
  const manifest = JSON.parse(await fs.readFile(manifestPath, "utf8"));
  const missing = [];
  for (const script of manifest.content_scripts?.flatMap((item) => item.js) ?? []) {
    const scriptPath = path.join(root, "src", script);
    if (!(await exists(scriptPath))) missing.push(script);
  }
  if (manifest.background?.service_worker) {
    const workerPath = path.join(root, "src", manifest.background.service_worker);
    if (!(await exists(workerPath))) missing.push(manifest.background.service_worker);
  }
  if (manifest.options_page) {
    const optionsPath = path.join(root, "src", manifest.options_page);
    if (!(await exists(optionsPath))) missing.push(manifest.options_page);
  }
  if (missing.length) {
    throw new Error(`${name} manifest references missing files: ${missing.join(", ")}`);
  }
}

async function main() {
  const command = process.argv[2] ?? "all";
  if (command === "clean") {
    await removeOutputs();
    return;
  }
  if (command === "check") {
    for (const name of Object.keys(targets)) await checkTarget(name);
    console.log("Extension manifests ok");
    return;
  }
  const names = command === "all" ? Object.keys(targets) : [command];
  for (const name of names) await checkTarget(name);
  await fs.mkdir(distRoot, { recursive: true });
  for (const name of names) await buildTarget(name);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
