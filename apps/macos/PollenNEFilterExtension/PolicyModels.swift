import Foundation

enum PollenVerdictAction: String, Codable {
    case allow
    case block
    case needMoreRules
}

struct PollenPolicyBundle: Codable {
    let version: String
    let generatedAt: Date
    let defaultAction: PollenVerdictAction
    let rules: [PollenRule]
}

struct PollenRule: Codable {
    let id: String
    let action: PollenVerdictAction
    let remoteHostSuffixes: [String]
    let remotePorts: [Int]
    let processBundleIds: [String]
    let reason: String
}

struct FlowMetadata: Codable {
    let flowIdentifier: String
    let direction: String
    let remoteHostname: String?
    let remoteAddress: String?
    let remotePort: Int?
    let sourceAppIdentifier: String?
    let timestamp: Date
}

struct PolicyDecision: Codable {
    let action: PollenVerdictAction
    let ruleId: String?
    let reason: String
    let auditRequired: Bool
}
