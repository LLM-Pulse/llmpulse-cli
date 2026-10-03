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
    ("GET", "/dimensions/shopping"),
    ("GET", "/dimensions/ads"),
    ("GET", "/dimensions/local_businesses"),
    ("GET", "/dimensions/owned_media"),
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
    ("PATCH", "/projects/{id}"),
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

    // The path list above only proves the CLI knows an endpoint exists. It says
    // nothing about the parameters that endpoint takes, and that is where the
    // CLI actually fell behind: by 1.46.0 it never sent prompt_type, brand_kind,
    // the four Search Console narrowing parameters, the answer mention and
    // citation filters, or the view/order/owned family the shopping, ads,
    // owned-media and Reddit lists are built around. Nothing failed, because
    // nothing compared them.
    //
    // This is a name-level check: a parameter the contract documents has to
    // appear somewhere in the CLI source. It does not prove the parameter is
    // wired to the right endpoint, which would need the query builders to be
    // data rather than code. It does catch the drift that happened, which is a
    // parameter being added to the API and the CLI never hearing about it.
    #[test]
    fn every_documented_query_parameter_is_known_to_the_cli() {
        // Deliberately not exposed, with the reason. Growing this list is a
        // decision; leaving it empty by accident is the bug above.
        const UNSUPPORTED_QUERY_PARAMS: &[(&str, &str)] = &[(
            "output",
            "the CLI renders json, table and csv itself from the JSON response, \
             so asking the API for a flat or csv body would bypass its own \
             formatter and collide with the global --output flag",
        )];

        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let bundled = fs::read_to_string(manifest_dir.join("openapi.json")).unwrap();
        let document: Value = serde_json::from_str(&bundled).unwrap();
        let components = &document["components"]["parameters"];
        let source = format!("{}{}", include_str!("commands.rs"), include_str!("cli.rs"));

        let exempt = UNSUPPORTED_QUERY_PARAMS
            .iter()
            .map(|(name, _)| *name)
            .collect::<BTreeSet<_>>();
        let mut missing = BTreeSet::new();

        for (path, entry) in document["paths"].as_object().unwrap() {
            for (method, operation) in entry.as_object().unwrap() {
                if !["get", "post", "put", "patch", "delete"].contains(&method.as_str()) {
                    continue;
                }
                let tags = operation["tags"].as_array().cloned().unwrap_or_default();
                if tags
                    .iter()
                    .any(|tag| matches!(tag.as_str(), Some("MCP" | "OAuth" | "OAuth 2.1")))
                {
                    continue;
                }

                let shared = entry["parameters"].as_array().cloned().unwrap_or_default();
                let own = operation["parameters"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                for parameter in shared.iter().chain(own.iter()) {
                    let parameter = match parameter["$ref"].as_str() {
                        Some(reference) => &components[reference.rsplit('/').next().unwrap()],
                        None => parameter,
                    };
                    if parameter["in"].as_str() != Some("query") {
                        continue;
                    }
                    let name = parameter["name"].as_str().unwrap();
                    if exempt.contains(name) {
                        continue;
                    }
                    // Rails array parameters are sent as name[].
                    if source.contains(&format!("\"{name}\""))
                        || source.contains(&format!("\"{name}[]\""))
                    {
                        continue;
                    }
                    missing.insert(format!("{name} ({} {path})", method.to_uppercase()));
                }
            }
        }

        assert!(
            missing.is_empty(),
            "these documented query parameters are not sent by the CLI. Wire each one \
             into its command, or add it to UNSUPPORTED_QUERY_PARAMS with the reason: {missing:#?}"
        );
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
