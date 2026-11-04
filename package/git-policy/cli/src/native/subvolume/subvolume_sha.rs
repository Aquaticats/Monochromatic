//! What:
//!  SHA-1 and SHA-256 of a byte string, written out in full.
//! Why:
//!  The last bytes of a Git index file are a checksum of everything before them (SHA-1, or SHA-256 in a repository of
//!  that object format), and Git refuses an index whose checksum is wrong. The index stat rewrite of the subvolume
//!  migration must write a new checksum. The crate has no hash function, a new dependency was not shown to be needed
//!  for two short functions, and the controls compare both functions with the published test vectors and, in the
//!  migration controls, with the checksum real Git accepts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { createHash } from 'node:crypto';
//! const sha1 = (bytes: Uint8Array) => createHash('sha1').update(bytes).digest();
//! ```

/// The bytes in one block of both functions.
const BLOCK: usize = 64;

/// The SHA-256 round constants: the first 32 bits of the fractional
///  parts of the cube roots of the first 64 primes (FIPS 180-4, section 4.2.2).
const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01,
    0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
    0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08,
    0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// What:
///  The message padded as both functions require: a 1 bit, zeros, and the bit length as 64 big-endian bits, to a
///  multiple of 64 bytes.
///  `Vec<u8>` is an owned byte list (sibling `&[u8]` is a borrowed view).
/// Why:
///   Both functions hash the padded message block by block.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pad(bytes: Uint8Array): Uint8Array;
/// ```
fn padded(bytes: &[u8]) -> Vec<u8> {
    let mut message: Vec<u8> = bytes.to_vec();
    message.push(0x80);
    while message.len() % BLOCK != BLOCK - 8 {
        message.push(0);
    }
    let bit_length: u64 = (bytes.len() as u64).wrapping_mul(8);
    message.extend_from_slice(&bit_length.to_be_bytes());
    return message;
}

/// What:
///  The SHA-1 digest (20 bytes) of `bytes`.
///  `[u8; 20]` is a fixed-size byte array (sibling `Vec<u8>` is growable).
/// Why:
///   The checksum of a SHA-1 repository's index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// createHash('sha1').update(bytes).digest()
/// ```
pub fn sha1(bytes: &[u8]) -> [u8; 20] {
    let mut state: [u32; 5] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];
    // The whole blocks are folded in place; only the last partial block is copied and padded, so a multi-megabyte index is
    //  not copied to be hashed.
    let whole: usize = bytes.len() / BLOCK * BLOCK;
    for block in bytes[..whole].as_chunks::<BLOCK>().0 {
        sha1_block(&mut state, block);
    }
    let mut tail: Vec<u8> = bytes[whole..].to_vec();
    tail.push(0x80);
    while tail.len() % BLOCK != BLOCK - 8 {
        tail.push(0);
    }
    tail.extend_from_slice(&(bytes.len() as u64).wrapping_mul(8).to_be_bytes());
    for block in tail.as_chunks::<BLOCK>().0 {
        sha1_block(&mut state, block);
    }
    let mut digest: [u8; 20] = [0; 20];
    for (index, word) in state.iter().enumerate() {
        digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    return digest;
}

/// One SHA-1 round: `vars` holds the working variables a to e, and `mixed` is the round's boolean function of b, c and d.
fn sha1_round(vars: &mut [u32; 5], mixed: u32, constant: u32, word: u32) {
    let [a, b, c, d, e] = *vars;
    let next: u32 = a.rotate_left(5).wrapping_add(mixed).wrapping_add(e).wrapping_add(constant).wrapping_add(word);
    *vars = [next, a, b.rotate_left(30), c, d];
}

/// Fold one 64-byte block into the SHA-1 state, with the four groups of twenty rounds written out so no round chooses its function.
fn sha1_block(state: &mut [u32; 5], block: &[u8; BLOCK]) {
    let mut words: [u32; 80] = [0; 80];
    for (slot, quad) in words.iter_mut().zip(block.as_chunks::<4>().0) {
        *slot = u32::from_be_bytes(*quad);
    }
    for index in 16..80 {
        words[index] = (words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16]).rotate_left(1);
    }
    let mut vars: [u32; 5] = *state;
    for word in &words[..20] {
        let [_, b, c, d, _] = vars;
        sha1_round(&mut vars, (b & c) | (!b & d), 0x5a827999, *word);
    }
    for word in &words[20..40] {
        let [_, b, c, d, _] = vars;
        sha1_round(&mut vars, b ^ c ^ d, 0x6ed9eba1, *word);
    }
    for word in &words[40..60] {
        let [_, b, c, d, _] = vars;
        sha1_round(&mut vars, (b & c) | (b & d) | (c & d), 0x8f1bbcdc, *word);
    }
    for word in &words[60..] {
        let [_, b, c, d, _] = vars;
        sha1_round(&mut vars, b ^ c ^ d, 0xca62c1d6, *word);
    }
    for (slot, add) in state.iter_mut().zip(vars) {
        *slot = slot.wrapping_add(add);
    }
}

/// What:
///  The SHA-256 digest (32 bytes) of `bytes`.
/// Why:
///   The checksum of a SHA-256 repository's index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// createHash('sha256').update(bytes).digest()
/// ```
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut state: [u32; 8] =
        [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    for block in padded(bytes).chunks(BLOCK) {
        let mut words: [u32; 64] = [0; 64];
        for (index, quad) in block.chunks(4).enumerate() {
            words[index] = u32::from_be_bytes([quad[0], quad[1], quad[2], quad[3]]);
        }
        for index in 16..64 {
            let low: u32 = words[index - 15].rotate_right(7) ^ words[index - 15].rotate_right(18) ^ (words[index - 15] >> 3);
            let high: u32 = words[index - 2].rotate_right(17) ^ words[index - 2].rotate_right(19) ^ (words[index - 2] >> 10);
            words[index] = words[index - 16].wrapping_add(low).wrapping_add(words[index - 7]).wrapping_add(high);
        }
        let mut working: [u32; 8] = state;
        for index in 0..64 {
            let [a, b, c, d, e, f, g, h] = working;
            let sum1: u32 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice: u32 = (e & f) ^ (!e & g);
            let first: u32 = h.wrapping_add(sum1).wrapping_add(choice).wrapping_add(K256[index]).wrapping_add(words[index]);
            let sum0: u32 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority: u32 = (a & b) ^ (a & c) ^ (b & c);
            let second: u32 = sum0.wrapping_add(majority);
            working = [first.wrapping_add(second), a, b, c, d.wrapping_add(first), e, f, g];
        }
        for (slot, add) in state.iter_mut().zip(working) {
            *slot = slot.wrapping_add(add);
        }
    }
    let mut digest: [u8; 32] = [0; 32];
    for (index, word) in state.iter().enumerate() {
        digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    return digest;
}

/// SHA control vectors stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_sha_tests.rs"]
mod tests;
