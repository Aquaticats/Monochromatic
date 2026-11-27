//! What:
//! The identity text this shadow carries inside its own program file:
//! `MONOCHROMATIC-SHADOW-IDENTITY/1 <crate> <version>`,
//! placed in an ELF section of its own
//! (`.monochromatic_shadow_identity`),
//! and the function that reads back what this program says it is.
//! Why:
//! A candidate is judged by what its file is and never by its path or its version,
//! so every shadow carries the same words,
//! and a program that carries them is a shadow of any version and under any name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const identity = 'MONOCHROMATIC-SHADOW-IDENTITY/1 ' + PACKAGE_NAME + ' ' + PACKAGE_VERSION;
//! function ownIdentity(): string { return identity.slice(IDENTITY_PREFIX.length); }
//! ```

/// The name of the ELF section that holds the identity text.
pub const IDENTITY_SECTION: &str = ".monochromatic_shadow_identity";

/// The first words of every identity text;
/// what follows is the crate's name and version.
pub const IDENTITY_PREFIX: &str = "MONOCHROMATIC-SHADOW-IDENTITY/1 ";

/// The identity text this program carries,
/// written in full so that the compiler places it in the section below.
const CARRIED_TEXT: &str = concat!(
    "MONOCHROMATIC-SHADOW-IDENTITY/1 ",
    env!("CARGO_PKG_NAME"),
    " ",
    env!("CARGO_PKG_VERSION")
);

/// Copies a text into an array of its own length,
/// which a `static` needs.
const fn copy_into<const LENGTH: usize>(text: &str) -> [u8; LENGTH] {
    let source: &[u8] = text.as_bytes();
    let mut copy: [u8; LENGTH] = [0; LENGTH];
    let mut index: usize = 0;
    while index < LENGTH {
        copy[index] = source[index];
        index += 1;
    }
    return copy;
}

/// What:
/// The identity text as bytes,
/// placed in its own section of the program file.
/// Why:
/// Every shadow carries it,
/// so a candidate on `PATH` that carries it is a shadow whatever its path,
/// name or version.
/// `#[used]` keeps the linker from dropping a value nothing reads.
#[used]
#[unsafe(link_section = ".monochromatic_shadow_identity")]
static CARRIED: [u8; CARRIED_TEXT.len()] = copy_into(CARRIED_TEXT);

/// What:
/// `pub fn own_identity() -> &'static str` is what this program says it is:
/// the crate name,
/// a space and the version.
/// Why:
/// The arrival marker and the messages name the shadow with it.
/// It reads the carried bytes,
/// so the section is referenced and kept in every build.
pub fn own_identity() -> &'static str {
    let carried: &[u8] = std::hint::black_box(&CARRIED);
    return std::str::from_utf8(&carried[IDENTITY_PREFIX.len()..]).unwrap_or("a shadow");
}
