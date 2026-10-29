# forbidden-regex-fuzz

libFuzzer (cargo-fuzz) harnesses for the `forbidden-regex` engine.
 Mirrors
`package/forbidden-strings/cli.fuzz`.
 The Rust toolchain (nightly) comes from the repo-root
mise tools;
 no per-package `[tools]`.

## Targets

- `fuzz_compile`:
   arbitrary bytes as a pattern and as a ruleset.
   `compile`,
  `compile_lenient`,
   and matching must reject cleanly,
   never panic or hang.
- `fuzz_from_bytes`:
   arbitrary bytes through `Regex::from_bytes` / `RegexSet::from_bytes`.
  The security boundary:
   a decoded automaton that passes `validate` must run without an
  out-of-bounds read or panic on attacker-influenced input.
- `fuzz_roundtrip`:
   a generated valid pattern compiled,
   serialized,
   and reloaded must
  give the same verdict on the generated content.
   Serialization is verdict-preserving.
- `fuzz_differential`:
   a generated pattern in the shared dialect (no `&`/`~`) compared
  against the `regex` crate (`unicode(false)`,
   byte mode) on single-line content.
   Byte
  verdicts must agree.
- `fuzz_line_matches`:
   a generated ruleset and a generated multi-line buffer
  (lines joined with a mix of `\n`,
   `\r\n`,
   and an unterminated final
  line) compared against per-line `matches()` sliced out by hand per the
  documented contract.
   `RegexSet::line_matches` must agree with that naive
  per-line delegation on every line,
   including empty ones.

The structured pattern/content generator is `src/generators.rs`
(`PatternAndContent`,
 bounded depth and repetition;
 records `uses_algebra` so the
differential target skips set algebra `regex` cannot express).
 The same file's
`RulesetAndBuffer` generates a small compiled ruleset plus a matching
multi-line buffer and `starts` array for `fuzz_line_matches`.

## Commands

- `mise run //package/forbidden-strings/core.fuzz:list` lists targets.
- `mise run //package/forbidden-strings/core.fuzz:build` builds all targets (ASAN).
- `mise run //package/forbidden-strings/core.fuzz:smoke` runs each target 30s.
- `mise run //package/forbidden-strings/core.fuzz:run <target> -- -max_total_time=60` runs one,
  or replays a crash artifact passed before `--`.

All tasks pin `--target x86_64-unknown-linux-gnu --sanitizer address`;
 macOS/Windows
hosts need a per-platform target switch.
