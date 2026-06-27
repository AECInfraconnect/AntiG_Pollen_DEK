import { readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, "../..");
const catalogPath = resolve(
  repoRoot,
  "apps/local-admin-dashboard/src/data/contextual-help.compact.json",
);
const outputPath = resolve(repoRoot, "docs/generated/contextual-help-catalog.md");

const topics = JSON.parse(readFileSync(catalogPath, "utf8"));

const lines = [
  "# Generated Contextual Help Catalog",
  "",
  "This file is generated from `apps/local-admin-dashboard/src/data/contextual-help.compact.json`.",
  "Edit the compact catalog, then run `node scripts/docs/build-contextual-help-docs.mjs`.",
  "",
];

for (const topic of topics) {
  lines.push(`## ${topic.title}`);
  lines.push("");
  lines.push(`Topic ID: \`${topic.id}\``);
  lines.push("");
  lines.push(topic.summary);
  lines.push("");
  for (const item of topic.guidance) {
    lines.push(`- ${item}`);
  }
  lines.push("");
  lines.push(
    `Source: \`${topic.sourceDoc}${topic.sourceAnchor ? `#${topic.sourceAnchor}` : ""}\``,
  );
  if (topic.relatedTopicIds?.length) {
    lines.push(`Related: ${topic.relatedTopicIds.map((id) => `\`${id}\``).join(", ")}`);
  }
  lines.push("");
}

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${lines.join("\n")}\n`, "utf8");
