const fs = require("fs");
const file = "crates/dek-agent-discovery/src/identity.rs";
let content = fs.readFileSync(file, "utf8");

content = content.replace(
    /"openclaw" => "OpenClaw",/g,
    `"openclaw" => "OpenClaw",
        "antigravity" => "Antigravity",`
);

fs.writeFileSync(file, content);
console.log("Updated identity.rs");

