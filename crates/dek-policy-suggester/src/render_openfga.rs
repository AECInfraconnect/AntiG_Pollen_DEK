use crate::model::{PolicyArtifact, PolicySuggestion};

pub fn render_openfga(_suggestion: &PolicySuggestion) -> PolicyArtifact {
    let content =
        "model\n  schema 1.1\n\ntype user\ntype resource\n  relations\n    define viewer: [user]"
            .to_string();
    PolicyArtifact {
        language: crate::model::SuggestedPolicyLanguage::OpenFga,
        name: "model.fga".to_string(),
        content,
    }
}
