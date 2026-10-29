# Final-newline normalization plan

## Status

The hk baseline was implemented and verified on 2026-07-09,
with tsdown Node output explicitly exempted during implementation.
The durable cli-git migration was implemented on 2026-07-11 under issue `#355`.

`final-newline` is now an enabled-by-default core cli-git policy at error severity.
It runs after `add-explicit` in core order and remains warn-safe.
Pre-forward commit handling applies corrections only to a private index,
then commits canonical blobs while preserving partially staged worktree tails byte-for-byte.
Manual push and direct check report findings without rewriting content.
Direct fix converges against a private index,
installs selected worktree files atomically,
and verifies that real index bytes remain unchanged.

The policy canonicalizes every in-scope,
non-empty UTF-8 text file to exactly one trailing LF.
It skips empty,
NUL-containing,
invalid UTF-8,
deleted,
symlink,
and submodule candidates.
The exact hk exclusion families remain unchanged:
`package/forbidden-strings/cli.fuzz/corpus/**`,
`package/test-fixture/toml-edit/src/**`,
and `**/dist/final/node/**`.
Tsdown outputs therefore keep producer-native missing final LF.
Git continues to track actual bytes;
it is not configured to hide newline-only changes.

The final hk-era tracked-file scan inspected 7,074 paths and found zero in-scope violations:
5,446 compliant text files,
67 binary or non-UTF-8 files,
and 1,561 excluded paths.
Those counts remain historical evidence rather than current-tree claims.
Issue `#357` removed the transitional hk and Pkl infrastructure after independent cli-git CI passed.

## Goal

Prevent missing final newlines and extra blank lines at end of file from entering commits,
while preserving binary files and text-shaped test data whose exact bytes are part of the test.

The user-facing outcomes are:

- Committing a normal text file with an invalid ending commits a canonical private-index blob without rewriting
  unstaged worktree bytes.
- `git cli-git fix -- <pathspec>...` and `git cli-git fix --all` canonicalize selected worktree files without changing
  real index bytes.
- `git cli-git check` and manual push report invalid endings without rewriting worktree or committed content.
- A partially staged file keeps its worktree tail byte-for-byte while its would-be committed blob is corrected.
- Generated license copies remain canonical after their normal producer runs.
- Tsdown Node outputs remain byte-identical to producer output and outside newline enforcement.
- Fuzz corpora and parser fixtures remain byte-identical.

## Current state and measured scope

The tracked-file scan inspected 7,060 paths in the checkout used for planning.
It found 5,445 compliant text-like files,
67 binary or non-UTF-8 paths,
one empty file,
and 1,547 non-empty text-like violations.

The violations split by ownership:

- 1,344 files under `package/forbidden-strings/cli.fuzz/corpus/`.
  These are fuzz inputs whose bytes are data,
  not source formatting.
- 52 TOML inputs under `package/test-fixture/toml-edit/src/`.
  Missing final newlines can be part of parser behavior under test.
- 123 copies of `GPL-3.0-or-later.txt`.
  `file-enforcer.config.ts` copies package-local license texts verbatim from the canonical root source.
- 18 tracked outputs under `package/claude-code-plugin/*/dist/final/node/`.
  Tsdown 0.22.4 emits these JavaScript and declaration files without a final LF;
  the implementation amendment reclassified them as intentional compact-output exemptions.
- 10 ordinary source or configuration files.

The sole empty file is `test/file.txt`.
History says it was added only to retain an otherwise empty `test/` directory.
Nothing references it,
and Git does not otherwise track directories.
It will be deleted rather than turned into a newline-only file.

Counts are evidence from the planning scan,
not permanent constants.
The implementation verification must enumerate the then-current tree again.

## Why Git attributes are insufficient

`.gitattributes` already contains:

```gitattributes
* text=auto eol=lf
```

That policy normalizes line-ending style,
meaning CRLF versus LF.
It does not normalize how many LF bytes terminate a file.
A direct `git hash-object --stdin` check produced three different blob IDs for `x`,
`x\n`,
and `x\n\n`.
Git therefore correctly treats these as different contents.

A Git clean filter could rewrite the content while adding it to the index,
but it is rejected here:

- Filter commands live in Git configuration,
  so a clone without the local driver silently receives pass-through behavior unless the filter is marked required.
- Required filters make ordinary Git operations depend on another configured executable.
- A filter makes the worktree and index intentionally disagree,
  which obscures what `git status` means.
- Applying it broadly risks rewriting byte-sensitive test data.

The repository will normalize before commit instead of asking Git to conceal a real byte difference.

## Historical interim mechanism

Before the durable cli-git migration,
the repository used hk's built-in `newlines` step as an interim local fixer and checker.
At hk 1.50.0,
`pkl/builtins/newlines.pkl` delegates to `hk util end-of-file-fixer`:

- Check mode emits a unified diff and fails on missing or multiple final LF bytes.
- Fix mode removes repeated final LF bytes and appends one LF.
- Binary-looking files and empty files are skipped.
- The builtin selects text-like paths,
  then the utility performs its own content check.

The interim implementation aligned the executable and Pkl package at hk 1.50.0 before retirement.
Issue `#357` removed the root config,
both active tool declarations,
and all legacy and active hk/Pkl lock entries.

The configured step will be equivalent to:

```pkl
local finalNewline = (Builtins.newlines) {
  batch = true
  exclude = List(
    "**/dist/final/node/**",
    "package/forbidden-strings/cli.fuzz/corpus/**",
    "package/test-fixture/toml-edit/src/**",
  )
}
```

`batch = true` is required,
not an optimization.
The full-tree prototype without it reached the operating system's argument-size limit.
The batched prototype split the tree into bounded invocations and completed.

The fixture exclusions are deliberately narrow.
A generic `**/corpus/**` exclusion would silently exempt future corpora that may contain ordinary text.
A broad `package/test-fixture/**` exclusion would exempt source and metadata that are not byte-sensitive.

The output exclusion is intentionally directory-wide.
The shared tsdown Node config owns `dist/final/node`,
and every file beneath that producer boundary keeps producer-native bytes.
The current 18 tracked outputs all omit the final LF,
so the exemption saves 18 bytes in the measured baseline.

## Historical hook behavior

### Pre-commit

Add `finalNewline` to the existing `pre-commit` hook in read-only mode and keep:

```pkl
stash = "git"
```

Git stashing is essential for checking staged bytes instead of a partially staged file's worktree version:
hk temporarily removes unstaged edits,
runs the check against staged content,
then reapplies the unstaged patch.

Pre-commit auto-fix is intentionally disabled.
A verified hk 1.50.0 edge case duplicates the boundary LF when staged content lacks final LF and an unstaged tail
begins at that exact boundary.
`doc/troubleshooting/hk-partial-staging-final-newline.md` records the source trace and upstream-compatible fix.
Until that fix ships in a verified hk release,
read-only rejection is the only safe pre-commit behavior.

An actual disposable-repository commit verified that:

- Missing and multiple final LF bytes reject the commit without changing `HEAD`.
- The invalid staged blobs remain byte-identical after rejection.
- A partially staged worktree remains byte-identical after rejection.
- The explicit fix normalizes worktree files while leaving staged blobs unchanged.
- After explicitly staging corrected bytes,
  a real commit succeeds.
- A NUL-containing binary and all three excluded path families stay byte-identical.

### Pre-push and explicit check

Add the same step to `pre-push` and `check` without fix mode.
These surfaces fail when an in-scope violation remains;
they do not rewrite content.

### Explicit fix

Add a `fix` hook containing the newline step with `fix = true`.
This makes the repository-wide cleanup available as:

```sh
hk fix --all --step final-newline --no-stage
```

`--no-stage` is required for baseline cleanup.
It lets the implementer inspect the changes and stage explicit path groups,
rather than letting a bulk fixer sweep unrelated concurrent work into the index.
A disposable repository verified that this option normalized the worktree while leaving the staged blob untouched.

Full-tree verification uses:

```sh
hk check --all --step final-newline
```

The explicit step selection avoids coupling this policy's verification to unrelated hk steps.

## Generated-file ownership

A one-time normalization commit is insufficient when a generator recreates the old ending.
Each generated family therefore needs an explicit ownership decision:
canonicalize at its source or exclude the producer boundary.

### License copies

Normalize `LICENSES/GPL-3.0-or-later.txt`,
then run the repository file-enforcer task.
`file-enforcer.config.ts` will propagate the canonical bytes to package-local `LICENSES/` directories.
The package copies are not edited as independent sources.

`LGPL-3.0-or-later.txt` and `CC-BY-SA-4.0.txt` already have compliant endings;
they need no content change.

### Tsdown Node outputs

Tsdown 0.22.4 with the current minifier emits the tracked `.mjs` and `.d.mts` files with no final LF.
That producer-native result is now intentional:
every generated file saves the otherwise mandatory final-LF byte.
The shared `**/dist/final/node/**` hk exclusion applies to pre-commit,
pre-push,
explicit check,
and explicit fix surfaces.

No tsdown post-build normalizer is installed.
The shared Node config remains byte-transparent,
and rebuilding the eight affected plugin packages restores all 18 tracked files to zero final LF bytes.
A real commit containing those 18 files verified that the pre-commit hook omitted the newline step for them.

Rejected normalization techniques remain useful evidence:

- `footer: '\n'` emitted two final LF bytes because Rolldown inserts a separator before footer content.
- Disabling minified code generation fixed JavaScript but not declarations and changed output formatting.
- A `build:done` normalizer worked,
  including multi-entry coordination,
  but its reads,
  writes,
  and added output byte conflict with the compact-output decision.

## Baseline cleanup

After hook and generator changes are in place:

1. Delete `test/file.txt`.
2. Normalize the canonical GPL source and run file-enforcer.
3. Rebuild every tracked Claude Code plugin Node artifact through its package mise task,
   preserving producer-native missing final LF bytes.
4. Run the newline fixer with `--no-stage` to catch the remaining ordinary files;
   the output-directory exclusion keeps rebuilt artifacts untouched.
5. Inspect the complete diff.
6. Confirm no excluded corpus or TOML parser fixture changed.
7. Stage explicit path groups and commit checkpoints according to repository policy.

The ordinary files identified during planning were:

- `.idea/modules.xml`
- `.remarkignore`
- `package-paused/webapp-content/messages-demo/src/server.ts`
- `package/mutation-test/cli/src/host/cli-options.ts`
- `package/linter/kotlin/build.gradle.kts`
- `package/toml-edit/core/src/fuzz/coverage-v8.ts`
- `package/toml-edit/core/src/values.ts`
- `package/pi-plugin/auto-mode/src/git-worktree-read-allowlist.ts`
- `package/ssg/aquati.cat/public/manifest.webmanifest`
- `package/ssg/aquati.cat/src/content/en/on-humanity.mdx`

The implementation must trust the fresh scan over this historical list if the tree has changed.

## Architecture tradeoff and migration

The migration selected cli-git as the durable owner.
`package/git-policy/cli/src/policy-engine/final-newline-policy.ts` owns policy registration and lifecycle behavior;
`final-newline-normalize.ts` owns exact-byte classification;
`final-newline-patch.ts` owns destination-grammar Git patch generation.
The shared commit transaction applies pre-forward patches only to a private index.
The direct-fix transaction projects selected worktree bytes into another private index,
converges whole-policy passes,
revalidates concurrent worktree changes,
and installs sibling-file replacements atomically.

Issue `#357` removed hk's duplicate transitional checks and fallback fixer.
The historical pre-commit fixer remained disabled through retirement because hk 1.50.0 can alter a partially staged
worktree at an EOF-tail boundary.
Git clean filters remain rejected because local driver configuration and worktree/index opacity conflict with the
platform's exact-byte model.
Formatter-only enforcement remains incomplete because it cannot protect every in-scope text file.

Issue `#356` added independent final-newline CI in `.github/workflows/final-newline.yml`
(Removed on 2026-10-14 at the owner's request, together with `package/git-policy/cli/bin/final-newline-workflow.mjs`:
the check rebuilt the wrapper on every pull request.).
Hosted run `29171565809` passed while invoking cli-git's direct checker through typed Node orchestration over a
disposable clone without loading unrelated hk or Pkl tooling.
Issues `#356` and `#357` completed the independent CI gate and hk/Pkl retirement.

## Issue #520 evidence and resolution

A disposable foreign repository exercised the installed `node_modules/.bin/git` shim.
A canonical file produced no `final-newline/noncanonical-final-newline` warning or fix summary.
A file missing its final LF warned on `git add -- file.txt`;
the former pathspec commit fixed only the private index and landed canonical `HEAD` bytes,
but left the worktree missing that LF (`git status --short` showed ` M file.txt`).
Adding it again staged the old bytes,
then normalization removed the entire selected tree difference from `HEAD`,
so Git reported `no changes added to commit` while leaving staged bytes behind.
The original pre-fix worktree bytes of most files in the issue were not recorded.
Canonical committed blobs therefore do not establish that their pre-add worktree bytes were canonical;
the reported doubled and relocated LF patterns have not been reproduced or assigned a cause.

The selected-path fix now records only paths changed by the core final-newline policy,
compares initial staged and worktree bytes,
and reconciles matching worktree copies after commit installation.
The worktree copy of a partially staged file remains byte-identical.
If the settled tree equals `HEAD`,
the transaction installs the selected canonical index and eligible worktree copies without creating a commit;
it emits `commit-normalization/no-change` with exit `1` instead of forwarding an empty commit to Git.
A journal distinguishes that operation from an interrupted real commit for startup recovery.
The wrapper remains repository-agnostic,
including repositories without cli-git config.

`package/git-policy/cli/src/trust/fixture/built-final-newline-reconciliation-consumer.ts` verifies canonical input,
missing and repeated LF,
index/worktree cleanup,
concurrent edits,
permissions,
interrupted real commits,
and explicit `--allow-empty`.
`built-final-newline-partial-consumer.ts` verifies partial staging preserves its unstaged tail.
`built-final-newline-normalization-recovery.ts` exercises normalization-only recovery with differing and byte-identical
index snapshots.
Neither fixture claims a reproduction of the original doubled or relocated LF incident.

The issue-specific verification passed with `mise run //package/git-policy/cli:build:js:node`,
`mise run //package/git-policy/cli:lint:types`,
`mise run //package/git-policy/cli:test:unit`,
and `mise run //package/git-policy/cli:test:built:trust`.
The rebuilt `node_modules/.bin/git` also left a disposable foreign repository clean after a corrected pathspec commit;
for a normalization-only selection it reported no commit and cleaned the selected index and worktree.
A file-scoped Oxlint pass over the changed production TypeScript files reported zero warnings and errors.
The file-scoped task's positive control reported the existing `test-import(require-eventual-artifact)` errors in
`package/git-policy/cli/src/policy-engine/final-newline-policy.unit.test.ts`.
Package-wide Oxlint remains red on that rule in unchanged unit tests and an unrelated
`unicorn(consistent-function-scoping)` warning in `package/git-policy/cli/src/trust/account-root.ts`.
Those diagnostics do not meet the historical whole-package zero-lint criterion;
they are not evidence that issue #520's changed production files pass package-wide lint.

## Limitations

- Clients that bypass the PATH-shadowed executable also bypass local cli-git policy enforcement;
  independent CI remains the backstop.
- Only LF runs are collapsed.
  The existing `.gitattributes` policy remains responsible for CRLF-to-LF normalization.
- Hk 1.50.0's retired fixer could insert a blank boundary line when it added the staged file's missing final LF at the
  exact point where an unstaged tail began.
  Cli-git avoids that merge path entirely:
  commit correction first changes only its private index,
  then synchronizes selected worktree files only when their initial bytes matched the staged candidate and still match
  immediately before installation.
  Partial staging remains untouched.
  A noncooperating filesystem writer may still change a path after the final check and before its pathname replacement.
  Direct fix changes selected worktree files only after exact concurrency and real-index checks.
- Generated license normalization stays attached to file-enforcer's canonical source.
- Tsdown's `dist/final/node` tree is a deliberate compact-output exception;
  paths moved outside that boundary become subject to normal enforcement.

## Documentation

Implementation updates:

- `doc/decision/cli-git-policies-platform.md`:
  record the historical interim hk behavior,
  migration obligation,
  and completed retirement.
- `doc/troubleshooting/hk-partial-staging-final-newline.md`:
  record hk 1.50.0's partial-staging boundary merge,
  exact-byte reproduction,
  consumer limitation,
  and upstream prototype.
- `doc/troubleshooting/tsdown-final-newline.md`:
  record the tsdown 0.22.4 byte endings,
  source trace,
  reproduction,
  rejected normalization approaches,
  compact-output exception,
  and upstream-filing decision.
- This plan:
  update status and any measured details that change during implementation.

## Verification and acceptance criteria

The original hk baseline criteria remain historical evidence.
The durable cli-git migration additionally requires:

- Core registration is enabled by default at error severity and ordered after `add-explicit`.
- Unit fixtures cover exact normalization,
  binary and empty preservation,
  all exclusion families,
  single-path patch grammar,
  eight changed passes,
  cross-policy cycle detection,
  final-pass-only summaries,
  and Git-byte-ordered paths.
- Disposable Git fixtures prove transactional commit correction,
  partial-staging worktree preservation,
  direct-fix real-index neutrality,
  and read-only check and push behavior.
- Packed built-shim fixtures invoke check,
  fix,
  commit,
  patch selection,
  interactive selection,
  and push through the shipped `index.mjs`.
- Cli-git build,
  Oxlint with zero warnings,
  type checks,
  unit tests,
  packed trust,
  and independent forbidden-strings scanning pass.

The hk baseline was complete only when all of these held:

- `hk.pkl` evaluates with the pinned hk executable and Pkl package version.
- An actual Git commit fixture proves missing and multiple final LF cases are rejected without mutation.
- The same fixture proves partial staged and unstaged bytes are restored exactly.
- An explicit no-stage fix followed by explicit staging proves corrected content commits successfully.
- Binary and excluded fixture hashes remain unchanged.
- `hk check --all --step final-newline` passes.
- A fresh tracked-file scan reports no in-scope non-empty text violations.
- Fuzz corpus,
  TOML fixture,
  and `dist/final/node` paths have no fixer-induced diff.
- File-enforcer regeneration is idempotent after the canonical GPL change.
- Rebuilding tracked plugin artifacts produces zero final LF bytes in all 18 tracked JavaScript and declaration files.
- The rebuilt files save one byte each and remain unchanged by pre-commit,
  explicit check,
  and explicit fix surfaces.
- Config-tsdown lint and type checking pass with zero warnings or errors.
- The representative plugin build and end-user executable invocation pass.
- The plan,
  decision,
  and troubleshooting documents match the implemented behavior.

## Reference sources

- Git attributes documentation:
  <https://git-scm.com/docs/gitattributes>
- hk 1.50.0 newline builtin:
  <https://github.com/jdx/hk/blob/v1.50.0/pkl/builtins/newlines.pkl>
- hk 1.50.0 end-of-file fixer:
  <https://github.com/jdx/hk/blob/v1.50.0/src/cli/util/end_of_file_fixer.rs>
- hk hook behavior:
  <https://hk.jdx.dev/hooks.html>
