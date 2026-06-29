import referenceDefinitions from "../data/entity-reference-intel.compact.json";

export interface ReferenceIntel {
  id: string;
  title: string;
  vendor: string;
  category: string;
  description: string;
  controlNotes: string;
  sourceLabel: string;
  sourceUrl: string;
  reviewedAt: string;
  keywords: string[];
  vendorKeywords?: string[];
  entityKinds: Array<"agent" | "tool" | "resource" | "policy" | "identity">;
  visualIdentity?: {
    icon: string;
    label: string;
    background: string;
    foreground: string;
    sourceLabel: string;
  };
  expectedCapabilities?: Array<{
    id: string;
    label: string;
    keywords: string[];
  }>;
  observeGuide?: {
    summary: string;
    probeKeys: string[];
    signals: Array<{
      label: string;
      probeKeys: string[];
      meaning: string;
      detail: string;
      userAction: string;
    }>;
    caveat: string;
  };
}

const REFERENCES = referenceDefinitions as ReferenceIntel[];

function normalize(value?: string | null) {
  return (value ?? "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
}

export function findReferenceIntel(input: {
  entityKind: ReferenceIntel["entityKinds"][number];
  name?: string;
  vendor?: string;
  type?: string;
  runtimeName?: string;
  uri?: string;
  host?: string;
  path?: string;
}) {
  const haystack = normalize(
    [
      input.name,
      input.vendor,
      input.type,
      input.runtimeName,
      input.uri,
      input.host,
      input.path,
    ]
      .filter(Boolean)
      .join(" "),
  );
  const vendor = normalize(input.vendor);
  const normalizedName = normalize(input.name);
  const normalizedRuntime = normalize(input.runtimeName);

  return REFERENCES.map((reference) => {
    if (!reference.entityKinds.includes(input.entityKind)) return false;
    let score = 0;
    for (const keyword of reference.keywords) {
      const normalizedKeyword = normalize(keyword);
      if (!normalizedKeyword || !haystack.includes(normalizedKeyword)) continue;
      const specificKeyword = normalizedKeyword !== vendor;
      score += specificKeyword ? 20 + normalizedKeyword.length : 3;
      if (normalizedName.includes(normalizedKeyword)) score += 20;
      if (normalizedRuntime.includes(normalizedKeyword)) score += 10;
    }
    const title = normalize(reference.title);
    if (title && (normalizedName.includes(title) || haystack.includes(title))) {
      score += 50;
    }
    const vendorMatch =
      vendor &&
      reference.vendorKeywords?.some((keyword) =>
        vendor.includes(normalize(keyword)),
      );
    if (vendorMatch) score += 2;
    return score > 0 ? { reference, score } : false;
  })
    .filter((entry): entry is { reference: ReferenceIntel; score: number } =>
      Boolean(entry),
    )
    .sort((left, right) => right.score - left.score)
    .map((entry) => entry.reference);
}

export function findAgentReferenceIntel(input: {
  name?: string;
  vendor?: string;
  agentType?: string;
  runtimeName?: string;
}) {
  return findReferenceIntel({
    entityKind: "agent",
    name: input.name,
    vendor: input.vendor,
    type: input.agentType,
    runtimeName: input.runtimeName,
  });
}

export function assessExpectedCapabilities(
  references: ReferenceIntel[],
  observedCapabilities: string[],
) {
  const observed = normalize(observedCapabilities.join(" "));
  return references.flatMap((reference) =>
    (reference.expectedCapabilities ?? []).map((capability) => ({
      ...capability,
      referenceId: reference.id,
      referenceTitle: reference.title,
      detected: capability.keywords.some((keyword) =>
        observed.includes(normalize(keyword)),
      ),
    })),
  );
}

export function matchObserveGuideSignals(
  reference: ReferenceIntel | undefined,
  observedTerms: Array<string | undefined | null>,
) {
  const guide = reference?.observeGuide;
  if (!guide) return [];
  const observed = normalize(observedTerms.filter(Boolean).join(" "));
  return guide.signals.map((signal) => ({
    ...signal,
    detected: signal.probeKeys.some((key) => observed.includes(normalize(key))),
  }));
}
