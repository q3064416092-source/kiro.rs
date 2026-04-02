use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::anthropic::types::Model;

const DEFAULT_CONTEXT_WINDOW: i32 = 200_000;
const EXTENDED_CONTEXT_WINDOW: i32 = 1_000_000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum CredentialTier {
    #[default]
    Any,
    Opus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CustomModel {
    pub id: String,
    pub display_name: String,
    pub model_type: String,
    pub max_tokens: i32,
    pub owned_by: String,
    #[serde(default = "default_context_window")]
    pub context_window: i32,
    #[serde(default)]
    pub supports_thinking: bool,
    #[serde(default)]
    pub credential_tier: CredentialTier,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "targetModel"
    )]
    pub upstream_model_id: Option<String>,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertCustomModel {
    pub id: String,
    pub display_name: String,
    pub model_type: String,
    pub max_tokens: i32,
    pub owned_by: String,
    #[serde(default = "default_context_window")]
    pub context_window: i32,
    #[serde(default)]
    pub supports_thinking: bool,
    #[serde(default)]
    pub credential_tier: CredentialTier,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelListResponse {
    pub built_in: Vec<Model>,
    pub custom: Vec<CustomModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedModel {
    pub public_id: String,
    pub upstream_model_id: String,
    pub display_name: String,
    pub model_type: String,
    pub max_tokens: i32,
    pub owned_by: String,
    pub context_window: i32,
    pub supports_thinking: bool,
    pub credential_tier: CredentialTier,
}

#[derive(Debug, Clone)]
pub struct ModelManager {
    path: PathBuf,
    custom: Arc<RwLock<Vec<CustomModel>>>,
}

#[derive(Debug, Clone, Copy)]
struct BuiltInModelDefinition {
    public_id: &'static str,
    upstream_model_id: &'static str,
    created: i64,
    owned_by: &'static str,
    display_name: &'static str,
    model_type: &'static str,
    max_tokens: i32,
    context_window: i32,
    supports_thinking: bool,
    credential_tier: CredentialTier,
}

const BUILT_IN_MODELS: &[BuiltInModelDefinition] = &[
    BuiltInModelDefinition {
        public_id: "claude-opus-4-6",
        upstream_model_id: "claude-opus-4.6",
        created: 1770163200,
        owned_by: "anthropic",
        display_name: "Claude Opus 4.6",
        model_type: "chat",
        max_tokens: 64000,
        context_window: EXTENDED_CONTEXT_WINDOW,
        supports_thinking: true,
        credential_tier: CredentialTier::Opus,
    },
    BuiltInModelDefinition {
        public_id: "claude-opus-4-6-thinking",
        upstream_model_id: "claude-opus-4.6",
        created: 1770163200,
        owned_by: "anthropic",
        display_name: "Claude Opus 4.6 (Thinking)",
        model_type: "chat",
        max_tokens: 64000,
        context_window: EXTENDED_CONTEXT_WINDOW,
        supports_thinking: true,
        credential_tier: CredentialTier::Opus,
    },
    BuiltInModelDefinition {
        public_id: "claude-sonnet-4-6",
        upstream_model_id: "claude-sonnet-4.6",
        created: 1771286400,
        owned_by: "anthropic",
        display_name: "Claude Sonnet 4.6",
        model_type: "chat",
        max_tokens: 64000,
        context_window: EXTENDED_CONTEXT_WINDOW,
        supports_thinking: true,
        credential_tier: CredentialTier::Any,
    },
];

fn default_context_window() -> i32 {
    DEFAULT_CONTEXT_WINDOW
}

fn infer_context_window(model_id: &str) -> i32 {
    let model_id = model_id.to_ascii_lowercase();
    if model_id.contains("4.6") || model_id.contains("4-6") {
        EXTENDED_CONTEXT_WINDOW
    } else {
        DEFAULT_CONTEXT_WINDOW
    }
}

fn infer_credential_tier(model_id: &str) -> CredentialTier {
    if model_id.to_ascii_lowercase().contains("opus") {
        CredentialTier::Opus
    } else {
        CredentialTier::Any
    }
}

fn infer_supports_thinking(model_id: &str) -> bool {
    let model_id = model_id.to_ascii_lowercase();
    model_id.contains("thinking") || model_id.contains("sonnet") || model_id.contains("opus")
}

fn builtin_model_definition(model_id: &str) -> Option<BuiltInModelDefinition> {
    BUILT_IN_MODELS
        .iter()
        .copied()
        .find(|item| item.public_id == model_id)
}

fn compatibility_model(model_id: &str) -> Option<ResolvedModel> {
    let model_lower = model_id.to_ascii_lowercase();
    let upstream_model_id = if model_lower.contains("sonnet") {
        if model_lower.contains("4-6") || model_lower.contains("4.6") {
            "claude-sonnet-4.6".to_string()
        } else {
            "claude-sonnet-4.5".to_string()
        }
    } else if model_lower.contains("opus") {
        if model_lower.contains("4-5") || model_lower.contains("4.5") {
            "claude-opus-4.5".to_string()
        } else {
            "claude-opus-4.6".to_string()
        }
    } else if model_lower.contains("haiku") {
        "claude-haiku-4.5".to_string()
    } else {
        return None;
    };

    Some(ResolvedModel {
        public_id: model_id.to_string(),
        upstream_model_id: upstream_model_id.clone(),
        display_name: model_id.to_string(),
        model_type: "chat".to_string(),
        max_tokens: 64000,
        owned_by: "anthropic".to_string(),
        context_window: infer_context_window(&upstream_model_id),
        supports_thinking: infer_supports_thinking(model_id),
        credential_tier: infer_credential_tier(&upstream_model_id),
    })
}

impl BuiltInModelDefinition {
    fn as_model(self) -> Model {
        Model {
            id: self.public_id.to_string(),
            object: "model".to_string(),
            created: self.created,
            owned_by: self.owned_by.to_string(),
            display_name: self.display_name.to_string(),
            model_type: self.model_type.to_string(),
            max_tokens: self.max_tokens,
        }
    }

    fn as_resolved_model(self) -> ResolvedModel {
        ResolvedModel {
            public_id: self.public_id.to_string(),
            upstream_model_id: self.upstream_model_id.to_string(),
            display_name: self.display_name.to_string(),
            model_type: self.model_type.to_string(),
            max_tokens: self.max_tokens,
            owned_by: self.owned_by.to_string(),
            context_window: self.context_window,
            supports_thinking: self.supports_thinking,
            credential_tier: self.credential_tier,
        }
    }
}

impl CustomModel {
    pub fn effective_upstream_model_id(&self) -> &str {
        self.upstream_model_id.as_deref().unwrap_or(&self.id)
    }

    pub fn is_legacy_alias(&self) -> bool {
        self.upstream_model_id
            .as_deref()
            .is_some_and(|upstream| upstream != self.id)
    }

    fn normalize(mut self) -> Self {
        let effective_upstream = self.effective_upstream_model_id().to_string();
        let is_legacy_alias = self.is_legacy_alias();

        if self.context_window <= 0
            || (is_legacy_alias && self.context_window == DEFAULT_CONTEXT_WINDOW)
        {
            self.context_window = infer_context_window(&effective_upstream);
        }
        if is_legacy_alias && !self.supports_thinking {
            self.supports_thinking = infer_supports_thinking(&effective_upstream);
        }
        if is_legacy_alias && self.credential_tier == CredentialTier::Any {
            self.credential_tier = infer_credential_tier(&effective_upstream);
        }

        self
    }

    pub fn as_resolved_model(&self) -> ResolvedModel {
        ResolvedModel {
            public_id: self.id.clone(),
            upstream_model_id: self.effective_upstream_model_id().to_string(),
            display_name: self.display_name.clone(),
            model_type: self.model_type.clone(),
            max_tokens: self.max_tokens,
            owned_by: self.owned_by.clone(),
            context_window: self.context_window,
            supports_thinking: self.supports_thinking,
            credential_tier: self.credential_tier,
        }
    }
}

pub fn built_in_models() -> Vec<Model> {
    BUILT_IN_MODELS
        .iter()
        .copied()
        .map(BuiltInModelDefinition::as_model)
        .collect()
}

impl ModelManager {
    pub fn load(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            fs::write(&path, "[]").with_context(|| format!("create {} failed", path.display()))?;
        }

        let custom = match fs::read_to_string(&path) {
            Ok(content) if content.trim().is_empty() => Vec::new(),
            Ok(content) => match serde_json::from_str::<Vec<CustomModel>>(&content) {
                Ok(models) => models.into_iter().map(CustomModel::normalize).collect(),
                Err(error) => {
                    tracing::warn!(path = %path.display(), %error, "failed to parse custom models, starting empty");
                    Vec::new()
                }
            },
            Err(error) => bail!("failed to read custom models config: {}", error),
        };

        Ok(Self {
            path,
            custom: Arc::new(RwLock::new(custom)),
        })
    }

    pub fn list_models(&self) -> ModelListResponse {
        ModelListResponse {
            built_in: built_in_models(),
            custom: self.custom.read().clone(),
        }
    }

    pub fn resolve_requested_model(&self, model_id: &str) -> Option<ResolvedModel> {
        if let Some(custom) = self.custom.read().iter().find(|item| item.id == model_id) {
            return Some(custom.as_resolved_model());
        }

        if let Some(definition) = builtin_model_definition(model_id) {
            return Some(definition.as_resolved_model());
        }

        compatibility_model(model_id)
    }

    pub fn add_model(&self, request: UpsertCustomModel) -> Result<CustomModel> {
        let mut current = self.custom.read().clone();
        self.validate_new_model(&request, &current)?;

        let model = CustomModel {
            id: request.id,
            display_name: request.display_name,
            model_type: request.model_type,
            max_tokens: request.max_tokens,
            owned_by: request.owned_by,
            context_window: request.context_window,
            supports_thinking: request.supports_thinking,
            credential_tier: request.credential_tier,
            upstream_model_id: None,
            created: chrono::Utc::now().timestamp(),
        }
        .normalize();

        current.push(model.clone());
        self.persist_and_swap(current)?;
        Ok(model)
    }

    pub fn update_model(&self, id: &str, request: UpsertCustomModel) -> Result<CustomModel> {
        let mut current = self.custom.read().clone();

        let index = current
            .iter()
            .position(|m| m.id == id)
            .ok_or_else(|| anyhow::anyhow!("model '{}' not found", id))?;

        self.validate_updated_model(id, &request, &current)?;

        let original_created = current[index].created;
        let updated = CustomModel {
            id: request.id,
            display_name: request.display_name,
            model_type: request.model_type,
            max_tokens: request.max_tokens,
            owned_by: request.owned_by,
            context_window: request.context_window,
            supports_thinking: request.supports_thinking,
            credential_tier: request.credential_tier,
            upstream_model_id: None,
            created: original_created,
        }
        .normalize();

        current[index] = updated.clone();
        self.persist_and_swap(current)?;
        Ok(updated)
    }

    pub fn delete_model(&self, id: &str) -> Result<()> {
        let mut current = self.custom.read().clone();

        let index = current
            .iter()
            .position(|m| m.id == id)
            .ok_or_else(|| anyhow::anyhow!("model '{}' not found", id))?;

        current.remove(index);
        self.persist_and_swap(current)?;
        Ok(())
    }

    fn validate_new_model(
        &self,
        request: &UpsertCustomModel,
        current: &[CustomModel],
    ) -> Result<()> {
        if builtin_model_definition(&request.id).is_some() {
            bail!("model id '{}' conflicts with a built-in model", request.id);
        }

        if current.iter().any(|m| m.id == request.id) {
            bail!("model id '{}' already exists", request.id);
        }

        self.validate_model_request(request)
    }

    fn validate_updated_model(
        &self,
        current_id: &str,
        request: &UpsertCustomModel,
        current: &[CustomModel],
    ) -> Result<()> {
        if request.id != current_id && builtin_model_definition(&request.id).is_some() {
            bail!("model id '{}' conflicts with a built-in model", request.id);
        }

        if request.id != current_id && current.iter().any(|m| m.id == request.id) {
            bail!("model id '{}' already exists", request.id);
        }

        self.validate_model_request(request)
    }

    fn validate_model_request(&self, request: &UpsertCustomModel) -> Result<()> {
        if request.id.trim().is_empty() {
            bail!("model id cannot be empty");
        }
        if request.display_name.trim().is_empty() {
            bail!("display name cannot be empty");
        }
        if request.model_type.trim().is_empty() {
            bail!("model type cannot be empty");
        }
        if request.owned_by.trim().is_empty() {
            bail!("owned by cannot be empty");
        }
        if request.max_tokens <= 0 {
            bail!("max tokens must be greater than 0");
        }
        if request.context_window <= 0 {
            bail!("context window must be greater than 0");
        }

        Ok(())
    }

    fn persist_and_swap(&self, new_models: Vec<CustomModel>) -> Result<()> {
        let content = serde_json::to_string_pretty(&new_models)
            .context("failed to serialize custom models")?;

        fs::write(&self.path, content).with_context(|| {
            format!(
                "failed to write custom models config: {}",
                self.path.display()
            )
        })?;

        *self.custom.write() = new_models;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{}-{}.json", name, uuid::Uuid::new_v4()))
    }

    fn sample_request(id: &str) -> UpsertCustomModel {
        UpsertCustomModel {
            id: id.to_string(),
            display_name: "My Model".to_string(),
            model_type: "chat".to_string(),
            max_tokens: 32000,
            owned_by: "custom".to_string(),
            context_window: 500000,
            supports_thinking: true,
            credential_tier: CredentialTier::Any,
        }
    }

    #[test]
    fn test_load_missing_file_returns_empty_manager() {
        let path = temp_path("custom-models-missing");
        let manager = ModelManager::load(path.clone()).unwrap();
        let response = manager.list_models();
        assert!(response.custom.is_empty());
        assert!(path.exists());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_add_custom_model_persists_to_disk() {
        let path = temp_path("custom-models-add");
        let manager = ModelManager::load(path.clone()).unwrap();

        manager.add_model(sample_request("my-fast-model")).unwrap();

        let persisted = fs::read_to_string(&path).unwrap();
        assert!(persisted.contains("my-fast-model"));
        assert!(!persisted.contains("targetModel"));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_add_rejects_builtin_id_conflict() {
        let path = temp_path("custom-models-conflict");
        let manager = ModelManager::load(path.clone()).unwrap();
        let error = manager
            .add_model(sample_request("claude-opus-4-6"))
            .unwrap_err();

        assert!(error.to_string().contains("built-in"));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_resolve_custom_model_uses_same_upstream_id() {
        let path = temp_path("custom-models-resolve");
        let manager = ModelManager::load(path.clone()).unwrap();
        manager
            .add_model(sample_request("claude-sonnet-4.7"))
            .unwrap();

        let resolved = manager
            .resolve_requested_model("claude-sonnet-4.7")
            .unwrap();
        assert_eq!(resolved.public_id, "claude-sonnet-4.7");
        assert_eq!(resolved.upstream_model_id, "claude-sonnet-4.7");
        assert_eq!(resolved.context_window, 500000);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_add_custom_model_preserves_explicit_capabilities() {
        let path = temp_path("custom-models-explicit-capabilities");
        let manager = ModelManager::load(path.clone()).unwrap();
        let mut request = sample_request("claude-sonnet-4.7-lite");
        request.supports_thinking = false;
        request.credential_tier = CredentialTier::Any;

        let model = manager.add_model(request).unwrap();
        assert!(!model.supports_thinking);
        assert_eq!(model.credential_tier, CredentialTier::Any);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_load_legacy_target_model_alias() {
        let path = temp_path("custom-models-legacy");
        fs::write(
            &path,
            r#"[{"id":"team-sonnet","displayName":"Team Sonnet","modelType":"chat","maxTokens":64000,"ownedBy":"custom","targetModel":"claude-sonnet-4.6","created":1}]"#,
        )
        .unwrap();

        let manager = ModelManager::load(path.clone()).unwrap();
        let model = manager.list_models().custom.remove(0);
        assert!(model.is_legacy_alias());
        assert_eq!(model.effective_upstream_model_id(), "claude-sonnet-4.6");
        assert_eq!(model.context_window, EXTENDED_CONTEXT_WINDOW);

        let resolved = manager.resolve_requested_model("team-sonnet").unwrap();
        assert_eq!(resolved.upstream_model_id, "claude-sonnet-4.6");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_resolve_compatibility_model() {
        let path = temp_path("custom-models-compat");
        let manager = ModelManager::load(path.clone()).unwrap();

        let resolved = manager.resolve_requested_model("claude-opus-4").unwrap();
        assert_eq!(resolved.upstream_model_id, "claude-opus-4.6");
        assert_eq!(resolved.credential_tier, CredentialTier::Opus);

        fs::remove_file(path).unwrap();
    }
}
