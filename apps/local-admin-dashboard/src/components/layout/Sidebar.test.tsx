import { describe, it, expect } from "vitest";
import { getNavItems } from "../../navigation/menu";

describe("Sidebar Configuration", () => {
  it("should hide technical terms in desktop_simple mode", () => {
    const nav = getNavItems("desktop_simple");
    const labels = nav.map(item => item.label.en).join(" ");
    
    expect(labels).not.toContain("PEP");
    expect(labels).not.toContain("PDP");
    expect(labels).not.toContain("WFP");
    expect(labels).not.toContain("eBPF");
    expect(labels).not.toContain("NetworkExtension");
  });

  it("should show PEP/PDP in enterprise_server mode", () => {
    const nav = getNavItems("enterprise_server");
    const labels = nav.map(item => item.label.en).join(" ");
    
    expect(labels).toContain("PEP");
    expect(labels).toContain("PDP");
  });
});

