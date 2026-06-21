use crate::model::{PolicyArtifact, PolicySuggestion};

pub fn render_cedar(suggestion: &PolicySuggestion) -> PolicyArtifact {
    let content = format!(
        "// {}\n// {}\npermit(\n    principal,\n    action,\n    resource\n);",
        suggestion.title, suggestion.summary
    );
    PolicyArtifact {
        language: crate::model::SuggestedPolicyLanguage::Cedar,
        name: "policy.cedar".to_string(),
        content,
    }
}
