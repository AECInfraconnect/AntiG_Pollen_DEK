use crate::model::{PolicyArtifact, PolicySuggestion};

pub fn render_openfga(_suggestion: &PolicySuggestion) -> PolicyArtifact {
    let content =
        "model\n  schema 1.1\n\ntype user\ntype resource\n  relations\n    define viewer: [user]"
            .to_string();
    PolicyArtifact {
        language: "openfga".to_string(),
        name: "model.fga".to_string(),
        content,
    }
}
