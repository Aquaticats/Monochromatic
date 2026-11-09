# Invocation and argument-route findings on `main`

Audit record:
2026-11-09T05:59:59-05:00.
Audited source:
`b38de34bb23ba5fd29fcbc70e8be218ff9746edd`,
the starting revision of this worktree.
`git diff main -- package/git-policy/cli/ doc/decision/` was empty when the audit started;
the observed `main` revision was `566be9a823d58f9532b14a48d54980a8e8b4fec4`.
No other agent's branch was inspected.
This is an audit,
not authorization to implement the suggestions.

Scope:
`package/git-policy/cli/src/native/invocation/`,
the configuration-loading classifier,
and the command readers that select policies and index-writer coordination.
Source paths in this page are repository-relative.
Git source references name Git 2.56.0's `git.c` and `builtin/reset.c`,
read from the existing source checkout at `${HOME}/temp/agent/git-v2.56.0-2026-11-03`.
Linux only.

## Requirements and existing coverage

The owner's requirements in the audit brief are:

- "Commits should never fail."
- "never discard user data"
- "nothing should ever wait forever or indefinitely"
- "We should never invoke ourselves recursively..."

The related accepted decision,
`doc/decision/cli-git-rust-rewrite.md:11`,
says "Only latest stable Git is supported."
Its preservation requirement at `:18` is
"Preserve the existing required policy,
transaction,
concurrency,
recovery,
worktree,
and push behavior."
The selected version here is Git 2.56.0.

`doc/decision/cli-git-concurrent-commits.md:470` requires:
"Forwarded index writers coordinate with landings through the cli-git landing lock".
The implementation now waits for the commit service's index hold rather than the historical landing lock;
`package/git-policy/cli/src/native/lock/index_writer_forward.rs:213` is that wait.
The obligation to route index writers through coordination remains relevant to the reset finding.

The owner's bounded-wait amendment is recorded in
`doc/handover/cli-git-rust-implementation.md:7140`:
"nothing should ever wait forever or indefinitely",
with "Return with an error" and "No,
keep it for a retry (Recommended)" when the bound is reached.
The no-self-start requirement is also in
`package/git-policy/cli/SPEC.md:4718`.

The handover's `## Open review findings` was read before probing.
This page does not re-report:

- Aliases missing the discarding guards:
  `cli-git-discarding-commands-route-findings-on-main.md`,
  finding "An alias reaches no guard and leaves no trace".
- Literal-pathspec mode emptying a commit's candidate set:
  `cli-git-policy-and-candidate-route-findings-on-main.md`.
- A launcher defeating the wrapper's recursion checks:
  `cli-git-entry-and-service-lifecycle-route-findings-on-main.md`.
- Hook configuration precedence:
  `cli-git-hook-route-findings-on-main.md`.

The alias finding here is different:
Git and the wrapper do not execute the same command.
It is not another demonstration that the same destructive alias misses a guard.

## Findings ranked by what they risk

### Expanded aliases execute destructive commands that Git refuses or does not select

Status:
**reproduced through the built wrapper and native Git**.
Risk:
uncommitted tracked bytes are overwritten by a command the original Git invocation does not execute.
This directly touches "never discard user data".

The shared cause is
`package/git-policy/cli/src/native/invocation/entry_forward_action_planning.rs:77`:
`Route::Alias => command.arguments.clone()`.
The wrapper does not merely use its alias interpretation to choose a route.
It replaces the caller's arguments with that interpretation before running real Git.

The expansion in
`package/git-policy/cli/src/native/invocation/forwarded_command.rs:369`
differs from Git in the cases recorded in the named subsections.
Each fixture begins with a commit containing `tracked.txt` with `BASE\n`,
then overwrites the worktree file with `USER DATA\n` without staging it.
The two arms use independent repositories.
No real user data was put at risk.

#### A directory-changing alias that Git rejects resets another repository

Sequence:

1.  Make source and victim repositories,
    each with the baseline commit.
2.  Leave `USER DATA\n` in the victim's tracked worktree file.
3.  From the source repository run
    `git -c 'alias.audit=-C <victim> reset --hard' audit`.
4.  Run the same argument vector through the built wrapper in the equivalent fixture.

Native Git 2.56.0 refuses the alias before executing `reset`:

```text
# Native Git, alias-environment-native
status=128
fatal: alias 'audit' changes environment variables.
You can use '!git' in the alias to do this
tracked.txt="USER DATA\n"
```

The wrapper executes the reset in the victim:

```text
# Built wrapper, alias-environment-wrapper
status=0
trace: built-in: git reset --hard
HEAD is now at b777e79 baseline
tracked.txt="BASE\n"
```

The wrapper accepts the alias's `-C` as another global prefix at
`package/git-policy/cli/src/native/invocation/forwarded_command.rs:375`,
then substitutes the expansion at
`package/git-policy/cli/src/native/invocation/entry_forward_action_planning.rs:77`.
It never applies Git's alias-specific environmental-change rejection.
Git 2.56.0 `git.c:316` sets `envchanged` for a nonempty `-C`,
and `git.c:418` to `:422` rejects that expansion.

This is not a claim that every global option is forbidden in an alias.
For example,
Git's `-c` branch does not set that flag.
The problem is accepting options without applying the separate alias rule.

#### An existing external command is replaced by a destructive alias of its name

Sequence:

1.  Provide an executable named `git-audit-external` that only prints
    `EXTERNAL: no mutation`.
2.  Set `GIT_EXEC_PATH` to the fixture directory containing that executable.
3.  With the tracked file dirty,
    run
    `git -c 'alias.audit-external=reset --hard' audit-external`.
4.  Run the same invocation through the built wrapper in the equivalent fixture.

Native Git selects the external command and leaves the file alone:

```text
# Native Git, external-precedence-native
status=0
trace: exec: git-audit-external
EXTERNAL: no mutation
tracked.txt="USER DATA\n"
```

The wrapper selects the alias instead:

```text
# Built wrapper, external-precedence-wrapper
status=0
trace: built-in: git reset --hard
HEAD is now at b777e79 baseline
tracked.txt="BASE\n"
```

`package/git-policy/cli/src/native/invocation/forwarded_command.rs:381`
checks built-in names,
then `:393` looks up the alias without checking for a dashed external command.
Git 2.56.0 `git.c:905` calls `execv_dashed_external` before
`git.c:912` calls `handle_alias`.
The wrong interpretation becomes an actual wrong command only when the forwarding code substitutes it.

#### Trimming a malformed alias turns Git's refusal into a destructive command

Sequence:

1.  Leave the tracked file dirty.
2.  Run `git -c 'alias.audit= reset --hard' audit`,
    preserving the space immediately after `=` inside that single argument.
3.  Run the same argument vector through the built wrapper in the equivalent fixture.

Native Git retains the empty first command word:

```text
# Native Git, alias-trim-native
status=1
trace: alias expansion: audit => '' reset --hard
trace: exec: git- reset --hard
expansion of alias 'audit' failed; '' is not a git command
tracked.txt="USER DATA\n"
```

The wrapper removes the space and runs the reset:

```text
# Built wrapper, alias-trim-wrapper
status=0
trace: built-in: git reset --hard
HEAD is now at b777e79 baseline
tracked.txt="BASE\n"
```

`package/git-policy/cli/src/native/invocation/forwarded_command.rs:221`
trims whitespace with JavaScript semantics,
and `:396` applies that transformation before splitting.
Git 2.56.0 `git.c:414` splits the alias value itself.
The result quoted here is for a command-line `-c` value;
it does not claim that an unquoted config-file value preserves the same leading space.

Suggestion,
not an accepted fix:
preserve Git's alias resolution semantics before substituting any expansion.
That includes external-command precedence,
Git's alias-only restrictions on globals,
and exact value bytes with only the lookup protocol's terminator removed.
Returning to Git for expansion must also preserve the reason expansion was substituted:
`entry_forward_action_planning.rs:74` explains that hooks-off options must follow the alias's own `-c` options.
Simply deleting substitution without preserving that invariant would be an incomplete fix.

### Reset can write the real index while its classifier skips writer coordination

Status:
**classification reproduced with failing Rust seams;
index mutation reproduced with native Git**.
No concurrent landing or resulting commit failure was reproduced.
Risk:
the route omits the coordination that protects a landing from another writer.
This touches "Commits should never fail."
and the accepted forwarded-writer requirement.

`package/git-policy/cli/src/native/lock/index_writer_commands.rs:194` to `:200`
returns `false` when any collected option matches `--soft`.
It does not model Git's final reset mode.
The token collector at `:98` to `:105` also does not distinguish an option from a separated option value.

The two independent sequences are:

- Stage a modification to `tracked.txt`,
  then run `git reset --soft --mixed`.
  Git's last mode is mixed,
  so the staged modification is removed from the index.
  The wrapper nevertheless returns `writes_index=false` upon seeing the first mode.
- Stage a modification to `tracked.txt`;
  create a file literally named `--soft` containing `tracked.txt\n`;
  run `git reset --pathspec-from-file --soft`.
  Git consumes `--soft` as the pathspec-file name and performs the default mixed reset.
  The wrapper instead reads that filename as the soft mode.

Native output and the positive control:

```text
# Native Git, reset-last-mode
$ git reset --soft --mixed
status=0
Unstaged changes after reset:
M       tracked.txt
cached paths before="tracked.txt\n"
cached paths after=""

# Native Git, reset-file-value
$ git reset --pathspec-from-file --soft
status=0
Unstaged changes after reset:
M       tracked.txt
cached paths before="tracked.txt\n"
cached paths after=""

# Native Git, reset-soft-control
$ git reset --soft
status=0
cached paths before="tracked.txt\n"
cached paths after="tracked.txt\n"
```

The uncommitted seams call production `resolve_forwarded_command`,
then production `is_index_writer`,
without starting any concurrent operation:

```text
# Filtered nextest run, audit_invocation_reset
AUDIT reset --soft --mixed: route=Direct, writes_index=false
Git's final --mixed writes the index but the wrapper skips coordination
FAIL invocation::forwarded_command::tests::audit_invocation_reset_last_mode_writes_index

AUDIT reset --pathspec-from-file --soft: route=Direct, writes_index=false
Git consumes --soft as a file name but the wrapper skips coordination
FAIL invocation::forwarded_command::tests::audit_invocation_reset_pathspec_file_is_not_a_mode

Summary: 2 tests run: 0 passed, 2 failed, 5011 skipped
```

Git 2.56.0 `builtin/reset.c:354` to `:368`
sets the selected reset mode as options are parsed;
`:489` takes the non-soft index-writing branch.
The wrapper consumes the wrong boolean at
`package/git-policy/cli/src/native/invocation/entry_forward_action_planning.rs:84`.
Its nonwriter branch at `:132` never constructs `Action::ForwardCoordinated`.
The coordinated route's service-hold wait and retry therefore cannot protect these commands.
This consequence is read from the code;
this audit does not claim to have demonstrated corruption or a lost commit.

Suggestion,
not an accepted fix:
use the existing `command_reset` option-table reader to obtain the final selected mode,
and preserve the distinction between options,
values,
and pathspecs before deciding whether the command writes the index.
Add controls for mode order and separated option-looking filenames.

## Reproduction environment and evidence

The wrapper was built from the audited source using
`mise run //package/git-policy/cli:native:build`.
It is the debug executable at
`package/git-policy/cli/target/debug/monochromatic-git-policy-cli`.
The only source change used for reproduction is a temporary test-only addition to
`invocation/forwarded_command_tests.rs`;
no production code was changed.
The temporary tests are not to be committed.

The local harness and its raw JSONL output are under
`package/git-policy/cli/target/invocation-audit/`:
`differential.ts` and `differential.jsonl`.
The directory is ignored by `.gitignore`'s `target/` rule.
All fixtures,
homes,
runtime directories,
and script writes stay in this worktree's build directory.
Native Git is invoked as plain `git` with a controlled `PATH=/usr/bin:/bin`;
the installed wrapper and running service are not test targets.
The harness prints no ambient environment or real Git configuration.

The native build and Rust seam run each have a cgroup bounded to
`CPUQuota=400%`,
`MemoryMax=6G`,
`TasksMax=256`,
and `IOWeight=10`,
with a `600s` deadline.
The differential harness has
`CPUQuota=200%`,
`MemoryMax=1G`,
`TasksMax=128`,
and `IOWeight=10`,
with a `180s` outer deadline and a `20s` deadline for each Git or wrapper command.
No synthetic load,
mutation campaign,
or full test suite was run.

## Verification still in progress

The remaining audit work is the global-option and environment compatibility controls,
restoring the temporary test sidecar,
rendering the document,
and running the required Markdown fixer and linter.
The ranked findings are already backed by the quoted results;
no result for the remaining checks is claimed yet.

## What I checked and found sound

- `global_arguments.rs` reads native argument bytes and does not convert them to UTF-8 while locating boundaries.
  Its `--` handling is not a generic stop rule:
  it classifies the global spelling as invalid and leaves Git to report it.
- `wrapper_invocation.rs` strips controls by parser-provided token position,
  not by matching every argument's text.
  Post-separator pathspecs and option values are not intended to become escape hatches.
- A genuine final soft reset does not change the index in the native positive control.
  The reported reset defect is not an assumption that every reset writes it.
- Startup resolution failures stop rather than explicitly falling back to unguarded native Git.
  `entry.rs:209` returns `failure` when `resolve_real_git` fails;
  `entry.rs:393` reports a refused process replacement as an engine failure.
  The separate launcher-recursion defect is already recorded in the entry-route audit.
