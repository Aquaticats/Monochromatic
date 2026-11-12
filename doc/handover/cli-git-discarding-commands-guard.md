# The discarding-commands guard: every file a form overwrites or deletes

Branch: `fix/git-policy-cli-discarding-commands-guard-every-file-they-overwrite-or-delete`
in `.claude/worktrees/pi-discarding-commands-guard-every-file-they-overwrite-or-delete`,
started at `main` (release 0.6.1-alpha.12). Worktree of run: `pi-discarding-commands-guard-every-file-they-overwrite-or-delete`.

## Why the run exists

Two independent audits of 0.6.1-alpha.12 ran the discarding commands through the built wrapper
beside real Git and found the same hole: the guard decided from a question that does not see every
file the command would overwrite or delete. Each case below ended with exit 0, the file gone, no
notice and no copy.

Reports:

- `/var/home/user/temp/agent/kimi-trials/kimi-audit-alpha-12-discarding-commands/report.md`
- `/var/home/user/temp/agent/sonnet-audits/alpha-12-merge-discarding-commands-lfs-and-package-repair/report.md`

Run scratch (reproduction scripts, fixtures, measurements):
`/var/home/user/temp/agent/pi-discard-guard/`.

## What is done

### Merges (before any fix)

- `e311f1cd8d` merges `fix/git-policy-cli-review-findings-clean-saves-every-nested-repository`
  (clean and fast; its own tests showed 50/50 unit and 36/38 binary, the two failures being
  `worktree_clone::a_clean_ordinary_main_checkout_is_cloned_and_equals_gits_own_checkout` and
  `worktree_snapshot::a_clean_main_checkout_is_snapshotted_and_equals_gits_own_checkout`, which
  fail because a snapshot of this host's main checkout takes longer than the 5 second notice under
  a load average above 100; they fail the same way alone).
- `16aba260d9` merges `fix/git-policy-cli-clean-inside-a-shell-alias-is-measured-where-git-runs-it`
  after resolving six conflicts. The shell-alias branch's second dry run at one force was dropped
  rather than kept beside the walk main now uses, because Git reads no bare repository as a
  repository and the walk that main keeps does. Its `run_in` plumbing stayed.
- `4e9fe0b911` reads a clean's printed names from the directory the dry run starts in. Without it
  the shell-alias branch's executable control failed on the merge (0 saved nested repositories).

### Fixes, each with its finding

1. `e9ff3da12f` (finding 4, force abbreviations): `command_discard.rs` now resolves a long option
   by Git's own rule (exact name, or the only sibling starting that way) with each command's
   sibling names from its own table, so `rm --f`, `mv --f` and `checkout-index --f` are forces.
   `--abort`, `--staged`, `--worktree`, `--source`, `--conflict` and `--help` are read the same way.
2. `3b93f9ee02` (findings 1, 2, 3, 5 and 6 in one commit, from one cause):
   - `command/command_written_paths.rs` reads which tree paths a form writes (whole tree, the tree
     paths a pathspec reaches, or the index), and `repo_read/repository_facts_written_paths.rs`
     asks Git for them.
   - `repo_read/repository_facts_paths_in_the_way.rs` asks which of those paths the worktree holds
     and Git did not put there (untracked and ignored listings, `--directory`, only the outermost
     asked path, because a pathspec inside a nested repository makes Git answer nothing at all).
   - `policy/policy_linked_worktree_saved_overwrite.rs` copies each file and moves each directory
     whole to `monochromatic-git-policy-cli/discarded`, prints one line per entry naming the copy,
     and records `saved-overwritten`; only a copy that cannot be made refuses, with the reason.
     The old ignored-file refusal is gone, so `checkout <rev> -- .env` and
     `restore --source=<rev> -- .env` now end alike: both save and run.
   - `repo_read/repository_facts_edits_the_status_hides.rs` reads the `assume-unchanged` marks and
     compares the worktree with the index, so a marked edit gets what the same edit gets unmarked.
   - a clean's nested-repository reading no longer waits for `-d` (finding 5's pathspec case), and
     the interactive transform likewise.
   - an undefined command word is forwarded with `-c help.autocorrect=0`, so Git prints its own
     error and the most similar command and runs nothing (finding 6).
3. `553800d6aa` (finding 1 and 3 tests): the audits' two matrices as a binary test,
   `src/native/binary/binary_discard_in_the_way_matrix_tests.rs`, and four controls that expected
   the old refusal rewritten to expect the save.
4. `fe9459d104`: the unit controls of the questions and the forwarded words, plus one stale
   expectation that could never have passed (an abort whose operation waits for nothing is left to
   Git before any status question).

Reproductions of all six findings, before and after, ran through the built wrapper in disposable
repositories with their own `HOME` and `XDG_RUNTIME_DIR`:
`pi-discard-guard/f01_untracked_in_the_way.ts` and `pi-discard-guard/f02_to_f06.ts`
(`WRAPPER=<built binary> node <script>`).

## What is still open

- **Documentation.** `SPEC.md` and `README.md` still describe the tracked-only question and the
  ignored-file refusal. The passages to rewrite: `SPEC.md:16111-16129` (the guard's question),
  `16066-16109` (aliases and unknown names, for the autocorrect switch), the ignores-a-tracked-path
  refusal passage (`~16320-16345` and the audit fact `ignoredAndTracked`), and the clean's
  `-d` gate; `README.md` where it lists what the guard asks and what the abort forms refuse.
  Nothing under `doc/decision/` may be edited, and nothing there covers this guard.
- **Cost measurement.** The guard's added cost on `reset --hard` and `checkout --force` in a
  worktree of this repository's size, before and after, with a same-binary control for the spread.
  Not started.
- **Clippy** at the final commit: `mise run //package/git-policy/cli:native:clippy`. Not run yet.
- **Binary suites** beyond `discard::` and `discard_in_the_way_matrix::` (policy, controls,
  forwarding, builtin) are not run since the last code change.
- **Final report** for the brief: the matrix, the cost, the clippy and test summaries, and which
  discarding command is known to still destroy a file without a refusal or a copy.
- **Push.** `git push` is refused for want of a login (`could not read Username for
  'https://github.com'`); the coordinating session pushes.

## Where the code is

- `src/native/command/command_written_paths.rs` and `_tests.rs`: the form's written paths.
- `src/native/repo_read/repository_facts_written_paths.rs`: the queries for them.
- `src/native/repo_read/repository_facts_paths_in_the_way.rs` and `_tests.rs`: the entries in the way.
- `src/native/repo_read/repository_facts_edits_the_status_hides.rs` and `_tests.rs`: the marks.
- `src/native/policy/policy_linked_worktree_saved_overwrite.rs` and `_tests.rs`: the save.
- `src/native/command/rule_linked_worktree_saved_overwrite_messages.rs` and `_tests.rs`: the words.
- `src/native/policy/policy_linked_worktree_nested_repositories.rs`: one place a form is let
  through, now saving what it overwrites, then the clean's nested repositories.
- `src/native/policy/policy_linked_worktree.rs`: the status question and the stash question; the
  ignored-file refusal block is gone.
- `src/native/invocation/forwarded_command.rs`: `missing_word` and `without_autocorrection`.
- `src/native/command/rule_worktree_dirty.rs`: the two listings and the reading of an answer.

## Results already measured

- Unit tests of the touched modules: `1433 tests run: 1433 passed, 5933 skipped`
  (`mise run //package/git-policy/cli:native:test:host -- -E 'test(policy) | test(command) |
  test(invocation) | test(repo_read)'`).
- `discard::` binary controls before the four rewritten ones: 41/45; after the rewrite the four
  pass (see `553800d6aa` and the run of `test(discard::a_forced)`: 4/4).
- `discard_in_the_way_matrix::` 2/2 (`every_form_over_an_untracked_file_keeps_or_saves_it`,
  `a_marked_edit_is_treated_as_the_edit_it_is`).
- Every finding reproduced through the built wrapper after the fixes: `RESULT finding-1: no defect
  reproduced` and `RESULT findings-2-6: no defect reproduced`.

## Known remaining hole (for the report)

`git clean` keeps its recorded "allow and log" answer for ignored and untracked files (owner's
2026-11-10 decision, out of scope for this run), so `git clean -fx` and `git clean -fX` still
delete those files with no copy. Nested repositories are moved aside. `mv --f` moves the edit to
the destination rather than copying it, which the guard now refuses like `mv --fo`.
