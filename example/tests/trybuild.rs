#[test]
fn examples() {
    let t = trybuild::TestCases::new();
    t.compile_fail("examples/compile_fail/*.rs");
    t.pass("examples/pass/*.rs");
}
