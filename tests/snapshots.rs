use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

fn run_lox(file: &str) -> String {
    let mut cmd = Command::cargo_bin("loxide").unwrap();
    cmd.arg("--plain")
        .arg(format!("tests/fixtures/{}.lox", file));

    let output = cmd.output().unwrap();
    let mut result = String::new();
    result.push_str(&String::from_utf8_lossy(&output.stdout));
    result.push_str(&String::from_utf8_lossy(&output.stderr));
    result
}

macro_rules! test_snapshot {
    ($id:ident) => {
        #[test]
        fn $id() {
            let file = stringify!($id);
            let mut settings = insta::Settings::clone_current();
            settings.set_snapshot_path("fixtures/");
            settings.set_prepend_module_to_snapshot(false);
            settings.set_input_file(file);
            settings.bind(|| {
                insta::assert_snapshot!(run_lox(file));
            })
        }
    };
}

test_snapshot!(blocks);
test_snapshot!(variables);
test_snapshot!(conditional);
