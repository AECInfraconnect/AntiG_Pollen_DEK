import fs from "node:fs";
import yaml from "yaml";

const errors = yaml.parse(fs.readFileSync("catalog/error-codes.yaml", "utf8"));
const decisions = yaml.parse(fs.readFileSync("catalog/decision-enums.yaml", "utf8"));

function fail(message: string): never {
  console.error(`semantic-contract-lint: ${message}`);
  process.exit(1);
}

for (const [code, def] of Object.entries(errors.errors ?? {})) {
  if (!/^[A-Z0-9_]+$/.test(code)) fail(`invalid error code format: ${code}`);
  if (!(def as any).dek_behavior) fail(`missing dek_behavior for ${code}`);
}

const requiredDecisions = ["allow", "deny", "observe", "redact"];
const decisionValues = new Set(decisions.decisions ?? []);
for (const d of requiredDecisions) {
  if (!decisionValues.has(d)) fail(`missing canonical decision enum: ${d}`);
}

console.log("semantic contract lint passed");
