//! What:
//!  The SHA-256 digest of one byte buffer,
//!  as lowercase hexadecimal text.
//! Why:
//!  A smudged Git LFS file's object id is by definition the SHA-256 of its bytes,
//! and this crate has no approved hashing dependency.
//! The algorithm is FIPS 180-4 section 6.2;
//!  its fixed vectors are checked in the sibling test file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // createHash('sha256').update(bytes).digest('hex')
//! ```

/// What:
///  The 64 round constants,
///  the first 32 bits of the fractional parts of the cube roots of the first 64 primes.
/// Why:
///  FIPS 180-4 section 4.2.2 fixes these values;
///  a wrong constant changes every digest.
/// `u32` (not `u64` or `usize`) because SHA-256 is defined over 32-bit words on every platform.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ROUND = new Uint32Array([0x428a2f98, /* ... */]);
/// ```
const ROUND: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// What:
///  The initial hash value,
///  the first 32 bits of the fractional parts of the square roots of the first 8 primes.
/// Why:
///  FIPS 180-4 section 5.3.3 fixes the starting state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const INITIAL = new Uint32Array([0x6a09e667, /* ... */]);
/// ```
const INITIAL: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

/// What:
///  The FIPS 180-4 `Ch` function:
///  each result bit is the `y` bit where the `x` bit is 1,
/// and the `z` bit where the `x` bit is 0.
/// Why:
///  FIPS spells it `(x & y) ^ (!x & z)`.
///  Those two halves never share a set bit,
///  so in that
/// spelling `^` and `|` give the same value and replacing one with the other changes no digest.
/// This spelling is the same function,
///  and each of its operators changes some result bit when replaced,
/// so the fixed vectors notice any such change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function choose(x: number, y: number, z: number): number { return (z ^ (x & (y ^ z))) >>> 0; }
/// ```
fn choose(x: u32, y: u32, z: u32) -> u32 {
    return z ^ (x & (y ^ z));
}

/// What:
///  The FIPS 180-4 `Maj` function:
///  each result bit is the value at least two of the three input bits share.
/// Why:
///  FIPS spells it `(x & y) ^ (x & z) ^ (y & z)`,
///  where either `^` can become `|` without changing
/// the value,
///  because at most one or all three of the terms are set.
///  Here `x` and `y` are the majority
/// where they agree,
///  and `z` breaks the tie where they differ;
///  replacing any operator changes some result bit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function majority(x: number, y: number, z: number): number { return (y ^ ((x ^ y) & (y ^ z))) >>> 0; }
/// ```
fn majority(x: u32, y: u32, z: u32) -> u32 {
    return y ^ ((x ^ y) & (y ^ z));
}

/// What:
///  Mix one 64-byte block into the running eight-word state.
/// Why:
///  The digest is this compression applied to every padded block in order.
/// `&mut [u32; 8]` lends the caller's state for in-place update;
///  `&[u8]` lends the block read-only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compress(state: Uint32Array, block: Uint8Array): void;
/// ```
fn compress(state: &mut [u32; 8], block: &[u8]) {
    // The message schedule: 16 big-endian words from the block, then 48 derived words.
    let mut schedule: [u32; 64] = [0; 64];
    // as_chunks::<4>() views the block as its 16 four-byte groups; the unused remainder is empty.
    let (groups, _): (&[[u8; 4]], &[u8]) = block.as_chunks::<4>();
    for (index, group) in groups.iter().enumerate() {
        schedule[index] = u32::from_be_bytes(*group);
    }
    for index in 16..64 {
        let before: u32 = schedule[index - 15];
        let recent: u32 = schedule[index - 2];
        let small_zero: u32 = before.rotate_right(7) ^ before.rotate_right(18) ^ (before >> 3);
        let small_one: u32 = recent.rotate_right(17) ^ recent.rotate_right(19) ^ (recent >> 10);
        // wrapping_add is addition modulo 2^32; plain `+` would panic on overflow in debug builds.
        schedule[index] = schedule[index - 16]
            .wrapping_add(small_zero)
            .wrapping_add(schedule[index - 7])
            .wrapping_add(small_one);
    }
    // Copy the eight working variables; arrays of u32 copy by value.
    let mut work: [u32; 8] = *state;
    for index in 0..64 {
        let big_one: u32 =
            work[4].rotate_right(6) ^ work[4].rotate_right(11) ^ work[4].rotate_right(25);
        let chosen: u32 = choose(work[4], work[5], work[6]);
        let first: u32 = work[7]
            .wrapping_add(big_one)
            .wrapping_add(chosen)
            .wrapping_add(ROUND[index])
            .wrapping_add(schedule[index]);
        let big_zero: u32 =
            work[0].rotate_right(2) ^ work[0].rotate_right(13) ^ work[0].rotate_right(22);
        let agreed: u32 = majority(work[0], work[1], work[2]);
        let second: u32 = big_zero.wrapping_add(agreed);
        work[7] = work[6];
        work[6] = work[5];
        work[5] = work[4];
        work[4] = work[3].wrapping_add(first);
        work[3] = work[2];
        work[2] = work[1];
        work[1] = work[0];
        work[0] = first.wrapping_add(second);
    }
    for index in 0..8 {
        state[index] = state[index].wrapping_add(work[index]);
    }
}

/// What:
///  Hash a complete in-memory buffer and return 64 lowercase hexadecimal characters.
/// Why:
///  Git LFS object ids use exactly this spelling;
///  the caller compares and embeds them as text.
/// `String` (not `&str`) because the digest is newly built and outlives this call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sha256Hex(bytes: Uint8Array): string;
/// ```
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut state: [u32; 8] = INITIAL;
    // What: `as_chunks::<64>()` splits the input into a borrowed list of whole 64-byte arrays, `&[[u8; 64]]`,
    // and the borrowed remainder shorter than one block, `&[u8]`; `::<64>` names the block length.
    // Why: Whole blocks come straight from the input without copying, and the standard library computes
    // both parts, so there is no hand-stepped offset whose mutation could spin and no boundary arithmetic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const whole = bytes.length - (bytes.length % 64);
    // const blocks = range(0, whole, 64).map((offset) => bytes.subarray(offset, offset + 64));
    // const rest = bytes.subarray(whole);
    // ```
    let (blocks, rest): (&[[u8; 64]], &[u8]) = bytes.as_chunks::<64>();
    // Each `&[u8; 64]` block is passed where a `&[u8]` slice is expected; Rust converts the reference itself.
    for block in blocks {
        compress(&mut state, block);
    }
    // The tail holds the remaining bytes, the 0x80 marker, zero padding and the 64-bit bit length.
    let mut tail: [u8; 128] = [0; 128];
    tail[..rest.len()].copy_from_slice(rest);
    tail[rest.len()] = 0x80;
    // One block suffices only when the marker and the 8 length bytes both fit after the remainder.
    let tail_length: usize = if rest.len() < 56 { 64 } else { 128 };
    // `as u64` widens losslessly on every supported platform; the length is counted in bits.
    let bits: u64 = (bytes.len() as u64).wrapping_mul(8);
    tail[tail_length - 8..tail_length].copy_from_slice(&bits.to_be_bytes());
    compress(&mut state, &tail[..64]);
    if tail_length == 128 {
        compress(&mut state, &tail[64..128]);
    }
    let mut output: String = String::with_capacity(64);
    for word in state {
        output.push_str(format!("{word:08x}").as_str());
    }
    return output;
}

/// Fixed FIPS vectors and block-boundary lengths stay outside release artifacts.
#[cfg(test)]
#[path = "markdown_lfs_sha256_tests.rs"]
mod tests;
