# Scratch scripts folded into packages, and the ones that are one-offs

Written 2026-11-10T18:50:55-05:00 by the agent that owns the two groups whose package was not chosen
in `~/temp/agent/pi-runs/inventory-of-scratch-scripts-by-owning-package.json`:
"GitHub issue work" and "other".
That inventory sorted 137 scripts by file name,
so this record comes from reading each of the 137.

Two packages were built for the parts that repeat.
The rest are one-offs:
some record their result in a document,
some record it nowhere,
and some belong to a package another agent is folding,
so they stay in the coordinator's folder for that agent.

Nothing here deletes a scratch script.
The coordinator switches the sessions over and deletes the scratch copies after this branch lands.

## GitHub issue work: `package/issue-acceptance/cli`

Forty-nine issue scripts differ only in their data:
which issues,
which checks of `main` decide each one,
and the words the issue receives.
`issue-acceptance settle` takes that as a plan file,
decides every check first,
and writes on GitHub only with `--apply`.

The checks the family needed are all in the command:
`absent`,
`present`,
`exactly`,
`atLeast`,
`exists`,
`gone`,
`fileContains`,
`fileLacks`,
`pathMatching`,
and `onBranch`,
plus a plan entry that comments and leaves the issue open,
and `commitCorrections` for the two runs that posted a correction on a commit message.

These forty-nine scripts are replaced by one `settle` run with a plan:

- `close-bash-limit-issues.ts`
- `close-catalog-stub-vmsync-design-issues.ts`
- `close-conf-kv-store-issues.ts`
- `close-doc-folder-citation-issues.ts`
- `close-done-app-issues.ts`
- `close-figma-to-penpot-issues.ts`
- `close-file-enforcer-perf-issues.ts`
- `close-git-executable-issues.ts`
- `close-issues-1233-to-1237.ts`
- `close-issues-after-auto-mode-rebuild.ts`
- `close-issues-after-citation-sweep.ts`
- `close-issues-after-claude-code-plugin-landing.ts`
- `close-issues-after-import-attributes-landing.ts`
- `close-issues-after-open-issues-landing.ts`
- `close-issues-after-package-test-gaps.ts`
- `close-issues-after-repository-tooling.ts`
- `close-issues-from-landing-rounds-18-to-20.ts`
- `close-issues-newer-half-2026-11-09.ts`
- `close-issues-older-half-2026-11-09.ts`
- `close-issues-settled-by-test-coverage.ts`
- `close-kiwi-bench-ide-issues.ts`
- `close-logger-css-model-selection-issues.ts`
- `close-model-review-issues.ts`
- `close-module-doc-issues.ts`
- `close-paused-forge-editord-issues.ts`
- `close-pnpr-vm-builder-goal-issues.ts`
- `close-prose-issues-whose-acceptance-holds.ts`
- `close-rand-issues.ts`
- `close-rolldown-1269-and-1271.ts`
- `close-rolldown-and-fixture-issues.ts`
- `close-round-12-issues.ts`
- `close-round-13-issues.ts`
- `close-round-23-issues.ts`
- `close-round-35-issues.ts`
- `close-round-36-issues.ts`
- `close-round-60-issues.ts`
- `close-round-40-issues.ts`
- `close-round-60-issues.ts`
- `close-rss-fs-path-file-manager-morph-issues.ts`
- `close-search-fetch-issues.ts`
- `close-settled-file-enforcer-issues.ts`
- `close-settled-linter-issues.ts`
- `close-settled-mpm-shadow-issues.ts`
- `close-settled-search-shadow-issues.ts`
- `close-settled-wrapper-round-3-issues.ts`
- `close-terminal-exec-electron-issues.ts`
- `close-test-expect-issues.ts`
- `close-typeface-issues.ts`
- `close-wrapper-issue-branch-issues.ts`
- `comment-on-wrong-copies-issues.ts`

Two of those runs carry a part a plan cannot express,
and the part stays a one-off:

- `close-settled-linter-issues.ts` closes nine issues,
  which `settle` does,
  and also opened one new issue with `gh issue create`.
  Creating an issue is `package/open-code-review-issue/cli`'s job,
  and that package is create-only by design,
  so the open part was run once and is not folded here.
- `close-round-60-issues.ts` read each issue's state before closing it,
  which `settle --apply` now does for every issue it settles.

Three listing scripts are replaced by the command's read-only subcommands:

- `show-issue-bodies.ts` is `issue-acceptance show --repo OWNER/NAME NUMBER...`.
- `show-acceptance-of-issues.ts` is the same call with `--acceptance`,
  which prints the title and the sections naming what the issue asked for.
- `ready-bugs-by-package.ts` is `issue-acceptance ready-bugs --repo OWNER/NAME [--out FILE]`.

### Owner answers: `package/owner-answer/cli`

Five scripts recorded the owner's answers in `doc/handover/questions-for-the-owner.md`
by hard-coding the block and the headings it replaced.
`owner-answer --answers FILE` takes the block and every part to remove as data,
refuses a heading the file no longer holds,
and writes only with `--apply`.
These five are replaced by one run with an answers file:

- `apply-answers-of-2026-11-10-to-questions-file.ts`
- `record-answers-to-owner-file.ts`
- `record-owner-message-container-base.ts`
- `record-plain-answers-of-the-morning.ts`
- `record-question-tool-answers-batch-1.ts`

`record-is-instead-of-remeda.ts` replaced one note with a later message from the owner.
Its result is in `doc/handover/questions-for-the-owner.md`,
under the answer that first named `remedajs.com` and then `sindresorhus/is`.
It is a one-off and safe to delete.

### Left for the coordinator in this group

Two scripts in this group read the Pi runs' own `summary.json` files,
which is `package/pi-shared/subagent-driver`'s subject,
not GitHub's:

- `owner-choices-in-reports.ts`
- `show-issue-verdicts-of-finished-runs.ts`

Two more are one-offs whose result no document records:

- `who-asks-for-the-lfs-username.ts` walks process parents to name the command behind a credential prompt.
  The subject is recorded in `doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md`
  and `doc/troubleshooting/lfs-credential-in-url.md`,
  but neither names the ancestry walk or the command it found,
  so the script is safe to delete only if the session that ran it still holds that answer.
- `sweep-prose-issue-acceptance-on-main.ts` ran fence-aware sweeps of the prose issues against `main`'s blobs.
  Its counts went into the closing comments of the prose issues themselves,
  and the durable form of those sweeps is `monochromatic-lint` with the plan checks of `issue-acceptance`,
  so no doc names the script.

## The other group: seventy-seven probes and one-off measurements

Most of this group measures one thing once.
A probe that answered a question is worth keeping only when a document cites it,
and only two documents cite one:

### Copied beside the document that cites it

`doc/planning/filesystem-for-the-repository-volume.md` cited two scripts by their path in the coordinator's folder.
Both were copied unchanged beside that document,
and the document now points at the copy:

- `compare-deletes-on-three-filesystems.ts` is now `doc/planning/filesystem-for-the-repository-volume.compare-deletes.ts`.
- `enable-trim-on-the-860.ts` is now `doc/planning/filesystem-for-the-repository-volume.enable-trim.ts`.

Five other documents cite a script from this folder that belongs to another agent's subject;
those are listed under the package they belong to below,
and their citations stay as they are until that package lands.

### Recorded in a document

Each of these is a one-off whose result a document holds,
so the scratch copy is safe to delete:

- `read-btrfs-and-crypt-settings.ts`,
  `how-each-filesystem-deletes-a-file.ts` and `how-openzfs-deletes-a-file.ts`:
  `doc/planning/filesystem-for-the-repository-volume.md`
  records the dm-crypt,
  queue,
  btrfs and TRIM readings,
  and quotes the `unlink` source of each filesystem.
- `compare-cachyos-and-arch-contents.ts` and `how-heavy-are-container-bases.ts`:
  `doc/decision/container-base-distro.md` holds the decision,
  and `doc/handover/questions-for-the-owner.md` records the owner's answer with the sizes the probes measured.
- `apply-wg-quicker-pi-exemption.ts`:
  `doc/planning/wg-quicker.md`.
- `retirement-fixture-rows-added.ts`:
  `doc/troubleshooting/deps-update-api-drift.md` and `doc/decision/pi-model-retirement.md`
  record the regeneration of the retirement fixture.
- `dispatch-music-player-dry-run-when-pushed.ts`:
  `doc/runbook/release-the-music-player.md` holds the dry-run step it dispatched.
- `where-the-lfs-setting-comes-from.ts`:
  `doc/troubleshooting/lfs-credential-in-url.md` records which config scope supplies the LFS settings,
  with every value masked.
- `turn-off-claude-ai-skills-sync.ts`:
  `doc/handover/questions-for-the-owner.md` records the owner's instruction and that both sync settings are now `false`.
- `apply-kopia-resilience.ts` and `wait-for-kopia-job-outcome.ts`:
  the owner's instruction is recorded in `doc/handover/questions-for-the-owner.md`;
  the settings the script applied are not described by any document,
  so these are safe to delete only with the owner's instruction as the record.

### One-offs with no record

These answer a question that no document holds.
Deleting them loses the only written form of what they measured:

- `add-guardrail-to-pi-settings.ts` and `remove-guardrail-from-pi-settings.ts`
- `apply-pcloud-crash-capture.ts`,
  `pcloud-crash-capture-facts.ts`,
  `pcloud-crash-capture-facts-2.ts`,
  `probe-pcloud-capture-mechanisms.ts` and `wait-for-next-pcloud-crash.ts`
- `kopia-job-facts.ts` and `kopia-pack-write-errors-and-restart-control.ts`
- `brace-and-extglob-forms-through-the-parser.ts` and `fix-plain-brace-words-the-analyzer-refused.ts`
- `distinct-sources-of-credential-prompts.ts`,
  whose subject document,
  `doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md`,
  names one cause and not the set it found
- `what-fetch-says-when-a-body-is-cut.ts`
- `is-it-cloudflare-facts.ts` and `wifi-and-host-facts-for-cut-offs.ts`
- `measure-rule-lengths.ts`,
  which measures `AGENTS.md` rules against rule `RLM`
- `probe.ts`,
  the argv runner the coordinator used for one-off command lists
- `sort-scratch-scripts-by-owning-package.ts`,
  whose output is the inventory itself
- `move-function-before-caller.ts`,
  `move-server-regions.ts` and `move-transport-regions.ts`
- `create-logger.instrumented.ts`,
  `server-resources-head.ts`,
  `unix.fixed1482.ts` and `with-host-timeout.current.ts`,
  which are instrumented or mutated copies of files that live in packages
- `delete-move-leftovers.ts`,
  `delete-old-860-trees.ts` and `what-the-old-860-trees-are.ts`,
  whose owner answers are in `doc/handover/questions-for-the-owner.md`
  but whose lists of deleted paths are in no document

### Left for the coordinator: another agent's package

These belong to the three packages the other agents are folding.
They stay in the coordinator's folder,
and this record only names them.

`package/git-policy/cli`,
the wrapper and its own measurements:

- `are-the-worktree-adds-still-moving.ts`
- `what-are-worktree-adds-deleting.ts`
- `what-are-the-stuck-worktree-adds-waiting-on.ts`
  (cited by `doc/planning/cli-git-worktree-add-stand-ins-off-the-waiting-path.md`)
- `how-long-will-the-worktree-adds-take.ts`
- `what-is-jamming-the-volume.ts`
- `does-the-wrapper-stop-each-whole-tree-add.ts`
- `what-does-each-whole-tree-spelling-select.ts`
- `find-functions-lost-in-merges.ts`
  (cited by `doc/handover/cli-git-rust-implementation.md`)
- `does-a-unit-survive-its-waiting-client.ts`
  (cited by `doc/handover/cli-git-rust-implementation.md`)
- `probe-btrfs-subvolume-delete-and-rename.ts`
  (cited by `doc/planning/cli-git-worktree-add-cost-of-its-trash.md`)

`package/agent-branch-landing/cli`,
landing and the checks before it:

- `check-error-handling-branch-on-main.ts`
- `check-fuzz-branch-after-merging-main.ts`
- `env-local-json-read-test-branch-against-main.ts`
- `compare-todays-landing-messages-with-their-commits.ts`
- `show-package-lint-findings.ts`
- `with-clause-whitespace-red-then-green.ts`
- `hold-cargo-shadow-task-wiring-until-release.ts`
- `restore-cargo-shadow-task-wiring.ts`
- `lock-and-install-cargo-shadow-alpha-8.ts`
  (the wiring these three move is recorded in `doc/handover/cargo-shadow-lean-build-folders.md`)
- `rebuild-and-compare-claude-code-plugin-bundles.ts`

`package/pi-shared/subagent-driver`,
starting and watching agents and reviews:

- `count-the-twenty-slots.ts`
- `wait-for-pi-units-to-start.ts`
- `wait-for-pi-agents-to-appear.ts`
- `wait-until-sixteen-pi-agents-run.ts`
- `wait-for-launchers-to-become-units.ts`
- `wait-for-first-model-turn-of-sol-audits.ts`
- `when-did-these-runs-end.ts`
- `how-did-the-recent-wrapper-runs-end.ts`
- `how-fast-is-the-work.ts`
- `combine-round-82.ts`
- `compare-the-other-provider-for-the-bundle.ts`
- `review-branch-with-sol.ts`
  (cited by `doc/handover/cli-git-rust-implementation.md`)
- `find-the-commits-two-reviews-read.ts`

## Counts

- 137 scripts read:
  60 in "GitHub issue work" and 77 in "other".
- 49 replaced by `issue-acceptance settle`,
  two of which also carry a one-off part named above.
- 3 replaced by `issue-acceptance show` and `issue-acceptance ready-bugs`.
- 5 replaced by `owner-answer`.
- 3 one-offs in the issue group,
  each with the record or the gap named above.
- 12 one-offs in the other group whose result a document records.
- 28 one-offs in the other group that no document records.
- 2 copied beside the document that cites them.
- 35 left for the coordinator because they belong to another agent's package.

## What a session still has to write

The commands take the varying parts as data,
so each round still needs its own plan file or answers file.
Those files are per-round notes,
not package code:
a session writes one under its own scratch folder,
runs `issue-acceptance settle --plan` or `owner-answer --answers`,
and the plan is worth keeping only while the round is open.
