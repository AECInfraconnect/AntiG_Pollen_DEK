use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorDomain {
    Enrollment,
    Identity,
    Mtls,
    Config,
    Bundle,
    Activation,
    Policy,
    Pdp,
    Pep,
    Wasm,
    Telemetry,
    Storage,
    Update,
    Ebpf,
    Platform,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetryClass {
    NoRetry,
    RetryImmediate,
    RetryWithBackoff,
    RetryAfterReauth,
    RetryAfterAdminAction,
    FatalRequiresReinstall,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SafetyAction {
    DenyRequest,
    KeepLastKnownGood,
    EnterObserveOnly,
    EnterDegradedMode,
    StopService,
    QuarantineArtifact,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorEnvelope {
    pub error_id: String,
    pub domain: ErrorDomain,
    pub code: String,
    pub message: String,
    pub safe_message: String,
    pub retry_class: RetryClass,
    pub safety_action: SafetyAction,
    pub tenant_id: Option<String>,
    pub device_id: Option<String>,
    pub bundle_version: Option<String>,
    pub request_id: Option<String>,
    pub timestamp: String,
    pub remediation: Option<String>,
}

impl std::fmt::Display for ErrorEnvelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}: {}", self.domain, self.code, self.message)
    }
}

impl std::error::Error for ErrorEnvelope {}

impl ErrorEnvelope {
    /// Maps the internal domain and safety action to a standard HTTP status code.
    pub fn to_http_status(&self) -> u16 {
        match self.domain {
            ErrorDomain::Identity | ErrorDomain::Mtls => 401, // Unauthorized
            ErrorDomain::Pep | ErrorDomain::Pdp | ErrorDomain::Policy => {
                if self.safety_action == SafetyAction::DenyRequest {
                    403 // Forbidden
                } else {
                    400 // Bad Request (e.g. invalid policy input)
                }
            }
            ErrorDomain::Enrollment | ErrorDomain::Config | ErrorDomain::Bundle => 400, // Bad Request
            ErrorDomain::Activation | ErrorDomain::Update => 409, // Conflict / Unprocessable
            ErrorDomain::Wasm | ErrorDomain::Ebpf | ErrorDomain::Platform | ErrorDomain::Storage => 500, // Internal Server Error
            ErrorDomain::Telemetry => 503, // Service Unavailable (queue full etc)
        }
    }

    /// Maps the internal domain and safety action to a standard gRPC Code.
    pub fn to_grpc_status(&self) -> tonic::Code {
        match self.domain {
            ErrorDomain::Identity | ErrorDomain::Mtls => tonic::Code::Unauthenticated,
            ErrorDomain::Pep | ErrorDomain::Pdp | ErrorDomain::Policy => {
                if self.safety_action == SafetyAction::DenyRequest {
                    tonic::Code::PermissionDenied
                } else {
                    tonic::Code::InvalidArgument
                }
            }
            ErrorDomain::Enrollment | ErrorDomain::Config | ErrorDomain::Bundle => tonic::Code::InvalidArgument,
            ErrorDomain::Activation | ErrorDomain::Update => tonic::Code::FailedPrecondition,
            ErrorDomain::Wasm | ErrorDomain::Ebpf | ErrorDomain::Platform | ErrorDomain::Storage => tonic::Code::Internal,
            ErrorDomain::Telemetry => tonic::Code::Unavailable,
        }
    }
}
