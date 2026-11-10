//! What:
//!      This Rust module adds batch methods to [`Regex`] and [`RegexSet`].
//!  A Rust
//!           module is closest to a private TypeScript file inside a package.
//!  The public
//!           methods return `Vec<bool>` values (owned growable arrays of booleans,
//!  not
//!           borrowed `&[bool]` slices or fixed `[bool; N]` arrays),
//!  and the hidden hooks
//!           let the benchmark force scalar,
//!  interleaved,
//!  tight,
//!  and Sheng layouts.
//! Why:
//!      This file is the Rust module that groups the batch implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module batch: see exported functions and types below.
//! ```

/// Imports the per-line checks and the first-seen set that the batch resolution uses.
use super::line_checks::{any_group_matches, any_line_start_match, CheckedFull, RuleCheck, RuleCollector};

/// What:
///     Imports the public matcher types this module extends.
/// Why:
///      The code below uses `Regex`,
///  `RegexSet` directly;
///  importing from
///          `./super` keeps each call site focused on the matcher logic instead of the full Rust
///          path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Regex, RegexSet } from "./super";
/// ```
use super::{Regex, RegexSet};

/// What:
///     Imports the anchored line-start match used by the per-line resolution.
/// Why:
///      The code below uses `line_start_match` directly;
///  importing from `crate/build` keeps
///          each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { line_start_match } from "crate/build";
/// ```
use crate::build::line_start_match;

/// Lines advanced together per exact-length bucket in [`Regex::is_match_batch_bucketed`].
///
/// What:
///  the bucket width,
///  thirty-two.
///  Why:
///  the cross-arch sweep found 32 the sweet
/// spot,
///  enough independent transition chains to saturate memory-level parallelism while
/// the per-column bookkeeping stays small;
///  16 is slightly behind and 64 regresses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BATCH_BUCKET: number = 32;
/// ```
const BATCH_BUCKET: usize = 32;

/// What:
///     Runs one benchmark batch through a named table-back-end kernel.
///     Falls back to the per-line loop when the pattern has no table.
/// Why:
///      The benchmark hooks differ only in the kernel they name.
///      One shape therefore serves all of them.
///
/// Invocation forms:
/// - `table_kernel_dispatch!(self, lines, kernel)` for a kernel with no width.
/// - `table_kernel_dispatch!(self, lines, kernel::<WIDTH>)` for a const width.
///
/// Expands to a block expression that the calling hook returns.
/// Each hook stays a real,
/// documented `pub fn` that `rg` finds by signature.
macro_rules! table_kernel_dispatch {
    ( $receiver:expr, $lines:expr, $kernel:ident $(::<$width:ident>)? ) => {{
        let lines = $lines;
        let mut out = vec![false; lines.len()];
        match $receiver.engine.table_dfa() {
            Some(dfa) => dfa.$kernel$(::<$width>)?(lines, &mut out),
            None => $receiver.engine.is_match_batch(lines, &mut out),
        }
        out
    }};
}

/// What:
///     Many-lines matching for a single compiled pattern.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl Regex {
    /// Reports,
    ///  per line,
    ///  whether the pattern matches a substring of that line.
    ///
    /// What:
    ///  returns one verdict per input line.
    ///  Why:
    ///  the batch face of
    /// [`Regex::is_match`];
    ///  a table-backed pattern with no required literal over a large
    /// batch runs the Sheng permute kernel,
    ///  every other shape loops the per-line match.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_match_batch(lines: Uint8Array[]): boolean[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// //! A batch of two lines:
    /// //! the line holding the key prefix matches,
    /// //! and the other does not.
    /// let re = forbidden_regex::compile("AKIA[A-Z2-7]{4}").unwrap();
    /// let lines: &[&[u8]] = &[b"AKIA2345", b"nope"];
    /// assert_eq!(re.is_match_batch(lines), vec![true, false]);
    /// ```
    pub fn is_match_batch(&self, lines: &[&[u8]]) -> Vec<bool> {
        let mut out = vec![false; lines.len()];
        self.engine.is_match_batch(lines, &mut out);
        return out
    }

    /// Reports per line,
    ///  grouping equal-length lines so the DFA advances many at once.
    ///
    /// What:
    ///  for a table-backed pattern,
    ///  sorts the line indices by length,
    ///  runs the
    /// branchless equal-length kernel over each exact-length bucket [`BATCH_BUCKET`] lines
    /// at a time,
    ///  and scatters verdicts back to input order;
    ///  other back-ends fall through
    /// to the per-line loop.
    ///  Why:
    ///  a single full-scan DFA is latency-bound per line,
    ///  but a
    /// bucket of equal-length lines exposes independent transition chains the core
    /// overlaps.
    ///  Measured 1.04x-1.09x on x86 and 1.32x-1.37x on arm64 over the per-line
    /// loop in the low-match regime a secret scanner sees;
    ///  it does not help when most
    /// lines match (the per-line loop early-exits) so it is opt-in,
    ///  not the default.
    ///  The
    /// internal sort is near-linear on already length-sorted input,
    ///  so a caller that
    /// pre-buckets pays almost nothing for it.
    ///
    /// Not a `table_kernel_dispatch` call.
    /// It orders the lines by length before dispatching.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_match_batch_bucketed(lines: Uint8Array[]): boolean[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// //! The bucketed batch answers exactly what the plain batch answers,
    /// //! on a line that matches,
    /// //! a line that is too short,
    /// //! and a line that does not match.
    /// let re = forbidden_regex::compile("[0-9a-f]{32}").unwrap();
    /// let lines: &[&[u8]] = &[b"deadbeefdeadbeefdeadbeefdeadbeef", b"short", b"nope"];
    /// assert_eq!(re.is_match_batch_bucketed(lines), re.is_match_batch(lines));
    /// ```
    pub fn is_match_batch_bucketed(&self, lines: &[&[u8]]) -> Vec<bool> {
        let mut out = vec![false; lines.len()];
        let Some(dfa) = self.engine.table_dfa() else {
            self.engine.is_match_batch(lines, &mut out);
            return out;
        };
        let mut keyed: Vec<(usize, usize)> = Vec::with_capacity(lines.len());
        for (index, line) in lines.iter().enumerate() {
            keyed.push((line.len(), index));
        }
        // Sorting the (length, index) pairs orders by length and keeps equal lengths in input order.
        keyed.sort_unstable();
        let mut order: Vec<usize> = Vec::with_capacity(keyed.len());
        for &(_, index) in &keyed {
            order.push(index);
        }
        let mut start = 0;
        while start < order.len() {
            let len = lines[order[start]].len();
            let mut end = start + 1;
            while end < order.len() && lines[order[end]].len() == len {
                end += 1;
            }
            let mut bucket: Vec<&[u8]> = Vec::with_capacity(end - start);
            for &index in &order[start..end] {
                bucket.push(lines[index]);
            }
            let mut verdicts = vec![false; bucket.len()];
            dfa.is_match_batch_tight_w::<BATCH_BUCKET>(&bucket, &mut verdicts);
            for (slot, &index) in order[start..end].iter().enumerate() {
                out[index] = verdicts[slot];
            }
            start = end;
        }
        return out
    }

    /// Benchmark hook:
    ///  forces the scalar per-line kernel.
    ///
    /// What:
    ///  runs the scalar batch on the table back-end,
    ///  else the ordinary per-line
    /// loop.
    ///  Why:
    ///  the baseline the interleaved,
    ///  tight,
    ///  and Sheng kernels are timed against.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_match_batch_scalar(lines: Uint8Array[]): boolean[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    #[doc(hidden)]
    pub fn is_match_batch_scalar(&self, lines: &[&[u8]]) -> Vec<bool> {
        table_kernel_dispatch!(self, lines, is_match_batch_scalar)
    }

    /// Benchmark hook:
    ///  forces the interleaved-scalar kernel.
    ///
    /// What:
    ///  runs the interleaved batch on the table back-end,
    ///  else the per-line loop.
    /// Why:
    ///  measures the memory-level parallelism of N independent scalar transition chains
    /// against the scalar baseline.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_match_batch_interleaved(lines: Uint8Array[]): boolean[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    #[doc(hidden)]
    pub fn is_match_batch_interleaved(&self, lines: &[&[u8]]) -> Vec<bool> {
        table_kernel_dispatch!(self, lines, is_match_batch_interleaved)
    }

    /// Benchmark hook:
    ///  reports whether this pattern compiled to the table DFA back-end.
    ///
    /// What:
    ///  true when the engine is `EngineKind::Table`.
    ///  Why:
    ///  the batch kernels only
    /// diverge on a table back-end,
    ///  so the benchmark confirms a microbench pattern is
    /// one before trusting its kernel-versus-kernel numbers.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_table(): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    #[doc(hidden)]
    pub fn is_table(&self) -> bool {
        return self.engine.table_dfa().is_some()
    }

    /// Benchmark hook:
    ///  interleaved-scalar kernel at an explicit bucket width `N`.
    ///
    /// What:
    ///  forces the interleaved batch at `N` lanes on the table back-end,
    ///  else the
    /// per-line loop.
    ///  Why:
    ///  sweeps how bucket size trades memory-level parallelism against
    /// per-chunk overhead.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function batch_inter_w(lines: Uint8Array[]): boolean[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    #[doc(hidden)]
    pub fn batch_inter_w<const N: usize>(&self, lines: &[&[u8]]) -> Vec<bool> {
        table_kernel_dispatch!(self, lines, is_match_batch_interleaved_w::<N>)
    }

    /// Benchmark hook:
    ///  branchless equal-length kernel at bucket width `N`.
    ///
    /// What:
    ///  forces the tight batch at `N` lanes on the table back-end;
    ///  `lines` must all
    /// share one byte length (an exact-length bucket).
    ///  Why:
    ///  measures the MLP ceiling once
    /// the per-lane early-exit branches are removed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function batch_tight_w(lines: Uint8Array[]): boolean[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    #[doc(hidden)]
    pub fn batch_tight_w<const N: usize>(&self, lines: &[&[u8]]) -> Vec<bool> {
        table_kernel_dispatch!(self, lines, is_match_batch_tight_w::<N>)
    }

    /// Benchmark hook:
    ///  Sheng in-register transition kernel (no bucketing needed).
    ///
    /// What:
    ///  forces the Sheng per-line scan on a table back-end of at most 64 states,
    ///  else
    /// the per-line loop.
    ///  Why:
    ///  measures the permute-transition path that attacks per-byte
    /// latency directly,
    ///  a different axis from the across-lines bucketed kernels.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function batch_sheng(lines: Uint8Array[]): boolean[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    #[doc(hidden)]
    pub fn batch_sheng(&self, lines: &[&[u8]]) -> Vec<bool> {
        table_kernel_dispatch!(self, lines, is_match_batch_sheng)
    }

    /// Benchmark hook:
    ///  two-byte composed Sheng kernel.
    ///
    /// What:
    ///  forces the two-byte Sheng scan on a qualifying table back-end (position-
    /// independent acceptance,
    ///  at most 64 states and 16 classes),
    ///  else the scalar batch.
    /// Why:
    ///  measures whether one permute per two bytes beats the one-byte Sheng.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function batch_sheng2(lines: Uint8Array[]): boolean[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    #[doc(hidden)]
    pub fn batch_sheng2(&self, lines: &[&[u8]]) -> Vec<bool> {
        table_kernel_dispatch!(self, lines, is_match_batch_sheng2)
    }
}

/// What:
///     Many-lines matching for a whole ruleset.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl RegexSet {
    /// Reports,
    ///  per line,
    ///  whether any rule matches a substring of that line.
    ///
    /// What:
    ///  one verdict per input line,
    ///  equal to calling [`RegexSet::is_match`] on
    /// each.
    ///  Why:
    ///  the batch face of the set matcher;
    ///  the consumer scans a file by
    /// handing every line at once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_match_batch(lines: Uint8Array[]): boolean[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// //! Each line is answered by whether any rule of the set matches it.
    /// let set = forbidden_regex::RegexSet::new(&["AKIA[A-Z2-7]{4}", "secret"]).unwrap();
    /// let lines: &[&[u8]] = &[b"AKIA2345", b"all clear", b"a secret here"];
    /// assert_eq!(set.is_match_batch(lines), vec![true, false, true]);
    /// ```
    pub fn is_match_batch(&self, lines: &[&[u8]]) -> Vec<bool> {
        let mut verdicts: Vec<bool> = Vec::with_capacity(lines.len());
        for line in lines {
            verdicts.push(self.is_match(line));
        }
        return verdicts
    }

    /// Returns the `(line index, rule index)` pairs the lines in `buf` match.
    ///
    /// What:
    ///  one full-width SIMD prefilter sweep over `buf` marks which lines
    /// hold a seeded literal,
    ///  then each line's byte range from its `starts`
    /// offset to the next offset (or `buf`'s end for the last line),
    ///  with the
    /// trailing newline and one trailing carriage return excluded,
    ///  is resolved:
    /// the seeded-rule gate runs only on a swept-candidate line,
    ///  while the
    /// line-start-anchored and literal-free rules run on every non-empty line,
    /// giving the same ids [`RegexSet::matches`] would.
    ///  Every matching rule id
    /// becomes a pair,
    ///  an empty line (after that exclusion) yields none,
    ///  and
    /// pairs are emitted in ascending line index with each line's ids in
    /// `matches()` order.
    ///  The line index is the 0-based position in `starts`,
    /// which the scanner maps to its own 1-based output.
    ///  Why:
    ///  the negative-line
    /// cost is the per-line gate scan,
    ///  and short lines starve the SIMD prefilter;
    /// one long-buffer sweep runs it at full width and skips the gate on every
    /// line with no seed.
    ///  The newline separators sitting in `buf` between lines
    /// keep a seed from matching across a line boundary,
    ///  so the sweep never
    /// attributes a cross-line hit.
    ///  The reference oracle is a per-line `matches()`
    /// loop,
    ///  kept in the batch tests.
    ///
    /// # Preconditions
    ///
    /// `starts` ascends,
    ///  its first offset is 0,
    ///  and every offset indexes within
    /// `buf`.
    ///  The caller guarantees this,
    ///  so it is not checked at runtime.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function line_matches(buf: Uint8Array, starts: number[]): [number, number][] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// //! Each match is reported as its line index and the index of the rule that matched.
    /// let set = forbidden_regex::RegexSet::new(&["AKIA[A-Z2-7]{4}", "secret"]).unwrap();
    /// let buf = b"AKIA2345\nall clear\na secret here";
    /// let starts = [0usize, 9, 19];
    /// assert_eq!(set.line_matches(buf, &starts), vec![(0, 0), (2, 1)]);
    /// ```
    pub fn line_matches(&self, buf: &[u8], starts: &[usize]) -> Vec<(usize, usize)> {
        // What:    One SIMD prefilter sweep over the whole buffer marks which
        //          lines hold a seeded literal, so the per-line loop runs the
        //          seeded-rule gate only where a seed is actually present.
        // Why:     Sweeping the long buffer at full SIMD width replaces the
        //          per-line gate scan that short lines would starve; the newlines
        //          left in `buf` stop a seed from spanning two lines.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        let candidate = self.sweep_candidates(buf, starts);
        let mut hits: Vec<(usize, usize)> = Vec::new();
        for index in 0..starts.len() {
            let start = starts[index];
            // What:    Raw end is the next line's start, or the buffer end for the
            //          last line; dropping one trailing `\n` then one trailing `\r`
            //          recovers the line content, mirroring the scanner's split-on-`\n`
            //          plus strip-one-`\r`.
            // Why:     The terminator (and a CRLF carriage return) is not part of the
            //          line the matcher sees.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Same step as the Rust statement below, written with ordinary TS objects/functions.
            // ```
            let mut end = starts.get(index + 1).copied().unwrap_or(buf.len());
            if end > start && buf[end - 1] == b'\n' {
                end -= 1;
            }
            if end > start && buf[end - 1] == b'\r' {
                end -= 1;
            }
            // What:    An empty line (after terminator exclusion) contributes no pairs
            //          and is never handed to the matcher.
            // Why:     The contract skips empty lines, and the engine expects a
            //          non-empty line.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Same step as the Rust statement below, written with ordinary TS objects/functions.
            // ```
            if end == start {
                continue;
            }
            for rule in self.resolve_matches(&buf[start..end], candidate[index]) {
                hits.push((index, rule));
            }
        }
        return hits
    }

    /// Benchmark hook:
    ///  batch via one concatenated-buffer gate sweep.
    ///
    /// What:
    ///  joins the lines with `\n` separators,
    ///  sweeps the SIMD prefilter once over
    /// the whole buffer to mark which lines hold a seed,
    ///  then resolves each line.
    ///  Why:
    /// the negative-line cost is the per-line prefilter,
    ///  and short lines starve Teddy's
    /// SIMD;
    ///  one long-buffer sweep runs it at full width and skips the per-line gate on
    /// every line with no seed.
    ///  Equivalent to [`RegexSet::is_match_batch`] line for line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_match_batch_concat(lines: Uint8Array[]): boolean[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    #[doc(hidden)]
    pub fn is_match_batch_concat(&self, lines: &[&[u8]]) -> Vec<bool> {
        let count = lines.len();
        if count == 0 {
            return Vec::new();
        }
        let mut total: usize = 0;
        for line in lines {
            total += line.len() + 1;
        }
        let mut buf = Vec::with_capacity(total);
        let mut starts = Vec::with_capacity(count);
        for line in lines {
            starts.push(buf.len());
            buf.extend_from_slice(line);
            buf.push(b'\n');
        }
        let candidate = self.sweep_candidates(&buf, &starts);
        let mut verdicts: Vec<bool> = Vec::with_capacity(count);
        for index in 0..count {
            let end = starts.get(index + 1).copied().unwrap_or(buf.len()) - 1;
            verdicts.push(self.resolve_line(&buf[starts[index]..end], candidate[index]));
        }
        return verdicts
    }

    /// Marks,
    ///  by one prefilter sweep over `buf`,
    ///  which lines contain a seeded literal.
    ///
    /// What:
    ///  walks the prefilter from hit to hit,
    ///  attributing each to its line and
    /// jumping to the next line start.
    ///  Why:
    ///  one SIMD pass replaces a per-line prefilter
    /// call,
    ///  and jumping past a flagged line keeps the sweep over the negative gaps.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function sweep_candidates(buf: Uint8Array, starts: number[]): boolean[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn sweep_candidates(&self, buf: &[u8], starts: &[usize]) -> Vec<bool> {
        let mut candidate = vec![false; starts.len()];
        let mut at = 0;
        while let Some(hit) = self.gate.prefilter_find_from(buf, at) {
            // The line holding the hit is the last one that starts at or before it;
            // the line starts strictly increase, so a search finds either that start or the slot after it.
            let line = match starts.binary_search(&hit) {
                Ok(exact) => exact,
                Err(after) => after - 1,
            };
            candidate[line] = true;
            at = starts.get(line + 1).copied().unwrap_or(buf.len());
            if at >= buf.len() {
                break;
            }
        }
        return candidate
    }

    /// Resolves one line's verdict given whether the sweep found a seed in it.
    ///
    /// What:
    ///  runs the seeded-rule gate only when a seed is present,
    ///  then the line-start
    /// rules and the literal-free groups,
    ///  mirroring [`RegexSet::is_match`].
    ///  Why:
    ///  skipping
    /// the gate on a seedless line is exactly what the prefilter would have done per
    /// line,
    ///  so the verdict is unchanged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function resolve_line(line: Uint8Array, has_seed: boolean): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn resolve_line(&self, line: &[u8], has_seed: bool) -> bool {
        if has_seed {
            let mut rule_check = RuleCheck { set: self, line, checked: CheckedFull::new() };
            if self.gate.any_candidate(line, &mut rule_check) {
                return true;
            }
        }
        if self.line_start_candidate(line) && any_line_start_match(&self.line_start, line) {
            return true;
        }
        return any_group_matches(&self.seedless_groups, line)
    }

    /// Collects one line's matching rule ids given whether the sweep found a seed
    /// in it.
    ///
    /// What:
    ///  runs the seeded-rule gate only when a seed is present,
    ///  then the
    /// line-start rules and the literal-free rules,
    ///  returning each matching rule
    /// id in ascending,
    ///  deduplicated order;
    ///  equal to [`RegexSet::matches`] line
    /// for line,
    ///  and the attribution twin of the boolean `resolve_line`.
    ///  Why:
    /// skipping the gate on a seedless line is exactly what the prefilter would
    /// have done per line,
    ///  so the rule ids are unchanged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function resolve_matches(line: Uint8Array, has_seed: boolean): number[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn resolve_matches(&self, line: &[u8], has_seed: bool) -> Vec<usize> {
        let mut collector = RuleCollector { set: self, line, checked: CheckedFull::new(), hits: Vec::new() };
        if has_seed {
            self.gate.for_each_candidate(line, &mut collector);
        }
        let mut hits: Vec<usize> = collector.hits;
        if self.line_start_candidate(line) {
            for (engine, &id) in self.line_start.iter().zip(&self.line_start_ids) {
                if line_start_match(engine, line) {
                    hits.push(id);
                }
            }
        }
        for &id in &self.seedless_ids {
            if self.rules[id].is_match(line) {
                hits.push(id);
            }
        }
        hits.sort_unstable();
        hits.dedup();
        return hits
    }
}

/// What:
///     Unit tests for the batch API,
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
#[path = "batch_tests.rs"]
mod tests;
