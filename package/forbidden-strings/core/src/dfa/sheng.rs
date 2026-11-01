//! Sheng-style in-register transition kernel:
//!  a faster single-line DFA scan.
//!
//! What:
//!  for a DFA of at most 64 states,
//!  fold each input byte's whole transition column
//! into one 64-byte permute table,
//!  so the per-byte step is a register permute (`vpermb`
//! on AVX-512VBMI,
//!  `vqtbl4q` on NEON) instead of a dependent table load.
//!  Why:
//!  a per-line
//! scan's critical chain is the dependent transition load (~5 cycles);
//!  a permute shortens
//! it,
//!  and needs no length sorting (a different axis from the across-lines batch).
//!  The
//! whole DFA state stays in a vector:
//!  every lane starts at the same state and every step
//! applies the same permute,
//!  so all lanes stay equal and lane 0 is the state.
//! Position-dependent acceptance folds in branchlessly:
//!  each byte accumulates
//! `acc |= accept_mask[state] & ctx_bit[byte]` in-register,
//!  tested once at the end.
//!  The
//! permute is the explicit intrinsic,
//!  not portable SIMD's `swizzle_dyn`,
//!  which scalarizes
//! a 64-byte dynamic shuffle instead of emitting `vpermb`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module sheng: see exported functions and types below.
//! ```

/// What:
///     Imports the AVX-512 permute and bitwise intrinsics for the x86 path.
/// Why:
///      The code below uses `__m512i`,
///  `_mm512_and_si512`,
///  `_mm512_loadu_si512`,
///          `_mm512_or_si512`,
///  `_mm512_permutexvar_epi8`,
///  `_mm512_set1_epi8`,
///          `_mm512_setzero_si512`,
///  `_mm512_test_epi8_mask` directly;
///  importing from
///          `std/arch/x86_64` keeps each call site focused on the matcher logic instead of the
///          full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import {
///   __m512i,
///   _mm512_and_si512,
///   _mm512_loadu_si512,
///   _mm512_or_si512,
///   _mm512_permutexvar_epi8,
///   _mm512_set1_epi8,
///   _mm512_setzero_si512,
///   _mm512_test_epi8_mask,
/// } from "std/arch/x86_64";
/// ```
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m512i, _mm512_and_si512, _mm512_loadu_si512, _mm512_or_si512, _mm512_permutexvar_epi8,
    _mm512_set1_epi8, _mm512_setzero_si512, _mm512_test_epi8_mask,
};

/// What:
///     Imports the NEON table-lookup and reduce intrinsics for the arm64 path.
/// Why:
///      The code below uses `uint8x16_t`,
///  `vandq_u8`,
///  `vdupq_n_u8`,
///  `vld1q_u8_x4`,
///          `vmaxvq_u8`,
///  `vorrq_u8`,
///  `vqtbl4q_u8` directly;
///  importing from `std/arch/aarch64`
///          keeps each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import {
///   uint8x16_t,
///   vandq_u8,
///   vdupq_n_u8,
///   vld1q_u8_x4,
///   vmaxvq_u8,
///   vorrq_u8,
///   vqtbl4q_u8,
/// } from "std/arch/aarch64";
/// ```
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::{
    uint8x16_t, vandq_u8, vdupq_n_u8, vld1q_u8_x4, vmaxvq_u8, vorrq_u8, vqtbl4q_u8,
};

/// What:
///     Imports the DFA table and its per-boundary acceptance-bit helper.
/// Why:
///      The code below uses `Dfa`,
///  `accept_bit` directly;
///  importing from `crate/dfa/table`
///          keeps each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Dfa, accept_bit } from "crate/dfa/table";
/// ```
use crate::dfa::table::{Dfa, accept_bit};

/// What:
///     Largest state count the permute kernel supports (one lane per state).
/// Why:
///      The program gives this fixed value a name so every caller uses the same setting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHENG_MAX_STATES: number = 64;
/// ```
const SHENG_MAX_STATES: usize = 64;

/// Precomputed permute tables for one DFA's Sheng scan.
///
/// What:
///  per input byte a 64-byte next-state column,
///  the per-state acceptance masks,
///  the
/// per-byte acceptance context bit,
///  the start state,
///  and the end-of-input bit.
///  Why:
///  built
/// once per batch call and reused for every line,
///  so the 16 KiB of columns amortizes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShengTables = {
///   // fields documented in Rust above
/// };
/// ```
struct ShengTables {
    /// What:
    ///     `trans[byte][state]` = next state after `byte` from `state` (other lanes
    ///          unused).
    /// Why:
    ///      `trans` stores `trans[byte][state]` = next state after `byte` from `state`
    ///          (other lanes unused),
    ///  so matcher code reads that precomputed state by name
    ///          instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// trans: number[][];
    /// ```
    trans: Vec<[u8; SHENG_MAX_STATES]>,
    /// What:
    ///     `accept[state]` = the state's 4-bit acceptance mask.
    /// Why:
    ///      `accept` stores `accept[state]` = the state's 4-bit acceptance mask,
    ///  so matcher
    ///          code reads that precomputed state by name instead of recomputing or passing it
    ///          separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// accept: number[];
    /// ```
    accept: [u8; SHENG_MAX_STATES],
    /// What:
    ///     `ctx[byte]` = the acceptance bit a match would need given that byte follows.
    /// Why:
    ///      `ctx` stores `ctx[byte]` = the acceptance bit a match would need given that byte
    ///          follows,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ctx: number[];
    /// ```
    ctx: [u8; 256],
    /// What:
    ///     Start state id.
    /// Why:
    ///      `start` stores start state id,
    ///  so matcher code reads that precomputed state by
    ///          name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// start: number;
    /// ```
    start: u8,
    /// What:
    ///     Acceptance bit for the end-of-input boundary.
    /// Why:
    ///      `end_bit` stores acceptance bit for the end-of-input boundary,
    ///  so matcher code
    ///          reads that precomputed state by name instead of recomputing or passing it
    ///          separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// end_bit: number;
    /// ```
    end_bit: u8,
}

/// What:
///     Building the Sheng tables and scanning lines with them.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl Dfa {
    /// Builds the Sheng permute tables,
    ///  or `None` when the DFA has over 64 states.
    ///
    /// What:
    ///  fills a 64-byte next-state column per byte value (folding the byte class in),
    /// the per-state accept masks,
    ///  and the per-byte context bits.
    ///  Why:
    ///  the scan then needs
    /// only `trans[byte]` and a permute per step,
    ///  no class or transition load on the chain.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function build_sheng(): ShengTables | null {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn build_sheng(&self) -> Option<ShengTables> {
        let states = self.num_states as usize;
        if states > SHENG_MAX_STATES {
            return None;
        }
        let nc = self.nclasses as usize;
        let mut trans = vec![[0u8; SHENG_MAX_STATES]; 256];
        let mut ctx = [0u8; 256];
        for byte in 0..256usize {
            let class = self.class_map[byte] as usize;
            ctx[byte] = accept_bit(self.class_word[class], self.class_newline[class]);
            // What:    `state` strides `self.trans` by `state*nc+class`, so there is no single
            //          slice to iterate; clippy's enumerate hint cannot apply.
            // Why:     The surrounding function uses this step to keep the matcher behavior
            //          correct at this point.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Same step as the Rust statement below, written with ordinary TS objects/functions.
            // ```
            #[allow(clippy::needless_range_loop)]
            for state in 0..states {
                trans[byte][state] = self.trans[state * nc + class] as u8;
            }
        }
        let mut accept = [0u8; SHENG_MAX_STATES];
        accept[..states].copy_from_slice(&self.accept[..states]);
        return Some(ShengTables {
            trans,
            accept,
            ctx,
            start: self.start as u8,
            end_bit: accept_bit(false, true),
        })
    }

    /// Fills `out[i]` with whether the DFA matches `lines[i]`,
    ///  via the Sheng kernel.
    ///
    /// What:
    ///  builds the permute tables once (falling back to the scalar batch when the DFA
    /// is too large or the host lacks the permute),
    ///  then scans each line.
    ///  Why:
    ///  the entry
    /// the benchmark and the public hook drive;
    ///  runtime-gated so the wide permute only runs
    /// where its instruction exists.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_match_batch_sheng(lines: Uint8Array[], out: boolean[]): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn is_match_batch_sheng(&self, lines: &[&[u8]], out: &mut [bool]) {
        let Some(tables) = self.build_sheng() else {
            self.is_match_batch_scalar(lines, out);
            return;
        };
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx512vbmi")
                && is_x86_feature_detected!("avx512bw")
                && is_x86_feature_detected!("avx512f")
            {
                // What:    Safety: guarded by the matching runtime feature detection above.
                // Why:     This explains the exact invariant that makes the unsafe Rust
                //          operation valid.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // // Same step as the Rust statement below, written with ordinary TS objects/functions.
                // ```
                unsafe { sheng_all_avx512(&tables, lines, out) };
            } else {
                self.is_match_batch_scalar(lines, out);
            }
        }
        #[cfg(target_arch = "aarch64")]
        for (line, slot) in lines.iter().zip(out.iter_mut()) {
            // What:    Safety: NEON (and vqtbl4q) is baseline on aarch64.
            // Why:     This explains the exact invariant that makes the unsafe Rust operation
            //          valid.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Same step as the Rust statement below, written with ordinary TS objects/functions.
            // ```
            *slot = unsafe { sheng_line_neon(&tables, line) };
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        self.is_match_batch_scalar(lines, out);
    }
}

/// Scans every line with the AVX-512 `vpermb` Sheng kernel.
///
/// What:
///  per line keeps the state in a zmm register,
///  per byte folds the masked acceptance
/// into `acc` and permutes to the next state,
///  then tests the end-of-input boundary.
///  Why:
/// `vpermb` is the only step on the critical chain;
///  the column load and accumulate are off
/// it.
///  The whole loop is one `#[target_feature]` function so every intrinsic is in scope.
///
/// # Safety
///
/// The caller must have confirmed AVX-512F,
///  AVX-512BW,
///  and AVX-512VBMI at runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sheng_all_avx512(tables: ShengTables, lines: Uint8Array[], out: boolean[]): void {
///   // Rust body below is the implementation.
/// }
/// ```
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi")]
unsafe fn sheng_all_avx512(tables: &ShengTables, lines: &[&[u8]], out: &mut [bool]) {
    let accept = unsafe { _mm512_loadu_si512(tables.accept.as_ptr().cast::<__m512i>()) };
    for (line, slot) in lines.iter().zip(out.iter_mut()) {
        let mut state = _mm512_set1_epi8(tables.start as i8);
        let mut acc = _mm512_setzero_si512();
        for &byte in *line {
            let column =
                unsafe { _mm512_loadu_si512(tables.trans[byte as usize].as_ptr().cast::<__m512i>()) };
            let masks = _mm512_permutexvar_epi8(state, accept);
            let ctx = _mm512_set1_epi8(tables.ctx[byte as usize] as i8);
            acc = _mm512_or_si512(acc, _mm512_and_si512(masks, ctx));
            state = _mm512_permutexvar_epi8(state, column);
        }
        let masks = _mm512_permutexvar_epi8(state, accept);
        let end = _mm512_set1_epi8(tables.end_bit as i8);
        acc = _mm512_or_si512(acc, _mm512_and_si512(masks, end));
        *slot = _mm512_test_epi8_mask(acc, acc) != 0;
    }
}

/// Scans one line with the NEON `vqtbl4q` Sheng kernel.
///
/// What:
///  the same accumulate-and-permute scan using a 64-byte NEON table lookup.
///  Why:
///  the
/// arm64 equivalent of `vpermb`;
///  `vqtbl4q_u8` indexes a four-register 64-byte table.
///
/// # Safety
///
/// NEON (and `vqtbl4q`) is baseline on aarch64,
///  so this is always valid there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sheng_line_neon(tables: ShengTables, line: Uint8Array): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn sheng_line_neon(tables: &ShengTables, line: &[u8]) -> bool {
    // What:    Edition 2024: an `unsafe fn` body is not an implicit unsafe context, and every
    //          NEON intrinsic here is unsafe, so the whole scan runs inside one unsafe block.
    // Why:     The surrounding function uses this step to keep the matcher behavior correct at
    //          this point.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    unsafe {
        let accept = vld1q_u8_x4(tables.accept.as_ptr());
        let mut state: uint8x16_t = vdupq_n_u8(tables.start);
        let mut acc: uint8x16_t = vdupq_n_u8(0);
        for &byte in line {
            let column = vld1q_u8_x4(tables.trans[byte as usize].as_ptr());
            let masks = vqtbl4q_u8(accept, state);
            let ctx = vdupq_n_u8(tables.ctx[byte as usize]);
            acc = vorrq_u8(acc, vandq_u8(masks, ctx));
            state = vqtbl4q_u8(column, state);
        }
        let masks = vqtbl4q_u8(accept, state);
        let end = vdupq_n_u8(tables.end_bit);
        acc = vorrq_u8(acc, vandq_u8(masks, end));
        vmaxvq_u8(acc) != 0
    }
}

/// What:
///     Unit tests for the Sheng kernel,
///  in a sidecar (max-lines exempt).
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./tests";
/// ```
#[cfg(test)]
#[path = "sheng_tests.rs"]
mod tests;
