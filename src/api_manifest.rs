pub const SUPPORTED_REST_OPERATIONS: &[(&str, &str)] = &[
    ("GET", "/ping"),
    ("GET", "/metrics/timeseries"),
    ("GET", "/metrics/summary"),
    ("GET", "/metrics/prompt_summary"),
    ("GET", "/metrics/sov"),
    ("GET", "/metrics/top_sources"),
    ("GET", "/metrics/agent_traffic"),
    ("GET", "/metrics/ai_traffic"),
    ("GET", "/search_console/summary"),
    ("GET", "/search_console/timeseries"),
    ("GET", "/search_console/queries"),
    ("GET", "/search_console/pages"),
    ("GET", "/dimensions/projects"),
    ("GET", "/dimensions/projects/{id}"),
    ("GET", "/dimensions/competitors"),
    ("GET", "/dimensions/competitors/{id}"),
    ("GET", "/dimensions/collections"),
    ("GET", "/dimensions/tags"),
    ("GET", "/dimensions/models"),
    ("GET", "/dimensions/locales"),
    ("GET", "/dimensions/sentiments"),
    ("GET", "/dimensions/prompts"),
    ("GET", "/dimensions/prompt_executions"),
    ("GET", "/dimensions/sources"),
    ("GET", "/dimensions/mentions"),
    ("GET", "/dimensions/citations"),
    ("GET", "/dimensions/competitor_mentions"),
    ("GET", "/dimensions/competitor_citations"),
    ("GET", "/dimensions/all_mentions"),
    ("GET", "/dimensions/all_citations"),
    ("GET", "/dimensions/agent_bots"),
    ("GET", "/citation_intelligence/groups"),
    ("GET", "/citation_intelligence/mentions_by_domain"),
    ("GET", "/citation_intelligence/urls/{url_sha256}"),
    (
        "GET",
        "/citation_intelligence/urls/{url_sha256}/occurrences",
    ),
    ("GET", "/citation_intelligence/urls/{url_sha256}/content"),
    ("GET", "/reports/ai_model_insights/summary"),
    ("GET", "/reports/ai_model_insights/position_distribution"),
    ("GET", "/reports/ai_model_insights/ai_overview_results"),
    ("GET", "/answers"),
    ("GET", "/answers/{id}"),
    ("GET", "/sentiments"),
    ("GET", "/recommendations"),
    ("POST", "/recommendations"),
    ("GET", "/recommendations/{id}"),
    ("POST", "/project_drafts"),
    ("GET", "/project_drafts/{id}"),
    ("PATCH", "/project_drafts/{id}"),
    ("POST", "/project_drafts/{id}/finalize"),
    ("POST", "/projects"),
    ("POST", "/prompts"),
    ("DELETE", "/prompts/{id}"),
    ("POST", "/prompts/assign_tags"),
    ("POST", "/competitors"),
    ("PATCH", "/competitors/{id}"),
    ("DELETE", "/competitors/{id}"),
    ("POST", "/collections"),
    ("PATCH", "/collections/{id}"),
    ("DELETE", "/collections/{id}"),
    ("GET", "/annotations"),
    ("POST", "/annotations"),
    ("PATCH", "/annotations/{id}"),
    ("DELETE", "/annotations/{id}"),
    ("POST", "/technical_geo_reports"),
    ("GET", "/intelligence_tasks"),
    ("POST", "/intelligence_tasks"),
    ("GET", "/intelligence_tasks/{id}"),
    ("GET", "/webhooks"),
    ("POST", "/webhooks"),
    ("DELETE", "/webhooks/{id}"),
    ("GET", "/webhooks/sample/{event_type}"),
];

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, fs, path::Path};

    use serde_json::Value;

    use super::*;

    #[test]
    fn matches_the_openapi_rest_surface() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let bundled = fs::read_to_string(manifest_dir.join("openapi.json")).unwrap();
        let monorepo_document = manifest_dir.join("../../public/openapi.json");
        if monorepo_document.exists() {
            assert_eq!(bundled, fs::read_to_string(monorepo_document).unwrap());
        }
        let document: Value = serde_json::from_str(&bundled).unwrap();
        let mut documented = BTreeSet::new();
        for (path, entry) in document["paths"].as_object().unwrap() {
            for (method, operation) in entry.as_object().unwrap() {
                if !["get", "post", "put", "patch", "delete"].contains(&method.as_str()) {
                    continue;
                }
                let tags = operation["tags"].as_array().cloned().unwrap_or_default();
                let protocol_only = tags
                    .iter()
                    .any(|tag| matches!(tag.as_str(), Some("MCP" | "OAuth")));
                if !protocol_only {
                    documented.insert((method.to_uppercase(), path.clone()));
                }
            }
        }
        let supported = SUPPORTED_REST_OPERATIONS
            .iter()
            .map(|(method, path)| (method.to_string(), path.to_string()))
            .collect::<BTreeSet<_>>();
        assert_eq!(supported, documented);
    }

    #[test]
    fn every_manifest_path_is_wired_into_the_command_dispatcher() {
        let source = include_str!("commands.rs");
        for (_, path) in SUPPORTED_REST_OPERATIONS {
            let literal = path.split('{').next().unwrap().trim_end_matches('/');
            assert!(
                source.contains(literal),
                "REST operation path is not wired into commands.rs: {path}"
            );
        }
    }
}
