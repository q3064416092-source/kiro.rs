use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::anthropic::types::Model;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CustomModel {
    pub id: String,
    pub display_name: String,
    pub model_type: String,
    pub max_tokens: i32,
    pub owned_by: String,
    pub target_model: String,
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
    pub target_model: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelListResponse {
    pub built_in: Vec<Model>,
    pub custom: Vec<CustomModel>,
}

#[derive(Debug, Clone)]
pub struct ModelManager {
    path: PathBuf,
    custom: Arc<RwLock<Vec<CustomModel>>>,
}

pub fn built_in_models() -> Vec<Model> {
    vec![
        Model {
            id: "claude-opus-4-6".to_string(),
            object: "model".to_string(),
            created: 1770163200,
            owned_by: "anthropic".to_string(),
            display_name: "Claude Opus 4.6".to_string(),
            model_type: "chat".to_string(),
            max_tokens: 64000,
        },
        Model {
            id: "claude-opus-4-6-thinking".to_string(),
            object: "model".to_string(),
            created: 1770163200,
            owned_by: "anthropic".to_string(),
            display_name: "Claude Opus 4.6 (Thinking)".to_string(),
            model_type: "chat".to_string(),
            max_tokens: 64000,
        },
        Model {
            id: "claude-sonnet-4-6".to_string(),
            object: "model".to_string(),
            created: 1771286400,
            owned_by: "anthropic".to_string(),
            display_name: "Claude Sonnet 4.6".to_string(),
            model_type: "chat".to_string(),
            max_tokens: 64000,
        },
    ]
}

impl ModelManager {
    pub fn load(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            fs::write(&path, "[]").with_context(|| format!("创建 {} 失败", path.display()))?;
        }

        let custom = match fs::read_to_string(&path) {
            Ok(content) if content.trim().is_empty() => Vec::new(),
            Ok(content) => match serde_json::from_str::<Vec<CustomModel>>(&content) {
                Ok(models) => models,
                Err(error) => {
                    tracing::warn!(path = %path.display(), %error, "自定义模型配置解析失败，使用空列表启动");
                    Vec::new()
                }
            },
            Err(error) => bail!("读取自定义模型配置失败: {}", error),
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

    pub fn resolve_model_id(&self, model_id: &str) -> String {
        self.custom
            .read()
            .iter()
            .find(|item| item.id == model_id)
            .map(|item| item.target_model.clone())
            .unwrap_or_else(|| model_id.to_string())
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
            target_model: request.target_model,
            created: chrono::Utc::now().timestamp(),
        };
        
        current.push(model.clone());
        self.persist_and_swap(current)?;
        Ok(model)
    }

    pub fn update_model(&self, id: &str, request: UpsertCustomModel) -> Result<CustomModel> {
        let mut current = self.custom.read().clone();
        
        let index = current
            .iter()
            .position(|m| m.id == id)
            .ok_or_else(|| anyhow::anyhow!("模型 '{}' 不存在", id))?;
        
        self.validate_target_model(&request.target_model)?;
        
        let original_created = current[index].created;
        let updated = CustomModel {
            id: request.id,
            display_name: request.display_name,
            model_type: request.model_type,
            max_tokens: request.max_tokens,
            owned_by: request.owned_by,
            target_model: request.target_model,
            created: original_created,
        };
        
        current[index] = updated.clone();
        self.persist_and_swap(current)?;
        Ok(updated)
    }

    pub fn delete_model(&self, id: &str) -> Result<()> {
        let mut current = self.custom.read().clone();
        
        let index = current
            .iter()
            .position(|m| m.id == id)
            .ok_or_else(|| anyhow::anyhow!("模型 '{}' 不存在", id))?;
        
        current.remove(index);
        self.persist_and_swap(current)?;
        Ok(())
    }

    fn validate_new_model(&self, request: &UpsertCustomModel, current: &[CustomModel]) -> Result<()> {
        let builtin_ids: Vec<String> = built_in_models().iter().map(|m| m.id.clone()).collect();
        
        if builtin_ids.contains(&request.id) {
            bail!("模型 ID '{}' 与内置模型冲突", request.id);
        }
        
        if current.iter().any(|m| m.id == request.id) {
            bail!("模型 ID '{}' 已存在", request.id);
        }
        
        self.validate_target_model(&request.target_model)?;
        Ok(())
    }

    fn validate_target_model(&self, target_model: &str) -> Result<()> {
        let builtin_ids: Vec<String> = built_in_models().iter().map(|m| m.id.clone()).collect();
        
        if !builtin_ids.contains(&target_model.to_string()) {
            bail!(
                "目标模型 '{}' 不存在，可用模型：{:?}",
                target_model,
                builtin_ids
            );
        }
        
        Ok(())
    }

    fn persist_and_swap(&self, new_models: Vec<CustomModel>) -> Result<()> {
        let content = serde_json::to_string_pretty(&new_models)
            .context("序列化自定义模型失败")?;
        
        fs::write(&self.path, content)
            .with_context(|| format!("写入自定义模型配置失败: {}", self.path.display()))?;
        
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
        let request = UpsertCustomModel {
            id: "my-fast-model".to_string(),
            display_name: "My Fast Model".to_string(),
            model_type: "chat".to_string(),
            max_tokens: 32000,
            owned_by: "custom".to_string(),
            target_model: "claude-sonnet-4-6".to_string(),
        };

        manager.add_model(request).unwrap();

        let persisted = fs::read_to_string(&path).unwrap();
        assert!(persisted.contains("my-fast-model"));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_add_rejects_builtin_id_conflict() {
        let path = temp_path("custom-models-conflict");
        let manager = ModelManager::load(path.clone()).unwrap();
        let error = manager
            .add_model(UpsertCustomModel {
                id: "claude-opus-4-6".to_string(),
                display_name: "Conflict".to_string(),
                model_type: "chat".to_string(),
                max_tokens: 32000,
                owned_by: "custom".to_string(),
                target_model: "claude-sonnet-4-6".to_string(),
            })
            .unwrap_err();

        assert!(error.to_string().contains("内置模型"));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_resolve_model_maps_to_target_model() {
        let path = temp_path("custom-models-resolve");
        let manager = ModelManager::load(path.clone()).unwrap();
        manager
            .add_model(UpsertCustomModel {
                id: "my-opus".to_string(),
                display_name: "My Opus".to_string(),
                model_type: "chat".to_string(),
                max_tokens: 64000,
                owned_by: "custom".to_string(),
                target_model: "claude-opus-4-6".to_string(),
            })
            .unwrap();

        assert_eq!(manager.resolve_model_id("my-opus"), "claude-opus-4-6");
        assert_eq!(manager.resolve_model_id("claude-opus-4-6"), "claude-opus-4-6");
        fs::remove_file(path).unwrap();
    }
}
