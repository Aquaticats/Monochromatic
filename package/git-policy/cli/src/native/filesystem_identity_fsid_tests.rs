//! Controls for the in-process `f_fsid` spelling.

use super::f_fsid_text;
use std::path::Path;

/// What GNU `stat --file-system --format=%i` prints for `path`, trimmed.
fn stat_text(path: &Path) -> String {
    let output: std::process::Output = std::process::Command::new("stat")
        .arg("--file-system")
        .arg("--format=%i")
        .arg(path)
        .output()
        .expect("stat starts");
    assert!(output.status.success(), "stat refused {}", path.display());
    return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
}

/// The in-process text equals what `stat` prints, on a disk path, on a pseudo filesystem and on the root.
#[test]
fn the_text_equals_what_stat_prints() {
    let package: &Path = Path::new(env!("CARGO_MANIFEST_DIR"));
    for path in [package, Path::new("/"), Path::new("/proc"), Path::new("/dev")] {
        if !path.exists() {
            println!("skipped: {} is absent here", path.display());
            continue;
        }
        assert_eq!(f_fsid_text(path).as_deref(), Some(stat_text(path).as_str()), "{}", path.display());
    }
}

/// A path the kernel refuses has no text, and a path with an interior NUL never reaches the call.
#[test]
fn a_refused_path_has_no_text() {
    assert_eq!(f_fsid_text(Path::new("/nonexistent/entry/for/statfs")), None);
    assert_eq!(f_fsid_text(Path::new("/pro\0c")), None);
}

/// The text is lowercase hexadecimal with no leading zero, the shape the identity normalizer accepts unchanged.
#[test]
fn the_text_is_plain_lowercase_hexadecimal() {
    let text: String = f_fsid_text(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("a text");
    assert!(!text.is_empty());
    assert!(text.chars().all(|character: char| { return character.is_ascii_digit() || ('a'..='f').contains(&character); }), "{text}");
    assert!(text == "0" || !text.starts_with('0'), "{text}");
}
