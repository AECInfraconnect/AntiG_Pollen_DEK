const LEGACY_DASHBOARD_STORAGE_KEYS = [
  "pollek.aiActivity.demoEvents",
  "pollek.activity.demoEvents",
  "pollek.discovery.demoCandidates",
  "pollek.discovery.fallbackCandidates",
  "pollek.guard.demoIncidents",
  "pollek.plugins.fixtureRegistry",
  "pollek.scan.cachedDemo",
  "pollek.timeline.mockEvents",
  "pollek.mockCloud.profile",
];

const LEGACY_DASHBOARD_STORAGE_PREFIXES = [
  "pollek.demo.",
  "pollek.fixture.",
  "pollek.fallback.",
  "pollek.mock.",
];

export function cleanupLegacyDashboardStorage() {
  if (typeof window === "undefined") return;
  const storage = window.localStorage;
  for (const key of LEGACY_DASHBOARD_STORAGE_KEYS) {
    storage.removeItem(key);
  }
  const keys: string[] = [];
  for (let index = 0; index < storage.length; index += 1) {
    const key = storage.key(index);
    if (key) keys.push(key);
  }
  for (const key of keys) {
    if (LEGACY_DASHBOARD_STORAGE_PREFIXES.some((prefix) => key.startsWith(prefix))) {
      storage.removeItem(key);
    }
  }
  if (storage.getItem("dek_admin_profile") === "mock-cloud") {
    storage.removeItem("dek_admin_profile");
  }
}
