import os
files = [
    "POLLEN_DEK_AGENT_NAME_RESOLUTION_FIX_2026-06-24.md",
    "POLLEN_DEK_FIX_UNKNOWN_AGENT_IDENTITY.md",
    "POLLEN_DEK_GTM_ONBOARDING_HANDOFF_2026-06-24.md",
    "POLLEN_DEK_PDP_PEP_ROUTING_FRIENDLY_STATUS_HANDOFF_2026-06-24.md",
    "POLLEN_DEK_POLICY_FIRST_UX_UI_REDESIGN_2026-06-24.md",
    "POLLEN_DEK_PRODUCTION_HARDENING_HANDOFF_2026-06-23.md"
]
banner = "> [!WARNING]\n> **superseded by FINALIZATION 2026-06-25 + READY_FOR_TEST_PLAN**\n> This document is kept for reference only.\n\n"
for f in files:
    if os.path.exists(f):
        with open(f, "r", encoding="utf-8") as file:
            content = file.read()
        if "superseded by FINALIZATION" not in content:
            with open(f, "w", encoding="utf-8") as file:
                file.write(banner + content)
        print(f"Updated {f}")

