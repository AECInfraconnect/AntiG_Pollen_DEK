
const fs = require("fs");
let file = "crates/dek-deployment-planner/src/suggestion.rs";
let content = fs.readFileSync(file, "utf8");

// We know tests block is from "#[cfg(test)]" to the matching "}" before "pub fn suggest_for_agent".
let startIdx = content.indexOf("#[cfg(test)]");
if (startIdx !== -1) {
    let beforeTest = content.substring(0, startIdx);
    let afterTestIdx = content.indexOf("pub fn suggest_for_agent");
    if (afterTestIdx !== -1) {
        let testBlock = content.substring(startIdx, afterTestIdx);
        let afterTest = content.substring(afterTestIdx);
        let newContent = beforeTest + afterTest + "\n" + testBlock;
        fs.writeFileSync(file, newContent);
    }
}

