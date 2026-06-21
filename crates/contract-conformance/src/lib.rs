#[cfg(test)]
mod tests {
    use serde_json::Value;
    use std::fs;

    #[test]
    fn test_bundle_envelope_schema_valid() {
        let schema_str =
            fs::read_to_string("../../contracts/schemas/bundle-envelope.v1.schema.json").unwrap();
        let schema_json: Value = serde_json::from_str(&schema_str).unwrap();

        let valid_envelope: Value = serde_json::json!({
            "schema_version": "bundle-envelope.v1",
            "manifest": { "key": "value" },
            "signatures": [
                { "signature_id": "test", "payload": "xyz" }
            ]
        });

        assert!(jsonschema::is_valid(&schema_json, &valid_envelope));
    }
}
