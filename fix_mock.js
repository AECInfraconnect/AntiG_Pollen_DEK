const fs = require("fs");
const file = "crates/local-control-plane/src/policy_first_api.rs";
let content = fs.readFileSync(file, "utf8");

content = content.replace(
    /let candidate = dek_agent_discovery::model::DiscoveredAgentCandidateV2 \{[\s\S]*?\};\s*let res = dek_enforcement_api::feasibility::assess\(&candidate, ControlLevel::Enforce, &snap\);/g,
    `let pol = dek_enforcement_api::planner::Policy {
        id: "mock_pol".into(),
        requested_level: dek_enforcement_api::planner::ControlLevel::Enforce,
    };
    let res = dek_enforcement_api::planner::assess_feasibility(&pol, &snap);`
);

fs.writeFileSync(file, content);
console.log("Updated policy_first_api.rs mock");

