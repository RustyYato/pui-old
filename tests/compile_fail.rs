use trybuild;

#[test]
fn compile_fail() {
    let cases = trybuild::TestCases::new();

    cases.compile_fail("tests/compile_fail/*.rs");
    cases.compile_fail("tests/compile_fail/scoped/*.rs");

    if cfg!(feature = "std") {
        cases.compile_fail("tests/compile_fail/thread_local/*.rs");
    } else {
        cases.compile_fail("tests/compile_fail/no_std/*/*.rs");
    }
}
