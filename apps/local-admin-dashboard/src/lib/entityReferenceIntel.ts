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
}

const REFERENCES = referenceDefinitions as ReferenceIntel[];

function normalize(value?: string | null) {
  return (value ?? "").toLowerCase().replace(/[^a-z0-9]+/g, " ").trim();
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

  return REFERENCES.filter((reference) => {
    if (!reference.entityKinds.includes(input.entityKind)) return false;
    const keywordMatch = reference.keywords.some((keyword) =>
      haystack.includes(normalize(keyword)),
    );
    const vendorMatch =
      vendor &&
      reference.vendorKeywords?.some((keyword) =>
        vendor.includes(normalize(keyword)),
      );
    return keywordMatch || vendorMatch;
  });
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
