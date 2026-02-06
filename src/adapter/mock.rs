use super::ToolAdapter;
use crate::scenario::Scenario;
use std::path::Path;

pub struct MockAdapter;

impl MockAdapter {
    pub fn generate_transcript(&self, _scenario: &Scenario) -> String {
        // Generate a simple mock transcript without executing any commands
        // This is used for testing the framework without requiring a real tool
        "mock command output\nMock execution completed successfully".to_string()
    }
}

impl ToolAdapter for MockAdapter {
    fn is_available(&self) -> Result<super::ToolStatus, super::AdapterError> {
        Ok(super::ToolStatus {
            available: true,
            authenticated: true,
        })
    }

    fn check_availability(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn run(
        &self,
        scenario: &Scenario,
        _cwd: &Path,
        _model: Option<&str>,
        _timeout_secs: u64,
    ) -> anyhow::Result<(String, i32, Option<f64>, Option<super::TokenUsage>)> {
        // Generate mock output without executing any commands
        let transcript = self.generate_transcript(scenario);
        Ok((transcript, 0, None, None))
    }
}
