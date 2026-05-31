use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct Model {
    pub name: String,
}

// ---- Ollama Cloud catalog (OpenAI-compatible /v1/models) ----
#[derive(Deserialize)]
struct OpenAiModels {
    data: Vec<OpenAiModel>,
}

#[derive(Deserialize)]
struct OpenAiModel {
    id: String,
}

const CLOUD_MODELS_URL: &str = "https://ollama.com/v1/models";

/// Convert a cloud catalog id into the name ollama uses to run it.
/// Tagged names append `-cloud` (gpt-oss:120b -> gpt-oss:120b-cloud);
/// untagged names append `:cloud` (glm-4.6 -> glm-4.6:cloud).
fn to_cloud_ref(id: &str) -> String {
    // already a cloud ref? leave as-is
    if id.ends_with("-cloud") || id.ends_with(":cloud") {
        return id.to_string();
    }
    if id.contains(':') {
        format!("{id}-cloud")
    } else {
        format!("{id}:cloud")
    }
}

/// Full cloud model catalog from the Ollama subscription, as launchable names.
/// Listing does not require an API key.
pub async fn list_cloud_models() -> Result<Vec<Model>> {
    let resp = reqwest::Client::new()
        .get(CLOUD_MODELS_URL)
        .send()
        .await
        .context("failed to reach ollama.com cloud catalog")?;
    let parsed: OpenAiModels = resp.json().await.context("invalid /v1/models response")?;
    let mut models: Vec<Model> = parsed
        .data
        .into_iter()
        .map(|m| Model { name: to_cloud_ref(&m.id) })
        .collect();
    models.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::to_cloud_ref;

    #[test]
    fn cloud_ref_rule() {
        assert_eq!(to_cloud_ref("gpt-oss:120b"), "gpt-oss:120b-cloud");
        assert_eq!(to_cloud_ref("glm-4.6"), "glm-4.6:cloud");
        assert_eq!(to_cloud_ref("deepseek-v4-pro"), "deepseek-v4-pro:cloud");
        assert_eq!(to_cloud_ref("gemma4:31b-cloud"), "gemma4:31b-cloud");
        assert_eq!(to_cloud_ref("deepseek-v4-pro:cloud"), "deepseek-v4-pro:cloud");
    }
}
