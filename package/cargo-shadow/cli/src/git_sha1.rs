//! What:
//! SHA-1,
//! only to check that a stored Git object has the name it is stored under.
//! Why:
//! Round 6 (fifth review 5a F3):
//! an object file whose contents hash to another name is not the object Git would read
//! (Git checks the signature before it peels),
//! so the reader must check it too or decline.
//! This is an integrity check of data the repository owner wrote,
//! not a defence against a collision attack:
//! a mismatch only
//! makes the reader decline and leave the answer to Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { createHash } from 'node:crypto';
//! export const sha1 = (...parts: Uint8Array[]): Uint8Array => createHash('sha1').update(...).digest();
//! ```

/// The bytes in one block of SHA-1 input.
const BLOCK_BYTES: usize = 64;

/// The bytes in a SHA-1 digest.
pub(crate) const DIGEST_BYTES: usize = 20;

/// What:
/// `fn compress(state: &mut [u32; 5], block: &[u8])` mixes one 64-byte block into the state.
/// Why:
/// It is the round function of SHA-1 (FIPS 180-4,
/// section 6.1.2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compress(state: Uint32Array, block: Uint8Array): void
/// ```
fn compress(state: &mut [u32; 5], block: &[u8]) {
    let mut words: [u32; 80] = [0; 80];
    for (index, chunk) in block.as_chunks::<4>().0.iter().enumerate() {
        words[index] = u32::from_be_bytes(*chunk);
    }
    for index in 16..80 {
        words[index] = (words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16]).rotate_left(1);
    }
    let [mut a, mut b, mut c, mut d, mut e] = *state;
    for (index, word) in words.iter().enumerate() {
        let (mix, constant): (u32, u32) = match index {
            0..=19 => ((b & c) | (!b & d), 0x5a82_7999),
            20..=39 => (b ^ c ^ d, 0x6ed9_eba1),
            40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc),
            _ => (b ^ c ^ d, 0xca62_c1d6),
        };
        let next: u32 = a.rotate_left(5).wrapping_add(mix).wrapping_add(e).wrapping_add(constant).wrapping_add(*word);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = next;
    }
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
}

/// What:
/// `pub(crate) fn sha1(parts: &[&[u8]]) -> [u8; DIGEST_BYTES]` is the SHA-1 digest of the parts joined in order.
/// Why:
/// An object's name is the digest of its header and its body,
/// which are held apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sha1(parts: Uint8Array[]): Uint8Array
/// ```
pub(crate) fn sha1(parts: &[&[u8]]) -> [u8; DIGEST_BYTES] {
    let mut state: [u32; 5] = [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476, 0xc3d2_e1f0];
    let mut pending: Vec<u8> = Vec::with_capacity(BLOCK_BYTES * 2);
    let mut length: u64 = 0;
    for part in parts {
        length += part.len() as u64;
        pending.extend_from_slice(part);
        let whole: usize = pending.len() / BLOCK_BYTES * BLOCK_BYTES;
        for block in pending[..whole].as_chunks::<BLOCK_BYTES>().0 {
            compress(&mut state, block);
        }
        pending.drain(..whole);
    }
    pending.push(0x80);
    while pending.len() % BLOCK_BYTES != BLOCK_BYTES - 8 {
        pending.push(0);
    }
    pending.extend_from_slice(&(length * 8).to_be_bytes());
    for block in pending.as_chunks::<BLOCK_BYTES>().0 {
        compress(&mut state, block);
    }
    let mut digest: [u8; DIGEST_BYTES] = [0; DIGEST_BYTES];
    for (slot, word) in digest.as_chunks_mut::<4>().0.iter_mut().zip(state) {
        slot.copy_from_slice(&word.to_be_bytes());
    }
    return digest;
}

/// What:
/// `pub(crate) fn object_name(kind: &str, body: &[u8]) -> [u8; DIGEST_BYTES]` is the name Git gives an object:
/// the digest of `<kind> <length>\0` and the body.
/// Why:
/// Comparing it with the name an object is stored under shows that the stored bytes are that object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function objectName(kind: string, body: Uint8Array): Uint8Array
/// ```
pub(crate) fn object_name(kind: &str, body: &[u8]) -> [u8; DIGEST_BYTES] {
    let header: String = format!("{kind} {}\0", body.len());
    return sha1(&[header.as_bytes(), body]);
}
