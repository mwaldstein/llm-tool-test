use crate::results::{Cache, CacheKey, ResultRecord};

pub fn compute_cache_key(scenario_yaml: &str, prompt: &str, tool: &str, model: &str) -> CacheKey {
    CacheKey::compute(scenario_yaml, prompt, tool, model)
}

pub fn check_cache(cache: &Cache, cache_key: &CacheKey) -> anyhow::Result<Option<ResultRecord>> {
    Ok(cache.get(cache_key))
}
