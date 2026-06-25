const fs = require("fs");
const file = "crates/dek-secure-spool/src/lib.rs";
let content = fs.readFileSync(file, "utf8");
content = content.replace(/    }\n        }\n    }\n}\n$/g, "    }\n}\n");
fs.writeFileSync(file, content);

