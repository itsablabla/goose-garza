use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::services::acp::GooseAcpManager;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationTranscribeRequest {
    pub audio: String,
    pub mime_type: String,
    pub provider: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationTranscribeResponse {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationModelOptionResponse {
    pub id: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationProviderStatusResponse {
    pub configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    pub description: String,
    pub uses_provider_config: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_config_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_model: Option<String>,
    #[serde(default)]
    pub available_models: Vec<DictationModelOptionResponse>,
}

#[tauri::command]
pub async fn transcribe_dictation(
    app_handle: AppHandle,
    request: DictationTranscribeRequest,
) -> Result<DictationTranscribeResponse, String> {
    let manager = GooseAcpManager::start(app_handle).await?;
    let raw = manager
        .call_ext(
            "goose/dictation/transcribe",
            serde_json::json!({
                "audio": request.audio,
                "mimeType": request.mime_type,
                "provider": request.provider,
            }),
        )
        .await?;
    serde_json::from_str(&raw).map_err(|e| format!("Failed to parse transcribe response: {e}"))
}

#[tauri::command]
pub async fn get_dictation_config(
    app_handle: AppHandle,
) -> Result<std::collections::HashMap<String, DictationProviderStatusResponse>, String> {
    let manager = GooseAcpManager::start(app_handle).await?;
    let raw = manager
        .call_ext("goose/dictation/config", serde_json::json!({}))
        .await?;

    #[derive(Deserialize)]
    struct ConfigResponse {
        providers: std::collections::HashMap<String, DictationProviderStatusResponse>,
    }

    let response: ConfigResponse =
        serde_json::from_str(&raw).map_err(|e| format!("Failed to parse config response: {e}"))?;
    Ok(response.providers)
}
