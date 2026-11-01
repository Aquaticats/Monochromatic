//! Differential controls for the configuration text reader:
//!  whenever it answers, `git config --file <file> --list -z` prints the same entries;
//!  whenever Git refuses the file, the reader declines.

use super::*;
use crate::test_support::{fixture, git_output, remove};
use std::path::Path;

/// Every spelling the controls feed both readers; the expectation is Git's own output, not a literal.
const CORPUS: &[&str] = &[
    "[core]\n\tbare = false\n\trepositoryformatversion = 0\n",
    "[core]\nbare\n",
    "[Core]\n  Bare = TRUE\n",
    "[remote \"origin\"]\n\turl = https://example.invalid/a.git\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n",
    "[remote \"Mixed.Case\"]\nurl = u\n",
    "[branch \"feature/x\"]\n\tremote = origin\n\tmerge = refs/heads/feature/x\n",
    "[section.legacy]\nkey = value\n",
    "[core] bare = true\n",
    "# comment\n; other\n[a]\nb = c # trailing comment\nd = \"quoted # kept\" ; tail\n",
    "[a]\nb = one \\\ntwo\n",
    "[a]\nb = \"a\\tb\\n\\\\ \\\"q\\\"\"\n",
    "[a]\nb =   spaced   inside   value   \n",
    "[a]\nb =\n",
    "[a]\nb\nc = 1\n",
    "[a]\r\nb = crlf\r\nc\r\n",
    "\u{feff}[a]\nb = bom\n",
    "[a \"sub\\\"quote\"]\nk = v\n",
    "[a \"back\\\\slash\"]\nk = v\n",
    "[a]\nb = c\nb = d\n",
    "[include]\npath = ~/other\n[includeIf \"gitdir:/x/\"]\npath = ../y\n",
    "[a]\nbad = \"open\n",
    "[a]\nbad = \\q\n",
    "[a\n",
    "b = no section\n",
    "[a]\n1bad = x\n",
    "[a]\nk = v\n[]\n",
    "[a b]\nk = v\n",
    "[a]\nk = é\n",
    "[a]\nk = v\n\n\n[b]\nk = w",
];

/// The entries Git prints for `file`, as `key`, `key\nvalue` pieces, or `None` when Git refuses the file.
fn from_git(directory: &Path, file: &Path) -> Option<Vec<Vec<u8>>> {
    let output = git_output(directory, &["config", "--file", file.to_str().expect("UTF-8 path"), "--list", "-z"]);
    if !output.status.success() {
        return None;
    }
    let mut pieces: Vec<Vec<u8>> = Vec::new();
    for piece in output.stdout.split(|byte| { return *byte == 0; }) {
        if !piece.is_empty() {
            pieces.push(piece.to_vec());
        }
    }
    return Some(pieces);
}

/// The reader's entries spelled as Git prints them.
fn spelled(entries: &[ConfigEntry]) -> Vec<Vec<u8>> {
    return entries
        .iter()
        .map(|entry| {
            let mut piece: Vec<u8> = entry.key.as_bytes().to_vec();
            if let Some(value) = &entry.value {
                piece.push(b'\n');
                piece.extend_from_slice(value);
            }
            return piece;
        })
        .collect();
}

/// Every corpus text is either read exactly as Git reads it or declined.
#[test]
fn the_reader_agrees_with_git_or_declines() {
    let root = fixture("repo-read-config-text");
    let mut answered: usize = 0;
    for (index, text) in CORPUS.iter().enumerate() {
        let file = root.join(format!("config-{index}"));
        std::fs::write(&file, text.as_bytes()).expect("write corpus file");
        let git_view = from_git(root.as_path(), file.as_path());
        if let Some(entries) = parse_config_text(text.as_bytes()) {
            answered += 1;
            assert_eq!(Some(spelled(entries.as_slice())), git_view, "corpus {index}: {text:?}");
        }
        if git_view.is_none() {
            assert!(parse_config_text(text.as_bytes()).is_none(), "corpus {index} is refused by Git but read: {text:?}");
        }
    }
    // Positive control: the reader must answer the plain files, or the loop above proves nothing.
    assert!(answered * 2 > CORPUS.len(), "the reader declined too much of the corpus to prove anything");
    remove(root.as_path());
}

/// A NUL byte in a value cannot be a C string, so the reader declines it.
#[test]
fn a_nul_in_a_value_is_declined() {
    assert!(parse_config_text(b"[a]\nb = left\0right\n").is_none());
    assert!(parse_config_text(b"[a]\nb = fine\n").is_some());
}

/// A spelling Git reads and the reader leaves to Git is still safe, but the plain keys must be read.
#[test]
fn plain_repository_keys_are_read() {
    let entries = parse_config_text(b"[core]\n\tbare = false\n[remote \"origin\"]\n\turl = u\n").expect("plain file");
    assert_eq!(parse_bool(last_entry(entries.as_slice(), "core.bare").and_then(|entry| { return entry.value.as_deref(); })), Some(false));
    assert_eq!(last_entry(entries.as_slice(), "remote.origin.url").and_then(|entry| { return entry.value.clone(); }), Some(b"u".to_vec()));
}

/// Boolean spellings Git accepts are read, every other spelling is declined.
#[test]
fn booleans_decline_unknown_spellings() {
    assert_eq!(parse_bool(None), Some(true));
    assert_eq!(parse_bool(Some(b"Yes")), Some(true));
    assert_eq!(parse_bool(Some(b"off")), Some(false));
    assert_eq!(parse_bool(Some(b"")), None);
    assert_eq!(parse_bool(Some(b"2")), None);
}

/// A file above the size limit is declined without reading it.
#[test]
fn an_oversized_file_is_declined() {
    let big: Vec<u8> = vec![b'#'; CONFIG_TEXT_LIMIT + 1];
    assert!(parse_config_text(big.as_slice()).is_none());
}
