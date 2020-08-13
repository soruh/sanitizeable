#[test]
fn examples() {
    let t = trybuild::TestCases::new();
    t.compile_fail("test/compile_fail/*.rs");
    t.pass("test/pass/*.rs");
}
