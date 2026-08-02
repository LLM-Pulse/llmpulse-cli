use std::{
    fs,
    io::{self, IsTerminal},
    path::PathBuf,
    time::Duration,
};

use chrono::{Duration as ChronoDuration, Utc};
use clap::CommandFactory;
use dialoguer::{Confirm, Password, Select};
use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    cli::*,
    client::{ApiClient, ApiResponse},
    config::{self, Overrides},
    error::{CliError, Result},
    output::{self, OutputFormat},
    query::{Query, QueryExt, compact_object},
};

pub async fn run(cli: Cli) -> Result<()> {
    let Cli { global, command } = cli;
    match command {
        Command::Config { command } => run_config(command),
        Command::Completions(args) => {
            let mut command = Cli::command();
            clap_complete::generate(args.shell, &mut command, "llmpulse", &mut io::stdout());
            Ok(())
        }
        Command::Login => run_login(&global).await,
        command => {
            let client = client_from_global(&global)?;
            run_api_command(&client, &global, command).await
        }
    }
}

fn client_from_global(global: &GlobalArgs) -> Result<ApiClient> {
    ApiClient::new(config::resolve(Overrides {
        api_key: global.api_key.clone(),
        base_url: global.base_url.clone(),
        project_id: global.project_id,
        verbose: global.verbose,
    })?)
}

fn run_config(command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Set { key, value } => {
            config::set_value(&key, &value)?;
            println!("Set {key}");
        }
        ConfigCommand::Get { key } => {
            if let Some(mut value) = config::get_value(&key)? {
                if key == "api_key" {
                    value = config::mask_api_key(&value);
                }
                println!("{value}");
            }
        }
        ConfigCommand::Show => {
            let mut value = serde_json::to_value(config::read_config()?)?;
            if let Some(api_key) = value.get_mut("api_key") {
                if let Some(raw) = api_key.as_str() {
                    *api_key = Value::String(config::mask_api_key(raw));
                }
            }
            output::print(&value, OutputFormat::Json)?;
        }
        ConfigCommand::Use { profile } => {
            config::set_active_profile(&profile)?;
            println!("Active profile: {profile}");
        }
        ConfigCommand::Profiles => {
            let active = config::active_profile()?;
            for profile in config::list_profiles()? {
                let marker = if active.as_deref() == Some(profile.as_str()) {
                    "*"
                } else {
                    " "
                };
                println!("{marker} {profile}");
            }
        }
    }
    Ok(())
}

async fn run_login(global: &GlobalArgs) -> Result<()> {
    let interactive = io::stdin().is_terminal();
    let api_key = if let Some(api_key) = &global.api_key {
        api_key.clone()
    } else if let Ok(api_key) = std::env::var("LLMPULSE_API_KEY") {
        api_key
    } else if interactive {
        Password::new()
            .with_prompt("LLM Pulse API key")
            .interact()?
    } else {
        return Err(CliError::Usage(
            "Pass --api-key or set LLMPULSE_API_KEY when login is not interactive".into(),
        ));
    };

    let resolved = config::resolve(Overrides {
        api_key: Some(api_key.clone()),
        base_url: global.base_url.clone(),
        project_id: global.project_id,
        verbose: global.verbose,
    })?;
    let client = ApiClient::new(resolved)?;
    let mut ping_query = Query::new();
    ping_query.push_opt("project_id", global.project_id);
    client.get("/ping", &ping_query).await?;
    config::set_value("api_key", &api_key)?;
    if let Some(base_url) = &global.base_url {
        config::set_value("base_url", base_url)?;
    }

    let project_id = if let Some(project_id) = global.project_id {
        Some(project_id)
    } else if interactive {
        choose_project(&client).await?
    } else {
        None
    };
    if let Some(project_id) = project_id {
        config::set_value("default_project_id", &project_id.to_string())?;
    }
    println!("API key verified and configuration saved");
    Ok(())
}

async fn choose_project(client: &ApiClient) -> Result<Option<u64>> {
    let response = client.get("/dimensions/projects", &Vec::new()).await?;
    let Some(projects) = output::extract_array(&response.data) else {
        return Ok(None);
    };
    let choices = projects
        .iter()
        .filter_map(|project| {
            let id = project.get("id")?.as_u64()?;
            let name = project
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Unnamed project");
            Some((id, format!("{name} ({id})")))
        })
        .collect::<Vec<_>>();
    match choices.len() {
        0 => Ok(None),
        1 => Ok(Some(choices[0].0)),
        _ => {
            let labels = choices.iter().map(|(_, label)| label).collect::<Vec<_>>();
            let selected = Select::new()
                .with_prompt("Default project")
                .items(&labels)
                .interact()?;
            Ok(Some(choices[selected].0))
        }
    }
}

async fn run_api_command(client: &ApiClient, global: &GlobalArgs, command: Command) -> Result<()> {
    match command {
        Command::Ping => {
            let mut query = Query::new();
            query.push_opt("project_id", global.project_id);
            print_response(client.get("/ping", &query).await?, global.output, false)
        }
        Command::Status(args) => run_status(client, global, args.range).await,
        Command::Dimensions { command } => run_dimensions(client, global, command).await,
        Command::Metrics { command } => run_metrics(client, global, command).await,
        Command::SearchConsole { command } => run_search_console(client, global, command).await,
        Command::Citations { command } => run_citations(client, global, command).await,
        Command::Answers { command } => run_answers(client, global, command).await,
        Command::Sentiments { command } => run_sentiments(client, global, command).await,
        Command::Recommendations { command } => run_recommendations(client, global, command).await,
        Command::IntelligenceTasks { command } => {
            run_intelligence_tasks(client, global, command).await
        }
        Command::Prompts { command } => run_prompts(client, global, command).await,
        Command::Competitors { command } => run_competitors(client, global, command).await,
        Command::Projects { command } => run_projects(client, global, command).await,
        Command::Collections { command } => run_collections(client, global, command).await,
        Command::Annotations { command } => run_annotations(client, global, command).await,
        Command::Reports { command } => run_reports(client, global, command).await,
        Command::Webhooks { command } => run_webhooks(client, global, command).await,
        Command::Diff(args) => run_diff(client, global, args).await,
        Command::Report(args) => run_report(client, global, args).await,
        Command::Watch(args) => run_watch(client, global, args).await,
        Command::Export(args) => run_export(client, global, args).await,
        Command::Api(args) => run_raw_api(client, global, args).await,
        Command::Login | Command::Config { .. } | Command::Completions(_) => unreachable!(),
    }
}

fn project_query(client: &ApiClient) -> Result<Query> {
    Ok(vec![(
        "project_id".into(),
        client.project_id()?.to_string(),
    )])
}

fn add_dates(query: &mut Query, dates: &DateArgs, include_range: bool) {
    if include_range {
        query.push_opt("range", dates.range);
    }
    query.push_opt("from", dates.from.as_deref());
    query.push_opt("to", dates.to.as_deref());
}

fn add_filters(query: &mut Query, filters: &FilterArgs, include_range: bool) {
    query.push_opt("model", filters.model.as_deref());
    query.push_opt("collection_id", filters.collection_id);
    query.push_opt("country_code", filters.country_code.as_deref());
    query.push_opt("language_code", filters.language_code.as_deref());
    query.push_opt("prompt", filters.prompt);
    add_dates(query, &filters.dates, include_range);
}

fn add_pagination(query: &mut Query, global: &GlobalArgs) {
    query.push_opt("page", global.page);
    query.push_opt("per_page", global.per_page);
}

fn add_csv_values<T: ToString>(query: &mut Query, key: &str, values: &[T]) {
    if !values.is_empty() {
        query.push((
            key.to_owned(),
            values
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        ));
    }
}

async fn list_get(
    client: &ApiClient,
    global: &GlobalArgs,
    path: &str,
    mut query: Query,
) -> Result<()> {
    if global.all {
        let value = fetch_all(client, path, query).await?;
        output::print(&value, global.output)?;
        if let Some(rows) = value.as_array() {
            eprintln!("Fetched {} records", rows.len());
        }
        return Ok(());
    }
    add_pagination(&mut query, global);
    print_response(client.get(path, &query).await?, global.output, true)
}

async fn fetch_all(client: &ApiClient, path: &str, base_query: Query) -> Result<Value> {
    let mut all = Vec::new();
    let mut page = 1;
    loop {
        let mut query = base_query.clone();
        query.push(("page".into(), page.to_string()));
        query.push(("per_page".into(), "100".into()));
        let response = client.get(path, &query).await?;
        let Some(rows) = output::extract_array(&response.data) else {
            return Ok(response.data);
        };
        let count = rows.len();
        all.extend(rows.iter().cloned());
        if count < 100 {
            break;
        }
        page += 1;
    }
    Ok(Value::Array(all))
}

fn print_response(response: ApiResponse, format: OutputFormat, list: bool) -> Result<()> {
    let value = if list {
        output::extract_array(&response.data)
            .map(|rows| Value::Array(rows.clone()))
            .unwrap_or_else(|| response.data.clone())
    } else {
        response.data.clone()
    };
    output::print(&value, format)?;
    if list {
        output::print_pagination_meta(&response.data);
    }
    Ok(())
}

fn optional_vec<T: Serialize>(values: &[T]) -> Value {
    if values.is_empty() {
        Value::Null
    } else {
        json!(values)
    }
}

fn confirm_delete(resource: &str, yes: bool) -> Result<()> {
    if yes {
        return Ok(());
    }
    if !io::stdin().is_terminal() {
        return Err(CliError::Usage(format!(
            "Deleting {resource} requires --yes when input is not interactive"
        )));
    }
    let confirmed = Confirm::new()
        .with_prompt(format!("Delete {resource}? This cannot be undone"))
        .default(false)
        .interact()?;
    if confirmed {
        Ok(())
    } else {
        Err(CliError::Usage("Delete cancelled".into()))
    }
}

async fn run_dimensions(
    client: &ApiClient,
    global: &GlobalArgs,
    command: DimensionsCommand,
) -> Result<()> {
    match command {
        DimensionsCommand::Projects { command } => match command {
            ProjectsDimensionCommand::List => {
                list_get(client, global, "/dimensions/projects", Query::new()).await
            }
            ProjectsDimensionCommand::Get { id } => print_response(
                client
                    .get(&format!("/dimensions/projects/{id}"), &Vec::new())
                    .await?,
                global.output,
                false,
            ),
        },
        DimensionsCommand::Competitors { command } => match command {
            CompetitorsDimensionCommand::List(args) => {
                dimension_list(client, global, "/dimensions/competitors", args).await
            }
            CompetitorsDimensionCommand::Get { id } => print_response(
                client
                    .get(&format!("/dimensions/competitors/{id}"), &Vec::new())
                    .await?,
                global.output,
                false,
            ),
        },
        DimensionsCommand::Collections(args) => {
            dimension_list(client, global, "/dimensions/collections", args).await
        }
        DimensionsCommand::Tags(args) => {
            dimension_list(client, global, "/dimensions/tags", args).await
        }
        DimensionsCommand::Models(args) => {
            dimension_list(client, global, "/dimensions/models", args).await
        }
        DimensionsCommand::Locales(args) => {
            dimension_list(client, global, "/dimensions/locales", args).await
        }
        DimensionsCommand::Sentiments(args) => {
            dimension_list(client, global, "/dimensions/sentiments", args).await
        }
        DimensionsCommand::Prompts(args) => {
            dimension_list(client, global, "/dimensions/prompts", args).await
        }
        DimensionsCommand::Executions(args) => {
            dimension_list(client, global, "/dimensions/prompt_executions", args).await
        }
        DimensionsCommand::Sources(args) => {
            dimension_list(client, global, "/dimensions/sources", args).await
        }
        DimensionsCommand::Mentions(args) => {
            dimension_list(client, global, "/dimensions/mentions", args).await
        }
        DimensionsCommand::Citations(args) => {
            dimension_list(client, global, "/dimensions/citations", args).await
        }
        DimensionsCommand::CompetitorMentions(args) => {
            dimension_list(client, global, "/dimensions/competitor_mentions", args).await
        }
        DimensionsCommand::CompetitorCitations(args) => {
            dimension_list(client, global, "/dimensions/competitor_citations", args).await
        }
        DimensionsCommand::AllMentions(args) => {
            dimension_list(client, global, "/dimensions/all_mentions", args).await
        }
        DimensionsCommand::AllCitations(args) => {
            dimension_list(client, global, "/dimensions/all_citations", args).await
        }
        DimensionsCommand::AgentBots => print_response(
            client.get("/dimensions/agent_bots", &Vec::new()).await?,
            global.output,
            false,
        ),
    }
}

async fn dimension_list(
    client: &ApiClient,
    global: &GlobalArgs,
    path: &str,
    args: DimensionListArgs,
) -> Result<()> {
    let mut query = project_query(client)?;
    add_filters(&mut query, &args.filters, false);
    add_csv_values(&mut query, "competitors", &args.competitors);
    query.push_bool("include_project_brand", args.include_project_brand);
    list_get(client, global, path, query).await
}

async fn run_metrics(
    client: &ApiClient,
    global: &GlobalArgs,
    command: MetricsCommand,
) -> Result<()> {
    match command {
        MetricsCommand::Timeseries(args) => {
            metric_get(client, global, "/metrics/timeseries", args, false).await
        }
        MetricsCommand::Summary(args) => {
            metric_get(client, global, "/metrics/summary", args, false).await
        }
        MetricsCommand::PromptSummary(args) => {
            let mut query = metric_query(client, &args.metrics)?;
            query.push_opt("sort", args.sort);
            query.push_opt("sort_dir", Some(args.sort_dir));
            query.push_opt("breakdown", args.breakdown);
            list_get(client, global, "/metrics/prompt_summary", query).await
        }
        MetricsCommand::Sov(args) => metric_get(client, global, "/metrics/sov", args, false).await,
        MetricsCommand::TopSources(args) => {
            let mut query = metric_query(client, &args.metrics)?;
            query.push_opt("sort", args.sort);
            query.push_opt("query", args.query);
            list_get(client, global, "/metrics/top_sources", query).await
        }
        MetricsCommand::AgentTraffic(args) => {
            let mut query = project_query(client)?;
            add_dates(&mut query, &args.dates, true);
            query.push_opt("bot", args.bot);
            query.push_opt("company", args.company);
            query.push_opt("group_by", args.group_by);
            query.push_opt("granularity", args.granularity);
            print_response(
                client.get("/metrics/agent_traffic", &query).await?,
                global.output,
                false,
            )
        }
        MetricsCommand::AiTraffic(args) => {
            let mut query = project_query(client)?;
            add_dates(&mut query, &args.dates, true);
            query.push_opt("source", args.source);
            query.push_opt("granularity", args.granularity);
            print_response(
                client.get("/metrics/ai_traffic", &query).await?,
                global.output,
                false,
            )
        }
        MetricsCommand::AiModelSummary(args) => {
            ai_model_get(
                client,
                global,
                "/reports/ai_model_insights/summary",
                args,
                false,
            )
            .await
        }
        MetricsCommand::AiModelPositions(args) => {
            ai_model_get(
                client,
                global,
                "/reports/ai_model_insights/position_distribution",
                args,
                false,
            )
            .await
        }
        MetricsCommand::AiOverviewResults(args) => {
            ai_model_get(
                client,
                global,
                "/reports/ai_model_insights/ai_overview_results",
                args,
                true,
            )
            .await
        }
    }
}

fn metric_query(client: &ApiClient, args: &MetricArgs) -> Result<Query> {
    let mut query = project_query(client)?;
    add_filters(&mut query, &args.filters, true);
    add_csv_values(&mut query, "metrics", &args.metrics);
    query.push_opt("granularity", args.granularity.as_deref());
    add_csv_values(&mut query, "competitors", &args.competitors);
    query.push_bool("include_project", args.include_project);
    query.push_opt("group_by", args.group_by.as_deref());
    Ok(query)
}

async fn metric_get(
    client: &ApiClient,
    global: &GlobalArgs,
    path: &str,
    args: MetricArgs,
    list: bool,
) -> Result<()> {
    let query = metric_query(client, &args)?;
    if list {
        list_get(client, global, path, query).await
    } else {
        print_response(client.get(path, &query).await?, global.output, false)
    }
}

async fn ai_model_get(
    client: &ApiClient,
    global: &GlobalArgs,
    path: &str,
    args: AiModelArgs,
    list: bool,
) -> Result<()> {
    let mut query = project_query(client)?;
    add_filters(&mut query, &args.filters, true);
    query.push_opt("granularity", args.granularity);
    add_csv_values(&mut query, "competitors", &args.competitors);
    query.push_opt("brand1", args.brand1);
    query.push_opt("brand2", args.brand2);
    if list {
        list_get(client, global, path, query).await
    } else {
        print_response(client.get(path, &query).await?, global.output, false)
    }
}

async fn run_search_console(
    client: &ApiClient,
    global: &GlobalArgs,
    command: SearchConsoleCommand,
) -> Result<()> {
    let (path, args, list) = match command {
        SearchConsoleCommand::Summary(args) => ("/search_console/summary", args, false),
        SearchConsoleCommand::Timeseries(args) => ("/search_console/timeseries", args, false),
        SearchConsoleCommand::Queries(args) => ("/search_console/queries", args, true),
        SearchConsoleCommand::Pages(args) => ("/search_console/pages", args, true),
    };
    let mut query = project_query(client)?;
    add_dates(&mut query, &args.dates, true);
    query.push_opt("dimension", args.dimension);
    query.push_opt("granularity", args.granularity);
    query.push_opt("sort", args.sort);
    if list {
        list_get(client, global, path, query).await
    } else {
        print_response(client.get(path, &query).await?, global.output, false)
    }
}

fn citation_query(client: &ApiClient, args: &CitationFilterArgs) -> Result<Query> {
    let mut query = project_query(client)?;
    add_filters(&mut query, &args.filters, false);
    query.push_opt("view", args.view.as_deref());
    query.push_opt("query", args.query.as_deref());
    query.push_opt("source_type", args.source_type.as_deref());
    query.push_opt("sentiment", args.sentiment.as_deref());
    query.push_opt("content_gap", args.content_gap.as_deref());
    Ok(query)
}

async fn run_citations(
    client: &ApiClient,
    global: &GlobalArgs,
    command: CitationsCommand,
) -> Result<()> {
    match command {
        CitationsCommand::List(args) => {
            let mut query = citation_query(client, &args.filters)?;
            query.push_opt("order", args.order);
            query.push_opt("direction", Some(args.direction));
            list_get(client, global, "/citation_intelligence/groups", query).await
        }
        CitationsCommand::MentionsByDomain { domains, filters } => {
            let mut query = project_query(client)?;
            add_filters(&mut query, &filters, false);
            for domain in domains {
                query.push(("domains[]".into(), domain));
            }
            print_response(
                client
                    .get("/citation_intelligence/mentions_by_domain", &query)
                    .await?,
                global.output,
                false,
            )
        }
        CitationsCommand::Get {
            url_sha256,
            filters,
        } => {
            let query = citation_query(client, &filters)?;
            print_response(
                client
                    .get(&format!("/citation_intelligence/urls/{url_sha256}"), &query)
                    .await?,
                global.output,
                false,
            )
        }
        CitationsCommand::Occurrences {
            url_sha256,
            filters,
        } => {
            let query = citation_query(client, &filters)?;
            list_get(
                client,
                global,
                &format!("/citation_intelligence/urls/{url_sha256}/occurrences"),
                query,
            )
            .await
        }
        CitationsCommand::Content { url_sha256 } => {
            let query = project_query(client)?;
            print_response(
                client
                    .get(
                        &format!("/citation_intelligence/urls/{url_sha256}/content"),
                        &query,
                    )
                    .await?,
                global.output,
                false,
            )
        }
    }
}

async fn run_answers(
    client: &ApiClient,
    global: &GlobalArgs,
    command: AnswersCommand,
) -> Result<()> {
    match command {
        AnswersCommand::List(filters) => {
            let mut query = project_query(client)?;
            add_filters(&mut query, &filters, false);
            list_get(client, global, "/answers", query).await
        }
        AnswersCommand::Get {
            id,
            include_source_page_details,
        } => {
            let mut query = project_query(client)?;
            query.push_bool("include_source_page_details", include_source_page_details);
            print_response(
                client.get(&format!("/answers/{id}"), &query).await?,
                global.output,
                false,
            )
        }
    }
}

async fn run_sentiments(
    client: &ApiClient,
    global: &GlobalArgs,
    command: SentimentsCommand,
) -> Result<()> {
    let SentimentsCommand::List(args) = command;
    let mut query = project_query(client)?;
    add_filters(&mut query, &args.filters, false);
    query.push_opt("competitor_id", args.competitor_id);
    query.push_bool("brand_only", args.brand_only);
    query.push_opt("analysis", args.analysis);
    list_get(client, global, "/sentiments", query).await
}

async fn run_recommendations(
    client: &ApiClient,
    global: &GlobalArgs,
    command: RecommendationsCommand,
) -> Result<()> {
    match command {
        RecommendationsCommand::List {
            recommendation_type,
            status,
        } => {
            let mut query = project_query(client)?;
            query.push_opt("recommendation_type", recommendation_type);
            query.push_opt("status", status);
            list_get(client, global, "/recommendations", query).await
        }
        RecommendationsCommand::Get {
            id,
            item_status,
            resolve_source_refs,
        } => {
            let mut query = project_query(client)?;
            query.push_opt("item_status", item_status);
            query.push((
                "resolve_source_refs".into(),
                resolve_source_refs.to_string(),
            ));
            print_response(
                client
                    .get(&format!("/recommendations/{id}"), &query)
                    .await?,
                global.output,
                false,
            )
        }
        RecommendationsCommand::Launch {
            recommendation_type,
        } => {
            let body = json!({
                "project_id": client.project_id()?,
                "recommendation_type": recommendation_type,
            });
            print_response(
                client.post("/recommendations", body).await?,
                global.output,
                false,
            )
        }
    }
}

async fn run_intelligence_tasks(
    client: &ApiClient,
    global: &GlobalArgs,
    command: IntelligenceTasksCommand,
) -> Result<()> {
    match command {
        IntelligenceTasksCommand::List { task_type, status } => {
            let mut query = project_query(client)?;
            query.push_opt("task_type", task_type);
            query.push_opt("status", status);
            list_get(client, global, "/intelligence_tasks", query).await
        }
        IntelligenceTasksCommand::Get { id } => {
            let query = project_query(client)?;
            print_response(
                client
                    .get(&format!("/intelligence_tasks/{id}"), &query)
                    .await?,
                global.output,
                false,
            )
        }
        IntelligenceTasksCommand::Create(args) => {
            let body = compact_object(json!({
                "project_id": client.project_id()?,
                "task_type": args.task_type,
                "prompt_id": args.prompt_id,
                "custom_topic": args.custom_topic,
                "user_instructions": args.instructions,
                "output_language_code": args.output_language_code,
                "existing_content": args.existing_content,
                "existing_content_url": args.existing_content_url,
            }));
            print_response(
                client.post("/intelligence_tasks", body).await?,
                global.output,
                false,
            )
        }
    }
}

async fn run_prompts(
    client: &ApiClient,
    global: &GlobalArgs,
    command: PromptsCommand,
) -> Result<()> {
    match command {
        PromptsCommand::Create(args) => {
            let body = json!({
                "project_id": client.project_id()?,
                "prompts": args.prompts,
                "country_code": args.country_code,
                "language_code": args.language_code,
            });
            print_response(client.post("/prompts", body).await?, global.output, false)
        }
        PromptsCommand::AssignTags(args) => {
            let body = compact_object(json!({
                "project_id": client.project_id()?,
                "prompt_ids": args.prompt_ids,
                "tag_ids": optional_vec(&args.tag_ids),
                "tag_names": optional_vec(&args.tag_names),
                "create_missing": args.create_missing,
            }));
            print_response(
                client.post("/prompts/assign_tags", body).await?,
                global.output,
                false,
            )
        }
        PromptsCommand::Delete { id, yes } => {
            confirm_delete(&format!("prompt {id}"), yes)?;
            let query = project_query(client)?;
            print_response(
                client.delete(&format!("/prompts/{id}"), &query).await?,
                global.output,
                false,
            )
        }
    }
}

async fn run_competitors(
    client: &ApiClient,
    global: &GlobalArgs,
    command: CompetitorsCommand,
) -> Result<()> {
    match command {
        CompetitorsCommand::Create(args) => {
            let body = compact_object(json!({
                "project_id": client.project_id()?,
                "brand_name": args.brand_name,
                "domain": args.domain,
                "matching_names": optional_vec(&args.matching_names),
            }));
            print_response(
                client.post("/competitors", body).await?,
                global.output,
                false,
            )
        }
        CompetitorsCommand::Update(args) => {
            if args.brand_name.is_none() && args.matching_names.is_empty() && args.color.is_none() {
                return Err(CliError::Usage(
                    "Provide --brand-name, --matching-names, or --color".into(),
                ));
            }
            let body = compact_object(json!({
                "project_id": client.project_id()?,
                "brand_name": args.brand_name,
                "matching_names": optional_vec(&args.matching_names),
                "color": args.color,
            }));
            print_response(
                client
                    .patch(&format!("/competitors/{}", args.id), body)
                    .await?,
                global.output,
                false,
            )
        }
        CompetitorsCommand::Delete { id, yes } => {
            confirm_delete(&format!("competitor {id}"), yes)?;
            let query = project_query(client)?;
            print_response(
                client.delete(&format!("/competitors/{id}"), &query).await?,
                global.output,
                false,
            )
        }
    }
}

async fn run_projects(
    client: &ApiClient,
    global: &GlobalArgs,
    command: ProjectsCommand,
) -> Result<()> {
    match command {
        ProjectsCommand::Create(args) => {
            let body = compact_object(json!({
                "website_url": args.website_url,
                "name": args.name,
                "main_country": args.main_country,
                "main_language": args.main_language,
                "brand_name": args.brand_name,
                "description": args.description,
                "industry": optional_vec(&args.industry),
                "matching_names": optional_vec(&args.matching_names),
                "prompts": optional_vec(&args.prompts),
                "competitors": optional_vec(&args.competitors),
                "external_identifier": args.external_identifier,
                "execute_prompts_immediately": !args.defer_execution,
            }));
            print_response(client.post("/projects", body).await?, global.output, false)
        }
        ProjectsCommand::DraftStart(args) => {
            let body = json!({
                "website_url": args.website_url,
                "main_country": args.main_country,
                "main_language": args.main_language,
                "suggest": !args.no_suggest,
            });
            print_response(
                client.post("/project_drafts", body).await?,
                global.output,
                false,
            )
        }
        ProjectsCommand::DraftGet {
            id,
            include_suggestions,
        } => {
            let query = vec![(
                "include_suggestions".into(),
                include_suggestions.to_string(),
            )];
            print_response(
                client.get(&format!("/project_drafts/{id}"), &query).await?,
                global.output,
                false,
            )
        }
        ProjectsCommand::DraftStep { id, step, payload } => {
            let mut body: Value = serde_json::from_str(&payload).map_err(|error| {
                CliError::Usage(format!("--payload must be a JSON object: {error}"))
            })?;
            let object = body
                .as_object_mut()
                .ok_or_else(|| CliError::Usage("--payload must be a JSON object".into()))?;
            object.insert("step".into(), Value::String(step));
            print_response(
                client.patch(&format!("/project_drafts/{id}"), body).await?,
                global.output,
                false,
            )
        }
        ProjectsCommand::DraftFinalize {
            id,
            defer_execution,
        } => {
            let body = if defer_execution {
                json!({"execute_prompts_immediately": false})
            } else {
                json!({})
            };
            print_response(
                client
                    .post(&format!("/project_drafts/{id}/finalize"), body)
                    .await?,
                global.output,
                false,
            )
        }
    }
}

async fn run_collections(
    client: &ApiClient,
    global: &GlobalArgs,
    command: CollectionsCommand,
) -> Result<()> {
    match command {
        CollectionsCommand::Create(args) => {
            let body = compact_object(json!({
                "project_id": client.project_id()?,
                "name": args.name,
                "description": args.description,
                "prompt_ids": optional_vec(&args.prompt_ids),
            }));
            print_response(
                client.post("/collections", body).await?,
                global.output,
                false,
            )
        }
        CollectionsCommand::Update(args) => {
            if args.name.is_none() && args.description.is_none() {
                return Err(CliError::Usage("Provide --name or --description".into()));
            }
            let body = compact_object(json!({
                "project_id": client.project_id()?,
                "name": args.name,
                "description": args.description,
            }));
            print_response(
                client
                    .patch(&format!("/collections/{}", args.id), body)
                    .await?,
                global.output,
                false,
            )
        }
        CollectionsCommand::Delete { id, yes } => {
            confirm_delete(&format!("collection {id}"), yes)?;
            let query = project_query(client)?;
            print_response(
                client.delete(&format!("/collections/{id}"), &query).await?,
                global.output,
                false,
            )
        }
    }
}

async fn run_annotations(
    client: &ApiClient,
    global: &GlobalArgs,
    command: AnnotationsCommand,
) -> Result<()> {
    match command {
        AnnotationsCommand::List(args) => {
            let mut query = project_query(client)?;
            query.push_opt("from", args.from);
            query.push_opt("to", args.to);
            query.push_opt("annotation_category_id", args.category_id);
            list_get(client, global, "/annotations", query).await
        }
        AnnotationsCommand::Create(args) => {
            let body = compact_object(json!({
                "project_id": client.project_id()?,
                "title": args.title,
                "description": args.description,
                "color": args.color,
                "annotation_date": args.date,
                "annotation_category_id": args.category_id,
            }));
            print_response(
                client.post("/annotations", body).await?,
                global.output,
                false,
            )
        }
        AnnotationsCommand::Update(args) => {
            if args.title.is_none()
                && args.description.is_none()
                && args.color.is_none()
                && args.date.is_none()
                && args.category_id.is_none()
            {
                return Err(CliError::Usage(
                    "Provide at least one annotation field to update".into(),
                ));
            }
            let body = compact_object(json!({
                "project_id": client.project_id()?,
                "title": args.title,
                "description": args.description,
                "color": args.color,
                "annotation_date": args.date,
                "annotation_category_id": args.category_id,
            }));
            print_response(
                client
                    .patch(&format!("/annotations/{}", args.id), body)
                    .await?,
                global.output,
                false,
            )
        }
        AnnotationsCommand::Delete { id, yes } => {
            confirm_delete(&format!("annotation {id}"), yes)?;
            let query = project_query(client)?;
            print_response(
                client.delete(&format!("/annotations/{id}"), &query).await?,
                global.output,
                false,
            )
        }
    }
}

async fn run_reports(
    client: &ApiClient,
    global: &GlobalArgs,
    command: ReportsCommand,
) -> Result<()> {
    match command {
        ReportsCommand::TechnicalGeo { url, country_code } => {
            let body = compact_object(json!({
                "project_id": client.project_id()?,
                "url": url,
                "country_code": country_code,
            }));
            print_response(
                client.post("/technical_geo_reports", body).await?,
                global.output,
                false,
            )
        }
    }
}

async fn run_webhooks(
    client: &ApiClient,
    global: &GlobalArgs,
    command: WebhooksCommand,
) -> Result<()> {
    match command {
        WebhooksCommand::List => {
            let mut query = Query::new();
            query.push_opt("project_id", global.project_id);
            list_get(client, global, "/webhooks", query).await
        }
        WebhooksCommand::Create {
            event_type,
            target_url,
        } => {
            let body = json!({
                "project_id": client.project_id()?,
                "event_type": event_type,
                "target_url": target_url,
            });
            print_response(client.post("/webhooks", body).await?, global.output, false)
        }
        WebhooksCommand::Delete { id, yes } => {
            confirm_delete(&format!("webhook {id}"), yes)?;
            print_response(
                client
                    .delete(&format!("/webhooks/{id}"), &Vec::new())
                    .await?,
                global.output,
                false,
            )
        }
        WebhooksCommand::Sample { event_type } => {
            let query = project_query(client)?;
            print_response(
                client
                    .get(&format!("/webhooks/sample/{event_type}"), &query)
                    .await?,
                global.output,
                false,
            )
        }
    }
}

async fn run_status(client: &ApiClient, global: &GlobalArgs, range: u32) -> Result<()> {
    let mut query = project_query(client)?;
    query.push(("range".into(), range.to_string()));
    let (summary, sov, timeseries, prompts) = tokio::try_join!(
        client.get("/metrics/summary", &query),
        client.get("/metrics/sov", &query),
        client.get("/metrics/timeseries", &query),
        client.get("/metrics/prompt_summary", &query),
    )?;
    let value = json!({
        "range_days": range,
        "summary": summary.data,
        "share_of_voice": sov.data,
        "timeseries": timeseries.data,
        "prompts": prompts.data,
    });
    output::print(&value, global.output)
}

async fn run_diff(client: &ApiClient, global: &GlobalArgs, args: DiffArgs) -> Result<()> {
    let current_to = Utc::now().date_naive();
    let current_from = current_to - ChronoDuration::days(i64::from(args.range.saturating_sub(1)));
    let previous_to = current_from - ChronoDuration::days(1);
    let previous_from =
        previous_to - ChronoDuration::days(i64::from(args.compare.saturating_sub(1)));

    let mut current_query = project_query(client)?;
    current_query.push(("from".into(), current_from.to_string()));
    current_query.push(("to".into(), current_to.to_string()));
    let mut previous_query = project_query(client)?;
    previous_query.push(("from".into(), previous_from.to_string()));
    previous_query.push(("to".into(), previous_to.to_string()));

    let (current, previous) = tokio::try_join!(
        client.get("/metrics/summary", &current_query),
        client.get("/metrics/summary", &previous_query),
    )?;
    let change = numeric_difference(&current.data, &previous.data);
    let value = json!({
        "current": {
            "from": current_from,
            "to": current_to,
            "data": current.data,
        },
        "previous": {
            "from": previous_from,
            "to": previous_to,
            "data": previous.data,
        },
        "change": change,
    });
    output::print(&value, global.output)
}

fn numeric_difference(current: &Value, previous: &Value) -> Value {
    match (current, previous) {
        (Value::Number(current), Value::Number(previous)) => {
            match (current.as_f64(), previous.as_f64()) {
                (Some(current), Some(previous)) => json!(current - previous),
                _ => Value::Null,
            }
        }
        (Value::Object(current), Value::Object(previous)) => Value::Object(
            current
                .iter()
                .filter_map(|(key, value)| {
                    previous
                        .get(key)
                        .map(|other| (key.clone(), numeric_difference(value, other)))
                })
                .filter(|(_, value)| !value.is_null())
                .collect(),
        ),
        _ => Value::Null,
    }
}

async fn run_report(client: &ApiClient, global: &GlobalArgs, args: ReportArgs) -> Result<()> {
    let mut query = project_query(client)?;
    query.push(("range".into(), args.range.to_string()));
    query.push(("per_page".into(), "10".into()));
    let (summary, sov, sources, prompts) = tokio::try_join!(
        client.get("/metrics/summary", &query),
        client.get("/metrics/sov", &query),
        client.get("/metrics/top_sources", &query),
        client.get("/metrics/prompt_summary", &query),
    )?;
    let value = json!({
        "range_days": args.range,
        "summary": summary.data,
        "share_of_voice": sov.data,
        "top_sources": sources.data,
        "top_prompts": prompts.data,
    });
    if args.markdown {
        println!("# LLM Pulse report\n");
        println!("Period: last {} days\n", args.range);
        for (heading, field) in [
            ("Summary", "summary"),
            ("Share of voice", "share_of_voice"),
            ("Top sources", "top_sources"),
            ("Top prompts", "top_prompts"),
        ] {
            println!("## {heading}\n");
            println!("```json");
            println!("{}", serde_json::to_string_pretty(&value[field])?);
            println!("```\n");
        }
        Ok(())
    } else {
        output::print(&value, global.output)
    }
}

async fn run_watch(client: &ApiClient, global: &GlobalArgs, args: WatchArgs) -> Result<()> {
    loop {
        if io::stdout().is_terminal() {
            print!("\x1b[2J\x1b[H");
        }
        run_status(client, global, args.range).await?;
        eprintln!(
            "Refreshing every {} seconds. Press Ctrl+C to stop.",
            args.interval
        );
        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(()),
            _ = tokio::time::sleep(Duration::from_secs(args.interval)) => {}
        }
    }
}

async fn run_export(client: &ApiClient, _global: &GlobalArgs, args: ExportArgs) -> Result<()> {
    let directory = PathBuf::from(&args.directory);
    fs::create_dir_all(&directory)?;
    let mut base_query = project_query(client)?;
    base_query.push_opt("from", args.from);
    base_query.push_opt("to", args.to);
    let resources = [
        ("prompts", "/dimensions/prompts"),
        ("prompt_executions", "/dimensions/prompt_executions"),
        ("mentions", "/dimensions/mentions"),
        ("citations", "/dimensions/citations"),
        ("competitor_mentions", "/dimensions/competitor_mentions"),
        ("competitor_citations", "/dimensions/competitor_citations"),
        ("answers", "/answers"),
        ("sentiments", "/sentiments"),
    ];
    for (name, path) in resources {
        let value = fetch_all(client, path, base_query.clone()).await?;
        fs::write(
            directory.join(format!("{name}.json")),
            serde_json::to_string_pretty(&value)? + "\n",
        )?;
        fs::write(
            directory.join(format!("{name}.csv")),
            output::format_value(&value, OutputFormat::Csv)?,
        )?;
        eprintln!("Exported {name}");
    }
    println!("Export complete: {}", directory.display());
    Ok(())
}

async fn run_raw_api(client: &ApiClient, global: &GlobalArgs, args: ApiArgs) -> Result<()> {
    let path = normalize_api_path(&args.path)?;
    let query = args
        .query
        .iter()
        .map(|pair| {
            pair.split_once('=')
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .ok_or_else(|| CliError::Usage(format!("Invalid query '{pair}'. Use KEY=VALUE")))
        })
        .collect::<Result<Query>>()?;
    let body = args
        .body
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(|error| CliError::Usage(format!("--body must be valid JSON: {error}")))?
        .unwrap_or_else(|| json!({}));
    let response = match args.method {
        HttpMethod::Get => client.get(&path, &query).await?,
        HttpMethod::Post => client.post(&path, body).await?,
        HttpMethod::Patch => client.patch(&path, body).await?,
        HttpMethod::Delete => client.delete(&path, &query).await?,
    };
    print_response(response, global.output, false)
}

fn normalize_api_path(path: &str) -> Result<String> {
    if path.contains("://")
        || path
            .chars()
            .any(|character| matches!(character, '\r' | '\n'))
    {
        return Err(CliError::Usage(
            "The API path must be relative to the configured base URL".into(),
        ));
    }
    Ok(if path.starts_with('/') {
        path.to_owned()
    } else {
        format!("/{path}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_api_only_accepts_relative_paths() {
        assert_eq!(
            normalize_api_path("metrics/summary").unwrap(),
            "/metrics/summary"
        );
        assert!(normalize_api_path("https://evil.example").is_err());
    }

    #[test]
    fn computes_nested_numeric_differences() {
        let current = json!({"summary": {"mentions": 10, "label": "now"}});
        let previous = json!({"summary": {"mentions": 4, "label": "before"}});
        assert_eq!(
            numeric_difference(&current, &previous),
            json!({"summary": {"mentions": 6.0}})
        );
    }
}
