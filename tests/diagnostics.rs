use std::path::PathBuf;

#[test]
fn run_valid_fixtures() {
    let valid_fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/valid");
    for valid_fixture in std::fs::read_dir(valid_fixtures_dir).unwrap() {
        let valid_fixture = valid_fixture.unwrap();
        let name = valid_fixture.file_name();
        dbg!(format!(
            "Running fixture {} ({})",
            name.to_str().unwrap(),
            valid_fixture.path().to_str().unwrap()
        ));
        let output = latch_rs::check(valid_fixture.path());
        assert_eq!(output, "Check done - 0 errors found");
    }
}

#[test]
fn run_invalid_fixtures() {
    let valid_fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/invalid");
    for valid_fixture in std::fs::read_dir(valid_fixtures_dir).unwrap() {
        let valid_fixture = valid_fixture.unwrap();
        let name = valid_fixture.file_name();
        let snapshot_name = name.to_str().unwrap().strip_suffix(".latch").unwrap();
        dbg!(format!(
            "Running fixture {} ({})",
            name.to_str().unwrap(),
            valid_fixture.path().to_str().unwrap()
        ));
        let output = latch_rs::check(valid_fixture.path());
        let mut settings = insta::Settings::clone_current();
        settings.add_filter(r"[^\s]*/fixtures/", "fixtures/");
        settings.bind(|| insta::assert_snapshot!(snapshot_name, output));
    }
}
