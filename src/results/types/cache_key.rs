//! Cache key for deduplicating test runs.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Cache key for deduplicating test runs.
///
/// Computed from scenario content, prompt, tool,
/// and model to uniquely identify a test configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct CacheKey {
    /// Hash of the scenario YAML content
    pub scenario_hash: String,
    /// Hash of the task prompt
    pub prompt_hash: String,
    /// Tool name
    pub tool: String,
    /// Model name
    pub model: String,
}

impl CacheKey {
    /// Compute a cache key from run parameters.
    ///
    /// Hashes the scenario YAML and prompt using SHA256,
    /// and combines with tool and model information.
    ///
    /// # Arguments
    ///
    /// * `scenario_yaml` - Raw scenario YAML content
    /// * `prompt` - Task prompt text
    /// * `tool` - Tool name
    /// * `model` - Model name
    ///
    /// # Returns
    ///
    /// A computed `CacheKey`
    pub fn compute(scenario_yaml: &str, prompt: &str, tool: &str, model: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(scenario_yaml.as_bytes());
        let scenario_hash = format!("{:x}", hasher.finalize());

        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        let prompt_hash = format!("{:x}", hasher.finalize());

        Self {
            scenario_hash,
            prompt_hash,
            tool: tool.to_string(),
            model: model.to_string(),
        }
    }

    /// Convert the cache key to a string representation.
    ///
    /// Used as the filename for cached results.
    ///
    /// # Returns
    ///
    /// A string combining all hash and identifier components
    pub fn as_string(&self) -> String {
        // Sanitize model name to avoid path separator issues in filenames
        let safe_model = self.model.replace(['/', '\\'], "_");
        format!(
            "{}_{}_{}_{}",
            self.scenario_hash, self.prompt_hash, self.tool, safe_model,
        )
    }
}
