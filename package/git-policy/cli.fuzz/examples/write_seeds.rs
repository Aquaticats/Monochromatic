//! What:
//!  Write every seed of `seed/wrapper_controls/` from `git_policy_cli_fuzz::seeds`,
//!  and print what each one asks for and spells.
//! Why:
//!  A seed's first bytes are the repository answers of the frontier view and the rest is its argument list,
//!  so the file is written from those two,
//!  never from a byte count a person keeps in step with the mode's width.
//!  `mise run //package/git-policy/cli.fuzz:seed:write` runs this,
//!  and `controls::tests::every_seed_spells_its_scenario` fails while a file here disagrees with the table.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const seed of SEEDS) writeFile(`seed/wrapper_controls/${seed.name}`, seed.bytes());
//! ```

/// The width of a mode, which is how many hex digits it takes.
use git_policy_cli_fuzz::controls::MODE_BYTES;
/// The seeds and the one encoding of them.
use git_policy_cli_fuzz::seeds::SEEDS;
/// Writing the files beside this package.
use std::fs;
/// The package's own directory, whatever the working directory is.
use std::path::{Path, PathBuf};

/// The two hex digits one byte takes.
const HEX_DIGITS_PER_BYTE: usize = 2;

fn main() {
    // Where the seeds live: beside this package's manifest, never the working directory.
    let directory: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("seed")
        .join("wrapper_controls");
    // How many digits a mode takes, which follows its own width.
    let digits: usize = MODE_BYTES * HEX_DIGITS_PER_BYTE;
    for seed in SEEDS {
        // The seed's bytes, which is the one encoding the control judges too.
        let bytes: Vec<u8> = seed.bytes();
        // The file those bytes belong in.
        let path: PathBuf = directory.join(seed.name);
        fs::write(path.as_path(), bytes.as_slice()).unwrap_or_else(|error| {
            panic!("the seed {path:?} could not be written: {error}")
        });
        // The answers as a reader wants them, one pair of digits per byte.
        let mode: String = format!("{:0width$x}", seed.mode(), width = digits);
        // The arguments as a reader wants them, with a byte no text holds shown as the character it is.
        let arguments: Vec<String> = seed
            .arguments
            .iter()
            .map(|argument| format!("{:?}", String::from_utf8_lossy(argument)))
            .collect();
        println!("{}: mode 0x{mode}, arguments [{}]", seed.name, arguments.join(", "));
    }
}
