//! What:
//!  Rewrite the committed `file-enforcer.schema.json` at the repository root from the config types.
//! Why:
//!  The schema is derived output;
//! `mise run //package/file-enforcer/core-rust:schema:write` regenerates it,
//! and the
//!  golden test fails until it does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeFile('file-enforcer.schema.json', configSchemaText());
//! ```

/// Imports `config_schema_text` from `monochromatic_file_enforcer::schema`.
use monochromatic_file_enforcer::schema::config_schema_text;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// What:
///  Write the schema text to the repository root,
/// two directories above this package's parent.
/// Why:
///  `CARGO_MANIFEST_DIR` is `package/file-enforcer/core-rust`,
/// so three `..` reach the root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function main(): void
/// ```
fn main() {
    let target: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../file-enforcer.schema.json");
    std::fs::write(&target, config_schema_text()).expect("the schema file is writable");
    println!("wrote {}", target.display());
}
