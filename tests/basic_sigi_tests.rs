mod run_sigi;

use run_sigi::sigi;

#[test]
fn sigi_version() {
    let res = sigi("_integ::version", &["--version"]);
    res.assert_success();
    res.assert_stdout_line_starts_with("sigi 3.7");
    res.assert_stderr_empty();
}
