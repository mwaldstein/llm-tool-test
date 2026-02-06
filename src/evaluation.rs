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
            Gate::MinNotes { count } => eval_min_notes(*count, env_root),
            Gate::MinLinks { count } => eval_min_links(*count, env_root),
            Gate::SearchHit { query } => eval_search_hit(query, env_root),
            Gate::NoteExists { id } => eval_note_exists(id, env_root),
            Gate::LinkExists {
                from,
                to,
                link_type,
            } => eval_link_exists(from, to, link_type, env_root),
            Gate::TagExists { tag } => eval_tag_exists(tag, env_root),
            Gate::ContentContains { id, substring } => {
                eval_content_contains(id, substring, env_root)
            }
            Gate::CommandSucceeds { command } => eval_command_succeeds(command, env_root),
            Gate::DoctorPasses => eval_doctor_passes(env_root),
            Gate::NoTranscriptErrors => {
                eval_no_transcript_errors(env_root, target_binary, command_pattern)
            }
        }
    }
}

fn eval_min_notes(count: usize, _env_root: &Path) -> GateResult {
    GateResult {
        gate_type: "MinNotes".to_string(),
        passed: false,
        message: format!(
            "MinNotes gate not implemented (requires qipu). Expected >= {} notes.",
            count
        ),
    }
}

fn eval_min_links(count: usize, _env_root: &Path) -> GateResult {
    GateResult {
        gate_type: "MinLinks".to_string(),
        passed: false,
        message: format!(
            "MinLinks gate not implemented (requires qipu). Expected >= {} links.",
            count
        ),
    }
}

fn eval_search_hit(query: &str, _env_root: &Path) -> GateResult {
    GateResult {
        gate_type: "SearchHit".to_string(),
        passed: false,
        message: format!(
            "SearchHit gate not implemented (requires qipu). Query: '{}'.",
            query
        ),
    }
}

fn eval_note_exists(id: &str, _env_root: &Path) -> GateResult {
    GateResult {
        gate_type: "NoteExists".to_string(),
        passed: false,
        message: format!(
            "NoteExists gate not implemented (requires qipu). Note ID: '{}'.",
            id
        ),
    }
}

fn eval_link_exists(from: &str, to: &str, link_type: &str, _env_root: &Path) -> GateResult {
    GateResult {
        gate_type: "LinkExists".to_string(),
        passed: false,
        message: format!(
            "LinkExists gate not implemented (requires qipu). Link: {} --[{}]--> {}.",
            from, link_type, to
        ),
    }
}

fn eval_tag_exists(tag: &str, _env_root: &Path) -> GateResult {
    GateResult {
        gate_type: "TagExists".to_string(),
        passed: false,
        message: format!(
            "TagExists gate not implemented (requires qipu). Tag: '{}'.",
            tag
        ),
    }
}

fn eval_content_contains(id: &str, substring: &str, _env_root: &Path) -> GateResult {
    GateResult {
        gate_type: "ContentContains".to_string(),
        passed: false,
        message: format!(
            "ContentContains gate not implemented (requires qipu). Note: '{}', substring: '{}'.",
            id, substring
        ),
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

fn eval_doctor_passes(_env_root: &Path) -> GateResult {
    GateResult {
        gate_type: "DoctorPasses".to_string(),
        passed: false,
        message: "DoctorPasses gate not implemented (requires qipu).".to_string(),
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
