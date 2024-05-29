mod run_sigi;

use run_sigi::{piping, sigi};

#[test]
fn sigi_interactive_preamble() {
    let res = sigi("_integ::interactive", &["interactive"]);
    res.assert_success();
    res.assert_stdout_line_starts_with("sigi 3.7");
    res.assert_stdout_line_starts_with(r#"Type "exit", "quit", or "q" to quit"#);
    res.assert_stdout_line_starts_with(
        r#"Type "?" for quick help, or "help" for a more verbose help message"#,
    );
    res.assert_stderr_empty();
}

#[test]
fn sigi_interactive_basic() {
    let res = piping(&["push hello world"]).into_sigi("_integ::interactive", &["interactive"]);
    res.assert_stdout_line_starts_with("sigi 3.7");
    res.assert_stdout_line_starts_with(r#"Type "exit", "quit", or "q" to quit"#);
    res.assert_stdout_line_starts_with(
        r#"Type "?" for quick help, or "help" for a more verbose help message"#,
    );
    res.assert_stdout_line_starts_with("Created: hello world");
    res.assert_stdout_line_starts_with("Ctrl+d: Buen biåhe!");
    res.assert_stderr_empty();
}

#[test]
fn sigi_interactive_basic_semicolons() {
    let res = piping(&["push goodbye; push hello; drop; drop"])
        .into_sigi("_integ::interactive_semicolons", &["interactive"]);

    res.assert_stderr_empty();
    res.assert_stdout_lines_eq(&[
        "*",
        r#"Type "exit", "quit", or "q" to quit. (On Unixy systems, Ctrl+C or Ctrl+D also work)"#,
        r#"Type "?" for quick help, or "help" for a more verbose help message."#,
        "",
        "Created: goodbye",
        "Created: hello",
        "Deleted: hello",
        "Now: goodbye",
        "Deleted: goodbye",
        "Now: NOTHING",
        "Ctrl+d: Buen biåhe!",
    ]);
}
