#![cfg_attr(miri, ignore)]

#[test]
fn integration() {
    let t = trybuild::TestCases::new();
    t.pass("tests/integration/*.rs");
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
