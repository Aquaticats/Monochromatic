# Handover: Rust fuzzing for monochromatic-jsonc-edit

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/rust-module/jsonc-edit.fuzz` is now `package/jsonc-edit/core-rust.fuzz`;
> `package/rust-module/jsonc-edit` is now `package/jsonc-edit/core-rust`;
> `package/module/jsonc-edit` is now `package/jsonc-edit/core-ts`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings.fuzz` is now `package/forbidden-strings/cli.fuzz`;
> `package/rust-module/forbidden-regex` is now `package/forbidden-strings/core`.

State as of 2026-09-26.
The sidecar exists,
builds,
and its unit tests pass.
The first bounded campaign found three crashes.
None is triaged yet.
This document is the resumption point.

## What is built

`package/rust-module/jsonc-edit.fuzz`,
beside the crate it fuzzes,
mirroring the other `<package>.fuzz` sidecars:

- Four libFuzzer targets:
   `fuzz_parse_emit_roundtrip`,
   `fuzz_reject_and_recover`,
   `fuzz_depth_envelope`,
   `fuzz_edit_invariants`.
- A structured `arbitrary` generator (`src/generators.rs`),
   shared invariant checks (`src/invariants.rs`) and an address picker (`src/path_pick.rs`).
- Thirteen unit tests,
   each invariant covered by a negative control that proves it can fail.
- Clippy clean under `-D warnings`,
   house Rust linter clean,
   ASAN binaries build through `mise run build`.
- Committed as `8de177bcf`,
   with the dictionary escape fix in `5c4d114f0`.

Each `[[bin]]` entry in `Cargo.toml` is load-bearing:
`cargo-fuzz` discovers targets through `cargo metadata`,
so a bare `fuzz_targets/` directory is invisible and `cargo fuzz list` prints nothing while
exiting 0.

## How to run a bounded campaign

Resource isolation is required,
and the mount must stay narrow.
Mounting the whole repository with `:z` fails on `lsetxattr` inside `node_modules` and would
relabel real user files,
so mount only the sidecar and disable label confinement instead:

```bash
podman run --rm --memory=2g --cpus=2 --security-opt label=disable \
  -v /var/home/user/Monochromatic/package/rust-module/jsonc-edit.fuzz:/work -w /work \
  localhost/mise-rust-test:fedora44 \
  sh -c 'for t in fuzz_parse_emit_roundtrip fuzz_reject_and_recover fuzz_depth_envelope fuzz_edit_invariants; do
    mkdir -p corpus/$t seed/$t
    ./target/x86_64-unknown-linux-gnu/release/$t corpus/$t seed/$t \
      -max_total_time=120 -rss_limit_mb=1536 -dict=dictionary/jsonc-edit.dict
  done'
```

The binaries are prebuilt on the host;
the container only needs a compatible glibc,
which `localhost/mise-rust-test:fedora44` provides.
`rg` is not installed in that image,
so replay crashes on the host instead of inside it.

## Campaign result, 2026-09-26

Four targets,
120 seconds each,
2 GiB and 2 CPU bounded:

- `fuzz_depth_envelope`:
   **clean**.
   593692 runs in 121 seconds,
   `cov: 284 ft: 1070 corp: 61/183b`,
   peak rss 393 Mb,
   no crashes.
   The nesting boundary and the malformed over-depth refusals held under sustained load.
- `fuzz_parse_emit_roundtrip`:
   **one crash**,
   `artifacts/campaign-2026-09-26/crash-5340dcc12e107cf99550cba3552928e99ef425b7`,
   14 bytes,
   Base64 `bmwA5HVlCg2KAOR1dUE=`.
   Panic at `src/invariants.rs:88`.
- `fuzz_reject_and_recover`:
   **one crash**,
   `artifacts/campaign-2026-09-26/crash-f4d33098b9cd52cfae84835bd188ad854191ec1b`,
   150 bytes,
   containing the literal `false` among binary noise.
   Panic at `src/invariants.rs:37`,
   and the backtrace names `assert_canonical_stability::{closure_env#1}`,
   which is the second closure in that function:
   the one that panics when **emission does not reparse**.
- `fuzz_edit_invariants`:
   **one crash on the empty input**,
   `artifacts/campaign-2026-09-26/crash-da39a3ee5e6b4b0d3255bfef95601890afd80709`
   (that hash is the SHA-1 of the empty string,
   and the file is 0 bytes).
   Panic at `fuzz_targets/fuzz_edit_invariants.rs:89`,
   backtrace through the target's own `unwrap_or_else` on a parse result.

Replay any of them on the host:

```bash
cd package/rust-module/jsonc-edit.fuzz
./target/x86_64-unknown-linux-gnu/release/fuzz_reject_and_recover \
  artifacts/campaign-2026-09-26/crash-f4d33098b9cd52cfae84835bd188ad854191ec1b
```

## Reading of each crash, not yet verified

These are inferences from panic locations and backtraces.
Each needs a minimized reproducer and a decision about whether the defect is in the crate,
the harness or the generator.

1. The empty-input edit crash is most likely a **harness** defect:
   with zero bytes,
   `GeneratedDocument::arbitrary` or `replacement_value` returns `Err(NotEnoughData)` and my
   `.expect(...)` turns that into a panic.
   Harnesses must skip inputs they cannot interpret rather than fail on them.
   Line 89 and the `replacement_value` expect are both candidates.
2. The reject-target crash is the one that matters most if it survives triage:
   `assert_canonical_stability` is only called after `parse_jsonc` succeeded,
   and closure #1 is the "emission must reparse" panic,
   so the crate would have emitted text it then refused to parse.
   That would be a product defect in `emit_jsonc_value`,
   not a harness problem.
   It must be reproduced with the mutated source printed,
   then minimized.
3. The roundtrip crash at `src/invariants.rs:88` needs the assertion message,
   which the replay filtered out.
   Line 88 is inside the comment-collection or preservation region of that file,
   so it is either a dropped comment during emission (product defect) or an over-strong
   expectation about merged or re-rendered comment bodies (harness defect).
   Rerun the replay without filtering to read the message.

## Next steps

1. Replay each artifact unfiltered and record the full panic message and the offending source text.
2. Minimize each:
   `cargo fuzz cmin` or `cargo fuzz tmin` per target,
   then reduce the JSONC source by hand to the smallest document that still fails.
3. Classify each as crate,
   generator or harness,
   and for crate defects write a unit test in `package/rust-module/jsonc-edit` that fails first.
4. Fix the harness so an uninterpretable input is skipped:
   no `.expect` on `Arbitrary` results inside a target.
5. Re-run the campaign until all four targets are clean for a full bounded round,
   then record the run counts here.
6. Task #23 is still open:
   mutation testing with `cargo-mutants`,
   which is not installed.
   A local image `localhost/mutation-test-runtime:ff8da957d5b0` exists and
   `package/rust-module/forbidden-regex/.cargo/mutants.toml` is the precedent to copy.
7. Task #19 is still open:
   no `AGENTS.md` rule states that sidecars live beside their package as `<pkgname>.<kind>`.
   Commit `b9c0e6e48` already moved `package/fuzz/forbidden-strings` to
   `package/cli/forbidden-strings.fuzz`,
   so task #20 is done and only the rule remains.

## Triage result, 2026-09-26

All three campaign crashes are triaged.
Two were product defects present in **both** maintained implementations,
one was an over-strict harness expectation.

1. **Product defect,
   fixed in both implementations.**
   A `//` comment ends at CR,
   LF or CRLF,
   but both emitters decided trailing form from LF alone (`isSingleLineComment` in
   `package/module/jsonc-edit/src/emit-comment.ts`,
   `single_line` in `package/rust-module/jsonc-edit/src/comment_merge.rs`) and split leading bodies
   on LF alone.
   A block comment body containing a bare CR was therefore emitted as a line comment that
   terminated early,
   leaving the rest of the body as code:
   the Rust port reported `unterminated block comment (at offset 242)` on its own emission,
   and the TypeScript package reproduced it as
   `unexpected character "b" (at offset 12)`.
   Fixed by counting every terminator for trailing form,
   preferring a block in leading form when the body carries CR and can be one,
   and splitting the block-unsafe fallback on every terminator.
   Guard tests:
   `block_comment_body_with_cr_survives_round_trip` and
   `merged_comment_body_with_cr_survives_round_trip` in
   `package/rust-module/jsonc-edit/src/parse_tests.rs`,
   and two round-trip cases in
   `package/module/jsonc-edit/src/stringify.unit.test.ts`.
   Both were shown failing before the fix.
2. **Product defect,
   fixed in both implementations.**
   Setting the document root to a scalar was accepted,
   producing a state whose canonical emission (`null`) its own parser rejects.
   The empty-input edit crash was this,
   not a harness budget problem:
   the fuzzer drew an empty address plus a scalar replacement.
   `jsonc_set` in `package/rust-module/jsonc-edit/src/edit_apply.rs` and `jsoncSet` in
   `package/module/jsonc-edit/src/edit-set.ts` now refuse a non-container root replacement.
   Guard test `root_set_to_a_scalar_is_refused` was shown failing with the guard removed and
   passing with it restored.
3. **Harness defect,
   fixed twice over.**
   `assert_comments_preserved` compared a whole merged body against the emission,
   and `fuzz_parse_emit_roundtrip` repeated that comparison for generator-recorded bodies.
   Canonical emission renders a merged multi-line body as one indented `//` line per body line,
   so the joined body is deliberately not a substring.
   Both now compare body lines,
   which still catches a dropped one;
   the negative control in `src/invariants_tests.rs` still fails when a comment is stripped.

After the fixes,
`fuzz_reject_and_recover` ran 880092 executions in 123 seconds with no crashes,
and all three original artifacts replay clean.

Two process lessons worth keeping:

- A replay command built as `./$PWD/target/.../fuzz_x` produced a doubled path,
   every replay silently failed to execute,
   and the `grep -c` for `SUMMARY: libFuzzer` read `0`,
   which looked like "no crash".
   Four targets were reported clean when one was not.
   A probe that cannot show a positive result proves nothing;
   replay the artifact and read the output before believing a clean count.
- libFuzzer wrote newly discovered units into the second corpus directory as well,
   polluting the tracked `seed/` tree with binary blobs.
   Tasks now copy seeds into `corpus/<target>` and pass only that directory.

## Campaign 4, after the fixes

Same bounds,
120 seconds per target:

- `fuzz_parse_emit_roundtrip`:
   clean,
   403182 runs.
- `fuzz_reject_and_recover`:
   clean,
   534637 runs.
- `fuzz_depth_envelope`:
   clean,
   93941 runs at roughly 776 exec/s,
   the slowest target because every run parses deep nesting.
- `fuzz_edit_invariants`:
   one crash,
   `artifacts/campaign-2026-09-26-c/crash-bb249c5e1ec5e56c724e817a135cc35ad203c5f0`,
   panicking on `deleted address still resolves`.

**Finding 4,
 harness defect,
 fixed.**
Deleting an array element shifts every later element down,
so the same index legitimately resolves afterwards to a different element.
The assertion was written as if deletion always vacated the address,
which is true only for object members.
It now splits by segment kind:
a deleted member must stop resolving and the parent must lose exactly one member,
while a deleted element must leave the parent with exactly one fewer element.
The artifact panicked before the rebuild and replays clean after it,
which is the positive control that the rebuilt binary is the one under test.

## Campaign 5, the clean full round

150 seconds per target,
same 2 GiB and 2 CPU bounds,
no crash artifacts written by any target:

- `fuzz_parse_emit_roundtrip`:
   13184 runs,
   clean.
   The low count against campaign 4 is the merged corpus being replayed and minimized first,
   not a slowdown in the target.
- `fuzz_reject_and_recover`:
   160114 runs,
   clean.
- `fuzz_depth_envelope`:
   625656 runs at 4143 exec/s,
   clean.
- `fuzz_edit_invariants`:
   452889 runs,
   clean.

## Mutation testing

`cargo mutants` 0.x over the crate,
test files excluded (`-E '_tests\.rs$'`),
found 489 mutants.
The first round caught 379 and missed 24.
Most survivors were real gaps rather than equivalent mutants,
and each now has a test naming the expected message or layout:
canonical indentation per nesting level,
trailing placement of a single-line comment,
a block comment at offset zero and at end of input,
a lone star or slash inside a block body,
a space inside a string,
a lone slash where a separator belongs,
and the six shape-mismatch refusals across lookup,
set and delete.
Round two,
scoped to the seven files that had survivors,
caught 262 and missed 10.
Six of those ten were real gaps and are now killed;
the final full-crate run reports:

- 485 mutants generated (test files excluded,
   four already excluded by configuration at that point)
- 397 caught by a failing test
- 34 caught by timeout,
   which is how an introduced infinite loop shows up
- 52 unviable,
   meaning the mutated code did not compile
- 2 survived

So of the 433 viable mutants,
431 were caught and the 2 survivors are now excluded with written proofs in
`package/rust-module/jsonc-edit/.cargo/mutants.toml`,
which holds six exclusions in total.
The crate lists 483 mutants with that configuration active.

Both survivors sit in `Scanner::capture_trailing`,
which consumes same-line trivia only (its whitespace arm takes space,
tab and CR but not LF) and has exactly one call site.
Replacing the comment arm's `offset + 1` lookahead with `offset * 1` makes the second disjunct
compare the current byte against `*` when the outer condition already fixed it to `/`,
and replacing it with `offset - 1` asks whether the byte *before* the slash is `*`,
which is unreachable because a block comment's `*/` is consumed as one unit by `comment()`.
Either way the arm narrows to `//` only,
and when it declines a `/*` the caller's next `trivia()` call captures the identical comment and
comma.
That was measured,
not only reasoned:
twelve inputs covering same-line and later-line trivia,
comment-then-comma,
tabs,
a lone star and a lone slash produced byte-identical outcomes under the unmutated build and under
each mutant,
by temporarily applying the mutation,
running the probe,
diffing and reverting.

Exclusions are anchored on line and column,
so a moved file stops matching and re-exposes the mutant instead of silently keeping the exclusion.
Re-verify the proof at that point rather than re-anchoring it.

## Related records

- `doc/troubleshooting/arbitrary-choose-empty-input.md`:
   why the generator's fresh-key retry loop hung,
   and why piping `cargo test` hides a hang.
- `doc/decision/jsonc-comment-line-ownership.md`:
   the ownership rule the invariants assume.
- `doc/decision/jsonc-edit-parser-foundation.md`:
   the adopted foundation,
   including the fast-path removal amendment.
- `doc/runbook/publish-crate-first-time.md`:
   the publication route;
   0.1.0 is published and Trusted Publishing is proven.
