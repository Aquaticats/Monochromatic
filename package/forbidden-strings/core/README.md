# forbidden-regex

A restricted regular-expression engine for byte-oriented,
 line-at-a-time secret
scanning.
 It parses a deliberately limited dialect,
 then chooses between counting
NFAs for bounded repetition,
 synchronized products for `&`/`~`,
 and capped
derivative DFAs when a transition table is the fastest safe representation.

It exists to power the `forbidden-strings` secret scanner.
 A `RegexSet` builds one
set-level gate over required literals,
 checks leading-literal rules anchored at the
literal hit,
 checks `^` rules at line start,
 and keeps any truly literal-free rules in
capped union DFAs.
 Clean lines avoid per-rule scans through the SIMD prefilter.
Unlike classic NFA or backtracking engines,
 the derivative and product back-ends make
intersection (`&`) and complement (`~(...)`) first-class operators.

## Match model

- Input is a single non-empty line,
   matched as raw bytes (`&[u8]`).
- Matching is unanchored search:
   a pattern matches if it matches any substring.
- `^`,
   `$`,
   and `\b` anchor to line and word boundaries;
   the word set is ASCII
  `[A-Za-z0-9_]`.
- `multiline` and verbose (`x`) mode are always on.
   There are no flags to pass.

## Supported constructs

- Literals,
   and the escapes `\t`,
   `\b` (word boundary),
   backslash-escaped
  metacharacters,
   and backslash-escaped whitespace.
- Character classes:
   `[abc]`,
   `[a-z]`,
   `[a-zA-Z]`,
   negated `[^...]`,
   and the
  shorthands `\d \w \s \D \W \S` (usable inside classes too).
- `.` matches any byte except a newline.
- Grouping and alternation:
   `(?:a|b)`.
   Groups are non-capturing only.
- Bounded repetition:
   `a?`,
   `a{3}`,
   `a{3,6}`.
- Anchors:
   `^`,
   `$`,
   `\b`.
- Set algebra:
   intersection `&`,
   complement `~(...)`.

Operators `&` and `|` take single-atom operands:
 a literal,
 a class,
 `.`,
 an
anchor,
 a `(?:...)` group,
 or a `~(...)`.
 A concatenation or a quantified atom
must be wrapped in `(?:...)` to be an operand,
 so there is no operator precedence
to remember and operators never mix with concatenation at one level.
 `~(...)` is
always parenthesized.

## Verbose mode

Because verbose mode is always on,
 unescaped whitespace outside character classes
is ignored,
 so a single rule may be written across many lines.
 To match a literal
space use `\<space>`,
 `\t`,
 or `[ ]`.
 A line whose first character is `#` is a
comment to end-of-line;
 a `#` anywhere else is a literal.
 Whitespace and `#`
inside `[...]` are literal.

## Rejected at compile time

Everything outside the supported set is a hard `CompileError` with the offending
position:
 `*`,
 `+`,
 unbounded `{n,}`,
 `\xNN`,
 capturing `(`,
 lookaround and inline
flags (`(?` not followed by `:`),
 backreferences,
 unknown escapes,
 unbalanced
brackets,
 stacked quantifiers,
 `{n,m}` with `n` greater than `m`,
 and repetition
whose expansion exceeds the configured cap.

A pattern that can match the empty string is also rejected,
 because under
unanchored search it would match every input.
 So `~(Y)` alone is rejected,
 while
`(?:X) & ~(Y)` with a concrete `X` compiles.

## Usage

```rust
// One pattern.
use forbidden_regex::{compile, Regex};

let re: Regex = compile("AKIA[A-Z2-7]{16}").unwrap();
assert!(re.is_match(b"key=AKIA0123456789ABCDEF7"));

// A whole ruleset through the gated matcher.
use forbidden_regex::RegexSet;

let set = RegexSet::new(&["AKIA[A-Z2-7]{16}", "ghp_[A-Za-z0-9]{36}"]).unwrap();
assert!(set.is_match(b"... AKIA0123456789ABCDEF7 ..."));
let hits: Vec<usize> = set.matches(b"... ghp_000000000000000000000000000000000000 ...").collect();
assert_eq!(hits, vec![1]);

// Persist a compiled set and reload it (the benchmark path).
let bytes = set.to_bytes().unwrap();
let reloaded = RegexSet::from_bytes(&bytes).unwrap();
assert!(reloaded.is_match(b"... AKIA0123456789ABCDEF7 ..."));
```

## Compile-time threading

`RegexSet::new` and `RegexSet::compile_lenient` build rules on worker threads,
one per available core (capped at the pattern count);
a one-pattern call builds on the calling thread.
Results are identical to a one-at-a-time build:
rule ids follow input order,
and a failing `new` returns the lowest-index rule's error.

- **Speed.**
  The `forbidden-strings` 259-rule baseline compiles in 18.2s to 21.3s across runs on 16 threads
  instead of 97.5s one rule at a time (opt-level 3),
  with byte-identical `to_bytes` output.
- **Memory.**
  Peak resident memory for that baseline rose from 0.88 GB to 5.9 GB,
  because rules that were built one after another now build at once.
- **Stack.**
  Workers get 64 MiB stacks,
  because deeply nested groups overflow Rust's default 2 MiB thread stack
  (see [the std thread defaults write-up](../../../doc/troubleshooting/rust-std-thread-defaults-for-compute.md)).
- **Small sets.**
  Thread startup adds about 40 microseconds to a two-pattern compile;
  from 8 patterns on, the measured cost matched the one-at-a-time build.

## Batch matching

For scanning many lines at once,
 `is_match_batch(&[&[u8]]) -> Vec<bool>` on `Regex` and
`RegexSet` returns one verdict per line,
 equal to calling `is_match` on each.

```rust
let re = forbidden_regex::compile("[0-9a-f]{32}").unwrap();
let lines: &[&[u8]] = &[b"deadbeefdeadbeefdeadbeefdeadbeef", b"nope"];
assert_eq!(re.is_match_batch(lines), vec![true, false]);
```

On `Regex`,
 a table-backed pattern with no required literal and at most 64 states over a large
batch routes through the Sheng in-register transition kernel:
 one `vpermb`
(AVX-512VBMI) or `vqtbl4q` (NEON) permute advances the DFA state,
 replacing
the dependent transition load,
 and when acceptance is position-independent a composed
table advances two bytes per permute.
 It is measured up to ~3.3x the per-line loop
on x86 and ~2.2x on arm64,
 falls back to the per-line scan otherwise,
 and is
runtime-detected (no nightly toolchain required).

`Regex::is_match_batch_bucketed(&[&[u8]]) -> Vec<bool>` is an opt-in path that groups lines
by exact length and runs a branchless equal-length kernel;
 it helps over-64-state table
patterns and is fastest when the caller already feeds length-sorted lines.

## Tasks

- `mise run //package/forbidden-strings/core:test` runs the unit and
  integration tests.
- `mise run lint:monochromatic-lint -- package/forbidden-strings/core`,
  from the repository root,
  enforces the code-line budget and required rustdoc.
- `mise run //package/forbidden-strings/core:lint:clippy` runs clippy.
- `mise run //package/forbidden-strings/core:bench` measures throughput
  against `regex`.
