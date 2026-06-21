use serde::{Deserialize, Serialize};

pub const LOCAL_TENANT_ID: &str = "local";
pub const LOCAL_WORKSPACE_ID: &str = "default";
pub const LOCAL_ENVIRONMENT_ID: &str = "local";
pub const LOCAL_ACTOR_ID: &str = "local-admin";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPlaneIdentity {
    pub tenant_id: String,
    pub workspace_id: String,
    pub environment_id: String,
    pub actor_id: String,
    pub auth_subject: Option<String>,
}

impl ControlPlaneIdentity {
    pub fn local_default() -> Self {
        Self {
            tenant_id: LOCAL_TENANT_ID.to_string(),
            workspace_id: LOCAL_WORKSPACE_ID.to_string(),
            environment_id: LOCAL_ENVIRONMENT_ID.to_string(),
            actor_id: LOCAL_ACTOR_ID.to_string(),
            auth_subject: Some("local-admin".to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlPlaneMode {
    LocalSingleUser,
    CloudMultiTenant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPlaneProfile {
    pub mode: ControlPlaneMode,
    pub base_url: String,
    pub tenant_id: String,
    pub workspace_id: String,
    pub environment_id: String,
    pub auth: ControlPlaneAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlPlaneAuth {
    NoneForDevOnly,
    LocalBearerToken {
        token: String,
    },
    OidcBearer {
        access_token: String,
    },
    Mtls {
        cert_path: String,
        key_path: String,
        ca_path: String,
    },
    SpiffeJwtSvid {
        jwt: String,
    },
}
