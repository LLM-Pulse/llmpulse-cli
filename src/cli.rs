use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::output::OutputFormat;

#[derive(Debug, Parser)]
#[command(
    name = "llmpulse",
    version,
    about = "Command-line client for LLM Pulse AI visibility analytics",
    propagate_version = true
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Args)]
pub struct GlobalArgs {
    #[arg(short = 'k', long, global = true, hide_env_values = true)]
    pub api_key: Option<String>,
    #[arg(long, global = true)]
    pub base_url: Option<String>,
    #[arg(short = 'p', long, global = true)]
    pub project_id: Option<u64>,
    #[arg(
        short = 'f',
        long = "format",
        global = true,
        value_enum,
        default_value_t
    )]
    pub output: OutputFormat,
    #[arg(long, global = true)]
    pub page: Option<u32>,
    #[arg(long, global = true, value_parser = clap::value_parser!(u32).range(1..=100))]
    pub per_page: Option<u32>,
    #[arg(long, global = true)]
    pub all: bool,
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Configure an API key and default project interactively.
    Login,
    /// Show a compact project dashboard.
    Status(StatusArgs),
    /// Read or change local CLI configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Check API connectivity and authentication.
    Ping,
    /// Query projects, prompts, mentions, citations, and other dimensions.
    Dimensions {
        #[command(subcommand)]
        command: DimensionsCommand,
    },
    /// Query visibility, share of voice, traffic, and model metrics.
    Metrics {
        #[command(subcommand)]
        command: MetricsCommand,
    },
    /// Query Google Search Console data.
    SearchConsole {
        #[command(subcommand)]
        command: SearchConsoleCommand,
    },
    /// Query citation intelligence and cited-page evidence.
    Citations {
        #[command(subcommand)]
        command: CitationsCommand,
    },
    /// Query AI model responses.
    Answers {
        #[command(subcommand)]
        command: AnswersCommand,
    },
    /// Query sentiment records.
    Sentiments {
        #[command(subcommand)]
        command: SentimentsCommand,
    },
    /// List, inspect, and launch recommendation runs.
    Recommendations {
        #[command(subcommand)]
        command: RecommendationsCommand,
    },
    /// List, inspect, create, edit, and revert GEO Writer tasks.
    IntelligenceTasks {
        #[command(subcommand)]
        command: IntelligenceTasksCommand,
    },
    /// Create and delete prompts or assign tags.
    Prompts {
        #[command(subcommand)]
        command: PromptsCommand,
    },
    /// Create, update, and delete competitors.
    Competitors {
        #[command(subcommand)]
        command: CompetitorsCommand,
    },
    /// Create a project directly or through a draft.
    Projects {
        #[command(subcommand)]
        command: ProjectsCommand,
    },
    /// Create, update, and delete collections.
    Collections {
        #[command(subcommand)]
        command: CollectionsCommand,
    },
    /// List, create, update, and delete timeline annotations.
    Annotations {
        #[command(subcommand)]
        command: AnnotationsCommand,
    },
    /// Queue, list, and inspect technical GEO reports, and edit or revert llms.txt report files.
    Reports {
        #[command(subcommand)]
        command: ReportsCommand,
    },
    /// Manage webhook subscriptions and sample payloads.
    Webhooks {
        #[command(subcommand)]
        command: WebhooksCommand,
    },
    /// Show plan, quota usage, and rate limits for the account.
    Account,
    /// List and inspect reputation reports.
    Reputation {
        #[command(subcommand)]
        command: ReputationCommand,
    },
    /// List and inspect custom AI studies.
    Studies {
        #[command(subcommand)]
        command: StudiesCommand,
    },
    /// Compare two adjacent date ranges.
    Diff(DiffArgs),
    /// Generate a visibility report.
    Report(ReportArgs),
    /// Refresh the project dashboard on an interval.
    Watch(WatchArgs),
    /// Export project data to JSON and CSV files.
    Export(ExportArgs),
    /// Generate a shell completion script.
    Completions(CompletionsArgs),
    /// Call a REST endpoint directly.
    Api(ApiArgs),
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    Set { key: String, value: String },
    Get { key: String },
    Show,
    Use { profile: String },
    Profiles,
}

#[derive(Clone, Debug, Default, Args)]
pub struct DateArgs {
    #[arg(long)]
    pub range: Option<u32>,
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
}

#[derive(Clone, Debug, Default, Args)]
pub struct FilterArgs {
    #[arg(long)]
    pub model: Option<String>,
    /// One collection/tag ID, or a comma-separated list.
    #[arg(long)]
    pub collection_id: Option<String>,
    #[arg(long)]
    pub country_code: Option<String>,
    #[arg(long)]
    pub language_code: Option<String>,
    #[arg(long)]
    pub prompt: Option<u64>,
    #[arg(long)]
    pub prompt_type: Option<String>,
    #[arg(long)]
    pub brand_kind: Option<String>,
    #[command(flatten)]
    pub dates: DateArgs,
}

#[derive(Clone, Debug, Default, Args)]
pub struct DimensionListArgs {
    #[command(flatten)]
    pub filters: FilterArgs,
    #[arg(long, value_delimiter = ',')]
    pub competitors: Vec<u64>,
    #[arg(long)]
    pub include_project_brand: bool,
    /// Which slice of the resource to list (shopping, ads, owned media, Reddit).
    #[arg(long)]
    pub view: Option<String>,
    #[arg(long)]
    pub order: Option<String>,
    #[arg(long)]
    pub direction: Option<String>,
    #[arg(long)]
    pub query: Option<String>,
    /// Restrict to your own properties (shopping, ads, owned media, Reddit).
    #[arg(long)]
    pub owned: bool,
    /// Owned media provider: youtube, instagram, facebook, tiktok, linkedin, mobile_apps.
    #[arg(long)]
    pub provider: Option<String>,
    /// Mobile app store: google_play or app_store.
    #[arg(long)]
    pub store: Option<String>,
    #[arg(long)]
    pub subreddit: Option<String>,
    #[arg(long)]
    pub author: Option<String>,
    /// Reddit thread status: open or archived.
    #[arg(long)]
    pub status: Option<String>,
    /// Brand name, as Reddit reports it.
    #[arg(long)]
    pub brand: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum DimensionsCommand {
    Projects {
        #[command(subcommand)]
        command: ProjectsDimensionCommand,
    },
    Competitors {
        #[command(subcommand)]
        command: CompetitorsDimensionCommand,
    },
    Collections(DimensionListArgs),
    Tags(DimensionListArgs),
    Models(DimensionListArgs),
    Locales(DimensionListArgs),
    Sentiments(DimensionListArgs),
    Prompts(DimensionListArgs),
    Executions(DimensionListArgs),
    Sources(DimensionListArgs),
    Mentions(DimensionListArgs),
    Citations(DimensionListArgs),
    CompetitorMentions(DimensionListArgs),
    CompetitorCitations(DimensionListArgs),
    AllMentions(DimensionListArgs),
    AllCitations(DimensionListArgs),
    AgentBots,
    Shopping(DimensionListArgs),
    Ads(DimensionListArgs),
    LocalBusinesses(DimensionListArgs),
    OwnedMedia(DimensionListArgs),
    Reddit(DimensionListArgs),
    QueryFanOuts(DimensionListArgs),
}

#[derive(Debug, Subcommand)]
pub enum ReputationCommand {
    List(Box<DimensionListArgs>),
    Get { id: u64 },
}

#[derive(Debug, Subcommand)]
pub enum StudiesCommand {
    List(Box<DimensionListArgs>),
    Get {
        id: u64,
    },
    Report {
        id: u64,
        report_id: u64,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        subject: Option<String>,
        #[arg(long)]
        dimension: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ProjectsDimensionCommand {
    List,
    Get { id: u64 },
}

#[derive(Debug, Subcommand)]
pub enum CompetitorsDimensionCommand {
    List(Box<DimensionListArgs>),
    Get { id: u64 },
}

#[derive(Clone, Debug, Default, Args)]
pub struct MetricArgs {
    #[command(flatten)]
    pub filters: FilterArgs,
    #[arg(long, value_delimiter = ',')]
    pub metrics: Vec<String>,
    #[arg(long)]
    pub granularity: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub competitors: Vec<u64>,
    #[arg(long)]
    pub include_project: bool,
    #[arg(long)]
    pub group_by: Option<String>,
}

#[derive(Clone, Debug, Args)]
pub struct PromptSummaryArgs {
    #[command(flatten)]
    pub metrics: MetricArgs,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long, default_value = "desc")]
    pub sort_dir: String,
    #[arg(long)]
    pub breakdown: Option<String>,
}

#[derive(Clone, Debug, Args)]
pub struct TopSourcesArgs {
    #[command(flatten)]
    pub metrics: MetricArgs,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub query: Option<String>,
}

#[derive(Clone, Debug, Args)]
pub struct AgentTrafficArgs {
    #[command(flatten)]
    pub dates: DateArgs,
    #[arg(long)]
    pub bot: Option<String>,
    #[arg(long)]
    pub company: Option<String>,
    #[arg(long)]
    pub group_by: Option<String>,
    #[arg(long)]
    pub granularity: Option<String>,
}

#[derive(Clone, Debug, Args)]
pub struct AiTrafficArgs {
    #[command(flatten)]
    pub dates: DateArgs,
    #[arg(long)]
    pub source: Option<String>,
    #[arg(long)]
    pub granularity: Option<String>,
}

#[derive(Clone, Debug, Args)]
pub struct AiModelArgs {
    #[command(flatten)]
    pub filters: FilterArgs,
    #[arg(long)]
    pub granularity: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub competitors: Vec<u64>,
    #[arg(long)]
    pub brand1: Option<u64>,
    #[arg(long)]
    pub brand2: Option<u64>,
}

#[derive(Debug, Subcommand)]
pub enum MetricsCommand {
    Timeseries(MetricArgs),
    Summary(MetricArgs),
    PromptSummary(PromptSummaryArgs),
    Sov(MetricArgs),
    TopSources(TopSourcesArgs),
    AgentTraffic(AgentTrafficArgs),
    AiTraffic(AiTrafficArgs),
    AiModelSummary(AiModelArgs),
    AiModelPositions(AiModelArgs),
    AiOverviewResults(AiModelArgs),
}

#[derive(Clone, Debug, Default, Args)]
pub struct SearchConsoleArgs {
    #[command(flatten)]
    pub dates: DateArgs,
    #[arg(long)]
    pub dimension: Option<String>,
    #[arg(long)]
    pub granularity: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    /// Search surface: web (default), image, video, news, discover, googleNews.
    #[arg(long)]
    pub search_type: Option<String>,
    /// final (default) counts only finalized rows; all includes the most recent days.
    #[arg(long)]
    pub data_state: Option<String>,
    /// JSON array of {dimension, operator, expression}, combined with AND.
    #[arg(long)]
    pub filters: Option<String>,
    /// Maximum breakdown rows on summary.
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Debug, Subcommand)]
pub enum SearchConsoleCommand {
    Summary(SearchConsoleArgs),
    Timeseries(SearchConsoleArgs),
    Queries(SearchConsoleArgs),
    Pages(SearchConsoleArgs),
}

#[derive(Clone, Debug, Default, Args)]
pub struct CitationFilterArgs {
    #[command(flatten)]
    pub filters: FilterArgs,
    #[arg(long)]
    pub view: Option<String>,
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long)]
    pub source_type: Option<String>,
    #[arg(long)]
    pub sentiment: Option<String>,
    #[arg(long)]
    pub content_gap: Option<String>,
}

#[derive(Debug, Args)]
pub struct CitationListArgs {
    #[command(flatten)]
    pub filters: CitationFilterArgs,
    #[arg(long)]
    pub order: Option<String>,
    #[arg(long, default_value = "desc")]
    pub direction: String,
}

#[derive(Debug, Subcommand)]
pub enum CitationsCommand {
    List(CitationListArgs),
    MentionsByDomain {
        #[arg(required = true, value_delimiter = ',')]
        domains: Vec<String>,
        #[command(flatten)]
        filters: FilterArgs,
    },
    Get {
        url_sha256: String,
        #[command(flatten)]
        filters: CitationFilterArgs,
    },
    Occurrences {
        url_sha256: String,
        #[command(flatten)]
        filters: CitationFilterArgs,
    },
    Content {
        url_sha256: String,
    },
}

#[derive(Debug, Args)]
pub struct AnswerListArgs {
    #[command(flatten)]
    pub filters: FilterArgs,
    #[arg(long, value_delimiter = ',')]
    pub competitors: Vec<u64>,
    #[arg(long)]
    pub query: Option<String>,
    /// mentions_you, not_mentions_you, mentions_competitor, you_and_competitor, no_brands, and so on.
    #[arg(long)]
    pub mention_filter: Option<String>,
    /// cites_you, not_cites_you, cites_competitor, you_and_competitor, cites_no_brands, and so on.
    #[arg(long)]
    pub citation_filter: Option<String>,
    /// Only answers where the model returned no result.
    #[arg(long)]
    pub no_result: bool,
}

#[derive(Debug, Subcommand)]
pub enum AnswersCommand {
    List(Box<AnswerListArgs>),
    Get {
        id: String,
        #[arg(long)]
        include_source_page_details: bool,
    },
}

#[derive(Debug, Args)]
pub struct SentimentListArgs {
    #[command(flatten)]
    pub filters: FilterArgs,
    #[arg(long)]
    pub competitor_id: Option<u64>,
    #[arg(long)]
    pub brand_only: bool,
    #[arg(long)]
    pub analysis: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum SentimentsCommand {
    List(SentimentListArgs),
}

#[derive(Debug, Subcommand)]
pub enum RecommendationsCommand {
    List {
        #[arg(long)]
        recommendation_type: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },
    Get {
        id: u64,
        #[arg(long)]
        item_status: Option<String>,
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        resolve_source_refs: bool,
    },
    Launch {
        #[arg(long, default_value = "ai_visibility")]
        recommendation_type: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum IntelligenceTasksCommand {
    List {
        #[arg(long)]
        task_type: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },
    Get {
        id: String,
    },
    Create(IntelligenceTaskCreateArgs),
    Update {
        id: String,
        #[arg(long)]
        edits: String,
    },
    Revert {
        id: String,
    },
}

#[derive(Debug, Args)]
pub struct IntelligenceTaskCreateArgs {
    #[arg(long)]
    pub task_type: String,
    #[arg(long)]
    pub prompt_id: Option<u64>,
    #[arg(long)]
    pub custom_topic: Option<String>,
    #[arg(long)]
    pub instructions: Option<String>,
    #[arg(long)]
    pub output_language_code: Option<String>,
    #[arg(long)]
    pub existing_content: Option<String>,
    #[arg(long)]
    pub existing_content_url: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum PromptsCommand {
    Create(PromptCreateArgs),
    AssignTags(PromptAssignTagsArgs),
    Delete {
        id: u64,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Args)]
pub struct PromptCreateArgs {
    #[arg(required = true)]
    pub prompts: Vec<String>,
    #[arg(long)]
    pub country_code: String,
    #[arg(long)]
    pub language_code: String,
}

#[derive(Debug, Args)]
pub struct PromptAssignTagsArgs {
    #[arg(long, required = true, value_delimiter = ',')]
    pub prompt_ids: Vec<u64>,
    #[arg(long, value_delimiter = ',')]
    pub tag_ids: Vec<u64>,
    #[arg(long, value_delimiter = ',')]
    pub tag_names: Vec<String>,
    #[arg(long)]
    pub create_missing: bool,
}

#[derive(Debug, Subcommand)]
pub enum CompetitorsCommand {
    Create(CompetitorCreateArgs),
    Update(CompetitorUpdateArgs),
    Delete {
        id: u64,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Args)]
pub struct CompetitorCreateArgs {
    #[arg(long)]
    pub brand_name: String,
    #[arg(long)]
    pub domain: String,
    #[arg(long, value_delimiter = ',')]
    pub matching_names: Vec<String>,
}

#[derive(Debug, Args)]
pub struct CompetitorUpdateArgs {
    pub id: u64,
    #[arg(long)]
    pub brand_name: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub matching_names: Vec<String>,
    #[arg(long)]
    pub color: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum ProjectsCommand {
    Create(ProjectCreateArgs),
    Update(ProjectUpdateArgs),
    DraftStart(ProjectDraftStartArgs),
    DraftGet {
        id: String,
        #[arg(long)]
        include_suggestions: bool,
    },
    DraftStep {
        id: String,
        #[arg(long)]
        step: String,
        #[arg(long)]
        payload: String,
    },
    DraftFinalize {
        id: String,
        #[arg(long)]
        defer_execution: bool,
    },
}

#[derive(Debug, Args)]
pub struct ProjectCreateArgs {
    #[arg(long)]
    pub website_url: String,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub main_country: String,
    #[arg(long)]
    pub main_language: String,
    #[arg(long)]
    pub brand_name: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub industry: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub matching_names: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub prompts: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub competitors: Vec<String>,
    #[arg(long)]
    pub business_model: Option<String>,
    #[arg(long)]
    pub target_audience: Option<String>,
    #[arg(long)]
    pub brand_voice: Option<String>,
    #[arg(long)]
    pub goals: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub primary_products: Vec<String>,
    #[arg(long)]
    pub external_identifier: Option<String>,
    #[arg(long)]
    pub defer_execution: bool,
}

#[derive(Debug, Args)]
pub struct ProjectUpdateArgs {
    pub id: u64,
    #[arg(long)]
    pub brand_name: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub industry: Option<String>,
    #[arg(long)]
    pub business_model: Option<String>,
    #[arg(long)]
    pub target_audience: Option<String>,
    #[arg(long)]
    pub brand_voice: Option<String>,
    #[arg(long)]
    pub goals: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub primary_products: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub matching_names: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ProjectDraftStartArgs {
    #[arg(long)]
    pub website_url: String,
    #[arg(long)]
    pub main_country: String,
    #[arg(long)]
    pub main_language: String,
    #[arg(long)]
    pub no_suggest: bool,
}

#[derive(Debug, Subcommand)]
pub enum CollectionsCommand {
    Create(CollectionCreateArgs),
    Update(CollectionUpdateArgs),
    Delete {
        id: u64,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Args)]
pub struct CollectionCreateArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub prompt_ids: Vec<u64>,
}

#[derive(Debug, Args)]
pub struct CollectionUpdateArgs {
    pub id: u64,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum AnnotationsCommand {
    List(AnnotationListArgs),
    Create(AnnotationWriteArgs),
    Update(AnnotationUpdateArgs),
    Delete {
        id: u64,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Args)]
pub struct AnnotationListArgs {
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
    #[arg(long)]
    pub category_id: Option<u64>,
}

#[derive(Debug, Args)]
pub struct AnnotationWriteArgs {
    #[arg(long)]
    pub title: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub color: Option<String>,
    #[arg(long)]
    pub date: Option<String>,
    #[arg(long)]
    pub category_id: Option<u64>,
}

#[derive(Debug, Args)]
pub struct AnnotationUpdateArgs {
    pub id: u64,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub color: Option<String>,
    #[arg(long)]
    pub date: Option<String>,
    #[arg(long)]
    pub category_id: Option<u64>,
}

#[derive(Debug, Subcommand)]
pub enum ReportsCommand {
    TechnicalGeo {
        #[arg(long)]
        url: String,
        #[arg(long)]
        country_code: Option<String>,
        #[arg(long)]
        output_language_code: Option<String>,
    },
    TechnicalGeoList {
        #[arg(long)]
        report_type: String,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        batch_id: Option<u64>,
    },
    TechnicalGeoGet {
        id: u64,
        #[arg(long)]
        report_type: String,
    },
    TechnicalGeoUpdateContent {
        id: u64,
        #[arg(long)]
        report_type: String,
        /// result_data.content_version of the report as last read.
        #[arg(long)]
        content_version: String,
        /// JSON object mapping llms_txt and/or llms_full_txt to the full replacement text.
        #[arg(long)]
        edits: String,
    },
    TechnicalGeoRevertContent {
        id: u64,
        #[arg(long)]
        report_type: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum WebhooksCommand {
    List,
    Create {
        #[arg(long)]
        event_type: String,
        #[arg(long)]
        target_url: String,
    },
    Delete {
        id: u64,
        #[arg(long)]
        yes: bool,
    },
    Sample {
        event_type: String,
    },
}

#[derive(Debug, Args)]
pub struct StatusArgs {
    #[arg(long, default_value_t = 7)]
    pub range: u32,
}

#[derive(Debug, Args)]
pub struct DiffArgs {
    #[arg(long, default_value_t = 7)]
    pub range: u32,
    #[arg(long, default_value_t = 7)]
    pub compare: u32,
}

#[derive(Debug, Args)]
pub struct ReportArgs {
    #[arg(long, default_value_t = 7)]
    pub range: u32,
    #[arg(long)]
    pub markdown: bool,
}

#[derive(Debug, Args)]
pub struct WatchArgs {
    #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(5..))]
    pub interval: u64,
    #[arg(long, default_value_t = 7)]
    pub range: u32,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    #[arg(long, default_value = "llmpulse-export")]
    pub directory: String,
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
}

#[derive(Debug, Args)]
pub struct CompletionsArgs {
    #[arg(value_enum)]
    pub shell: Shell,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum HttpMethod {
    Get,
    Post,
    Patch,
    Delete,
}

#[derive(Debug, Args)]
pub struct ApiArgs {
    #[arg(value_enum)]
    pub method: HttpMethod,
    pub path: String,
    #[arg(short = 'q', long = "query", value_name = "KEY=VALUE")]
    pub query: Vec<String>,
    #[arg(long, help = "JSON request body")]
    pub body: Option<String>,
}
