# Layout reader and a user-level `core.fsmonitor = true`

Note 2026-11-04T22:23:58-05:00: the program `cli-git-native` is now `monochromatic-git-policy-cli`,
the command word `git cli-git` is now `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` is now `monochromatic-git-policy-cli.config.jsonc`,
and the folders, logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
This record keeps the names of its time.

This handover records why the in-process layout reader declined every repository of the user whose own `~/.gitconfig` holds
`core.fsmonitor = true`,
what each caller of the reader does about it,
the measurements,
and the controls.
Respond with a veto of a named decision,
or with evidence that one does not hold.

The work is on the branch `reader-fsmonitor` (worktree `.claude/worktrees/reader-fsmonitor`),
based on main with
`merge-trigger-fix` merged in,
because `read_layout_after_success` is on that branch and not yet on main.

## The cause and how often it declines

The typed-value proof (`repo_read_config_typed.rs`) accepted only false spellings of `core.fsmonitor`,
because a control that
sets the key to a true word starts the monitor daemon.
`read_layout_in`,
`read_layout` and `read_layout_with` all apply that proof,
so one user-level line declines every repository.

Measured on this machine with a throwaway control (not committed),
naming the real Git as the wrapper does:
the readers answered 0 of 135 repositories (the main checkout and its 134 worktrees) with the real `~/.gitconfig`,
and 135 of 135 with only that one line removed from a copy of the file,
and 135 of 135 with the proof skipped.
So `core.fsmonitor` is the only value on this machine that the proof refuses.

## What a decline costs

A wrapped commit in a fixture whose repository configuration holds `core.fsmonitor = true`,
counted by `strace` in
`binary_budget_tests.rs` (Git 2.56.0),
before and after this branch:

- Before:
  13 more Git starts than the budget of a commit without the setting:
  `rev-parse` 6 instead of 1,
  `diff-index` 4 instead of 1,
  `ls-files` 2 instead of 0,
  and one each of `config`,
  `diff` and `diff-tree`.
- After:
  exactly the budget (`a_commit_with_core_fsmonitor_true_costs_what_one_without_costs`).

The three layout questions of a commit started 4 Git processes before and none after
(`worktree_git_dir`,
`resolve_landing_location` and `capture_invocation_layout`,
recorded by a recording Git:
two `rev-parse` location queries,
the `rev-parse` layout report and the `config --list` snapshot).
The same decline is what a key such as `diff.dirstat` causes now;
it measured the same 13.

## Can the value change an answer

Git's `lookup_fsmonitor_settings` (`fsmonitor-settings.c`) reads the key as a boolean first:
true selects the built-in daemon
(or silently selects nothing where the file system cannot host it),
false turns it off,
and any other value is the path of a hook program.
Measured with real Git,
the commands that read the index (`ls-files`,
`diff-index`,
`diff --cached`,
`diff-tree`) start the daemon
under the true word,
and `rev-parse`,
`show`,
`config`,
`for-each-ref` and `symbolic-ref` do not.
The readers start no daemon,
which changes no output.

- The shapes the query readers answer (`ls-files --stage`,
  `diff-index --cached`,
  `diff --cached`,
  `diff-tree`,
  `show`,
  `rev-parse`,
  `for-each-ref`,
  `symbolic-ref`,
  `remote get-url`) print the same bytes,
  standard error and exit status with
  the true word as without it.
  A differential run on 2026-11-02 compared them with real Git;
  with the daemon running and the index extended,
  and the committed control repeats that.
- Setting a hook path (`maybe` in the existing sweep) does change the output of `ls-files` and the cached diffs,
  so a hook path stays declined.
  Git also reads the bare name and the numbers other than zero as true;
  they decline here as well,
  which is conservative and not protection against a malformed value.
- Monitor failures are silent in Git 2.56.0 (finding of the review
  on daemon contact).
  Repro,
  2026-11-02,
  fresh repository with `core.fsmonitor = true` in the user file and no daemon:
  with a directory,
  a regular file or a dangling link at `.git/fsmonitor--daemon.ipc`,
  `ls-files --stage`,
  `diff-index --cached --raw HEAD`,
  `diff --cached --name-only HEAD` and `status --short`
  each printed nothing on standard error and exited 0.
  `GIT_TRACE2_PERF` showed the failure path was taken (`daemon failed to start`,
  `could not start IPC thread pool`),
  so Git reports it only in the trace.
  The committed control `the_query_readers_equal_git_when_the_monitor_cannot_start` compares the readers with Git in that state
  and checks that no monitor is running afterwards,
  so a later Git that starts printing is noticed.
  An exhausted-resource variant was not run:
  the user's `fs.inotify.max_user_instances` is 8192 and shared with the desktop session,
  and exhausting it would break the user's other programs,
  so no such state was made.
- Configuration the monitor reads once it is on,
  from Git's source (`fsmonitor-settings.c`,
  `fsmonitor.c`,
  `compat/fsmonitor/fsm-ipc-unix.c`,
  `builtin/fsmonitor--daemon.c`):
  `fsmonitor.allowRemote` (a boolean,
  read only for a work tree on a remote file system) and
  `fsmonitor.socketDir` (a path,
  read only for a Git directory on a remote file system,
  where `~user` of an unknown user dies)
  are client-side and are now typed in the proof;
  `core.fsmonitorHookVersion` is read only in hook mode,
  which stays declined;
  `fsmonitor.ipcThreads`,
  `fsmonitor.startTimeout` and `fsmonitor.announceStartup` are read by the daemon process only,
  and real Git printed nothing and exited 0 on index queries with each set to a malformed value,
  so they are not typed.
  A remote file system cannot be made in a test,
  so the control for the two typed keys asserts the decline,
  and Git's source is the evidence that Git would speak there.
- The index reader skips the optional `FSMN` and `UNTR` extensions a monitored index carries
  (`repo_read_index.rs`:
  only an unknown required extension,
  `link` and `sdir` are refused).
  A control writes both with a hook that answers "nothing changed",
  without starting the daemon.

## Decisions per caller

- The proof (`repo_read_config_typed.rs`):
  `Kind::OffOrBuiltinDaemon` accepts a false spelling or the words `true`,
  `yes`
  and `on`;
  it replaces `Kind::FalseOnly`.
  This answers `capture_invocation.rs`,
  `service_route.rs` (`worktree_git_dir`),
  `landing_location.rs`,
  `candidate_prediction.rs`,
  `repository_facts.rs`,
  `repo_read_query.rs` and the rest of the callers that run before any Git
  command:
  they compute paths and listings the value cannot change,
  and the proof keeps guarding the wording of a failure Git
  gives for a malformed value.
  The guard for a malformed value is not weakened:
  `maybe`,
  a path,
  `1`,
  `2` and the bare name still decline.
- `lfs_bounded_push.rs` (the auto-push) and `lfs_interactive.rs` (a typed push) read the layout after the plan
  came from Git queries that exited with status 0
  (an interactive push may hold settings from an earlier query),
  so Git had its say about the values those queries consult;
  that is not a certificate for every value,
  and the files are read again,
  not the snapshot the queries saw.
  A change between the two cannot make the push wrong:
  a plan that cannot be narrowed is still pushed by `git-lfs` and Git,
  and the structure checks still apply.
  they use `read_layout_after_success` and the new `read_layout_after_success_for` (which takes the `-C` options).
  A decline there sent the whole push to `git-lfs` for a full scan.
  Controls:
  a push and an auto-push in a repository holding `diff.dirstat` (the stand-in key that the proof refuses) start no
  `git-lfs` when the push adds no pointer;
  before,
  each started `git-lfs push origin main`.
- `management.rs` (`git cli-git service status`) needs no valid configuration,
  so it uses `read_layout_after_success_for`,
  given the invocation's global options:
  `-C <path>` chooses the worktree as for every other command (it used the process directory only,
  so `git -C B cli-git service status` run in A reported A),
  `-c <name>=<value>` changes nothing,
  and any other global option makes the status count any service.
  Control through the dispatch:
  `the_status_follows_the_dash_c_options_of_the_invocation`.
  It also never named the real Git to the reader,
  and the reader then cannot learn the system configuration file and declines
  every repository of every user without `GIT_CONFIG_NOSYSTEM`:
  the status counted any service instead of the worktree's.
  The command now resolves the real Git first,
  as the doctor does.
  This one is a defect for all users,
  not only this one.
  Controls:
  `the_status_names_the_worktree_for_a_user_the_typed_proof_cannot_vouch_for`,
  `the_status_needs_the_real_git_to_name_the_worktree`.

## Controls and red runs

Each mutant was applied by hand to the worktree source,
the targeted tests were run once under a capped unit
(CPU 300%,
memory 8G,
tasks 512,
I/O weight 30),
and the file was restored.
The runner and the mutant list are `~/temp/agent/reader-fsmonitor/run.ts` and `mutate.ts`;
the logs are `m-<mutant>.log` there.

- `proof-true-words-removed` (the proof back to false-only):
  fails
  `core_fsmonitor_accepts_the_switch_words_and_nothing_that_names_a_program`,
  `a_user_level_fsmonitor_true_keeps_the_layout_reader_answering`,
  `the_query_readers_equal_git_under_a_user_level_fsmonitor_true` and
  `a_user_level_fsmonitor_true_costs_the_layout_questions_no_git_process`.
- `proof-accepts-any-text`:
  fails the first two of those,
  which keep hook paths declined.
- `status-typed-proof` and `status-no-real-git`:
  fail the two status controls.
- `prefix-typed-proof`:
  fails `the_after_success_reader_for_a_prefix_follows_the_dash_c_options`.
- `index-rejects-monitor-extension`:
  fails `an_index_with_the_monitor_and_untracked_cache_extensions_is_read`.
- `lfs-interactive-typed-proof` and `lfs-bounded-typed-proof`:
  fail the push and the auto-push controls.
- `budget-proof-true-words-removed`:
  fails the commit budget with the 13 starts above.

## Limits

- The daemon runs only in two controls (the query differential and the budget scenario),
  each stopping it before the fixture
  goes;
  the controls report a file system that cannot host it and compare anyway.
- The commit measurement used a repository-level `core.fsmonitor = true`;
  the proof reads the repository file and the user
  file alike,
  and the unit controls use the user file.
- The after-success readers read the configuration files again.
  What that guarantees is the structure of the layout at the time of reading (owner,
  object format,
  `core.bare`,
  `core.worktree`,
  reftable,
  `HEAD`);
  what it does not guarantee is that the values are the ones the earlier Git command saw,
  or that every value is valid.
- Gate 47 failed `a_user_level_setting_the_reader_cannot_prove_does_not_stop_the_trigger` in `auto_push_trigger_tests.rs`,
  whose premise was that `core.fsmonitor = true` is refused.
  It now uses `diff.dirstat`,
  and a second case shows the true word
  lets the reader answer.
- The 13 starts were measured on Git 2.56.0 in the wrapper's hermetic fixture;
  a real commit adds starts the budget does not
  list when it is not a plain explicit-path commit.
- `management.rs`,
  `repo_read_layout.rs` and `repo_read_config_typed.rs` were over the crate's code-line limit before this
  branch and still are;
  they were not split here.

## 2026-11-02T15:51:29-05:00: monitor bound, service status outside a repository, and configured Trace2

- The owner bounded the work on the monitor (2026-11-02:
  "Why are we messing with git's fsmonitor?
  I already told you it's fine for git to run it.";
  earlier:
  "To stop the fsmonitor program - we don't need to."):
  the wrapper leaves it alone.
  The review's remaining monitor finding,
  a listener answering the monitor's socket with a malformed pkt-line reply,
  is an accepted limit
  recorded in the planning doc and `SPEC.md`;
  no query is declined for it.
- `git ... cli-git service status` now separates "outside a repository" from "cannot tell which repository".
  The reader resolves `-C` options;
  whatever it declines (`--git-dir`,
  `--work-tree`,
  `GIT_DIR`) is put once to the real Git
  (`rev-parse --absolute-git-dir`),
  whose "not a git repository" is the only outside answer;
  a Git that cannot answer,
  or no
  real Git,
  ends the command with a message that no service was counted.
  Control:
  `the_status_follows_the_dash_c_options_of_the_invocation` now selects a repository with no service by
  `-C` plus `--git-dir`,
  one outside any repository,
  and an unresolvable invocation.
- Configuration-enabled Trace2 (`trace2.normalTarget`,
  `trace2.eventTarget`,
  `trace2.perfTarget`,
  from `trace2/tr2_sysenv.c`;
  the other `trace2.*` keys only shape an active target) declines when set to anything but empty,
  `0` or `false`
  (`trace2/tr2_dst.c`,
  `tr2_dst_get_trace_fd`).
  Control:
  `a_configured_trace2_target_declines_and_an_inactive_one_does_not`,
  with a user-level target of `2` against real Git's standard error.
