# Trailing whitespace is not enforced

Decided by the owner on 2026-10-22 ("It's just not worth it to enforce it"),
asked by issue [#585][].

## What happened

Issue [#585][] measured 31,023 whitespace-only lines in 2,843 tracked TypeScript files
and proposed a `stylistic/no-trailing-spaces` rule
with a template-literal-safe autofix in `@monochromatic-dev/oxlint-plugin-stylistic`,
plus one `format:oxlint` sweep.
The issue also asked whether the git wrapper should enforce it instead.
Research for that question
(`doc/planning/trailing-whitespace-enforcement-layer.md`)
found no layer enforcing line-end whitespace today,
and the wrapper's `final-newline` policy rewriting only the run of LF bytes at end of file
(`package/git-policy/cli/src/native/policy_final_newline.rs`).
The owner judged enforcement not worth its cost:
the gain is cosmetic,
no single mechanism covers all the repository's languages well,
and a language-aware fixer exists for one language family only.

## The rule

- No check enforces trailing whitespace,
  in the git wrapper,
  the lint layer,
  or CI.
- `.editorconfig` keeps `trim_trailing_whitespace` as inspiration for editors and humans.
  No check verifies it,
  and it is not a promise the repository makes.
- The measured whitespace-only lines stay in the tree.
- `package/translation-repair/source-scan/src/trailing-whitespace.unit.test.ts`
  keeps guarding its own package alone,
  unchanged.
- Issue [#585][] is closed as not planned.

## Options rejected

- The `stylistic/no-trailing-spaces` rule with one sweep
  (option A of `doc/planning/trailing-whitespace-enforcement-layer.md`):
  implementable and literal-safe for TypeScript,
  but a 2,843-file sweep buys cosmetics,
  covers one language family,
  and still leaves the `.editorconfig` line unkept elsewhere.
- The same rule plus a CI gate
  (option C):
  all of option A's cost,
  plus gating every oxlint rule on every pull request.
- The same rule plus a delegating wrapper backstop
  (option D):
  all of option A's cost,
  plus lint toolchain startup on every commit.
- A byte-level wrapper policy
  (option B):
  no parser sits behind the wrapper's exact-byte seam,
  so it would strip template literal text,
  flag content holders such as `*.patch` diffs,
  and need its own design decision
  since the shipped policy catalog is closed
  (`doc/decision/cli-git-rust-rewrite.md`).

## Limits accepted

- An editor honouring `.editorconfig` still strips line-end whitespace on save,
  including template literal text where the whitespace is the value.
  No layer detects that change.
- Whitespace-only lines accumulate again,
  and future diffs keep showing them as noise.
- Any new enforcement proposal must re-justify against this decision
  and start from `doc/planning/trailing-whitespace-enforcement-layer.md`.

## Evidence

- Measured debt,
  enforcement surfaces,
  options and ranking:
  `doc/planning/trailing-whitespace-enforcement-layer.md`.
- Wrapper scope:
  `normalized_final_newline` in `package/git-policy/cli/src/native/policy_final_newline.rs`.

[#585]: https://github.com/Aquaticats/Monochromatic/issues/585
