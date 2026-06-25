
const fs = require("fs");
let file = "crates/dek-secure-spool/src/lib.rs";
let content = fs.readFileSync(file, "utf8");
content = content.replace("100 * 1, // tiny size forces rotation\n", "100, // tiny size forces rotation\n");
fs.writeFileSync(file, content);

