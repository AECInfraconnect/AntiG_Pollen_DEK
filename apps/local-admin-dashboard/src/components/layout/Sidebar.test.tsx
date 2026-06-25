import { describe, it, expect } from 'vitest';
import { NAV } from './Sidebar';

describe('Sidebar Configuration', () => {
  it('should have the correct simple mode routes', () => {
    expect(NAV.simple).toEqual(["/", "/agents", "/protect", "/activity"]);
  });

  it('should have the correct advanced mode routes', () => {
    expect(NAV.advanced).toEqual(["/", "/agents", "/protect", "/activity", "/capabilities", "/plugin-marketplace"]);
  });

  it('should have the correct enterprise mode routes', () => {
    expect(NAV.enterprise).toContain("/policy-presets");
    expect(NAV.enterprise).toContain("/bundles");
  });
});
