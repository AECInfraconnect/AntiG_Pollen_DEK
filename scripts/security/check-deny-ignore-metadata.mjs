import fs from "node:fs";

const denyToml = fs.readFileSync("deny.toml", "utf8").split(/\r?\n/);
const today = new Date();
today.setUTCHours(0, 0, 0, 0);

let inIgnoreList = false;
const failures = [];

function parseMetadata(comment) {
  const metadata = {};
  for (const part of comment.replace(/^#\s*/, "").split(",")) {
    const [key, ...rest] = part.trim().split(":");
    if (key && rest.length > 0) {
      metadata[key.trim()] = rest.join(":").trim();
    }
  }
  return metadata;
}

for (let index = 0; index < denyToml.length; index += 1) {
  const line = denyToml[index].trim();
  if (line === "ignore = [") {
    inIgnoreList = true;
    continue;
  }
  if (inIgnoreList && line === "]") {
    inIgnoreList = false;
    continue;
  }
  if (!inIgnoreList || !line.includes("RUSTSEC-")) continue;

  const advisory = line.match(/RUSTSEC-\d{4}-\d{4}/)?.[0] ?? line;
  const previous = denyToml[index - 1]?.trim() ?? "";
  const metadata = parseMetadata(previous);
  for (const field of ["owner", "expiry", "issue", "reason"]) {
    if (!metadata[field]) {
      failures.push(`${advisory}: missing ${field} metadata`);
    }
  }
  if (metadata.expiry) {
    const expiry = new Date(`${metadata.expiry}T00:00:00Z`);
    if (Number.isNaN(expiry.getTime())) {
      failures.push(`${advisory}: invalid expiry date '${metadata.expiry}'`);
    } else if (expiry < today) {
      failures.push(`${advisory}: ignore expired on ${metadata.expiry}`);
    }
  }
}

if (failures.length > 0) {
  console.error("deny.toml advisory ignore metadata check failed:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log("deny.toml advisory ignore metadata is current.");
