const fs = require("fs");
const file = "crates/dek-enforcement-api/Cargo.toml";
let content = fs.readFileSync(file, "utf8");
content = content.replace("dek-domain-schema = { path = \"../dek-domain-schema\" }", "dek-domain-schema = { path = \"../dek-domain-schema\" }\ndek-agent-discovery = { path = \"../dek-agent-discovery\" }");
fs.writeFileSync(file, content);
console.log("Updated Cargo.toml");

