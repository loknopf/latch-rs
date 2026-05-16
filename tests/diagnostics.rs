use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/invalid")
        .join(name)
}

fn check_snapshot(snapshot_name: &str, fixture_name: &str) {
    let output = latch_rs::check(fixture(fixture_name));
    let mut settings = insta::Settings::clone_current();
    settings.add_filter(r"[^\s]*/fixtures/", "fixtures/");
    settings.bind(|| insta::assert_snapshot!(snapshot_name, output));
}

#[test]
fn field_name_collision() {
    check_snapshot("field_name_collision", "field_name_collision.latch");
}

#[test]
fn field_overlap() {
    check_snapshot("field_overlap", "field_overlap.latch");
}

#[test]
fn parse_error_missing_name() {
    check_snapshot("parse_error_missing_name", "parse_error.latch");
}
