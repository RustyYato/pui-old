use trybuild;

#[test]
fn compile_fail() {
    let cases = trybuild::TestCases::new();

    cases.compile_fail("tests/compile_fail/scoped/*.rs");

    if !cfg!(feature = "std") {
        cases.compile_fail("tests/compile_fail/no_std/*/*.rs");
    }
}
