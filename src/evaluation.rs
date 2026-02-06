use crate::judge::{load_rubric, JudgeResponse};
use crate::scenario::{Gate, Scenario};
use crate::transcript::EfficiencyMetrics;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

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

pub trait GateEvaluator {
    fn evaluate(
        &self,
        env_root: &Path,
        target_binary: &str,
        command_pattern: Option<&str>,
    ) -> GateResult;
}

impl GateEvaluator for Gate {
    fn evaluate(
        &self,
        env_root: &Path,
        target_binary: &str,
        command_pattern: Option<&str>,
    ) -> GateResult {
        match self {
            Gate::CommandSucceeds { command } => eval_command_succeeds(command, env_root),
            Gate::CommandOutputContains { command, substring } => {
                eval_command_output_contains(command, substring, env_root)
            }
            Gate::CommandOutputMatches { command, pattern } => {
                eval_command_output_matches(command, pattern, env_root)
            }
            Gate::CommandJsonPath {
                command,
                path,
                assertion,
            } => eval_command_json_path(command, path, assertion, env_root),
            Gate::FileExists { path } => eval_file_exists(path, env_root),
            Gate::FileContains { path, substring } => eval_file_contains(path, substring, env_root),
            Gate::FileMatches { path, pattern } => eval_file_matches(path, pattern, env_root),
            Gate::NoTranscriptErrors => {
                eval_no_transcript_errors(env_root, target_binary, command_pattern)
            }
            Gate::Script {
                command,
                description,
            } => eval_script(command, description),
        }
    }
}

fn eval_command_succeeds(command: &str, env_root: &Path) -> GateResult {
    use std::process::Command;

    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return GateResult {
            gate_type: "CommandSucceeds".to_string(),
            passed: false,
            message: "Empty command".to_string(),
        };
    }

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .current_dir(env_root)
        .output();

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
    use std::process::Command;

    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(env_root)
        .output();

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
    use regex::Regex;
    use std::process::Command;

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

    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(env_root)
        .output();

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
    _env_root: &Path,
) -> GateResult {
    GateResult {
        gate_type: "CommandJsonPath".to_string(),
        passed: false,
        message: format!(
            "CommandJsonPath gate not implemented yet for command '{}', path '{}', assertion '{}'.",
            command, path, assertion
        ),
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
    use regex::Regex;

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

fn eval_script(command: &str, description: &str) -> GateResult {
    GateResult {
        gate_type: "Script".to_string(),
        passed: false,
        message: format!(
            "Script gate not implemented yet for '{}' ({})",
            command, description
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

fn evaluate_gates(
    gates: &[Gate],
    env_root: &Path,
    target_binary: &str,
    command_pattern: Option<&str>,
) -> (Vec<GateResult>, usize) {
    let mut details = Vec::new();
    let mut gates_passed = 0;

    for gate in gates {
        let result = gate.evaluate(env_root, target_binary, command_pattern);

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

pub fn evaluate(scenario: &Scenario, env_root: &Path, no_judge: bool) -> Result<EvaluationMetrics> {
    println!("Evaluating results for scenario: {}", scenario.name);

    let (details, gates_passed) = evaluate_gates(
        &scenario.evaluation.gates,
        env_root,
        &scenario.target.binary,
        scenario.target.command_pattern.as_deref(),
    );
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
