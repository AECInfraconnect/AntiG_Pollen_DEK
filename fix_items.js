
const fs = require("fs");
let file = "crates/dek-deployment-planner/src/suggestion.rs";
let content = fs.readFileSync(file, "utf8");
content = content.replace("#[cfg(test)]\nmod tests {", "#[cfg(test)]\n#[allow(clippy::items_after_test_module)]\nmod tests {");
fs.writeFileSync(file, content);

