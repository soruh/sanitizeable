#[test]
fn all() {
    let t = trybuild::TestCases::new();
    t.compile_fail("test/compile_fail/*");
    t.pass("test/pass/*");
}