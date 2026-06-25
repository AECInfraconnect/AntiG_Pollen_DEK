
const fs = require("fs");
let file = "crates/acceptance-tests/tests/desktop_fallback_test.rs";
let content = fs.readFileSync(file, "utf8");
content = content.replace(/ControlMethod::NetworkControl/g, "ControlMethod::SystemNetworkControl");
fs.writeFileSync(file, content);

file = "crates/acceptance-tests/tests/policy_first_ux_tests.rs";
content = fs.readFileSync(file, "utf8");
if (!content.includes("#![allow(clippy::expect_used)]")) {
    content = "#![allow(clippy::expect_used)]\n" + content;
}
fs.writeFileSync(file, content);

