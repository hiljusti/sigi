use std::io::{Read, Write};
use std::process::{Command, Output, Stdio};

pub const SIGI_PATH: &str = std::env!("CARGO_BIN_EXE_sigi");

pub fn sigi(stack: &str, args: &[&str]) -> SigiOutput {
    return Command::new(SIGI_PATH)
        .arg("--stack")
        .arg(stack)
        .args(args)
        .output()
        .expect("Error running process")
        .into();
}

pub fn piping(lines: &[&str]) -> SigiInput {
    SigiInput {
        stdin: lines.iter().map(|s| s.to_string()).collect(),
    }
}

pub struct SigiInput {
    stdin: Vec<String>,
}

impl SigiInput {
    pub fn into_sigi(self, stack: &str, args: &[&str]) -> SigiOutput {
        let stdin = self.stdin.join("\n");

        let process = Command::new(SIGI_PATH)
            .arg("--stack")
            .arg(stack)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Error running process");

        process
            .stdin
            .expect("Error sending stdin to sigi")
            .write_all(stdin.as_bytes())
            .unwrap();

        let mut stdout = String::new();
        process.stdout.unwrap().read_to_string(&mut stdout).unwrap();

        let mut stderr = String::new();
        process.stderr.unwrap().read_to_string(&mut stderr).unwrap();

        SigiOutput {
            status: SigiStatus::Unknown,
            stdout,
            stderr,
        }
    }
}

pub struct SigiOutput {
    status: SigiStatus,
    stdout: String,
    stderr: String,
}

#[derive(Debug, PartialEq)]
enum SigiStatus {
    Success,
    Failure,
    Unknown,
}

impl From<bool> for SigiStatus {
    fn from(b: bool) -> Self {
        match b {
            true => SigiStatus::Success,
            false => SigiStatus::Failure,
        }
    }
}

impl SigiOutput {
    pub fn assert_success(&self) {
        assert_eq!(self.status, SigiStatus::Success);
    }

    pub fn assert_failure(&self) {
        assert_eq!(self.status, SigiStatus::Failure);
    }

    pub fn assert_stdout_eq(&self, expected_stdout: &str) {
        assert_eq!(
            &self.stdout,
            expected_stdout,
            "sigi stdout did not exactly match expectation.\n{}",
            self.stdout_for_errors()
        );
    }

    pub fn assert_stdout_line_eq(&self, expected_line: &str) {
        let some_line_eqs = self.stdout.lines().any(|line| line == expected_line);
        assert!(
            some_line_eqs,
            "sigi stdout had no line matching: `{:?}`\n{}",
            expected_line,
            self.stdout_for_errors()
        );
    }

    pub fn assert_stdout_lines_eq(&self, expected_lines: &[&str]) {
        self.stdout
            .lines()
            .zip(expected_lines.iter())
            .enumerate()
            .for_each(|(i, (actual, expected))| {
                if *expected == "*" {
                    return;
                }
                assert_eq!(
                    &actual,
                    expected,
                    "sigi stdout did not match expected output `{}` on line {}\n{}",
                    expected_lines.join("\n"),
                    i,
                    self.stdout_for_errors()
                )
            });
    }

    pub fn assert_stdout_line_starts_with(&self, expected_prefix: &str) {
        let some_line_eqs = self
            .stdout
            .lines()
            .any(|line| line.starts_with(expected_prefix));
        assert!(
            some_line_eqs,
            "sigi stdout had no line starting with: {}\n{}",
            expected_prefix,
            self.stdout_for_errors()
        );
    }

    pub fn assert_stderr_empty(&self) {
        assert_eq!(
            &self.stderr,
            "",
            "sigi stderr was expected to be empty.\n{}",
            self.stderr_for_errors()
        );
    }

    fn stdout_for_errors(&self) -> String {
        format!(
            "===\nstdout:\n===\n{}\n===\n",
            self.stdout.lines().collect::<Vec<_>>().join("\n")
        )
    }

    fn stderr_for_errors(&self) -> String {
        format!(
            "===\nstderr:\n===\n{}\n===\n",
            self.stderr.lines().collect::<Vec<_>>().join("\n")
        )
    }
}

impl From<Output> for SigiOutput {
    fn from(output: Output) -> SigiOutput {
        SigiOutput {
            status: if output.status.success() {
                SigiStatus::Success
            } else {
                SigiStatus::Failure
            },
            stdout: String::from_utf8(output.stdout).expect("Couldn't read stdout"),
            stderr: String::from_utf8(output.stderr).expect("Couldn't read stderr"),
        }
    }
}

#[test]
fn assert_success() {
    let output = SigiOutput {
        status: true.into(),
        stdout: String::new(),
        stderr: String::new(),
    };

    output.assert_success();
}

#[test]
fn assert_failure() {
    let output = SigiOutput {
        status: false.into(),
        stdout: String::new(),
        stderr: String::new(),
    };

    output.assert_failure();
}

#[test]
fn assert_stdout_eq() {
    let output = SigiOutput {
        status: true.into(),
        stdout: "hello".to_string(),
        stderr: String::new(),
    };

    output.assert_stdout_eq("hello");
}

#[test]
fn assert_stdout_line_eq() {
    let output = SigiOutput {
        status: true.into(),
        stdout: "hey\nhello".to_string(),
        stderr: String::new(),
    };

    output.assert_stdout_line_eq("hello");
}

#[test]
fn assert_stdout_lines_eq() {
    let output = SigiOutput {
        status: true.into(),
        stdout: "hey\nhello there".to_string(),
        stderr: String::new(),
    };

    output.assert_stdout_lines_eq(&["hey", "hello there"]);
}

#[test]
fn assert_stdout_line_starts_with() {
    let output = SigiOutput {
        status: true.into(),
        stdout: "hey\nhello there".to_string(),
        stderr: String::new(),
    };

    output.assert_stdout_line_starts_with("hello");
}

#[test]
fn assert_stderr_empty() {
    let output = SigiOutput {
        status: true.into(),
        stdout: "hey\nhello there".to_string(),
        stderr: String::new(),
    };

    output.assert_stderr_empty();
}

#[test]
fn sigi_piping_basic() {
    let res = piping(&[]).into_sigi("_integ::basic", &["interactive"]);
    assert_eq!(res.status, SigiStatus::Unknown);
    res.assert_stdout_line_starts_with("sigi 3.7");
    res.assert_stderr_empty();
}
