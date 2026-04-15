use sacp::{JsonRpcRequest, JsonRpcResponse};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema descriptor for a single custom method, produced by the
/// `#[custom_methods]` macro's generated `custom_method_schemas()` function.
///
/// `params_schema` / `response_schema` hold `$ref` pointers or inline schemas
/// produced by `SchemaGenerator::subschema_for`. All referenced types are
/// collected in the generator's `$defs` map.
///
/// `params_type_name` / `response_type_name` carry the Rust struct name so the
/// binary can key `$defs` entries and annotate them with `x-method` / `x-side`.
#[derive(Debug, Serialize)]
pub struct CustomMethodSchema {
    pub method: String,
    pub params_schema: Option<schemars::Schema>,
    pub params_type_name: Option<String>,
    pub response_schema: Option<schemars::Schema>,
    pub response_type_name: Option<String>,
}

/// Add an extension to an active session.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/extensions/add", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct AddExtensionRequest {
    pub session_id: String,
    /// Extension configuration (see ExtensionConfig variants: Stdio, StreamableHttp, Builtin, Platform).
    #[serde(default)]
    pub config: serde_json::Value,
}

/// Remove an extension from an active session.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/extensions/remove", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct RemoveExtensionRequest {
    pub session_id: String,
    pub name: String,
}

/// List all tools available in a session.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/tools", response = GetToolsResponse)]
#[serde(rename_all = "camelCase")]
pub struct GetToolsRequest {
    pub session_id: String,
}

/// Tools response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct GetToolsResponse {
    /// Array of tool info objects with `name`, `description`, `parameters`, and optional `permission`.
    pub tools: Vec<serde_json::Value>,
}

/// Read a resource from an extension.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/resource/read", response = ReadResourceResponse)]
#[serde(rename_all = "camelCase")]
pub struct ReadResourceRequest {
    pub session_id: String,
    pub uri: String,
    pub extension_name: String,
}

/// Resource read response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct ReadResourceResponse {
    /// The resource result from the extension (MCP ReadResourceResult).
    #[serde(default)]
    pub result: serde_json::Value,
}

/// Update the working directory for a session.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/working_dir/update", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkingDirRequest {
    pub session_id: String,
    pub working_dir: String,
}

/// Delete a session.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "session/delete", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSessionRequest {
    pub session_id: String,
}

/// List configured extensions and any warnings.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/extensions", response = GetExtensionsResponse)]
pub struct GetExtensionsRequest {}

/// List configured extensions and any warnings.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct GetExtensionsResponse {
    /// Array of ExtensionEntry objects with `enabled` flag and config details.
    pub extensions: Vec<serde_json::Value>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/session/extensions", response = GetSessionExtensionsResponse)]
#[serde(rename_all = "camelCase")]
pub struct GetSessionExtensionsRequest {
    pub session_id: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct GetSessionExtensionsResponse {
    pub extensions: Vec<serde_json::Value>,
}

/// Atomically update the provider for a live session.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/session/provider/update", response = UpdateProviderResponse)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProviderRequest {
    pub session_id: String,
    pub provider: String,
    pub model: Option<String>,
    pub context_limit: Option<usize>,
    pub request_params: Option<HashMap<String, serde_json::Value>>,
}

/// Provider update response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProviderResponse {
    /// Refreshed session config options after the provider/model change.
    pub config_options: Vec<serde_json::Value>,
}

/// Read a single non-secret config value.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/read", response = ReadConfigResponse)]
#[serde(rename_all = "camelCase")]
pub struct ReadConfigRequest {
    pub key: String,
}

/// Config read response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
#[serde(rename_all = "camelCase")]
pub struct ReadConfigResponse {
    #[serde(default)]
    pub value: serde_json::Value,
}

/// Upsert a single non-secret config value.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/upsert", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct UpsertConfigRequest {
    pub key: String,
    pub value: serde_json::Value,
}

/// Remove a single non-secret config value.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/remove", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct RemoveConfigRequest {
    pub key: String,
}

/// Check whether a secret exists. Never returns the actual value.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/secret/check", response = CheckSecretResponse)]
#[serde(rename_all = "camelCase")]
pub struct CheckSecretRequest {
    pub key: String,
}

/// Secret check response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
#[serde(rename_all = "camelCase")]
pub struct CheckSecretResponse {
    pub exists: bool,
}

/// Set a secret value (write-only).
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/secret/upsert", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct UpsertSecretRequest {
    pub key: String,
    pub value: serde_json::Value,
}

/// Remove a secret.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/secret/remove", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct RemoveSecretRequest {
    pub key: String,
}

/// List providers available through goose, including the config-default sentinel.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/providers/list", response = ListProvidersResponse)]
pub struct ListProvidersRequest {}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProviderListEntry {
    pub id: String,
    pub label: String,
}

/// Provider list response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct ListProvidersResponse {
    pub providers: Vec<ProviderListEntry>,
}

/// Archive a session (soft delete).
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/session/archive", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSessionRequest {
    pub session_id: String,
}

/// Unarchive a previously archived session.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/session/unarchive", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct UnarchiveSessionRequest {
    pub session_id: String,
}

/// Export a session as a JSON string.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/session/export", response = ExportSessionResponse)]
#[serde(rename_all = "camelCase")]
pub struct ExportSessionRequest {
    pub session_id: String,
}

/// Export session response — raw JSON of the goose session with `conversation`.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct ExportSessionResponse {
    pub data: String,
}

/// Import a session from a JSON string.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/session/import", response = ImportSessionResponse)]
pub struct ImportSessionRequest {
    pub data: String,
}

/// Import session response — metadata about the newly created session.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
#[serde(rename_all = "camelCase")]
pub struct ImportSessionResponse {
    pub session_id: String,
    pub title: Option<String>,
    pub updated_at: Option<String>,
    pub message_count: u64,
}

/// List providers with full metadata (config keys, setup steps, etc.).
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/providers/details", response = GetProviderDetailsResponse)]
pub struct GetProviderDetailsRequest {}

/// Provider details response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct GetProviderDetailsResponse {
    pub providers: Vec<ProviderDetailEntry>,
}

/// Fetch the full list of models available for a specific provider.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/providers/models", response = GetProviderModelsResponse)]
#[serde(rename_all = "camelCase")]
pub struct GetProviderModelsRequest {
    pub provider_name: String,
}

/// Provider models response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct GetProviderModelsResponse {
    pub models: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDetailEntry {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub default_model: String,
    pub is_configured: bool,
    pub provider_type: String,
    pub config_keys: Vec<ProviderConfigKey>,
    #[serde(default)]
    pub setup_steps: Vec<String>,
    #[serde(default)]
    pub known_models: Vec<ModelEntry>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModelEntry {
    pub name: String,
    pub context_limit: usize,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfigKey {
    pub name: String,
    pub required: bool,
    pub secret: bool,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub oauth_flow: bool,
    #[serde(default)]
    pub device_code_flow: bool,
    #[serde(default)]
    pub primary: bool,
}

// ---------------------------------------------------------------------------
// Config management
// ---------------------------------------------------------------------------

/// Backup the config file.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/backup", response = ConfigMessageResponse)]
pub struct BackupConfigRequest {}

/// Recover a corrupted config file.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/recover", response = ConfigMessageResponse)]
pub struct RecoverConfigRequest {}

/// Validate the config file.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/validate", response = ConfigMessageResponse)]
pub struct ValidateConfigRequest {}

/// Initialize config from workspace init-config if it doesn't exist.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/init", response = ConfigMessageResponse)]
pub struct InitConfigRequest {}

/// Generic response carrying a message string.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct ConfigMessageResponse {
    pub message: String,
}

/// Read all non-secret config values.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/read_all", response = ReadAllConfigResponse)]
pub struct ReadAllConfigRequest {}

/// All config values response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct ReadAllConfigResponse {
    pub config: HashMap<String, serde_json::Value>,
}

/// Add or update an extension in persistent config.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/extensions/add", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct AddConfigExtensionRequest {
    pub name: String,
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Remove an extension from persistent config.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/extensions/remove", response = EmptyResponse)]
pub struct RemoveConfigExtensionRequest {
    pub name: String,
}

/// List available slash commands.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/slash_commands", response = GetSlashCommandsResponse)]
#[serde(rename_all = "camelCase")]
pub struct GetSlashCommandsRequest {
    pub working_dir: Option<String>,
}

/// Slash commands response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct GetSlashCommandsResponse {
    pub commands: Vec<serde_json::Value>,
}

/// Batch-update tool permission levels.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/permissions/upsert", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct UpsertPermissionsRequest {
    pub tool_permissions: Vec<ToolPermissionEntry>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolPermissionEntry {
    pub tool_name: String,
    pub permission: String,
}

/// Look up canonical model metadata.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/model_info", response = GetModelInfoResponse)]
pub struct GetModelInfoRequest {
    pub provider: String,
    pub model: String,
}

/// Model info response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
#[serde(rename_all = "camelCase")]
pub struct GetModelInfoResponse {
    pub model_info: Option<serde_json::Value>,
    pub source: String,
}

/// Validate that a provider can be instantiated.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/provider/check", response = EmptyResponse)]
pub struct CheckProviderRequest {
    pub provider: String,
}

/// Set the default provider and model in persistent config.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/provider/set", response = EmptyResponse)]
pub struct SetProviderRequest {
    pub provider: String,
    pub model: String,
}

/// Clean up cached credentials/tokens for a provider.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/provider/cleanup", response = ConfigMessageResponse)]
#[serde(rename_all = "camelCase")]
pub struct CleanupProviderCacheRequest {
    pub provider_name: String,
}

// ---------------------------------------------------------------------------
// Custom providers
// ---------------------------------------------------------------------------

/// Create a custom (declarative) provider.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/custom_providers/create", response = CreateCustomProviderResponse)]
#[serde(rename_all = "camelCase")]
pub struct CreateCustomProviderRequest {
    pub engine: String,
    pub display_name: String,
    pub api_url: String,
    pub api_key: String,
    pub models: Vec<String>,
    #[serde(default)]
    pub supports_streaming: Option<bool>,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub requires_auth: bool,
    #[serde(default)]
    pub catalog_provider_id: Option<String>,
    #[serde(default)]
    pub base_path: Option<String>,
}

/// Create custom provider response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
#[serde(rename_all = "camelCase")]
pub struct CreateCustomProviderResponse {
    pub provider_name: String,
}

/// Get a custom provider by ID.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/custom_providers/get", response = GetCustomProviderResponse)]
pub struct GetCustomProviderRequest {
    pub id: String,
}

/// Get custom provider response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct GetCustomProviderResponse {
    pub provider: serde_json::Value,
}

/// Update a custom provider.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/custom_providers/update", response = ConfigMessageResponse)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomProviderRequest {
    pub id: String,
    pub engine: String,
    pub display_name: String,
    pub api_url: String,
    pub api_key: String,
    pub models: Vec<String>,
    #[serde(default)]
    pub supports_streaming: Option<bool>,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub requires_auth: bool,
    #[serde(default)]
    pub catalog_provider_id: Option<String>,
    #[serde(default)]
    pub base_path: Option<String>,
}

/// Remove a custom provider.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/custom_providers/remove", response = EmptyResponse)]
pub struct RemoveCustomProviderRequest {
    pub id: String,
}

/// Get the provider catalog.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/provider_catalog", response = GetProviderCatalogResponse)]
pub struct GetProviderCatalogRequest {
    #[serde(default)]
    pub format: Option<String>,
}

/// Provider catalog response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct GetProviderCatalogResponse {
    pub providers: Vec<serde_json::Value>,
}

/// Get a provider catalog template by ID.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/config/provider_catalog/template", response = GetProviderCatalogTemplateResponse)]
pub struct GetProviderCatalogTemplateRequest {
    pub id: String,
}

/// Provider catalog template response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct GetProviderCatalogTemplateResponse {
    pub template: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Prompts
// ---------------------------------------------------------------------------

/// List all prompt templates.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/prompts/list", response = ListPromptsResponse)]
pub struct ListPromptsRequest {}

/// Prompts list response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct ListPromptsResponse {
    pub prompts: Vec<serde_json::Value>,
}

/// Get a specific prompt template.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/prompts/get", response = GetPromptResponse)]
pub struct GetPromptRequest {
    pub name: String,
}

/// Get prompt response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
#[serde(rename_all = "camelCase")]
pub struct GetPromptResponse {
    pub name: String,
    pub content: String,
    pub default_content: String,
    pub is_customized: bool,
}

/// Save a prompt template.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/prompts/save", response = EmptyResponse)]
pub struct SavePromptRequest {
    pub name: String,
    pub content: String,
}

/// Reset a prompt template to its default.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/prompts/reset", response = EmptyResponse)]
pub struct ResetPromptRequest {
    pub name: String,
}

// ---------------------------------------------------------------------------
// Session operations
// ---------------------------------------------------------------------------

/// Search sessions by query string.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/session/search", response = SearchSessionsResponse)]
#[serde(rename_all = "camelCase")]
pub struct SearchSessionsRequest {
    pub query: String,
    #[serde(default = "default_search_limit")]
    pub limit: usize,
    #[serde(default)]
    pub after_date: Option<String>,
    #[serde(default)]
    pub before_date: Option<String>,
}

fn default_search_limit() -> usize {
    10
}

/// Search sessions response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct SearchSessionsResponse {
    pub sessions: Vec<serde_json::Value>,
}

/// Get aggregate insights across all sessions.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/session/insights", response = GetSessionInsightsResponse)]
pub struct GetSessionInsightsRequest {}

/// Session insights response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct GetSessionInsightsResponse {
    pub insights: serde_json::Value,
}

/// Rename a session.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/session/rename", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct RenameSessionRequest {
    pub session_id: String,
    pub name: String,
}

/// Fork (branch/copy) a session.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/session/fork", response = GooseForkSessionResponse)]
#[serde(rename_all = "camelCase")]
pub struct GooseForkSessionRequest {
    pub session_id: String,
    #[serde(default)]
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub truncate: bool,
    #[serde(default)]
    pub copy: bool,
}

/// Fork session response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
#[serde(rename_all = "camelCase")]
pub struct GooseForkSessionResponse {
    pub session_id: String,
}

/// Update user-provided recipe parameter values.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/session/recipe_values/update", response = UpdateRecipeValuesResponse)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRecipeValuesRequest {
    pub session_id: String,
    pub user_recipe_values: HashMap<String, String>,
}

/// Update recipe values response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct UpdateRecipeValuesResponse {
    pub recipe: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Action required
// ---------------------------------------------------------------------------

/// Confirm or deny a tool permission request.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/action/confirm", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmToolActionRequest {
    pub id: String,
    #[serde(default)]
    pub principal_type: String,
    pub action: String,
    pub session_id: String,
}

// ---------------------------------------------------------------------------
// Agent lifecycle
// ---------------------------------------------------------------------------

/// Call a tool directly (not through agent loop).
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/tool/call", response = CallToolResponse)]
#[serde(rename_all = "camelCase")]
pub struct CallToolRequest {
    pub session_id: String,
    pub name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

/// Call tool response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
#[serde(rename_all = "camelCase")]
pub struct CallToolResponse {
    pub content: Vec<serde_json::Value>,
    #[serde(default)]
    pub structured_content: Option<serde_json::Value>,
    #[serde(default)]
    pub is_error: bool,
}

/// List goose apps (MCP app resources).
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/apps/list", response = ListAppsResponse)]
#[serde(rename_all = "camelCase")]
pub struct ListAppsRequest {
    #[serde(default)]
    pub session_id: Option<String>,
}

/// List apps response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct ListAppsResponse {
    pub apps: Vec<serde_json::Value>,
}

/// Import an app from HTML.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/apps/import", response = ImportAppResponse)]
pub struct ImportAppRequest {
    pub html: String,
}

/// Import app response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct ImportAppResponse {
    pub name: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Scheduling
// ---------------------------------------------------------------------------

/// Create a scheduled job.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/schedule/create", response = ScheduleJobResponse)]
pub struct CreateScheduleRequest {
    pub id: String,
    pub recipe: serde_json::Value,
    pub cron: String,
}

/// Scheduled job response (used for create/update).
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct ScheduleJobResponse {
    pub job: serde_json::Value,
}

/// List all scheduled jobs.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/schedule/list", response = ListSchedulesResponse)]
pub struct ListSchedulesRequest {}

/// List schedules response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct ListSchedulesResponse {
    pub jobs: Vec<serde_json::Value>,
}

/// Delete a scheduled job.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/schedule/delete", response = EmptyResponse)]
pub struct DeleteScheduleRequest {
    pub id: String,
}

/// Update a schedule's cron expression.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/schedule/update", response = ScheduleJobResponse)]
pub struct UpdateScheduleRequest {
    pub id: String,
    pub cron: String,
}

/// Run a scheduled job immediately.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/schedule/run_now", response = RunNowResponse)]
pub struct RunNowRequest {
    pub id: String,
}

/// Run now response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
#[serde(rename_all = "camelCase")]
pub struct RunNowResponse {
    pub session_id: String,
}

/// Pause a schedule.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/schedule/pause", response = EmptyResponse)]
pub struct PauseScheduleRequest {
    pub id: String,
}

/// Unpause a schedule.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/schedule/unpause", response = EmptyResponse)]
pub struct UnpauseScheduleRequest {
    pub id: String,
}

/// Kill a running scheduled job.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/schedule/kill", response = KillJobResponse)]
pub struct KillJobRequest {
    pub id: String,
}

/// Kill job response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct KillJobResponse {
    pub message: String,
}

/// Inspect a running scheduled job.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/schedule/inspect", response = InspectJobResponse)]
pub struct InspectJobRequest {
    pub id: String,
}

/// Inspect job response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
#[serde(rename_all = "camelCase")]
pub struct InspectJobResponse {
    pub session_id: Option<String>,
    pub process_start_time: Option<String>,
    pub running_duration_seconds: Option<i64>,
}

/// Get sessions for a schedule.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/schedule/sessions", response = ScheduleSessionsResponse)]
pub struct ScheduleSessionsRequest {
    pub id: String,
    #[serde(default = "default_schedule_sessions_limit")]
    pub limit: usize,
}

fn default_schedule_sessions_limit() -> usize {
    20
}

/// Schedule sessions response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct ScheduleSessionsResponse {
    pub sessions: Vec<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Recipes
// ---------------------------------------------------------------------------

/// Create a recipe from a session's conversation.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/recipe/create", response = CreateRecipeResponse)]
#[serde(rename_all = "camelCase")]
pub struct CreateRecipeRequest {
    pub session_id: String,
    #[serde(default)]
    pub author: Option<serde_json::Value>,
}

/// Create recipe response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct CreateRecipeResponse {
    pub recipe: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Encode a recipe into a deeplink.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/recipe/encode", response = EncodeRecipeResponse)]
pub struct EncodeRecipeRequest {
    pub recipe: serde_json::Value,
}

/// Encode recipe response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct EncodeRecipeResponse {
    pub deeplink: String,
}

/// Decode a deeplink into a recipe.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/recipe/decode", response = DecodeRecipeResponse)]
pub struct DecodeRecipeRequest {
    pub deeplink: String,
}

/// Decode recipe response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct DecodeRecipeResponse {
    pub recipe: serde_json::Value,
}

/// Scan a recipe for security warnings.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/recipe/scan", response = ScanRecipeResponse)]
pub struct ScanRecipeRequest {
    pub recipe: serde_json::Value,
}

/// Scan recipe response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
#[serde(rename_all = "camelCase")]
pub struct ScanRecipeResponse {
    pub has_security_warnings: bool,
}

/// List all saved recipes.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/recipe/list", response = ListRecipesResponse)]
pub struct ListRecipesRequest {}

/// List recipes response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct ListRecipesResponse {
    pub recipes: Vec<serde_json::Value>,
}

/// Delete a recipe by ID.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/recipe/delete", response = EmptyResponse)]
pub struct DeleteRecipeRequest {
    pub id: String,
}

/// Schedule a recipe.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/recipe/schedule", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleRecipeRequest {
    pub id: String,
    #[serde(default)]
    pub cron_schedule: Option<String>,
}

/// Set a slash command for a recipe.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/recipe/set_slash_command", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct SetRecipeSlashCommandRequest {
    pub id: String,
    #[serde(default)]
    pub slash_command: Option<String>,
}

/// Save a recipe to disk.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/recipe/save", response = SaveRecipeResponse)]
pub struct SaveRecipeRequest {
    pub recipe: serde_json::Value,
    #[serde(default)]
    pub id: Option<String>,
}

/// Save recipe response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
#[serde(rename_all = "camelCase")]
pub struct SaveRecipeResponse {
    pub id: String,
    pub file_name: String,
    pub file_path: String,
}

/// Parse recipe content (YAML/markdown) into a Recipe struct.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/recipe/parse", response = ParseRecipeResponse)]
pub struct ParseRecipeRequest {
    pub content: String,
}

/// Parse recipe response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct ParseRecipeResponse {
    pub recipe: serde_json::Value,
}

/// Convert a recipe to YAML.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/recipe/to_yaml", response = RecipeToYamlResponse)]
pub struct RecipeToYamlRequest {
    pub recipe: serde_json::Value,
}

/// Recipe to YAML response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct RecipeToYamlResponse {
    pub yaml: String,
}

// ---------------------------------------------------------------------------
// Telemetry
// ---------------------------------------------------------------------------

/// Send a telemetry event.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/telemetry/event", response = EmptyResponse)]
#[serde(rename_all = "camelCase")]
pub struct SendTelemetryRequest {
    pub event_name: String,
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Dictation
// ---------------------------------------------------------------------------

/// Transcribe audio.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/dictation/transcribe", response = TranscribeResponse)]
#[serde(rename_all = "camelCase")]
pub struct TranscribeRequest {
    pub audio: String,
    pub mime_type: String,
    pub provider: String,
}

/// Transcribe response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct TranscribeResponse {
    pub text: String,
}

/// Get dictation provider configuration/status.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/dictation/config", response = GetDictationConfigResponse)]
pub struct GetDictationConfigRequest {}

/// Dictation config response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct GetDictationConfigResponse {
    pub providers: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Features
// ---------------------------------------------------------------------------

/// Get compile-time feature flags.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcRequest)]
#[request(method = "_goose/features", response = GetFeaturesResponse)]
pub struct GetFeaturesRequest {}

/// Features response.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct GetFeaturesResponse {
    pub features: HashMap<String, bool>,
}

/// Empty success response for operations that return no data.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, JsonRpcResponse)]
pub struct EmptyResponse {}
