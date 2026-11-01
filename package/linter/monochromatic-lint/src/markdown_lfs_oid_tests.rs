//! What:
//!  Object-id spelling,
//!  pointer and digest controls.
//! Why:
//!  The id written into an object URL is frozen by `package/cli/markdown-lint/src/lfs-oid.unit.test.ts`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe(lfsOidOfFile.name, () => { /* smudged bytes, pointer, malformed pointer */ });
//! ```

/// Import the id operations under test and the digest used as the independent expectation.
use super::{is_lfs_oid, lfs_oid_of_bytes};
use crate::markdown_lfs_sha256::sha256_hex;

/// Declared oid used by the pointer fixtures.
const DECLARED: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

/// Exactly 64 lowercase hexadecimal characters are an id;
///  everything else is not.
#[test]
fn only_sixty_four_lowercase_hex_characters_are_an_oid() {
    assert!(is_lfs_oid(DECLARED));
    assert!(is_lfs_oid(
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    ));
    assert!(!is_lfs_oid(DECLARED.to_uppercase().as_str()));
    assert!(!is_lfs_oid(&DECLARED[1..]));
    assert!(!is_lfs_oid(format!("{DECLARED}c").as_str()));
    assert!(!is_lfs_oid(format!("g{}", &DECLARED[1..]).as_str()));
    assert!(!is_lfs_oid(format!("/{}", &DECLARED[1..]).as_str()));
    assert!(!is_lfs_oid(format!(":{}", &DECLARED[1..]).as_str()));
    assert!(!is_lfs_oid(format!("`{}", &DECLARED[1..]).as_str()));
    assert!(!is_lfs_oid(""));
}

/// Smudged bytes hash to their id;
///  a pointer yields the id it declares.
#[test]
fn smudged_bytes_are_hashed_and_pointers_are_read() {
    let bytes: &[u8] = b"PNG not really";
    // Measured independently with coreutils `sha256sum`.
    assert_eq!(
        lfs_oid_of_bytes(bytes),
        "28ea29e30bc216a87c8301cfb5ff987e4e2d17f1e5695b1ceb02a430ed453741"
    );
    let pointer: String =
        format!("version https://git-lfs.github.com/spec/v1\noid sha256:{DECLARED}\nsize 12\n");
    assert_eq!(lfs_oid_of_bytes(pointer.as_bytes()), DECLARED);
    let crlf: String = format!(
        "version https://git-lfs.github.com/spec/v1\r\noid sha256:{DECLARED}\r\nsize 12\r\n"
    );
    assert_eq!(lfs_oid_of_bytes(crlf.as_bytes()), DECLARED);
}

/// A pointer-like file without a well-formed oid line is hashed as ordinary bytes.
#[test]
fn malformed_pointers_are_hashed() {
    let malformed: &str = "version https://git-lfs.github.com/spec/v1\noid sha256:nope\nsize 1\n";
    assert_eq!(
        lfs_oid_of_bytes(malformed.as_bytes()),
        sha256_hex(malformed.as_bytes())
    );
    let headerless: String = format!("oid sha256:{DECLARED}\n");
    assert_eq!(
        lfs_oid_of_bytes(headerless.as_bytes()),
        sha256_hex(headerless.as_bytes())
    );
    let indented: String =
        format!("version https://git-lfs.github.com/spec/v1\n oid sha256:{DECLARED}\n");
    assert_eq!(
        lfs_oid_of_bytes(indented.as_bytes()),
        sha256_hex(indented.as_bytes())
    );
}

/// The first well-formed oid line wins,
///  and invalid UTF-8 after the header does not prevent pointer reading.
#[test]
fn the_first_well_formed_oid_line_is_selected() {
    let second: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    let pointer: String = format!(
        "version https://git-lfs.github.com/spec/v1\noid sha256:nope\noid sha256:{DECLARED}\noid sha256:{second}\n"
    );
    assert_eq!(lfs_oid_of_bytes(pointer.as_bytes()), DECLARED);
    let mut bytes: Vec<u8> = Vec::<u8>::from(
        format!("version https://git-lfs.github.com/spec/v1\noid sha256:{DECLARED}\n").as_bytes(),
    );
    bytes.push(0xff);
    assert_eq!(lfs_oid_of_bytes(bytes.as_slice()), DECLARED);
    let binary: [u8; 4] = [0xff, 0xfe, 0x00, 0x01];
    assert_eq!(lfs_oid_of_bytes(&binary), sha256_hex(&binary));
}
