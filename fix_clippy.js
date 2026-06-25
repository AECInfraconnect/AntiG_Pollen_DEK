
const fs = require("fs");

let file = "crates/dek-domain-schema/src/user_event.rs";
let content = fs.readFileSync(file, "utf8");
content = content.replace("use chrono::Utc;", "");
content = content.replace("#[cfg(test)]\nmod tests {", "#[cfg(test)]\n#[allow(clippy::unwrap_used)]\nmod tests {");
fs.writeFileSync(file, content);

file = "crates/dek-domain-schema/src/entity_card.rs";
content = fs.readFileSync(file, "utf8");
content = content.replace("#[cfg(test)]\nmod tests {", "#[cfg(test)]\n#[allow(clippy::unwrap_used)]\nmod tests {");
fs.writeFileSync(file, content);

file = "crates/dek-domain-schema/src/feasibility.rs";
content = fs.readFileSync(file, "utf8");
content = content.replace("#[cfg(test)]\nmod tests {", "#[cfg(test)]\n#[allow(clippy::unwrap_used)]\nmod tests {");
fs.writeFileSync(file, content);

