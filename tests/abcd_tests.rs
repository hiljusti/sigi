mod run_sigi;

use run_sigi::sigi;

#[test]
fn sigi_abcd_tests() {
    let stack = "_integ::abc";

    let res = sigi(stack, &["delete-all"]);
    res.assert_success();

    // ['a b c']
    let res = sigi(stack, &["push", "a", "b", "c"]);
    res.assert_success();
    res.assert_stdout_eq("Created: a b c\n");
    res.assert_stderr_empty();

    let res = sigi(stack, &["count"]);
    res.assert_success();
    res.assert_stdout_eq("1\n");
    res.assert_stderr_empty();

    let res = sigi(stack, &["delete"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Deleted: a b c", "Now: NOTHING"]);
    res.assert_stderr_empty();

    // ['a']
    let res = sigi(stack, &["push", "a"]);
    res.assert_success();
    res.assert_stdout_eq("Created: a\n");
    res.assert_stderr_empty();

    let res = sigi(stack, &["peek"]);
    res.assert_success();
    res.assert_stdout_eq("Now: a\n");
    res.assert_stderr_empty();

    let res = sigi(stack, &["list"]);
    res.assert_success();
    res.assert_stdout_eq("Now: a\n");
    res.assert_stderr_empty();

    // ['a', 'b']
    let res = sigi(stack, &["push", "b"]);
    res.assert_success();
    res.assert_stdout_eq("Created: b\n");
    res.assert_stderr_empty();

    let res = sigi(stack, &["peek"]);
    res.assert_success();
    res.assert_stdout_eq("Now: b\n");
    res.assert_stderr_empty();

    let res = sigi(stack, &["list"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Now: b", "  1: a"]);
    res.assert_stderr_empty();

    // ['a', 'b', 'c']
    let res = sigi(stack, &["push", "c"]);
    res.assert_success();
    res.assert_stdout_eq("Created: c\n");
    res.assert_stderr_empty();

    let res = sigi(stack, &["peek"]);
    res.assert_success();
    res.assert_stdout_eq("Now: c\n");
    res.assert_stderr_empty();

    let res = sigi(stack, &["list"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Now: c", "  1: b", "  2: a"]);
    res.assert_stderr_empty();

    // ['a', 'b', 'c', 'd']
    let res = sigi(stack, &["push", "d"]);
    res.assert_success();
    res.assert_stdout_eq("Created: d\n");
    res.assert_stderr_empty();

    let res = sigi(stack, &["peek"]);
    res.assert_success();
    res.assert_stdout_eq("Now: d\n");
    res.assert_stderr_empty();

    let res = sigi(stack, &["list"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Now: d", "  1: c", "  2: b", "  3: a"]);
    res.assert_stderr_empty();

    // swap
    let res = sigi(stack, &["swap"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Now: c", "  1: d", "  2: b", "  3: a"]);
    res.assert_stderr_empty();

    let res = sigi(stack, &["swap"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Now: d", "  1: c", "  2: b", "  3: a"]);
    res.assert_stderr_empty();

    // rot
    let res = sigi(stack, &["rot"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Now: c", "  1: b", "  2: d", "  3: a"]);
    res.assert_stderr_empty();

    let res = sigi(stack, &["rot"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Now: b", "  1: d", "  2: c", "  3: a"]);
    res.assert_stderr_empty();

    let res = sigi(stack, &["rot"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Now: d", "  1: c", "  2: b", "  3: a"]);
    res.assert_stderr_empty();

    // next
    let res = sigi(stack, &["next"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Now: c", "  1: b", "  2: a", "  3: d"]);
    res.assert_stderr_empty();

    let res = sigi(stack, &["next"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Now: b", "  1: a", "  2: d", "  3: c"]);
    res.assert_stderr_empty();

    let res = sigi(stack, &["next"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Now: a", "  1: d", "  2: c", "  3: b"]);
    res.assert_stderr_empty();

    let res = sigi(stack, &["next"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Now: d", "  1: c", "  2: b", "  3: a"]);
    res.assert_stderr_empty();

    // removal tests
    let res = sigi(stack, &["delete"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Deleted: d", "Now: c"]);
    res.assert_stderr_empty();

    let res = sigi(stack, &["complete", "1"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Completed: b", "Now: c"]);
    res.assert_stderr_empty();

    let res = sigi(stack, &["complete"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Completed: c", "Now: a"]);
    res.assert_stderr_empty();

    let res = sigi(stack, &["add", "b"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Created: b"]);
    res.assert_stderr_empty();

    let res = sigi(stack, &["add", "c"]);
    res.assert_success();
    res.assert_stdout_lines_eq(&["Created: c"]);
    res.assert_stderr_empty();

    let res = sigi(stack, &["delete-all"]);
    res.assert_success();
    res.assert_stdout_eq("Deleted: 3 items\n");
    res.assert_stderr_empty();
}
