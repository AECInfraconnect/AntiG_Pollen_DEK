
const fs = require("fs");
let file = "crates/dek-enforcement-api/src/planner.rs";
let content = fs.readFileSync(file, "utf8");
content = content.replace("per_domain.push(DomainFeasibility::ok(domain, &m, lvl));", "per_domain.push(DomainFeasibility::ok(domain, m, lvl));");
fs.writeFileSync(file, content);

