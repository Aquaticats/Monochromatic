//! What:
//! The accepted-specifier grammar:
//! which text a dependency may hold in an install section or a peer section.
//! Why:
//! The rule is syntax only (no catalog lookup),
//! so it fits in two small functions that every consumer shares.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // grammar.ts
//! export type SectionKind = 'install' | 'peer';
//! export function isAllowedSpecifier(kind: SectionKind, specifier: string): boolean { /* ... */ }
//! ```

/// What:
/// `pub enum SectionKind { Install, Peer }` is a type that is exactly one of two named cases.
/// `#[derive(...)]` asks the compiler to write standard code:
/// `Clone` and `Copy` (duplicate cheaply like a number),
/// `Debug` (printable in tests),
/// `Eq` and `PartialEq` (comparable with `==`).
/// Why:
/// Install sections and the peer section accept different literals,
/// and a two-case type makes that choice explicit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type SectionKind = 'install' | 'peer';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SectionKind {
    /// `dependencies`,
    /// `devDependencies` and `optionalDependencies`:
    /// also accept exactly `workspace:*`.
    Install,
    /// `peerDependencies`:
    /// also accept exactly `*`.
    Peer,
}

/// What:
/// `fn is_catalog_name_character(character: char) -> bool` is a private function over one `char`.
/// `char` is one Unicode character (4 bytes,
/// not a UTF-16 unit like a TS string element).
/// `&str` is a borrowed text view.
/// Why:
/// The catalog-name grammar is "one or more of ASCII letters,
/// digits,
/// `.`,
/// `_`,
/// `-`",
/// and this is the per-character test.
/// `char` and not `u8` because Rust iterates text by character,
/// and `u8` would let a multi-byte character through in pieces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isCatalogNameCharacter(character: string): boolean { return /^[A-Za-z0-9._-]$/.test(character); }
/// ```
fn is_catalog_name_character(character: char) -> bool {
    // What: `character.is_ascii_alphanumeric()` is true for A to Z, a to z and 0 to 9 only, never for accented letters.
    // `||` is logical or, the same as in TS. The last line has no `;`, so it is the returned value.
    // Why: Non-ASCII letters such as `é` must stay findings, because pnpm's own catalog-name rule is looser than ours.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return /[A-Za-z0-9]/.test(character) || character === '.' || character === '_' || character === '-';
    // ```
    return character.is_ascii_alphanumeric() || character == '.' || character == '_' || character == '-';
}

/// What:
/// `fn is_catalog_reference(specifier: &str) -> bool` is a private function.
/// `&str` borrows the text read-only.
/// Why:
/// Both section kinds accept `catalog:` followed by an optional name,
/// so the test lives once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isCatalogReference(specifier: string): boolean { return /^catalog:[A-Za-z0-9._-]*$/.test(specifier); }
/// ```
fn is_catalog_reference(specifier: &str) -> bool {
    // What: `specifier.strip_prefix("catalog:")` returns `Some(rest)` when the text starts with `catalog:` and `None` otherwise.
    // `if let Some(name) = ...` runs the block only for `Some`, binding the remainder to `name`.
    // Why: Case matters (`Catalog:` is refused) and nothing is trimmed, so the exact prefix is stripped and the rest is checked.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (specifier.startsWith('catalog:')) { const name = specifier.slice('catalog:'.length); /* ... */ }
    // ```
    if let Some(name) = specifier.strip_prefix("catalog:") {
        // What: `name.chars()` walks the text one `char` at a time; `.all(is_catalog_name_character)` is true when the
        // function returns true for every one, and also true for an empty remainder, which is exactly the bare `catalog:`.
        // Why: Bare `catalog:` selects the default catalog, and a named reference needs every character to be legal.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return [...name].every(isCatalogNameCharacter);
        // ```
        return name.chars().all(is_catalog_name_character);
    }
    return false;
}

/// What:
/// `pub fn is_allowed_specifier(kind: SectionKind, specifier: &str) -> bool` is the public test.
/// It takes the section kind
/// by value (`Copy`,
/// so free) and the text by borrow.
/// Why:
/// Exported so tests and callers that already hold a decoded string reach the rule without building a manifest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isAllowedSpecifier(kind: SectionKind, specifier: string): boolean
/// ```
pub fn is_allowed_specifier(kind: SectionKind, specifier: &str) -> bool {
    if is_catalog_reference(specifier) {
        return true;
    }
    // What: `match kind { ... }` picks one branch per case of `SectionKind`; the compiler checks both are covered.
    // `==` between two `&str` compares the text, exactly like `===` on TS strings.
    // Why: Install sections also accept exactly `workspace:*` and peers also accept exactly `*`; nothing else is allowed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return kind === 'install' ? specifier === 'workspace:*' : specifier === '*';
    // ```
    return match kind {
        SectionKind::Install => specifier == "workspace:*",
        SectionKind::Peer => specifier == "*",
    };
}
