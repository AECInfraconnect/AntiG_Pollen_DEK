import { describe, expect, it } from "vitest";
import { NAV } from "../../config/navigation";

const modes = [
  "desktop_simple",
  "desktop_advanced",
  "enterprise_cloud",
] as const;

function itemsForMode(mode: (typeof modes)[number]) {
  return NAV.flatMap((group) => group.items).filter((item) =>
    item.modes.includes(mode),
  );
}

describe("Sidebar Configuration", () => {
  it("hides technical control-plane terms in desktop_simple mode", () => {
    const labels = itemsForMode("desktop_simple")
      .map((item) => `${item.en} ${item.th}`)
      .join(" ");

    expect(labels).not.toContain("PEP");
    expect(labels).not.toContain("PDP");
    expect(labels).not.toContain("WFP");
    expect(labels).not.toContain("eBPF");
    expect(labels).not.toContain("NetworkExtension");
  });

  it("keeps core observe-first workflows reachable in every user mode", () => {
    const expectedIds = [
      "overview",
      "scan",
      "my-ai-apps",
      "ai-activity",
      "data-apps",
      "cost",
      "setup",
      "history",
    ];

    for (const mode of modes) {
      const visibleIds = itemsForMode(mode).map((item) => item.id);
      expect(visibleIds).toEqual(expect.arrayContaining(expectedIds));
    }
  });

  it("shows AI usage and cost as a normal activity workflow", () => {
    const activityGroup = NAV.find((group) => group.id === "activity");
    const costItem = activityGroup?.items.find((item) => item.id === "cost");

    expect(costItem?.en).toBe("AI Usage & Cost");
    expect(costItem?.th).toBe("การใช้งานและค่าใช้จ่าย AI");
    expect(costItem?.href).toBe("/cost-ledger");
    expect(costItem?.modes).toEqual([...modes]);
  });

  it("does not ship mojibake Thai labels in the real sidebar config", () => {
    const labels = NAV.flatMap((group) => [
      group.th,
      ...group.items.map((item) => item.th),
    ]).join(" ");

    expect(labels).not.toMatch(/\u0e40\u0e18|\u00e0|\u00c2|\ufffd/);
  });
});
