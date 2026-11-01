# cli-git native retirement lane

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/webapp-productivity/done-postcss` is now `package/done/variant-postcss`;
> `package/stub/throwing` is now `package/pnpm-override/stub-throwing`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/file-enforcer` is now `package/file-enforcer/core`;
> `package/module/css-edit.bench` is now `package/css/core-edit.bench`.

This file holds the current state of the retirement of the TypeScript Git policy wrapper;
 when the state changes,
replace the text instead of appending.
The plan is `doc/planning/cli-git-native-cutover.md`,
 sections `Retirement list` and `Order of the retirement commits`.

## Purpose and how to respond

Branch `feat/cli-git-native-retire`,
 based on `main` at `f4fc1fa213`,
 worktree `.claude/worktrees/cli-git-lane-retire`.
The branch deletes the TypeScript wrapper (every file of `package/git-policy/cli` outside `src/native/`) and the four
authoring packages `package/git-policy/{api,forbidden-strings,markdown-lint,repository}`.
To respond:
 merge the branch into `main`,
 then remove the ignored leftovers of the main checkout listed under
`Cleanup for the main checkout`,
 and read `Leftovers` for what the branch did not change.

The pinned incumbent revision,
 the last commit that contains the TypeScript,
 is
`f4fc1fa213ece65672e4314983a75266837892f9`.

## Commits

- `3584fdbc67` records the pinned revision in `doc/planning/cli-git-rust-behavior-ledger.md` (new section
  `Pinned incumbent revision`) and replaces the stale switch-state text of the queue item in
  `doc/handover/cli-git-rust-implementation.md`.
- `a2b56c548c` deletes the 6 `bin/differential-*.mjs` files,
   the 21 `bin/dependent-version-*.mjs` files,
   the tasks
  `native:config:differential`,
   `native:differential:commit` and `native:differential:dependent-version`,
   the ignored
  corpus test `corpus_from_environment` of `src/native/dependent_version_differential_tests.rs`,
   the four packages
  with their ignored `node_modules`,
   the three mirror blocks of `file-enforcer.config.ts` and
  `package/dev-script/file-enforcer/src/generated-policy-contracts.unit.test.ts`
  (deleted 2026-10-13 with the differentials, no replacement);
   it drops the cli devDependency on
  `git-policy-repository`,
   fixes the `markdown/autofix` sentence of `package/config/lfs-r2-worker/README.md`,
   and
  regenerates `pnpm-lock.yaml` and `package/config/pnpr/config.yaml`.
- `642b6d72cb` deletes the 478 tracked files of `src/` outside `src/native/`,
   `rolldown.node.config.ts`,
  `cli-git.config.ts`,
   `.github/workflows/cli-git-trust.yml`,
   the e2e surface probe files and the tasks `run`,
  `build:js`,
   `build:js:node`,
   `watch:build`,
   `watch:build:js`,
   `watch:build:js:node`,
   `verify:supported-runtime`,
  `pack:npm`,
   `test:built:trust`,
   `test:e2e:concurrent` and `test:e2e:concurrent:native:surface`;
   it reduces
  `package.json` and `tsconfig.json`,
   adds the task `lint:clippy`,
   edits the e2e runner,
   the Containerfile and
  `bin/e2e-native-container.mjs`,
   rewrites `SPEC.md`,
   and edits the root README,
   the forbidden-strings README,
   one
  comment of the oxlint plugin and `cli-git.config.jsonc`.
- `8cf1d0f5a6` removes `package.json:36` (the root dependency on the incumbent) and the matching `pnpm-lock.yaml`
  lines,
   and the `mise.toml` path entry `package/git-policy/cli/node_modules/.bin`
   (installed with the TypeScript wrapper, gone since 2026-10-13).
- `7c7e5c8140` names a constant that oxlint flagged and states the Git floor of the end-to-end run.
- `d42107ec3b` adds this file, the in-place notes to four lane records and the state paragraph of the cutover
  runbook.
- The commit after it rewrites the `SPEC.md` section `Compatibility and degradation` and the e2e README paragraphs that
  described a Git 2.40.0 floor, and corrects the cleanup list and the lint baseline of this file.

## Deviations and decisions

- The planned step 2 deleted `src/optional/`;
   it was deleted in the step 3 commit instead,
   because
  `src/authoring.ts` imported it and the TypeScript package would not have type-checked between the commits.
  The mirror blocks were deleted in step 2,
   so `sync:files` stopped regenerating the directory there.
- `valibot` and `nano-spawn` stayed in the manifest through step 2 (`src/` still imported them) and went in step 3.
- The lockfile was regenerated in steps 2 and 3 as well as 4,
   because each manifest change made the frozen install
  fail otherwise.
  Step 4 changed three lines.
- `test:e2e:concurrent:image` was kept and rewritten (the retirement list names the incumbent image,
   but the native
  image needs the same base):
   it runs `node bin/e2e-native-container.mjs base`,
   which builds
  `localhost/cli-git-concurrent-e2e:<tag>` from the Containerfile with no tarball.
  The native image builds the base when it does not exist,
   which removes the manual "build it first" step.
  `GIT_POLICY_E2E_BASE_TAG` (default `latest`) names another tag,
   so a trial build never replaces the shared image.
- An unset `E2E_WRAPPER` selects native;
   the value `incumbent` is refused.
- The step 4 `mise.toml` change is the one line the generator removes for the cli package.
  None of the harness's remaining dependencies (`config-typescript`,
   `module-caught-value`,
   `module-logger`,
  `module-test`) declares a `bin`,
   which is why `package/git-policy/cli/node_modules/.bin`
   (installed with the TypeScript wrapper, gone since 2026-10-13)
   no longer exists.
  `mise run sync:files` in this worktree also removes 13 other `node_modules/.bin` path entries (eight under
  `package/claude-code-plugin`,
   plus `package/dev-script/file-enforcer`,
   `package/mcp/mvm`,
  `package/module/css-edit.bench`,
   `package/stub/throwing` and `package/webapp-productivity/done-postcss`) because
  this worktree has no package build output for their binaries;
   those were not committed.
  The commit message of `8cf1d0f5a6` says 14 for that count;
   the count of other entries is 13,
   the 14th is the cli
  entry.
- The `SPEC.md` sections `Policy inputs and read sets` and `Public authoring declarations` described the TypeScript
  design.
  The first became `Policy re-run after replay` (the native revalidation converges every policy again;
   verified in the
  header of `src/native/commit_revalidation.rs` and by the ledger entry whose native state is "absent").
  The trust commands in `Management grammar` now describe the retired forms that print an explanation and exit `0`
  (verified in `src/native/management_tests.rs`).

## Gates

All through `node "${HOME}/temp/agent/gate-slot.ts"` with `GIT_POLICY_NATIVE_IMAGE_TAG=retire`.

- After step 2:
   `native:clippy` finished clean;
   `native:test:container` passed,
   test image
  `848309f046b963b8415c1445833732891105ebeb542f8de0af560568d5b0f593`:
   1660 library tests passed (5 ignored),
   118
  binary tests passed (1 ignored),
   and two further runs of a single test passed.
- After step 3 and again on the final head:
   `native:test:container` passed with the same counts,
   test image
  `523330ef13d5a5c48101d91357f8a6e2fd3e84873339e6c1a5499369d0adcbc8` (both runs produced the same image;
   the crate
  snapshot did not change between them);
   Clippy inside the gate finished clean;
   `lint:clippy` passed.
- E2E after step 3:
   `GIT_POLICY_E2E_BASE_TAG=retire GIT_POLICY_NATIVE_IMAGE_TAG=retire mise run
  //package/git-policy/cli:test:e2e:concurrent:native --git 2.55.0 --seed 1` exited 0:
   36 scenarios passed and 2
  skipped (the two `replay-unavailable-*` scenarios),
   the same result as before the retirement.
  The base image built under the tag `retire` in 10 seconds from cached layers,
   so the edited Containerfile builds.
  Without `--git`,
   the same task exited 1:
   52 of 114 runs failed,
   every one on Git 2.39.5 or 2.40.0,
   below the native
  floor of Git 2.55.0,
   and Git 2.55.0 passed 36 and skipped 2 (seed 91442351).
- `test:unit` of the cli package:
   all e2e unit tests passed,
   including the rewritten `wrapper-selection` tests.
  `lint:types` of the cli package passed.
  `lint:oxlint:paths` passed on the touched TypeScript files and on `bin/e2e-native-container.mjs`.
  The whole-package `lint:oxlint` reports diagnostics in `bin/build-git-test-image.mjs`,
   `bin/mutate-native-*.mjs`
  and `native-interop/*.mjs`,
   none in a file this branch changed.
  They predate the branch:
  on a detached worktree of `f4fc1fa213` the same two-file run (`bin/build-git-test-image.mjs`,
  `native-interop/support.mjs`) reported 98 warnings and 6 errors,
  and on this branch it reports the identical counts.
- `monochromatic-lint` on the touched Markdown files:
   no finding on an added line.
  Rendering checked with `render-docs.ts` for `SPEC.md`,
   the e2e README,
   the forbidden-strings README,
   the ledger and
  this file.
- `package/dev-script/file-enforcer:test:unit` fails in this worktree for 20 files with
  `Cannot find module .../package/dev-script/file-enforcer/dist/final/node/index.mjs` (no build output);
   the deleted
  `generated-policy-contracts.unit.test.ts` was not among them.

## Leftovers

- `native:interop` could not run after the retirement because it copied the main checkout's
  `package/git-policy/cli/dist` and `/opt/cli-git` into its image.
  The matrix then ran against an incumbent rebuilt from the pinned revision,
  and the user decided to delete the incumbent builder and every driver that needs a live incumbent afterwards
  (`doc/handover/cli-git-native-interop.md`).
- `slopo.ignore.txt` lines 45,
   50,
   69,
   200,
   309,
   324 and 373 are dismissals whose subjects are deleted files.
  They were not edited:
   the header counts ("9 live,
   10 dead" and the other sections) refer to a report generated on
  2026-10-07,
   and removing keys would make those counts wrong without a new report.
- The `SPEC.md` section `Compatibility and degradation` was rewritten for the Git 2.55.0 floor,
  and the e2e README says the 2.39.5 and 2.40.0 runs fail by design;
  the image still carries those two Git builds.
- `SPEC.md` still describes,
   in sections the plan did not name,
   the hook dispatcher,
   replay and lock behavior that
  the native crate shares with the incumbent;
   the sections were checked for dangling cross-references to deleted
  headings (none) but not re-verified line by line against the native source.
- `.changeset/config.json` still lists `@monochromatic-dev/git-policy-cli` under `ignore` (the package name is kept).
- Historical records (44 handover,
   45 planning,
   62 troubleshooting,
   17 audit,
   8 decision files and others) cite
  deleted TypeScript paths and stay as written;
   four lane records carry an in-place note for tasks they name.
- `doc/handover/cli-git-rust-implementation.md` still contains two sentences in the queue that are stale:
   "Current
  production tools remain active until cutover."
   and "The first crates.io publication still needs explicit user
  approval."
  Edits of both were refused by the tool's permission check and not retried.
  The native wrapper is the production tool and the crate is published.

## Cleanup for the main checkout

Do these only after the branch is merged and `pnpm install` has run there,
 with explicit paths (never `git clean -X`;
`.cache/cli-git-bin`,
 `.cache/forbidden-strings.rules.txt` and `package/git-policy/cli/target` are live).
Listed read-only on 2026-10-14 with `ls -d` and `git check-ignore --verbose`;
 every path is ignored by `.gitignore`.

- `package/git-policy/cli/dist` (holds `final`,
   `pack` and `perf`;
   2.2 MiB).
- `package/git-policy/cli/.cache`
  (gone since 2026-10-13; the launcher directory is `.cache/cli-git-bin` at the repository root)
  (holds `typescript`).
- `package/git-policy/api`,
   `package/git-policy/forbidden-strings`,
   `package/git-policy/markdown-lint` and
  `package/git-policy/repository`:
   after the merge only ignored `dist`,
   `.cache` and `node_modules` remain in each;
  remove the four directories.
- `node_modules/.bin/git`:
  the incumbent's shim (3,598 bytes).
  Measured in the worktree:
  after the root dependency was gone,
  `pnpm install --frozen-lockfile --offline` (also with `--force`) left the file in place,
  so `pnpm install` does not remove it.
  Remove that one file by explicit path.
  The first `PATH` entry `.cache/cli-git-bin` shadows it meanwhile.
- `package/git-policy/cli/node_modules` stays:
  pnpm manages it,
   and the harness's unit tests need its links to `module-test`,
  `module-logger` and `module-caught-value`.
  `pnpm install` in the main checkout prunes the incumbent's closure from it.
- After the merge,
   run `mise run sync:files` in the main checkout and confirm that `mise.toml` does not change.
- The main checkout has an uncommitted change to `pnpm-lock.yaml`;
   it was not touched and will collide with this
  branch's lockfile edits (three hunks in total) at merge.

## What remains

- The queue item "retirement of the old TypeScript implementation is verified complete" in
  `doc/handover/cli-git-rust-implementation.md` stays open until the trailing gates are recorded.

## Rerun commands

```sh
# Repository root of the worktree: the container gate and the end-to-end run.
GIT_POLICY_NATIVE_IMAGE_TAG=retire node "${HOME}/temp/agent/gate-slot.ts" \
  mise run //package/git-policy/cli:native:test:container
GIT_POLICY_E2E_BASE_TAG=retire GIT_POLICY_NATIVE_IMAGE_TAG=retire node "${HOME}/temp/agent/gate-slot.ts" \
  mise run //package/git-policy/cli:test:e2e:concurrent:native --git 2.55.0 --seed 1

# Light checks.
mise run //package/git-policy/cli:test:unit
mise run //package/git-policy/cli:lint:types
```
