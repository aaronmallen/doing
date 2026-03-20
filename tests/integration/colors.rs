use predicates::prelude::*;

use crate::helpers::DoingCmd;

#[test]
fn it_displays_section_headers() {
    let doing = DoingCmd::new();

    doing
        .run(["colors"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Foreground Colors:"))
        .stdout(predicate::str::contains("Bold/Bright Colors:"))
        .stdout(predicate::str::contains("Background Colors:"))
        .stdout(predicate::str::contains("Bold/Bright Backgrounds:"))
        .stdout(predicate::str::contains("Modifiers:"))
        .stdout(predicate::str::contains("Themes:"))
        .stdout(predicate::str::contains("Reset/Default:"));
}

#[test]
fn it_displays_hex_syntax_help() {
    let doing = DoingCmd::new();

    doing
        .run(["colors"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hex RGB syntax:"))
        .stdout(predicate::str::contains("%#FF5500"))
        .stdout(predicate::str::contains("%bg#00FF00"));
}

#[test]
fn it_displays_token_names() {
    let doing = DoingCmd::new();

    doing
        .run(["colors"])
        .assert()
        .success()
        .stdout(predicate::str::contains("%red"))
        .stdout(predicate::str::contains("%blue"))
        .stdout(predicate::str::contains("%bold"))
        .stdout(predicate::str::contains("%reset"));
}
