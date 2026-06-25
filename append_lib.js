const fs = require("fs");
const file = "crates/dek-enforcement-api/src/lib.rs";
let content = fs.readFileSync(file, "utf8");
content += "\npub mod warm_check;\npub mod feasibility;\n";
fs.writeFileSync(file, content);
console.log("Updated lib.rs");

