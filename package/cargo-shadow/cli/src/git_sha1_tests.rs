//! What:
//! Tests of the SHA-1 used for object names:
//! published vectors,
//! split inputs,
//! and Git itself as the oracle.
//! Why:
//! A wrong digest would decline objects Git accepts,
//! or worse,
//! confirm objects it rejects.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('sha1', () => { it('matches the FIPS 180 vectors', ...) })
//! ```

use std::io::Write;
use std::process::{Command, Stdio};

use crate::git_sha1::{object_name, sha1};

/// The digest as the 40 hexadecimal digits Git prints.
fn hex(digest: &[u8]) -> String {
    return digest.iter().map(|byte| { return format!("{byte:02x}"); }).collect();
}

/// FIPS 180 vector:
/// the empty message.
#[test]
fn the_digest_of_nothing_is_the_published_one() {
    assert_eq!(hex(&sha1(&[])), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
}

/// FIPS 180 vector:
/// "abc".
#[test]
fn the_digest_of_abc_is_the_published_one() {
    assert_eq!(hex(&sha1(&[b"abc"])), "a9993e364706816aba3e25717850c26c9cd0d89d");
}

/// FIPS 180 vector:
/// a message of two blocks.
#[test]
fn the_digest_of_a_two_block_message_is_the_published_one() {
    let message: &[u8] = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    assert_eq!(hex(&sha1(&[message])), "84983e441c3bd26ebaae4aa1f95129e5e54670f1");
}

/// The same bytes in different parts have the same digest.
#[test]
fn the_digest_does_not_depend_on_how_the_input_is_split() {
    let bytes: Vec<u8> = (0..1000_u32).map(|number| { return u8::try_from(number % 251).unwrap_or(0); }).collect();
    let whole = sha1(&[&bytes]);
    let split = sha1(&[&bytes[..63], &bytes[63..64], &bytes[64..500], &bytes[500..]]);
    assert_eq!(whole, split);
}

/// Git is the oracle for the name of a commit body.
#[test]
fn an_object_name_is_the_name_git_gives() {
    let body: &[u8] = b"tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904\nauthor A <a@b> 1 +0000\ncommitter A <a@b> 1 +0000\n\nmessage\n";
    let mut git = Command::new("/usr/bin/git").args(["hash-object", "-t", "commit", "--literally", "--stdin"]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect("git");
    git.stdin.take().expect("stdin").write_all(body).expect("write");
    let output = git.wait_with_output().expect("git ends");

    assert_eq!(hex(&object_name("commit", body)), String::from_utf8_lossy(&output.stdout).trim());
}
