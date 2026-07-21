use std::process::Command;

use ccusage_test_support::{Fixture, fs_fixture};

fn codex_fixture() -> Fixture {
    fs_fixture!({
        "codex/sessions/session.jsonl": r#"{"timestamp":"2026-01-02T00:00:00.000Z","type":"event_msg","payload":{"type":"token_count","info":{"model":"gpt-credit-test","last_token_usage":{"input_tokens":100,"cached_input_tokens":0,"output_tokens":0,"reasoning_output_tokens":0,"total_tokens":100}}}}"#,
        "ccusage.json": r#"{"defaults":{"pricingOverrides":{"gpt-credit-test":{"inputCostPerToken":0.0004,"outputCostPerToken":0.0}}}}"#,
    })
}

fn run_codex_table(fixture: &Fixture, report: &str, extra_args: &[&str], columns: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_ccusage"))
        .arg("codex")
        .arg(report)
        .args(extra_args)
        .arg("--config")
        .arg(fixture.path("ccusage.json"))
        .args([
            "--offline",
            "--no-color",
            "--single-thread",
            "--timezone",
            "UTC",
            "--speed",
            "standard",
        ])
        .env("CODEX_HOME", fixture.path("codex"))
        .env("COLUMNS", columns)
        .env("LOG_LEVEL", "0")
        .current_dir(fixture.root())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "ccusage failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn snapshots_codex_credit_tables_for_each_focused_report() {
    let fixture = codex_fixture();

    insta::assert_snapshot!(format!(
        "DAILY\n{}MONTHLY\n{}SESSION\n{}",
        run_codex_table(&fixture, "daily", &[], "200"),
        run_codex_table(&fixture, "monthly", &[], "200"),
        run_codex_table(&fixture, "session", &[], "200"),
    ));
}

#[test]
fn no_cost_hides_codex_credit_and_usd_columns() {
    let fixture = codex_fixture();

    let output = run_codex_table(&fixture, "daily", &["--no-cost"], "200");

    assert!(!output.contains("Credits"));
    assert!(!output.contains("Cost (USD)"));
    assert!(!output.contains("1.00"));
    assert!(!output.contains("$0.04"));
}

#[test]
fn codex_credit_table_fits_configured_terminal_width() {
    let fixture = codex_fixture();

    let output = run_codex_table(&fixture, "daily", &[], "120");

    let widest_line = output
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap();
    assert!(widest_line <= 120, "widest line was {widest_line} columns");
}
