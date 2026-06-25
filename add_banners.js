const fs = require("fs");
const files = [
    "POLLEN_DEK_AGENT_NAME_RESOLUTION_FIX_2026-06-24.md",
    "POLLEN_DEK_FIX_UNKNOWN_AGENT_IDENTITY.md",
    "POLLEN_DEK_GTM_ONBOARDING_HANDOFF_2026-06-24.md",
    "POLLEN_DEK_PDP_PEP_ROUTING_FRIENDLY_STATUS_HANDOFF_2026-06-24.md",
    "POLLEN_DEK_POLICY_FIRST_UX_UI_REDESIGN_2026-06-24.md",
    "POLLEN_DEK_PRODUCTION_HARDENING_HANDOFF_2026-06-23.md"
];
const banner = "> [!WARNING]\n> **superseded by FINALIZATION 2026-06-25 + READY_FOR_TEST_PLAN**\n> This document is kept for reference only.\n\n";
for (const f of files) {
    if (fs.existsSync(f)) {
        let content = fs.readFileSync(f, "utf-8");
        if (!content.includes("superseded by FINALIZATION")) {
            fs.writeFileSync(f, banner + content, "utf-8");
            console.log("Updated " + f);
        }
    }
}

