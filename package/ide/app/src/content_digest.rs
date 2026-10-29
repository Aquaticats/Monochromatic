//! The fixed 64-bit FNV-1a digest of some bytes.
//!
//! What:
//!  One function,
//!  shared by the build script (`build.rs` includes this file by path) and the
//!       application,
//!  so a digest recorded at build time is computed the same way at run time.
//! Why:
//!  The single executable records a digest of every embedded language file and checks it before
//!      use,
//!  so a damaged executable or cache is reported instead of loaded.
//!  The standard library's
//!      hasher may change between releases;
//!  FNV-1a is fixed by its definition.
//!  Changing any one byte
//!      always changes the digest,
//!  because every step (XOR with the byte,
//!  then multiplying by an odd
//!      constant modulo 2^64) is reversible.
//! Gotcha:
//!  This detects damage,
//!  not tampering.
//!  Anyone who can rewrite the cache can also rewrite the
//!         executable,
//!  so a keyed or cryptographic digest would not add protection here.

/// What:
///  The 64-bit FNV-1a hash of some bytes.
///  `&[u8]` is a borrowed read-only view of bytes (siblings:
///       `Vec<u8>`,
///  an owned growable list,
///  and `[u8; N]`,
///  a fixed-size array);
///  `u64` is an unsigned
///       64-bit integer (siblings `u32`,
///  `i64`).
///  `wrapping_mul` multiplies and keeps the low 64 bits
///       instead of failing on overflow.
/// Why:
///  A borrowed view lets callers hash embedded bytes and file contents without copying them,
///  and
///      64 bits keep accidental collisions out of reach for the few thousand files involved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fnv1a(bytes: Uint8Array): bigint {
///   let hash = 0xcbf29ce484222325n;
///   for (const byte of bytes) { hash ^= BigInt(byte); hash = (hash * 0x100000001b3n) & 0xffffffffffffffffn; }
///   return hash;
/// }
/// ```
pub const fn fnv1a(bytes: &[u8]) -> u64 {
    // What: `let mut` declares a variable that may be reassigned; the literal is FNV's 64-bit offset basis.
    // Why: Every FNV-1a digest starts from this fixed value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let hash = 0xcbf29ce484222325n;
    // ```
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    // What: `usize` is the index type (siblings `u32`, `u64`); a `while` loop walks the bytes by index.
    // Why: `const fn` (a function the compiler can also run while compiling, so tests can write digests
    //      as constants) does not allow `for` loops over iterators.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (let index = 0; index < bytes.length; index++) { ... }
    // ```
    let mut index: usize = 0;
    while index < bytes.len() {
        // What: `bytes[index] as u64` widens the byte from 8 to 64 bits.
        // Why: The XOR needs both sides at the hash's width.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // hash ^= BigInt(bytes[index]);
        // ```
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        index += 1;
    }
    return hash;
}
