//! What:
//!  The binary tests for the two ways a config file can be wrong.
//! Why:
//! A config that does not parse,
//! and a config whose fields are wrong,
//! each get one line naming the file and the position of the fault,
//! and both exit two.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe("the two ways a config file can be wrong", () => { it("...", () => { ... }); });
//! ```

use crate::support::{
    Harness,
    config_path,
};

#[test]
fn a_config_syntax_error_is_exit_two_with_path_line_and_column() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", "{\n  \"rules\": [,]\n}");
    let (code, lines) = harness.run(&["check"]);
    assert_eq!(code, 2);
    assert_eq!(lines[0]["code"], "config-syntax");
    let message = lines[0]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("file-enforcer.config.jsonc:2:"),
        "{message}"
    );
    assert!(config_path(&harness).exists());
}

#[test]
fn a_config_field_error_is_exit_two_with_path_line_column_and_pointer() {
    let harness = Harness::new();
    harness.write(
        "file-enforcer.config.jsonc",
        "{\n  \"rules\": [\n    {\"kind\": \"mirror\", \"id\": \"m\", \"from\": \"a\", \"to\": \"/abs\"}\n  ]\n}",
    );
    let (code, lines) = harness.run(&["check"]);
    assert_eq!(code, 2, "{lines:?}");
    assert_eq!(lines[0]["code"], "config-invalid");
    let message = lines[0]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("file-enforcer.config.jsonc:3:54: /rules/0/to: "),
        "{message}"
    );
}
