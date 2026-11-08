# monochromatic-lint: closure skips as warnings and the fence marker, record of 2026-11-07T23:10:20-05:00

This records three owner answers of 2026-11-07 (first section of `doc/handover/questions-for-the-owner.md`)
and what was built for them.
Release:
`monochromatic-lint` 0.5.2-alpha.7,
published to the GitHub release and not to crates.io (alpha label).

## Closure-check skips are warnings

The owner's answer,
in their own words:
"Fetch then warn."
The amendment is in `doc/decision/rust-closure-lint-dylint.md`,
section "Amendment of 2026-11-07T22:19:43-05:00".

- `lint:monochromatic-lint` in the root `mise.toml` has `depends = ["fetch:rust-dependencies"]`.
  `mise run --dry-run` showed the fetch first and the post-`--` paths reaching only the linter.
  One run of the fetch task took 15.7 s on a warm cache.
- A package that is not checked is one warning finding of `rust/avoidable-closure` at its `Cargo.toml`,
  one per package,
  also for a machine-wide gap and a missing driver.
  `ClosureOutcome.messages` and `with_messages` are replaced by `ClosureOutcome.skips`.
- Red test:
  `a_cold_cache_is_a_warning_finding_and_exit_zero` failed with 0 records against 1 before the change.
  Verified with the installed alpha.7 on a throwaway package with an uncached dependency:
  one `warn` record naming `pkg` and `cargo fetch --locked`,
  exit 0,
  and exit 1 under `--max-warnings 0`.
- A consequence the owner has not been asked about:
  `--quiet` and `--silent` now hide a skip with the other warnings,
  where the message on standard error was kept by `--silent`.

## The fence marker

The owner's answer:
"B:
exempt only \"Bad\" examples (Recommended)".
The word `monochromatic-lint-skip-rustdoc` after `rust` (or after a comma) exempts that fence from
`rust/require-rustdoc` only;
`rust/max-lines` still applies.
The README section "Exempting a fenced example from `rust/require-rustdoc`" documents it.

- Tests:
  `a_marked_fence_skips_require_rustdoc_and_an_unmarked_fence_still_warns` and
  `the_fence_marker_is_a_whole_word_after_a_space_or_a_comma`.
  The marker was written together with its tests,
  so the red state is shown by alpha.6 on the same fixture:
  the marked fence warned there and does not under alpha.7.
- Before:
  200 `rust/require-rustdoc` warnings in fenced Rust under `doc/audit` (133),
  `doc/planning` (41),
  `doc/research` (2) and the `dum-dum-non-ts` skill (24).
  After:
  the 176 of the documents are documented by hand (commit `4647f76f4b`,
  58 files,
  no finding left in them),
  and the 12 warnings of the six Bad fences of the skill are exempt.
- Done 2026-11-08T03:36:12-05:00:
  the 24 skill warnings were 12 in the six Bad fences and 12 in the six Good fences.
  Only the Bad fences carry the marker,
  as the answer says ("exempt only" the Bad ones),
  so the coordinator documented the Good fences by hand (commit `019c5f2b15`):
  each opens with a `//!` caption line,
  the two function examples gain a `///` dum-dum block,
  and the `Rule` example moves its type rationale onto `///` blocks on the struct and its fields,
  which is the skill's own first lesson (an item's dum-dum block is its doc comment).
  The skill file lints clean.
  The `//!` captions are the coordinator's choice and open to the owner's veto.
- Done 2026-11-08T03:39:43-05:00:
  `doc/handover` had 52 further `rust/require-rustdoc` warnings
  (`cli-git-native-candidates.md` 48,
  `cli-git-native-command-parser.md` 2,
  `resharp-panic-fix.md` 1,
  `slint-ide-0x.md` 1) outside the owner's list,
  in other sessions' documents.
  Under the owner's rule "Lint issues need to be taken care of asap" the coordinator documented them
  (commits `fd195ddb78` and `f6664812a4`):
  the API sketches gained one-line `///` summaries taken from the current source's rustdoc,
  and the one-line `enum` and `struct` sketches were split so each variant and field carries one.
  The formatter re-broke the lines of two more handovers with an unchanged word stream (commit `19e759079c`),
  and `monochromatic-lint doc/handover` then reported no finding.

## Decision records reflowed

The owner's answer:
"Allow whitespace-only reflow (Recommended)".
`mise run format:monochromatic-lint -- doc/decision` changed 32 files (commit `e0b808e244`).
`git diff --word-diff=porcelain -- doc/decision` showed no added or removed word,
and the whitespace-split word stream of `HEAD` equalled the working file's for every one of the 32 files.

## Also landed

The owner's answer "Ignore the seed folder (Recommended)":
`package/linter/monochromatic-lint.fuzz/seed/` is in the `ignored-trees` block of `monochromatic-lint.config.jsonc`
(commit `0154a68a97`).
Before:
5 findings in 2 seed files (the coordinator's message said 4),
exit 1;
after:
none,
exit 0;
a copy of `boundaries.md` in a sibling folder still reported its 2 findings.

## Commits

- Linter:
  `a36f4799f4` on `feat/monochromatic-lint-skip-warn-and-fence-marker`,
  merged by `fc2ffb9620`.
- Gates before the merge:
  `lint:types` (Clippy with `-D warnings`),
  `lint:container` (949 passed,
  3 skipped,
  then
  Clippy in the container),
  and the installed linter reported nothing on the package.
- Release bump `1afc9263bc` (dry run 42444572337,
  release 42444695146,
  publish 42444814648).
- Lock `e60b3db53d`,
  one tool moved.
- Documents:
  `ee184c38e7` (amendment),
  `7df987cd16` (`mise.toml`),
  `4647f76f4b`,
  `cfa9f741d8`.
