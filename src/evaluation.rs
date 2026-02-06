use crate::judge::{load_rubric, JudgeResponse};
use crate::scenario::{Gate, Scenario};
use crate::script_runner::ScriptRunner;
use crate::transcript::EfficiencyMetrics;
use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::path::Path;
use std::process::{Command, Output};

macro_rules! eval_gate {
    ($gate_type:expr, $expr:expr, |$result:ident| $closure:expr) => {
        match $expr {
            Ok($result) => {
                let (passed, message) = $closure;
                GateResult {
                    gate_type: $gate_type.to_string(),
                    passed,
                    message,
                }
            }
            Err(e) => GateResult {
                gate_type: $gate_type.to_string(),
                passed: false,
                message: format!("Evaluation error: {:#}", e),
            },
        }
    };
}

/// Context passed to gate evaluators, containing environment and optional script runner.
pub struct EvaluationContext<'a> {
    pub env_root: &'a Path,
    pub target_binary: &'a str,
    pub command_pattern: Option<&'a str>,
    pub script_runner: Option<&'a ScriptRunner>,
}

pub trait GateEvaluator {
    fn evaluate(&self, ctx: &EvaluationContext<'_>) -> GateResult;
}

impl GateEvaluator for Gate {
    fn evaluate(&self, ctx: &EvaluationContext<'_>) -> GateResult {
        match self {
            Gate::CommandSucceeds { command } => eval_command_succeeds(command, ctx.env_root),
            Gate::CommandOutputContains { command, substring } => {
                eval_command_output_contains(command, substring, ctx.env_root)
            }
            Gate::CommandOutputMatches { command, pattern } => {
                eval_command_output_matches(command, pattern, ctx.env_root)
            }
            Gate::CommandJsonPath {
                command,
                path,
                assertion,
            } => eval_command_json_path(command, path, assertion, ctx.env_root),
            Gate::FileExists { path } => eval_file_exists(path, ctx.env_root),
            Gate::FileContains { path, substring } => {
                eval_file_contains(path, substring, ctx.env_root)
            }
            Gate::FileMatches { path, pattern } => eval_file_matches(path, pattern, ctx.env_root),
            Gate::NoTranscriptErrors => {
                eval_no_transcript_errors(ctx.env_root, ctx.target_binary, ctx.command_pattern)
            }
            Gate::Script {
                command,
                description,
            } => eval_script(command, description, ctx.script_runner),
        }
    }
}

fn eval_command_succeeds(command: &str, env_root: &Path) -> GateResult {
    if command.trim().is_empty() {
        return GateResult {
            gate_type: "CommandSucceeds".to_string(),
            passed: false,
            message: "Empty command".to_string(),
        };
    }

    let output = run_shell_command(command, env_root);

    match output {
        Ok(output) => {
            let succeeds = output.status.success();
            GateResult {
                gate_type: "CommandSucceeds".to_string(),
                passed: succeeds,
                message: format!("Command '{}' succeeded: {}", command, succeeds),
            }
        }
        Err(e) => GateResult {
            gate_type: "CommandSucceeds".to_string(),
            passed: false,
            message: format!("Failed to execute command '{}': {}", command, e),
        },
    }
}

fn eval_command_output_contains(command: &str, substring: &str, env_root: &Path) -> GateResult {
    let output = run_shell_command(command, env_root);

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let passed = output.status.success() && stdout.contains(substring);
            GateResult {
                gate_type: "CommandOutputContains".to_string(),
                passed,
                message: format!(
                    "Command '{}' contains substring '{}': {}",
                    command, substring, passed
                ),
            }
        }
        Err(e) => GateResult {
            gate_type: "CommandOutputContains".to_string(),
            passed: false,
            message: format!("Failed to execute command '{}': {}", command, e),
        },
    }
}

fn eval_command_output_matches(command: &str, pattern: &str, env_root: &Path) -> GateResult {
    let regex = match Regex::new(pattern) {
        Ok(regex) => regex,
        Err(e) => {
            return GateResult {
                gate_type: "CommandOutputMatches".to_string(),
                passed: false,
                message: format!("Invalid regex pattern '{}': {}", pattern, e),
            }
        }
    };

    let output = run_shell_command(command, env_root);

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let passed = output.status.success() && regex.is_match(&stdout);
            GateResult {
                gate_type: "CommandOutputMatches".to_string(),
                passed,
                message: format!(
                    "Command '{}' matches pattern '{}': {}",
                    command, pattern, passed
                ),
            }
        }
        Err(e) => GateResult {
            gate_type: "CommandOutputMatches".to_string(),
            passed: false,
            message: format!("Failed to execute command '{}': {}", command, e),
        },
    }
}

fn eval_command_json_path(
    command: &str,
    path: &str,
    assertion: &str,
    env_root: &Path,
) -> GateResult {
    match run_shell_command(command, env_root) {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return GateResult {
                    gate_type: "CommandJsonPath".to_string(),
                    passed: false,
                    message: format!(
                        "Command '{}' failed with exit code {:?}: {}",
                        command,
                        output.status.code(),
                        stderr
                    ),
                };
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let json: Value = match serde_json::from_str(&stdout) {
                Ok(value) => value,
                Err(e) => {
                    return GateResult {
                        gate_type: "CommandJsonPath".to_string(),
                        passed: false,
                        message: format!("Command output is not valid JSON: {}", e),
                    };
                }
            };

            let resolved_value = match resolve_json_path(&json, path) {
                Ok(value) => value,
                Err(e) => {
                    return GateResult {
                        gate_type: "CommandJsonPath".to_string(),
                        passed: false,
                        message: format!("Invalid JSON path '{}': {}", path, e),
                    };
                }
            };

            let (passed, detail) = match evaluate_json_assertion(resolved_value, assertion) {
                Ok(result) => result,
                Err(e) => {
                    return GateResult {
                        gate_type: "CommandJsonPath".to_string(),
                        passed: false,
                        message: format!("Invalid assertion '{}': {}", assertion, e),
                    };
                }
            };

            GateResult {
                gate_type: "CommandJsonPath".to_string(),
                passed,
                message: format!(
                    "Path '{}' with assertion '{}' => {} ({})",
                    path, assertion, passed, detail
                ),
            }
        }
        Err(e) => GateResult {
            gate_type: "CommandJsonPath".to_string(),
            passed: false,
            message: format!("Failed to execute command '{}': {}", command, e),
        },
    }
}

fn eval_file_exists(path: &str, env_root: &Path) -> GateResult {
    let full_path = env_root.join(path);
    let passed = full_path.exists();
    GateResult {
        gate_type: "FileExists".to_string(),
        passed,
        message: format!("File '{}' exists: {}", full_path.display(), passed),
    }
}

fn eval_file_contains(path: &str, substring: &str, env_root: &Path) -> GateResult {
    let full_path = env_root.join(path);
    match std::fs::read_to_string(&full_path) {
        Ok(content) => {
            let passed = content.contains(substring);
            GateResult {
                gate_type: "FileContains".to_string(),
                passed,
                message: format!(
                    "File '{}' contains substring '{}': {}",
                    full_path.display(),
                    substring,
                    passed
                ),
            }
        }
        Err(e) => GateResult {
            gate_type: "FileContains".to_string(),
            passed: false,
            message: format!("Failed to read file '{}': {}", full_path.display(), e),
        },
    }
}

fn eval_file_matches(path: &str, pattern: &str, env_root: &Path) -> GateResult {
    let regex = match Regex::new(pattern) {
        Ok(regex) => regex,
        Err(e) => {
            return GateResult {
                gate_type: "FileMatches".to_string(),
                passed: false,
                message: format!("Invalid regex pattern '{}': {}", pattern, e),
            }
        }
    };

    let full_path = env_root.join(path);
    match std::fs::read_to_string(&full_path) {
        Ok(content) => {
            let passed = regex.is_match(&content);
            GateResult {
                gate_type: "FileMatches".to_string(),
                passed,
                message: format!(
                    "File '{}' matches pattern '{}': {}",
                    full_path.display(),
                    pattern,
                    passed
                ),
            }
        }
        Err(e) => GateResult {
            gate_type: "FileMatches".to_string(),
            passed: false,
            message: format!("Failed to read file '{}': {}", full_path.display(), e),
        },
    }
}

fn run_shell_command(command: &str, env_root: &Path) -> std::io::Result<Output> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(env_root)
        .output()
}

#[derive(Debug)]
enum JsonPathSegment {
    Key(String),
    Index(usize),
}

fn parse_json_path(path: &str) -> std::result::Result<Vec<JsonPathSegment>, String> {
    if !path.starts_with('$') {
        return Err("path must start with '$'".to_string());
    }

    if path == "$" {
        return Ok(Vec::new());
    }

    let chars: Vec<char> = path.chars().collect();
    let mut i = 1;
    let mut segments = Vec::new();

    while i < chars.len() {
        match chars[i] {
            '.' => {
                i += 1;
                let start = i;
                while i < chars.len() && chars[i] != '.' && chars[i] != '[' {
                    i += 1;
                }
                if start == i {
                    return Err("empty object key in path".to_string());
                }
                let key: String = chars[start..i].iter().collect();
                segments.push(JsonPathSegment::Key(key));
            }
            '[' => {
                i += 1;
                let start = i;
                while i < chars.len() && chars[i] != ']' {
                    i += 1;
                }
                if i >= chars.len() || chars[i] != ']' {
                    return Err("unclosed array index bracket".to_string());
                }
                let index_text: String = chars[start..i].iter().collect();
                let index = index_text
                    .parse::<usize>()
                    .map_err(|_| format!("invalid array index '{}'", index_text))?;
                segments.push(JsonPathSegment::Index(index));
                i += 1;
            }
            _ => return Err(format!("unexpected character '{}' in path", chars[i])),
        }
    }

    Ok(segments)
}

fn resolve_json_path<'a>(
    json: &'a Value,
    path: &str,
) -> std::result::Result<Option<&'a Value>, String> {
    let segments = parse_json_path(path)?;
    let mut current = json;

    for segment in segments {
        match segment {
            JsonPathSegment::Key(key) => {
                let Some(next) = current.get(&key) else {
                    return Ok(None);
                };
                current = next;
            }
            JsonPathSegment::Index(index) => {
                let Some(array) = current.as_array() else {
                    return Ok(None);
                };
                let Some(next) = array.get(index) else {
                    return Ok(None);
                };
                current = next;
            }
        }
    }

    Ok(Some(current))
}

fn evaluate_json_assertion(
    value: Option<&Value>,
    assertion: &str,
) -> std::result::Result<(bool, String), String> {
    let trimmed = assertion.trim();

    if trimmed == "exists" {
        let passed = matches!(value, Some(v) if !v.is_null());
        return Ok((passed, "value exists and is not null".to_string()));
    }

    if let Some(expected_text) = trimmed.strip_prefix("equals ") {
        let Some(actual) = value else {
            return Ok((false, "path not found".to_string()));
        };
        let expected = serde_json::from_str::<Value>(expected_text)
            .unwrap_or_else(|_| Value::String(expected_text.to_string()));
        let passed = actual == &expected;
        return Ok((passed, format!("actual={}, expected={}", actual, expected)));
    }

    if let Some(needle) = trimmed.strip_prefix("contains ") {
        let Some(actual) = value else {
            return Ok((false, "path not found".to_string()));
        };
        let Some(text) = actual.as_str() else {
            return Ok((false, "value is not a string".to_string()));
        };
        let passed = text.contains(needle);
        return Ok((passed, format!("substring='{}'", needle)));
    }

    let len_regex = Regex::new(r"^len\s*(>=|==|>)\s*(\d+)$").expect("valid len regex");
    if let Some(captures) = len_regex.captures(trimmed) {
        let Some(actual) = value else {
            return Ok((false, "path not found".to_string()));
        };
        let operator = captures
            .get(1)
            .map(|m| m.as_str())
            .ok_or_else(|| "missing length operator".to_string())?;
        let expected_len = captures
            .get(2)
            .ok_or_else(|| "missing length value".to_string())?
            .as_str()
            .parse::<usize>()
            .map_err(|_| "length must be a non-negative integer".to_string())?;

        let actual_len = if let Some(array) = actual.as_array() {
            array.len()
        } else if let Some(object) = actual.as_object() {
            object.len()
        } else {
            return Ok((false, "value is not an array or object".to_string()));
        };

        let passed = match operator {
            ">=" => actual_len >= expected_len,
            "==" => actual_len == expected_len,
            ">" => actual_len > expected_len,
            _ => return Err(format!("unsupported length operator '{}'", operator)),
        };

        return Ok((
            passed,
            format!("actual_len={} {} {}", actual_len, operator, expected_len),
        ));
    }

    Err("assertion must be one of: exists, equals <value>, contains <substring>, len >= N, len == N, len > N".to_string())
}

fn eval_script(
    command: &str,
    description: &str,
    script_runner: Option<&ScriptRunner>,
) -> GateResult {
    let runner = match script_runner {
        Some(r) => r,
        None => {
            return GateResult {
                gate_type: "Script".to_string(),
                passed: false,
                message: "Script runner not available for script gate evaluation".to_string(),
            };
        }
    };

    let result = match runner.run(command, 30) {
        Ok(r) => r,
        Err(e) => {
            return GateResult {
                gate_type: "Script".to_string(),
                passed: false,
                message: format!("Failed to execute script '{}': {}", command, e),
            };
        }
    };

    if result.timed_out {
        return GateResult {
            gate_type: "Script".to_string(),
            passed: false,
            message: format!("Script '{}' timed out after 30 seconds", command),
        };
    }

    // Try to parse stdout as JSON with {passed, message}
    #[derive(Deserialize)]
    struct ScriptGateOutput {
        passed: bool,
        message: Option<String>,
    }

    let stdout = result.stdout.trim();
    if let Ok(parsed) = serde_json::from_str::<ScriptGateOutput>(stdout) {
        return GateResult {
            gate_type: "Script".to_string(),
            passed: parsed.passed,
            message: parsed.message.unwrap_or_else(|| description.to_string()),
        };
    }

    // Fall back to exit code
    let passed = result.exit_code == 0;
    GateResult {
        gate_type: "Script".to_string(),
        passed,
        message: format!(
            "Script '{}' {} (exit code: {}, description: {})",
            command,
            if passed { "passed" } else { "failed" },
            result.exit_code,
            description
        ),
    }
}

fn eval_no_transcript_errors(
    env_root: &Path,
    target_binary: &str,
    command_pattern: Option<&str>,
) -> GateResult {
    eval_gate!(
        "NoTranscriptErrors",
        crate::eval_helpers::no_transcript_errors(env_root, target_binary, command_pattern),
        |no_errors| (
            no_errors,
            format!("Transcript has no command errors: {}", no_errors)
        )
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreTier {
    Excellent,
    Good,
    Acceptable,
    Poor,
}

impl ScoreTier {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.9 {
            ScoreTier::Excellent
        } else if score >= 0.7 {
            ScoreTier::Good
        } else if score >= 0.5 {
            ScoreTier::Acceptable
        } else {
            ScoreTier::Poor
        }
    }
}

impl fmt::Display for ScoreTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScoreTier::Excellent => write!(f, "Excellent"),
            ScoreTier::Good => write!(f, "Good"),
            ScoreTier::Acceptable => write!(f, "Acceptable"),
            ScoreTier::Poor => write!(f, "Poor"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvaluationMetrics {
    pub gates_passed: usize,
    pub gates_total: usize,
    pub details: Vec<GateResult>,
    pub judge_score: Option<f64>,
    pub judge_response: Option<JudgeResponse>,
    pub efficiency: EfficiencyMetrics,
    pub composite_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GateResult {
    pub gate_type: String,
    pub passed: bool,
    pub message: String,
}

fn evaluate_gates(gates: &[Gate], ctx: &EvaluationContext<'_>) -> (Vec<GateResult>, usize) {
    let mut details = Vec::new();
    let mut gates_passed = 0;

    for gate in gates {
        let result = gate.evaluate(ctx);

        if result.passed {
            println!("Gate {} passed: {}", result.gate_type, result.message);
            gates_passed += 1;
        } else {
            println!("Gate {} FAILED: {}", result.gate_type, result.message);
        }
        details.push(result);
    }

    (details, gates_passed)
}

fn run_judge_evaluation(
    scenario: &Scenario,
    env_root: &Path,
) -> Result<(Option<f64>, Option<JudgeResponse>)> {
    let judge_config = scenario.evaluation.judge.as_ref().unwrap();

    println!("Running LLM-as-judge evaluation...");
    let rubric_path = crate::utils::resolve_fixtures_path(&judge_config.rubric);
    let _rubric = load_rubric(&rubric_path)
        .with_context(|| format!("Failed to load rubric from {}", rubric_path.display()))?;

    let transcript_path = env_root.join("transcript.raw.txt");

    let runner = crate::session::SessionRunner::new();
    let prompt = format!(
        r#"Evaluate this LLM tool interaction.

Task: {}

Files to review:
- @{} - The interaction transcript

Use the rubric at {} for evaluation.

Return evaluation as JSON with this structure:
{{
  "scores": {{
    "criterion_id": <score_0_to_1>,
    ...
  }},
  "weighted_score": <weighted_average_0_to_1>,
  "confidence": <confidence_0_to_1>,
  "issues": ["issue1", "issue2", ...],
  "highlights": ["good_practice1", "good_practice2", ...]
}}

Provide JSON only, no additional text."#,
        scenario.task.prompt,
        transcript_path.display(),
        rubric_path.display()
    );

    let (output, exit_code) = runner
        .run_command("opencode", &["run", &prompt], env_root, 300)
        .context("Judge execution failed")?;

    if exit_code != 0 {
        anyhow::bail!("Judge exited with code {}: {}", exit_code, output);
    }

    let response: JudgeResponse = serde_json::from_str(&output)
        .with_context(|| format!("Failed to parse judge response: {}", output))?;

    println!(
        "Judge score: {:.2} (confidence: {:.2})",
        response.weighted_score, response.confidence
    );
    if !response.issues.is_empty() {
        println!("Issues: {}", response.issues.join(", "));
    }
    if !response.highlights.is_empty() {
        println!("Highlights: {}", response.highlights.join(", "));
    }

    Ok((Some(response.weighted_score), Some(response)))
}

fn maybe_run_judge(
    scenario: &Scenario,
    env_root: &Path,
    no_judge: bool,
) -> Result<(Option<f64>, Option<JudgeResponse>)> {
    if let Some(judge_config) = &scenario.evaluation.judge {
        if judge_config.enabled && !no_judge {
            return run_judge_evaluation(scenario, env_root);
        }
    }
    Ok((None, None))
}

fn compute_efficiency_or_default(
    env_root: &Path,
    target_binary: &str,
    command_pattern: Option<&str>,
) -> EfficiencyMetrics {
    crate::eval_helpers::compute_efficiency_metrics(env_root, target_binary, command_pattern)
        .unwrap_or(EfficiencyMetrics {
            total_commands: 0,
            unique_commands: 0,
            error_count: 0,
            retry_count: 0,
            help_invocations: 0,
            first_try_success_rate: 0.0,
            iteration_ratio: 0.0,
        })
}

fn build_metrics(
    scenario: &Scenario,
    env_root: &Path,
    details: Vec<GateResult>,
    gates_passed: usize,
    judge_score: Option<f64>,
    judge_response: Option<JudgeResponse>,
) -> EvaluationMetrics {
    let efficiency = compute_efficiency_or_default(
        env_root,
        &scenario.target.binary,
        scenario.target.command_pattern.as_deref(),
    );
    let composite_score = crate::eval_helpers::compute_composite_score(
        judge_score,
        gates_passed,
        scenario.evaluation.gates.len(),
        &efficiency,
    );

    EvaluationMetrics {
        gates_passed,
        gates_total: scenario.evaluation.gates.len(),
        details,
        judge_score,
        judge_response,
        efficiency,
        composite_score,
    }
}

pub fn evaluate(
    scenario: &Scenario,
    env_root: &Path,
    no_judge: bool,
    script_runner: Option<&ScriptRunner>,
) -> Result<EvaluationMetrics> {
    println!("Evaluating results for scenario: {}", scenario.name);

    let ctx = EvaluationContext {
        env_root,
        target_binary: &scenario.target.binary,
        command_pattern: scenario.target.command_pattern.as_deref(),
        script_runner,
    };

    let (details, gates_passed) = evaluate_gates(&scenario.evaluation.gates, &ctx);
    let (judge_score, judge_response) = maybe_run_judge(scenario, env_root, no_judge)?;
    let metrics = build_metrics(
        scenario,
        env_root,
        details,
        gates_passed,
        judge_score,
        judge_response,
    );

    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_env() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn command_succeeds_gate_passes_for_successful_command() {
        let env = temp_env();
        let result = eval_command_succeeds("true", env.path());
        assert!(result.passed);
    }

    #[test]
    fn command_succeeds_gate_fails_for_failing_command() {
        let env = temp_env();
        let result = eval_command_succeeds("false", env.path());
        assert!(!result.passed);
    }

    #[test]
    fn command_output_contains_gate_checks_stdout_substring() {
        let env = temp_env();
        let result = eval_command_output_contains("printf 'hello world'", "hello", env.path());
        assert!(result.passed);
    }

    #[test]
    fn command_output_matches_gate_checks_stdout_regex() {
        let env = temp_env();
        let result = eval_command_output_matches("printf 'abc-123'", r"abc-\d+", env.path());
        assert!(result.passed);
    }

    #[test]
    fn command_json_path_gate_supports_exists_assertion() {
        let env = temp_env();
        let result = eval_command_json_path(
            "printf '{\"meta\":{\"ok\":true}}'",
            "$.meta.ok",
            "exists",
            env.path(),
        );
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn command_json_path_gate_supports_equals_assertion() {
        let env = temp_env();
        let result =
            eval_command_json_path("printf '{\"count\":3}'", "$.count", "equals 3", env.path());
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn command_json_path_gate_supports_contains_assertion() {
        let env = temp_env();
        let result = eval_command_json_path(
            "printf '{\"msg\":\"build succeeded\"}'",
            "$.msg",
            "contains succeeded",
            env.path(),
        );
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn command_json_path_gate_supports_len_assertion() {
        let env = temp_env();
        let result = eval_command_json_path(
            "printf '{\"items\":[1,2,3]}'",
            "$.items",
            "len >= 3",
            env.path(),
        );
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn file_exists_gate_checks_relative_path() {
        let env = temp_env();
        fs::write(env.path().join("result.txt"), "ok").expect("write file");

        let result = eval_file_exists("result.txt", env.path());
        assert!(result.passed);
    }

    #[test]
    fn file_contains_gate_checks_file_contents() {
        let env = temp_env();
        fs::write(env.path().join("notes.md"), "status: complete").expect("write file");

        let result = eval_file_contains("notes.md", "complete", env.path());
        assert!(result.passed);
    }

    #[test]
    fn file_matches_gate_checks_file_regex() {
        let env = temp_env();
        fs::write(env.path().join("logs.txt"), "run-42 done").expect("write file");

        let result = eval_file_matches("logs.txt", r"run-\d+", env.path());
        assert!(result.passed);
    }

    #[test]
    fn script_gate_with_exit_code_success() {
        let temp = tempfile::tempdir().unwrap();
        let runner = ScriptRunner::new(
            temp.path().to_path_buf(),
            std::path::PathBuf::from("/tmp/results"),
            "test".to_string(),
            "test_agent".to_string(),
            "test_model".to_string(),
            None,
            None,
            std::collections::HashMap::new(),
        );

        let result = eval_script("true", "should pass", Some(&runner));
        assert!(result.passed, "Exit code 0 should pass: {}", result.message);
    }

    #[test]
    fn script_gate_with_exit_code_failure() {
        let temp = tempfile::tempdir().unwrap();
        let runner = ScriptRunner::new(
            temp.path().to_path_buf(),
            std::path::PathBuf::from("/tmp/results"),
            "test".to_string(),
            "test_agent".to_string(),
            "test_model".to_string(),
            None,
            None,
            std::collections::HashMap::new(),
        );

        let result = eval_script("false", "should fail", Some(&runner));
        assert!(
            !result.passed,
            "Exit code 1 should fail: {}",
            result.message
        );
    }

    #[test]
    fn script_gate_with_json_output() {
        let temp = tempfile::tempdir().unwrap();
        let runner = ScriptRunner::new(
            temp.path().to_path_buf(),
            std::path::PathBuf::from("/tmp/results"),
            "test".to_string(),
            "test_agent".to_string(),
            "test_model".to_string(),
            None,
            None,
            std::collections::HashMap::new(),
        );

        // Script outputs JSON with passed=true
        let result = eval_script(
            "echo '{\"passed\": true, \"message\": \"Custom check passed\"}'",
            "json gate",
            Some(&runner),
        );
        assert!(
            result.passed,
            "JSON passed=true should pass: {}",
            result.message
        );
        assert!(result.message.contains("Custom check passed"));
    }

    #[test]
    fn script_gate_with_json_output_failure() {
        let temp = tempfile::tempdir().unwrap();
        let runner = ScriptRunner::new(
            temp.path().to_path_buf(),
            std::path::PathBuf::from("/tmp/results"),
            "test".to_string(),
            "test_agent".to_string(),
            "test_model".to_string(),
            None,
            None,
            std::collections::HashMap::new(),
        );

        // Script outputs JSON with passed=false
        let result = eval_script(
            "echo '{\"passed\": false, \"message\": \"Custom check failed\"}'",
            "json gate",
            Some(&runner),
        );
        assert!(
            !result.passed,
            "JSON passed=false should fail: {}",
            result.message
        );
        assert!(result.message.contains("Custom check failed"));
    }

    #[test]
    fn script_gate_without_runner_fails() {
        let result = eval_script("true", "no runner", None);
        assert!(!result.passed);
        assert!(result.message.contains("Script runner not available"));
    }
}
