pub const SUPPORTED_REST_OPERATIONS: &[(&str, &str)] = &[
    ("GET", "/ping"),
    ("GET", "/account"),
    ("GET", "/metrics/timeseries"),
    ("GET", "/metrics/summary"),
    ("GET", "/metrics/prompt_summary"),
    ("GET", "/metrics/sov"),
    ("GET", "/metrics/top_sources"),
    ("GET", "/metrics/agent_traffic"),
    ("GET", "/metrics/ai_traffic"),
    ("GET", "/web_analytics/schema"),
    ("POST", "/web_analytics/query"),
    ("GET", "/search_console/properties"),
    ("GET", "/search_console/summary"),
    ("GET", "/search_console/timeseries"),
    ("GET", "/search_console/queries"),
    ("GET", "/search_console/pages"),
    ("GET", "/search_console/rows"),
    ("GET", "/search_console/opportunities"),
    ("GET", "/search_console/cannibalization"),
    ("GET", "/search_console/decay"),
    ("GET", "/search_console/brand_split"),
    ("GET", "/search_console/ai_visibility_gaps"),
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
    ("GET", "/dimensions/shopping"),
    ("GET", "/dimensions/ads"),
    ("GET", "/dimensions/local_businesses"),
    ("GET", "/dimensions/owned_media"),
    ("GET", "/dimensions/owned_media/summary"),
    ("GET", "/dimensions/owned_media/citing_prompts"),
    ("GET", "/dimensions/reddit"),
    ("GET", "/dimensions/query_fan_outs"),
    ("GET", "/reputation/reports"),
    ("GET", "/reputation/reports/{id}"),
    ("GET", "/studies"),
    ("GET", "/studies/{id}"),
    ("GET", "/studies/{id}/reports/{report_id}"),
    ("GET", "/citation_intelligence/groups"),
    ("GET", "/citation_intelligence/mentions_by_domain"),
    ("GET", "/citation_intelligence/urls/{url_sha256}"),
    (
        "GET",
        "/citation_intelligence/urls/{url_sha256}/occurrences",
    ),
    ("GET", "/citation_intelligence/urls/{url_sha256}/content"),
    ("GET", "/url_lists"),
    ("GET", "/url_lists/{id}"),
    ("POST", "/url_lists"),
    ("PATCH", "/url_lists/{id}"),
    ("DELETE", "/url_lists/{id}"),
    ("POST", "/url_lists/{id}/entries"),
    ("POST", "/url_lists/{id}/entries/remove"),
    ("GET", "/reports/ai_model_insights/summary"),
    ("GET", "/reports/ai_model_insights/position_distribution"),
    ("GET", "/reports/ai_model_insights/ai_overview_results"),
    ("GET", "/answers"),
    ("GET", "/answers/{id}"),
    ("GET", "/sentiments"),
    ("GET", "/reports/sentiment/topics"),
    ("GET", "/reports/sentiment/tags"),
    ("GET", "/recommendations"),
    ("POST", "/recommendations"),
    ("POST", "/reputation/reports"),
    ("GET", "/recommendations/{id}"),
    ("PATCH", "/recommendations/{id}/items/{item_id}"),
    ("GET", "/custom_reports"),
    ("GET", "/custom_reports/{id}"),
    ("POST", "/custom_reports/{id}/send"),
    ("POST", "/project_drafts"),
    ("GET", "/project_drafts/{id}"),
    ("PATCH", "/project_drafts/{id}"),
    ("POST", "/project_drafts/{id}/finalize"),
    ("POST", "/projects"),
    ("PATCH", "/projects/{id}"),
    ("DELETE", "/projects/{id}"),
    ("GET", "/projects/{id}/owned_media"),
    ("PATCH", "/projects/{id}/owned_media"),
    ("POST", "/prompts"),
    ("PATCH", "/prompts/{id}"),
    ("DELETE", "/prompts/{id}"),
    ("POST", "/prompts/assign_tags"),
    ("POST", "/prompts/remove_tags"),
    ("POST", "/prompts/import"),
    ("POST", "/prompt_generations"),
    ("GET", "/prompt_generations/{id}"),
    ("GET", "/suggested_prompts"),
    ("POST", "/suggested_prompts/accept"),
    ("POST", "/suggested_prompts/reject"),
    ("GET", "/competitors/suggestions"),
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
    ("GET", "/team_members"),
    ("GET", "/team_invitations"),
    ("POST", "/team_invitations"),
    ("DELETE", "/team_invitations/{id}"),
    ("DELETE", "/team_members/{id}"),
    ("GET", "/api_keys"),
    ("POST", "/api_keys"),
    ("DELETE", "/api_keys/{id}"),
    ("GET", "/store_connection"),
    ("GET", "/ai_orders"),
    ("PUT", "/ai_orders"),
    ("GET", "/catalog_prompt_suggestions"),
    ("POST", "/catalog_prompt_suggestions"),
    ("POST", "/catalog_prompt_suggestions/accept"),
    ("POST", "/catalog_prompt_suggestions/reject"),
    ("GET", "/technical_geo_reports"),
    ("GET", "/technical_geo_reports/{id}"),
    ("POST", "/technical_geo_reports"),
    ("PATCH", "/technical_geo_reports/{id}/content"),
    ("POST", "/technical_geo_reports/{id}/revert_content"),
    ("GET", "/intelligence_tasks"),
    ("POST", "/intelligence_tasks"),
    ("GET", "/intelligence_tasks/{id}"),
    ("PATCH", "/intelligence_tasks/{id}"),
    ("POST", "/intelligence_tasks/{id}/revert"),
    ("POST", "/intelligence_tasks/{id}/send_to_cms"),
    ("GET", "/cms/content"),
    ("GET", "/cms/content/{id}"),
    ("GET", "/geo_audits"),
    ("POST", "/geo_audits"),
    ("GET", "/geo_audits/{id}"),
    ("PATCH", "/geo_audits/{id}"),
    ("DELETE", "/geo_audits/{id}"),
    ("GET", "/geo_audits/{id}/comparison"),
    ("GET", "/geo_audits/{geo_audit_id}/runs"),
    ("POST", "/geo_audits/{geo_audit_id}/runs"),
    ("GET", "/geo_audits/{geo_audit_id}/runs/{sequence}"),
    ("GET", "/geo_audits/{geo_audit_id}/runs/{sequence}/findings"),
    ("GET", "/geo_audits/{geo_audit_id}/issues"),
    ("PATCH", "/geo_audits/{geo_audit_id}/issues/{id}"),
    ("GET", "/geo_alerts"),
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
                    .any(|tag| matches!(tag.as_str(), Some("MCP" | "OAuth" | "OAuth 2.1")));
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

    // Query parameters are held to their endpoint by tests/contract_wiring.rs,
    // which runs every command against a recording server.
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
