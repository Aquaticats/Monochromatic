# cli-git native policy engine

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

## Authority and scope

This records a scoped delegation of the Rust cli-git rewrite:
the native policy engine for every command that needs no commit transaction,
no worktree copy and no manual-push scanning.
The accepted approach is
[`cli-git-rust-implementation.md`](../planning/cli-git-rust-implementation.md);
the behavior inventory is
[`cli-git-rust-behavior-ledger.md`](../planning/cli-git-rust-behavior-ledger.md);
the layers this builds on are recorded in
[`cli-git-native-foundation.md`](cli-git-native-foundation.md)
and [`cli-git-native-command-parser.md`](cli-git-native-command-parser.md).

Settled before this delegation and not reopened:
Rust,
a repository-root `cli-git.config.jsonc`,
shipped policies only,
Git 2.56.0 only,
and the TypeScript executable staying the installed `git` until a later coordinated cutover.

The native executable is still a development artifact.
It is not installed and does not shadow `git`.
It now forwards guarded commands after running the ported policies,
and it refuses every command that needs work it does not do.
A commit,
a real push,
a `git add` with a content policy on,
and a worktree creation from a linked worktree are all refused.
Nothing here brings the wrapper close to cutover:
the commit path,
which is the reason for the rewrite,
is not started.

Candidate content and the forbidden-strings scanner belong to another delegation
(`candidate_*.rs` and `scanner_*.rs`)
and are not used by anything recorded here.

## Decisions by the human on 2026-10-05

The coordinating agent relayed five decisions as made by the human on 2026-10-05.
They are recorded as decided,
not as open to veto.

### Optional policies run only when listed

The five built-in policies run everywhere at their default severity.
Each of the four policies that used to come from plugins
(`markdown/autofix`,
`mono/forbidden-root-context`,
`mono/dependent-version-bump`,
`security/forbidden-strings`)
is off unless `cli-git.config.jsonc` names it,
whether or not a file exists.
An empty object and no file behave identically.
A policy listed with options but no severity gets its incumbent default severity.
A repository's translated configuration must therefore name every optional policy it relies on;
an unlisted one silently stops.
Commit `329de1b97` implements this and corrects the corresponding passages of
`cli-git-native-foundation.md`.

### The repository root is what Git reports

The root is the top level Git prints,
not the nearest directory holding a marker file.
The incumbent's nearest-marker walk is an intentional difference.
The consequence for the read-only fast path is recorded under "Hazard 5, the read-only fast path".

### A legacy configuration file is reported by check only

A legacy file left beside the JSONC file is reported by `git cli-git check`,
as one `configuration-warning` event per file on standard output,
and by no other command.
A legacy file without a JSONC file stays a migration error on every configuration-loading command.

### Two mutant kinds are never tried

`bin/mutate-native-container.mjs` passes `--exclude-re 'replace \+= with \*='`
and `--exclude-re 'replace -= with /='` to cargo-mutants on every campaign.
The human accepted that these two kinds are never tried.
The cause,
the cargo-mutants source that decides it,
the measured cost in this package
(34 of 1,230 mutants at commit `d1c02273a`)
and the tradeoffs are in
[`cargo-mutants-timeout-exit-status.md`](../troubleshooting/cargo-mutants-timeout-exit-status.md).
`replace += with -=` stays active as the check on every counter.

The three loops whose step mutant `replace += with -=` had timed out were rewritten to visit the argument slice
instead of stepping an index:
`mixed_command` in `config_loading.rs`,
`global_layout` in `global_arguments.rs`,
and `parse_direct` in `management_arguments.rs`.
None of the modules added by this delegation steps a loop index by hand.

### The failure code of a policy that could not finish follows its cause

A shipped policy that could not finish reports `content-unavailable`
when a candidate's bytes or a repository fact could not be read,
and `policy-incomplete` when the policy's own machinery failed:
its rules file could not be loaded,
its linter could not start,
or it hit an internal error.
`plugin-threw` is not carried over,
and no other code is added.

Commit `090c147e7` adds `PolicyIncomplete` to `diagnostics::EngineFailureCode`
with the wire spelling `policy-incomplete`,
a doc comment stating the rule,
and the spelling test.
No path in the engine reports it yet:
the only failure of a ported policy is a repository fact that could not be read,
which is `content-unavailable`.
The optional-policy phase is its first user.
The engine seam for it is `policy_engine::PolicyOutcome`,
whose `Failed` outcome the stage reports as `content-unavailable`;
a policy whose machinery can fail needs a second failing outcome there
and one more arm in `run_policy_stage`.

## Gate results

The gate is `mise run //package/git-policy/cli:native:test:container`:
a mount-free,
network-disabled container bounded to 2 GiB,
2 CPUs
and 128 PIDs,
on the audited Git 2.56.0 image.
It runs the unit target,
the binary-level target,
and Clippy with warnings denied.
Evidence directories are under `package/git-policy/cli/target/verification/`.
The baseline before this delegation was 322 unit and 21 binary-level tests.

### Slice A, wrapper controls

Commit `6b14f1249`:
333 unit and 21 binary-level tests,
evidence `native-lMmgW3`.

### Slice B, engine core

Commit `99646d0c6`:
363 unit and 21 binary-level tests,
evidence `native-gUdrZ4`.

### Human decisions on configuration

Commit `329de1b97`:
366 unit and 21 binary-level tests,
evidence `native-AXWZNN`.

### Slice C, facts, rule cores and transforms

Commit `a9010f435`:
407 unit and 21 binary-level tests,
evidence `native-GiOP83`.

### Loop rewrites and sequencer follow-up

Commits `9cc45dc2c`,
`d86b7463c`
and `d1c02273a`:
408 unit and 21 binary-level tests,
evidence `native-3hLjlH`.

### Slice D, lifecycle, refusal frontier and direct commands

Commit `3836299a7`:
440 unit and 36 binary-level tests,
evidence `native-FbeoYh`.
This was the first gate run of the slice;
no test needed a correction after it.

### Fuzz target and the status-format commit control

Commit `d8adb6015`:
440 unit and 36 binary-level tests,
run twice under separate image tags,
evidence `native-q6HftZ` and `native-LETrzq`.
The count is unchanged because the commit extends an existing binary-level test.

### Helper removal

Commit `4bef275e6` removes a function only tests called.
It was first checked on the host only;
the gate of commit `24cfeaaee` includes it.

### Classifier test

Commit `24cfeaaee`:
441 unit and 36 binary-level tests,
evidence `native-sgFHN6`.

### Shared command reading and single-body platform functions

Commits `e0042aeda` and `37d362484`:
443 unit and 37 binary-level tests.
The tree was gated four times under separate image tags,
one per part of the first mutation campaign over every file,
evidence `native-xi1ELC`,
`native-uKOShH`,
`native-aa2VKI`
and `native-nKKJSe`.
A fifth gate build ended in a Podman error and was started again;
see "Podman under concurrent use".

### Option scanners that say whether they took the next token

Commit `08847be01`:
443 unit and 37 binary-level tests,
evidence `native-TEGRqD`.

### Final tree

Commit `348d94cbe` is the last one that changes `package/git-policy/cli/src/native/`:
442 unit and 37 binary-level tests,
evidence `native-eqKHwb`.
The unit count is one lower because `detects_the_escape_hatch` left with the function it tested;
the other follow-up commits extend tests that already existed.
This gate also covers commits `66a3e9b9c`,
`52cc1494b`
and `5e5972f64`,
which had been checked on the host only.
Its image is the one the second and third mutation campaigns and the rerun ran against.
`mise run //package/git-policy/cli:native:clippy:windows` also passes on this tree.

### Tests run on the host

The delegation asked that repository-changing checks run only inside the container.
While iterating,
the unit target was also run on the host through `mise run //package/git-policy/cli:native:test:host`.
Those tests create repositories only in fresh directories under the system temporary directory,
with a cleared environment and no global or system Git configuration,
and never touch this repository or the real home directory.
The host has Git 2.55.0,
so 10 tests that compare against Git 2.56.0 fail there;
only the container gate counts as evidence.
The binary-level target,
which starts the wrapper itself,
was never run on the host.

## What the executable does today

### Lifecycle of a wrapped command

`entry::plan_invocation` decides one invocation in this order:

1.  A forward-target marker naming this executable stops the invocation,
    as in the foundation.
2.  Wrapper controls written before the subcommand are removed.
3.  `git cli-git ...` goes to the management commands.
4.  Real Git is resolved.
5.  `wrapped_command::run_wrapped_command` runs the lifecycle.

The lifecycle,
in `wrapped_command.rs`:

1.  An invocation that names no subcommand
    (a query,
    a global option Git refuses,
    a missing value,
    a bare `git`)
    is forwarded with its controls removed.
2.  A command that `classify_config_loading` skips uses the default settings.
    It reads no configuration,
    checks no lease and no leftover state,
    and asks Git only what a policy needs.
3.  Every other command is prepared:
    an inherited lease is refused;
    Git is asked once where the command runs;
    leftover transaction or worktree-copy state is refused;
    the worktree's configuration is loaded;
    a commit,
    a worktree creation or a possible alias that needs unported work is refused;
    and `git add` inside a worktree is marked as having content no policy can read yet.
4.  One pre-forward pass runs:
    built-in policies,
    then the fixed transforms,
    then optional policies.
5.  A `git push` that is not provably a dry run must clear the manual-push gate.
6.  The command is forwarded with the transformed arguments,
    after any warning events are written to standard error.

A stop writes its events to standard error and exits 1 for an error finding or a transform rejection,
and 2 for a failure or a refusal.

### Direct check and fix

`git cli-git check` and `git cli-git fix` validate the selected policy names,
resolve Git,
ask where they run,
refuse beside registered commit transactions,
load the configuration,
and run one pass with the `direct-check` or `direct-fix` trigger.
Events go to standard output.
`check` first reports legacy files,
and policy events continue that numbering.

Only `require-root` is both ported and declared for `direct-check`.
No ported policy declares `direct-fix`.
Every other policy that runs for a direct command reads the selected worktree files,
which is not ported,
so:

- `git cli-git check --all --policy require-root` gives a real answer:
  exit 0 at the top level,
  one finding and exit 1 below it.
- A direct command whose selection includes an enabled content policy stops with exit 2
  and a notice naming the policy,
  after the events gathered so far.
  With default settings that is every unfiltered `check` and `fix`,
  because `final-newline` is on.
- A direct command whose selection holds only policies that do not run for it exits 0 with no output.

The scope (`--all` or pathspecs) is parsed and validated by the management grammar but not read.
The incumbent projects the selected files before any policy runs,
so a pathspec error it would report is not reported here.

## Refusal frontier

Every refusal exits 2,
prints one line on standard error,
and leaves the repository untouched.
The line names what is missing and the command that did not run:

```text
cli-git: <what> is not implemented in this native development executable,
so git <command> was not run. Run it with the installed cli-git.
```

The two lines shown are one line of output,
joined by a space.
The reasons are the variants of `unported::Unported`.
Each has a binary-level test in `binary_frontier_tests.rs`,
`binary_controls_tests.rs`
or `binary_management_tests.rs`,
with a positive control showing that the neighbouring forwarded command really changes the repository.

### Refused commands

#### Every commit except a dry run

Ledger sources:
"Scope and applicability",
"Post-commit lifecycle",
"Trigger, skips, and push arguments",
"Candidate facts".
The decision is made on the command word `commit` alone,
then narrowed by Git's own option table:
only a region the table reads as a dry run is forwarded.
A region the table refuses is not known to be a dry run and is refused.
Git makes `--short`,
`--porcelain`,
`--long`
and `-z` dry runs by themselves,
and the table follows it;
those forms are forwarded.
Test: `frontier::a_real_commit_is_refused`,
ten refused forms from the top level and one from a subdirectory,
`HEAD` and the index unchanged,
and five forwarded dry-run forms compared with real Git.

#### `git add` while a content policy is on

Ledger sources:
"Candidate facts",
"Pre-forward lifecycle",
"final-newline".
The engine returns a typed "unavailable" for the first enabled content policy,
and the notice names that policy.
With default settings that policy is `final-newline`,
so a plain `git add` is refused.
`add-explicit` runs first,
so a bulk add is rejected with exit 1 before any content policy is asked.
Test: `frontier::add_is_refused_while_a_content_policy_cannot_read_what_it_would_stage`.
Positive controls:
`--no-enforce-final-newline`,
and a configuration that turns the policy off,
stage the file.

#### A push that may publish

Ledger source: "Manual-push lifecycle".
After the pre-forward pass,
a push that Git's table does not read as a dry run asks whether any policy declares the `manual-push` trigger
and is enabled.
If one is,
the engine's typed "unavailable" for that lifecycle becomes the refusal.
Test: `frontier::a_real_push_is_refused_at_the_manual_push_gate`:
three refused forms against a bare remote that stays empty,
a forwarded dry run compared with real Git,
and a positive control that publishes once the only such policy is escaped.

#### Worktree creation from a linked worktree or a bare repository

Ledger source: "Applicability and main-worktree bypass".
`git worktree add` and `git worktree move` are refused there unless `--no-worktree-copy` is given.
From the main worktree,
and outside a repository,
they are forwarded,
as the incumbent does.

#### A possible alias from a linked worktree or a bare repository

Ledger source: "Alias resolution and built-in command table".
A command word that Git 2.56.0 does not build in may be an alias for a worktree creation.
`git_builtins.rs` holds the 149 names of `git --list-cmds=builtins`,
compared with real Git in the gate.
`--no-worktree-copy` forwards such a command,
because the incumbent skips the copy before it resolves any alias.
Test for both: `frontier::worktree_creation_and_aliases_are_refused_from_a_linked_worktree`.

#### Leftover durable state

Ledger sources:
"Startup order",
"Recovery",
"Crash recovery",
"Index-writer coordination and landing lease".
A guarded command is refused when the invocation's Git directory holds any entry in `cli-git-transactions`,
or the legacy `cli-git-transaction`,
and,
from a linked worktree or a bare repository without `--no-worktree-copy`,
when the common directory holds any entry in `cli-git-worktree-copy/v1` other than `settlement.lock`.
A directory that cannot be listed counts as holding state.
This executable cannot tell a dead owner from a live one,
so it does not classify entries.
Direct `check` and `fix` are refused beside transaction state only.
Test: `frontier::leftover_state_and_leases_refuse_guarded_commands`
and `management::direct_commands_validate_run_ported_policies_and_refuse_the_rest`.

In a repository where the installed wrapper is in use,
the transaction registry is rarely empty:
this repository's own registry held two live transactions,
two staging directories several days old,
and a reservation lock when it was inspected.
In such a repository the native executable refuses every guarded command.
That is the intended state until recovery and the landing lock are ported.

#### An inherited lease

Ledger sources:
"Recovery",
"Index-writer coordination and landing lease",
"Applicability and main-worktree bypass".
A guarded command is refused when `CLI_GIT_PREPARATION_LEASE`,
`CLI_GIT_LANDING_LEASE`
or `CLI_GIT_WORKTREE_COPY_LEASE` is present in the environment,
whatever its value,
because checking a lease against the lock it names is not ported.
Same test as leftover state.

#### Direct check and fix with a content policy selected

Ledger sources:
"Direct check",
"Direct fix",
"Candidate facts".
Recorded under "Direct check and fix".

### Forwarded with a known omission

These parts of the incumbent's behavior are not ported and do not stop a command.
Each is a deliberate choice,
open to veto.

#### Index-writer coordination

Ledger source: "Index-writer coordination and landing lease".
The incumbent makes a command that writes the index wait for the landing lock.
The native executable does not take that lock.
The leftover-state refusal stands in for it:
a landing transaction has a registry entry,
and any entry refuses the command.
The check and the forward are not atomic,
so a transaction that registers in between is not seen.

#### Recovery on the read-only path

Ledger sources:
"Startup order",
"Crash recovery".
The incumbent recovers dead transactions and interrupted worktree copies before read-only commands too.
The native executable forwards a read-only command without looking,
because the fast path reads neither configuration nor leftover state.
A read-only command changes nothing a recovery would protect.
The research brief `cli-git-rust-open-decisions.md`,
section "Recovery before read-only commands",
reads the spec as requiring recovery there too once recovery is ported.
That conflicts with this fast path.
The coordinating agent will put it to the human with the questions of the transactions phase;
nothing here was changed for it.

#### Post-command output

Ledger source: "Post-command output".
The line after `git --version` and the note after `git status` are not printed:
forwarding replaces the process,
so nothing runs after Git.
The status-hints transform is applied,
so plain `git status` shows neither Git's hints nor the incumbent's note.

#### An aliased commit

`src/policy-engine/commit-transaction.ts:109-110` applies the commit transaction
only when the command word is literally `commit`.
An alias that expands to `commit` therefore runs as a plain Git commit under the incumbent,
without a transaction and without commit policies.
The native executable reproduces this:
from the main worktree a command word Git does not build in is forwarded.
The ledger does not list it as a defect.
Refusing every such word everywhere is a one-condition change in `refusal_frontier::command_frontier`
and would close the path at the cost of refusing every alias and external command.

## Hazards

### Hazard 1, controls before the subcommand

`wrapper_controls::strip_global_controls` removes every control from the global position
before any rule reads the command:
it removes the token at which Git's own global-option reader reports an invalid option,
if that token spells a control,
and repeats.
A control that is the value of `-C` or `-c` is left alone.
`entry::plan_invocation` runs it before the management namespace is detected,
so a control before `cli-git` does not hide the namespace;
the controls reach the direct command,
as they do under the incumbent.

Tests:
`controls::keep_going_before_the_subcommand_is_removed_before_any_rule_runs` runs exactly
`git --cli-git-keep-going status`.
From a subdirectory it exits 1 with one `require-root` finding and Git does not run;
at the top level its output equals `git -c advice.statusHints=false status`,
with a positive control showing that comparison can fail.
`controls::every_control_before_the_subcommand_is_removed` repeats this for all 14 general controls,
alone and between two of Git's global options,
and shows that three tokens that are not controls reach Git and are refused by it.
Both pass in the gate.

### Hazard 2, one escape-hatch mechanism

`wrapper_invocation::strip_wrapper_controls` is the only place that removes a wrapper token.
For a command with a ported option table the positions come from Git's own grammar:
`parse_options` reports each wrapper token in option position,
and the tokens are deleted by position.
The former `strip_escape_hatch` and its tests are deleted;
`escape_hatch.rs` now holds only the three spellings.

Test: `controls::a_path_named_like_a_hatch_is_forwarded_and_a_hatch_is_removed`.
With a file named `--no-enforce-worktree` and a second file staged,
`git reset --quiet -- --no-enforce-worktree` unstages only that file,
so Git received the token.
`git reset --hard` is rejected in the main worktree;
`git reset --hard -- --no-enforce-worktree` is rejected too;
`git reset --no-enforce-worktree --hard --quiet` runs and Git never sees the token.
Passes in the gate.

### Hazard 3, the refusal frontier

Recorded under "Refusal frontier".
The mutation runner's planted control "commit refusal frontier" replaces the commit refusal with a pass
and requires `frontier::a_real_commit_is_refused` to fail.

### Hazard 4, triggers that are not ported

`policy_engine::run_policy_stage` returns `StageEnd::Unavailable(Unavailable::Lifecycle(trigger))`
for the `post-commit` and `manual-push` triggers before any policy runs.
The exit code of that ending is 2.
Unit tests:
`policy_engine::tests::unported_triggers_are_unavailable_not_clean`
and `policy_pass::tests::an_unported_lifecycle_is_unavailable`.
At the binary level the `manual-push` trigger is reached by a real push
(`frontier::a_real_push_is_refused_at_the_manual_push_gate`).
The `post-commit` trigger cannot be reached from the executable,
because every commit that would land is refused first.

### Hazard 5, the read-only fast path

The requirement as written is not met,
and cannot be met together with the decision that the root is what Git reports.
`require-root` must reject `git status` from a subdirectory,
and the root it compares against comes from Git.
What holds instead:

- A command on the skip branch never reads configuration,
  leftover state or leases.
- It starts no Git process when no policy needs a repository fact:
  native queries,
  `version`
  and `help`.
- Otherwise it starts exactly one,
  the location query,
  before the forwarded command.
  The incumbent also starts one Git process on this path,
  to resolve the worktree identity.

Test: `policy::read_only_commands_start_at_most_the_location_query`.
Real Git is a shim that appends its arguments to a log and then runs Git.
Beside an invalid configuration file and a legacy file,
four exempt commands log exactly the forwarded command,
and seven other read-only commands log exactly the location query and then the forwarded command.
Positive controls in the same fixture:
a guarded command stops on the invalid configuration after one logged query,
and with a valid file logs the query and the command.
Passes in the gate.

On the skip branch the settings are the defaults,
as under the incumbent:
a repository whose file turns `require-root` off still gets it for `git status`.

### Hazard 6, a region Git refuses

The working answer was to forward unchanged.
What is implemented:
a region Git's table refuses does not short-circuit anything.
Each rule decides what it can without the region,
the transforms leave the region as written,
and the command is forwarded for Git to refuse.
Three exceptions follow from hazard 3,
which has no latitude:
a `commit` whose region is refused is refused by the wrapper,
a `push` whose region is refused counts as publishing,
and `git stash` is guarded in every form.
A mistaken refusal by an option table therefore cannot forward a commit or a publishing push;
it can still forward a command of another kind unguarded.

Test: `controls::a_region_git_refuses_is_forwarded_unless_the_command_is_never_forwarded`:
six commands whose output and exit code 129 equal real Git's,
then the three exceptions.
Passes in the gate.

## Engine interface for the optional-policy phase

### Stage and pass

- `policy_engine::PolicyChecks` is the seam:
  `check(policy, trigger) -> PolicyOutcome`.
  `PolicyOutcome` is `Findings(Vec<PolicyFinding>)`,
  `Unavailable(&'static str)`
  or `Failed(String)`.
- `policy_engine::run_policy_stage(request, policies, checks)` runs the given policies in order
  and returns `StageResult { events, end }`.
  `StageEnd` is `Completed`,
  `Stopped`,
  `Failed`
  or `Unavailable(..)`.
- `policy_engine::StageRequest` owns `trigger`,
  `config`,
  `controls`
  and `selected`.
- `policy_pass::run_policy_pass(request, checks)` runs the three stages
  and returns `PassResult { arguments, events, end }`.
- `policy_engine::pass_exit_code(events, end)` gives 0,
  1
  or 2.
- `policy_events::render_policy_events(first_sequence, events)` renders JSON Lines.
  Events are numbered when rendered,
  once per invocation.

### Where an optional policy plugs in

`policy_checks::ShippedChecks<F>` implements `PolicyChecks` with one `match` over `PolicyId`.
The five content policies share one arm,
`check_content`,
which answers from `CandidateSource`:
`None` means the lifecycle has no candidates and the policy is clean;
`NotPorted(needs)` means there are candidates nobody can read,
and the policy is unavailable.
Porting a content policy means giving `CandidateSource` a variant that carries readable candidates
and giving that policy its own arm.
The lifecycle sets the source in two places:
`wrapped_command::prepare_guarded_command` for `git add`,
and `management::plan_management` for direct commands.

### Fix passes

`policy_convergence::converge` is the bounded loop:
at most 8 changed passes,
a repeated state ends it as a cycle,
and an adjacent equal state is stable.
It is driven through the `FixPasses` trait and tested with scripted passes.
It has no production caller,
because no ported policy proposes a correction.
`PolicyFinding::fix_available` and `PolicyEvent::FixSummary` are carried and rendered for the same reason.

### Repository facts

`repository_facts::RepositoryFacts` is the only way a policy or transform asks Git:
`location`,
`index_vs_head`,
`sequencer_state`
and `remote_guess_creates_branch`.
`GitFacts` starts one process per question,
remembers the location,
and counts its processes in `queries`.
The location is one query,
shown here on two lines:

```sh
git <global options> rev-parse --path-format=absolute --is-bare-repository \
  --git-dir --git-common-dir --show-toplevel --show-prefix
```

It gives the worktree identity,
the top level,
and the directory below it after every `-C`.
`invocation_config::load_identity_config` loads configuration from that answer and starts no process.

## Choices open to veto

### The fast path asks Git one question

Recorded under "Hazard 5, the read-only fast path".

### A refused region forwards except for commit, push and stash

Recorded under "Hazard 6, a region Git refuses".

### The manual-push gate applies without a configuration file

The incumbent runs the manual-push gate only when a configuration was loaded
(`src/bin.ts:314`).
With an empty object and no file behaving identically,
the native gate applies whenever a policy with the `manual-push` trigger is enabled.
`final-newline` is such a policy and is on by default,
so in this phase a real push is refused in every repository unless that policy is off or escaped.
The research brief [`cli-git-rust-open-decisions.md`](../planning/cli-git-rust-open-decisions.md),
section "The manual-push gate without a configuration file",
reads the evidence the same way and lists the choice as settled by it.

### Controls on commands without a ported option table

For a command with no table,
and for a region Git refuses,
controls are recognized only directly after the command word,
where no option can claim them as a value.
For `git worktree add` and `git worktree move` they are also recognized directly after that second word.
The incumbent scans every token before `--`.

### `--no-worktree-copy` is a general control

It is recognized wherever the other general controls are,
including before the subcommand.

### Hatches of optional policies are always recognized

`--no-enforce-<name>` is removed for every shipped policy,
whether or not the configuration lists it.

### Leftover state refuses on any entry

Recorded under "Leftover durable state".

### A lease refuses on presence

Recorded under "An inherited lease".

### Engine failure of a fixed transform

A fixed transform whose repository fact could not be read ends the pass with `core-incomplete`.
The code of a policy that could not finish is decided;
see "The failure code of a policy that could not finish follows its cause".
Finding validation is by construction:
a `PolicyFinding` cannot hold an unknown code or a path outside its type,
so no code is needed for an invalid finding.

### require-root inside the Git directory

Inside `.git` Git reports no top level,
so `require-root` passes.

### The remote guess is one query

`git for-each-ref --format=%(refname) refs/heads/<name> refs/remotes/*/<name>`,
compared line by line,
decides whether `git switch <name>` or `git checkout <name>` would create a branch.

### Controls before the management namespace

`git --cli-git-keep-going cli-git check --all` applies keep-going to the direct command,
and `--no-enforce-<name>` there skips that policy,
as under the incumbent,
whose direct commands read controls from the global options
(`src/management.ts:291-307`).
A control after `cli-git` is a usage error.

### An options object alone lists a policy

`{ "policies": { "security/forbidden-strings": { ... } } }` names the policy at its default severity.
This is how "a policy listed with options but no severity" is written.

## Observations

### A linked sequencer head file

`git rev-parse --path-format=absolute --git-path CHERRY_PICK_HEAD` prints the target of a symbolic link,
not the name asked for
(measured on Git 2.55.0 on the host and pinned by a unit test that passes on Git 2.56.0 in the gate).
The byte-exact reader of that answer then refuses it,
and a pathless commit ends with `core-incomplete` and exit 2.
That is fail-closed.
Git itself would call the state a cherry-pick in progress.
The incumbent tests whatever path Git printed,
so it reads the link's target.
In this phase only a dry-run commit reaches the question.

### Reftable repositories

In a repository created with `--ref-format=reftable`,
a stopped cherry-pick has no `CHERRY_PICK_HEAD` file;
`git rev-parse --verify CHERRY_PICK_HEAD` still resolves
(measured on Git 2.55.0 on the host in a scratch repository).
The sequencer fact tests files,
as the incumbent's commit-only check does (`src/rule/commit-sequencer-check.ts:108-176`),
and reads "nothing in progress" there,
so a pathless concluding commit would be rejected by commit-only.
That blocks,
it does not let anything through.
It matters when the commit path is ported.

### Status-format options make a commit a dry run

The fuzz controls found that `git commit --short` is forwarded.
It is correct:
Git turns `--dry-run` on for `--short`,
`--porcelain`,
`--long`
and `-z`.
The binary-level commit test now pins those four forms against real Git.

### Superseded query

`worktree_identity::resolve_worktree_identity` has no production caller any more;
`GitFacts::location` asks the same question with one more output line.
It is kept for its real-Git tests.

### The repository's Rust linter is not part of the gate

The gate lints with Clippy only.
`mise run //package/git-policy/cli:native:lint:rust`,
added in commit `4133d2900`,
runs `package/linter/rust` over `src/native/`,
which checks the code-line budget and required rustdoc.
On the tree of commit `348d94cbe` it exits nonzero with 263 findings,
all of the rule `require-rustdoc` and none about the line budget:
212 on `use` statements,
33 on fields,
9 on functions,
8 on constants
and 1 on an impl block.
The 9 functions are the trait methods of `RepositoryFacts` implementations and one `check` method;
the 8 constants and the impl block are in test support.
Nothing was changed in response.
The delegation's gate does not include this linter
and its brief does not ask for it;
the findings are listed under "What remains".

## Mutation testing

### Runner changes

`bin/mutate-native-container.mjs` passes the two exclusions recorded under "Two mutant kinds are never tried".
`bin/native-planted-controls.mjs` had a planted control named "fail-closed policy stage",
which removed the stop of every repository-changing command in `entry.rs`.
That stop no longer exists.
The control is now "commit refusal frontier":
it replaces `return Some(Unported::CommitTransaction);` in `refusal_frontier.rs` with `return None;`
and requires `frontier::a_real_commit_is_refused` to fail.
Both scripts pass Oxlint with no finding.
`mise run //package/git-policy/cli:native:mutation:list` lists mutants without building anything.

### Campaigns

The tree of commit `d8adb6015` has 357 mutants in the modules this delegation added or changed,
after the two exclusions.
They are split over scoped campaigns,
each against a gate image of its own tag so that campaigns can run side by side.

The first attempt at the campaign over the three rewritten loops
(evidence `native-mutation-u9lW01`)
ended during the planted controls with
`podman rm failed ... database is locked`.
Four planted controls had been noticed by then.
That is a Podman failure under concurrent use,
not a test result,
and the campaign was started again.

Two campaigns started side by side at that load ended at the unmutated baseline,
one with a baseline test time of 91 seconds against the limit of 90
(evidence `native-mutation-Pnugm6`),
the other with four binary-level tests failing on
"the command hit the 5-second bound"
(evidence `native-mutation-xzWpxt`).
Neither is a result.
Both were started again when the load had fallen.

### Reading a result under load

The binary-level tests bound every wrapped command to 5 seconds,
and a loaded host trips that bound without any defect.
A mutant whose only failing test failed on that bound was not caught by a test:
it was caught by the load.
Each campaign's report is therefore read twice:
cargo-mutants' own summary,
and a pass over every caught mutant's log that sets aside the mutants whose failing tests all failed on the bound.
The pass was checked against the full campaign recorded in `cli-git-native-foundation.md`,
whose counts it reproduces.

### The three rewritten loops

Evidence `native-mutation-Gqhv1V`,
against the gate image of commit `3836299a7`:
94 mutants,
88 reported caught,
6 unviable,
0 missed
and 0 timeouts.
The earlier full campaign had 9 timeouts in these three files.
None remains:
the excluded kinds are not tried,
and the three `replace += with -=` mutants no longer exist,
because the loops no longer step an index.

The second reading found one mutant caught only by the bound:
`replace || with &&` on the `MissingValue` test in `classify_config_loading`.
It was a real survivor.
Before this delegation a binary-level test killed it,
because a global option error reached the classifier.
Now the lifecycle forwards every invocation without a subcommand before it classifies,
so only the classifier's own unit tests could kill it,
and none of them covered an option error.
Commit `24cfeaaee` adds `config_loading::tests::global_option_errors_keep_the_fast_path`.
Planting the mutant fails that test and no other unit test;
with the source restored it passes.

### Scoped campaigns before the final tree

- `wrapper_controls.rs`,
  `wrapper_invocation.rs`,
  `command_worktree.rs`,
  `policy_trigger.rs`,
  `policy_events.rs`,
  `git_builtins.rs`,
  `unported.rs`,
  `action.rs`,
  against the gate image of commit `d8adb6015`
  (evidence `native-mutation-nMRlpS`):
  84 mutants,
  76 caught,
  8 unviable,
  0 missed,
  0 timeouts.
  The second reading sets nothing aside.
- `repository_location.rs`,
  `repository_facts.rs`,
  `rule_branch_worktree.rs`,
  `rule_linked_worktree.rs`,
  `rule_add_explicit.rs`,
  `git_metadata.rs`
  (evidence `native-mutation-YIs2yn`):
  85 mutants,
  3 missed,
  and 1 more caught only by the 5-second bound.
  They are dispositioned under "Survivors removed with their code".
- `policy_engine.rs`,
  `policy_convergence.rs`,
  `policy_checks.rs`,
  `policy_transforms.rs`,
  `policy_pass.rs`,
  `wrapped_command.rs`
  (evidence `native-mutation-YCTtwI`):
  stopped before any mutant,
  because the planted control "commit refusal frontier" was not noticed.
  The cause is recorded under "Planted controls read the live tree".

### Planted controls read the live tree

`bin/native-planted-controls.mjs` plants each guard removal in the source file as it is in the working tree,
then builds it inside the gate image.
The gate image is a snapshot taken earlier.
When a file a control names is edited between the gate and the campaign,
the planted file no longer fits the snapshot.
In evidence `native-mutation-YCTtwI` the planted `refusal_frontier.rs` imported two functions
that the snapshot's `wrapper_invocation.rs` did not have yet,
the build failed,
the named test never ran,
and the runner refused to trust the campaign.
That refusal is the runner working as intended.
The files the controls name must not be edited between a gate and the campaign that uses its image.

### Survivors removed with their code

Each of these could not be killed by any test on Linux.
None is excluded;
the code that produced the mutant no longer exists.
Commit `37d362484`.

#### Function variants not compiled on Linux

`cli-git-native-foundation.md` records 8 missed mutants in `#[cfg(not(unix))]` variants of
`forwarding::outcome_of`,
`forwarding::replace_process_with_real_git`,
`git_metadata::path_from_git_bytes`,
`real_git_candidate::is_executable`
and `real_git_candidate::same_inode`.
cargo-mutants 27.1.0 skips an item only for `#[cfg(test)]`,
a test attribute
or `mutants::skip`
(`src/visit.rs:861-865`, read in the v27.1.0 sources),
so it mutates a function the Linux build never compiles,
and such a mutant always passes.
Two of them appeared again in evidence `native-mutation-YIs2yn`.

Each pair of variants is now one function.
The part every system shares is ordinary code,
and only the statement that differs sits in a `#[cfg(unix)]` or `#[cfg(not(unix))]` block.
The function-level mutants are therefore exercised by the Linux tests,
and the non-Unix blocks hold no operator a mutant could change.
`outcome_of` needs no non-Unix block at all:
an exit code,
else on Unix a signal,
else a general failure,
is on other systems exactly what its former variant computed.

Nothing compiled the non-Unix code before.
`mise run //package/git-policy/cli:native:clippy:windows` now type-checks and lints the library
for `x86_64-pc-windows-gnu` with warnings denied.
It passes on commit `37d362484`.
Its positive control:
a type error planted in the non-Unix block of `is_executable` fails that task
while the Linux Clippy task still passes,
and the task passes again with the source restored.
The task checks types only;
no Windows behavior is run.

#### Two equivalent mutants in the script read

`cli-git-native-foundation.md` records two equivalent mutants in `real_git_candidate.rs`:
`filled < limit` to `filled <= limit` in the loop of `read_up_to`,
and the read limit `MAX_SCRIPT_INSPECTION_BYTES + 1 - header.len()` in `classify_candidate`.
`read_up_to` now reads through the standard library's bounded reader and has no loop.
`classify_candidate` reads the whole bound after the 4-byte header and compares the joined length with the bound:
the header is at least one byte of any file that has one,
so the read reaches past the bound exactly when the script is too large,
and no arithmetic on the limit is left.
`script_inspection_bound_is_exact` still pins the bound from both sides.

#### An offset each rule core computed for itself

`decide_linked_worktree` and `decide_add_explicit` sliced the tokens after the command word
with `prefix_len + 1`.
The mutant `prefix_len * 1` hands the option table the command word as a leading positional token,
which changes no decision either rule makes,
so no test can tell the two apart.
`global_arguments::command_tokens` now returns the command word and the tokens after it,
split by the standard library,
and the three rule cores that read a command's own options take both from it.
`command_tokens_split_after_the_global_options` tests the split.

### Podman under concurrent use

Several sessions used Podman on this host at the same time as these runs.
Two failures came from that,
and neither is a test result:

- `podman rm failed ... database is locked`,
  during the planted controls of evidence `native-mutation-u9lW01`.
- A gate build that ended with
  `Error: committing container for step ... Args:[mkdir --parents /home/tester/.cargo] ...`
  `unpacking failed (error: exit status 1; output: mkdir /home: no such file or directory)`,
  when four gate builds committed their layers at once.

Each run was started again once and then succeeded.
The second campaign over every file gates once
and gives the one image a tag per part with `podman tag`,
so no two builds of the same layers run side by side.

### First campaign over every file

Tree of commit `37d362484`,
after the two exclusions:
1244 mutants in 67 files,
split by file into four parts of 311 that ran side by side,
each against a gate image of its own tag.
The 12 other files of `src/native/` hold option tables,
module declarations
and test support,
and cargo-mutants finds nothing to change in them.

The foundation's full campaign had 513 mutants,
because it ran before the command parser and the rule cores were merged.
[`cli-git-native-command-parser.md`](cli-git-native-command-parser.md)
records planted textual mutations for those modules
under "Planted mutations",
so this was the first run of cargo-mutants over them.

- Part 0
  (evidence `native-mutation-gYTjCf`,
  unmutated test time 29 seconds):
  274 caught,
  32 unviable,
  1 missed,
  4 timeouts.
- Part 1
  (evidence `native-mutation-GHKGah`,
  unmutated test time 33 seconds):
  287 caught,
  22 unviable,
  0 missed,
  2 timeouts.
- Part 2
  (evidence `native-mutation-INArmn`,
  unmutated test time 29 seconds):
  271 caught,
  39 unviable,
  1 missed,
  0 timeouts.
- Part 3
  (evidence `native-mutation-Q70PBa`,
  unmutated test time 23 seconds):
  264 caught,
  41 unviable,
  3 missed,
  3 timeouts.

Together:
1096 caught,
134 unviable,
5 missed
and 9 timeouts.
The second reading set no caught mutant aside in any part.
The 14 mutants that were not caught are dispositioned under
"Missed mutants of the first campaign",
"Mutants that stalled the option scanner"
and "Timeouts caused by a stalled host".

### Missed mutants of the first campaign

#### A wrapper-flag lookup only tests called

`command_options_query.rs`,
`==` to `!=` in `has_wrapper_flag`.
No production code called the function,
and its tests did not tell the two forms apart.
Commit `66a3e9b9c` removes the function;
the tests that used it now assert on the parsed record.

#### The shortest answer the sequencer path reader accepts

`rule_commit_sequencer.rs`,
`<` to `<=` in the length guard of `sequencer_head_paths`.
The guard is right as written,
and no test gave an answer of exactly the shortest accepted length.
Commit `52cc1494b` adds a test:
the three bare names `MERGE_HEAD`,
`CHERRY_PICK_HEAD`
and `REVERT_HEAD`,
each ended by a line feed,
are accepted,
and the same answer one byte shorter is refused.
Planting the mutant fails that test;
with the source restored it passes.

#### A length comparison the typo check already makes

`command_options_short.rs`,
`<` to `<=` in `scan_short_cluster`.
The condition read `position == 2 && position < token.len() && spells_long_option(...)`.
`spells_long_option` returns false for a cluster shorter than three letters,
so the middle term never changed the result
and the mutant was equivalent.
Commit `5e5972f64` removes the term.

#### A second detector of the commit-only hatch

`rule_commit_only.rs`,
`+` to `*` in `has_commit_only_escape_hatch`.
The function had no production caller:
the hatch is found and removed by position in `wrapper_controls.rs`.
Commit `348d94cbe` removes the function and its test.

#### Stash words without a dash

`command_stash.rs`,
`||` to `&&` in `check_top_level_token`.
The condition decides whether a leading word that is not a subcommand makes Git assume `push`.
No test gave a word that satisfies one side of the condition only.
Commit `348d94cbe` adds the words `xpush`,
`apus` followed by a path,
and `xlist`,
each of which Git reads as an assumed `push`.
Planting the mutant fails the test;
with the source restored it passes.

### Mutants that stalled the option scanner

Four timeouts were real:

- `command_options_long.rs`,
  `accept` replaced by `Ok(0)`.
- `command_options_long.rs`,
  `scan_long_option` replaced by `Ok(0)`.
- `command_options_short.rs`,
  `unknown_letter` replaced by `Ok(0)`.
- `command_options_short.rs`,
  `scan_short_cluster` replaced by `Ok(0)`.

Each of these functions returned how many tokens it had read,
and `parse_options` advanced its index by that count.
A count of zero left the tokenizer on the same token for ever.
The logs show it:
11 to 17 test result lines,
then `command_add::tests::... has been running for over 60 seconds`.

Commit `08847be01` removes the count.
The four functions return `Result<bool, OptionError>`,
where `true` means that the next token was the option's value,
and `parse_options` visits each token once in a `for` loop over the slice
and skips a token flagged as a value.
No return value can hold the loop on a token.
`command_status::has_status_hints_override` stepped an index in the same way
and now visits each token once,
remembering the option that awaits a value.
This is the fourth and fifth loop rewritten for the reason given under "The three rewritten loops".

### Timeouts caused by a stalled host

Five timeouts were not caused by the mutant:

- `forwarding.rs`,
  `exit_code` replaced by `-1`.
- `repository_facts.rs`,
  `!` deleted in `sequencer_state`.
- `config_loading.rs`,
  `+` to `*` in `mixed_command`.
- `command_config.rs`,
  `+=` to `-=` in `parse_config_region`.
- `child_environment.rs`,
  `==` to `!=` in `parse_config_count`.

None of them can hold a loop.
The first two replace a return value and invert a plain branch.
The third changes `index + 1 == arguments.len()`,
a comparison inside a `for` loop over the slice.
The fourth steps a counter down from zero,
which fails at once with an arithmetic overflow,
because the test profile checks integer overflow.
The fifth changes which byte counts as a sign,
after which the index still only moves forward.

Every part stalled at the same two moments,
although the parts ran different mutants of different files in separate containers:

- From 19:30:17 to 19:31:47,
  parts 0,
  1
  and 3 each had a mutant in its test phase,
  and all three reached the limit of 90 seconds in the same second:
  the mutants of `forwarding.rs`,
  `config_loading.rs`
  and `child_environment.rs`.
  Part 2 caught its mutant of that moment after 89.7 seconds of test time.
- From 20:11:03 to 20:12:36,
  parts 0 and 1 reached the limit within three seconds of each other:
  the mutants of `repository_facts.rs` and `command_config.rs`.
  Part 2 caught its mutant of that moment after 84.6 seconds,
  and part 3 caught two in a row after 61.6 and 47.1 seconds.
- In parts 0,
  1
  and 3 the mutant before the first stall,
  the stalled one
  and the two after it end in the same second or one second apart:
  19:30:11,
  19:31:47,
  19:32:13 or 19:32:14,
  and 19:32:31.
- No log has a test that ran for over 60 seconds.
  When the limit came,
  the unit target was still printing results:
  334,
  79,
  371,
  109
  and 354 of 443,
  in the order of the list of five.
- Away from the two moments a caught mutant fails in the unit target after about 5 seconds,
  the median of every part.
  The unmutated run,
  which also runs the binary-level target,
  took 23 to 33 seconds.

The times are the modification times of the mutant logs in the four evidence directories,
and the durations are those of `outcomes.json`.
The user journal of the host shows other sessions' Podman at work in both moments:
a container removed at 19:29:40 and another at 19:30:11,
the seconds at which three parts finished a mutant,
and a container creation that took 8 seconds at 20:11:45.
What stalled the host was not established.
The reading is that a wait shared by the whole host,
not processor load from the campaign,
held the tests,
so running fewer parts side by side would not avoid it.

All five files are in the second campaign,
which is the test of this reading.
It confirms it;
see "Second campaign over every file".

### Second campaign over every file

Tree of commit `348d94cbe`,
the final tree.
`mise run //package/git-policy/cli:native:mutation:list` finds 1234 mutants in 67 files.
The two exclusions remove 26 of them,
25 of the kind `replace += with *=`
and 1 of the kind `replace -= with /=`,
and no other mutant,
which leaves 1208.
They ran in four parts of 302 side by side,
all against the one gate image of evidence `native-eqKHwb`.

- Part 0
  (evidence `native-mutation-5mlFOt`,
  unmutated test time 16 seconds):
  270 caught,
  30 unviable,
  0 missed,
  2 timeouts.
- Part 1
  (evidence `native-mutation-cDqmRV`,
  unmutated test time 6 seconds):
  275 caught,
  25 unviable,
  0 missed,
  2 timeouts.
- Part 2
  (evidence `native-mutation-knEe30`,
  unmutated test time 7 seconds):
  260 caught,
  41 unviable,
  0 missed,
  1 timeout.
- Part 3
  (evidence `native-mutation-if52O1`,
  unmutated test time 23 seconds):
  263 caught,
  38 unviable,
  0 missed,
  1 timeout.

Together:
1068 caught,
134 unviable,
0 missed
and 6 timeouts.
The second reading set no caught mutant aside in any part.
Every part exited nonzero,
as the runner must with a timeout.

No mutant of the first campaign that was missed,
that stalled the scanner
or that timed out there
is among the six:

- The five mutants of "Timeouts caused by a stalled host" were each caught,
  after 4 to 10 seconds of test time.
  That is the confirmation the first campaign's reading waited for.
- The eight mutants that replace the result of the four scanner functions with `Ok(true)` or `Ok(false)`
  were each caught,
  after 1 to 19 seconds.
- The two missed mutants that were given a test,
  in `sequencer_head_paths` and `check_top_level_token`,
  were caught.
  The other three no longer exist.

The six timeouts are two stalls of the host,
read in the same way as those of the first campaign:

- From 20:57:22 to 20:58:53,
  parts 0 and 1 reached the limit in the same second:
  `entry.rs`,
  `run_process` replaced by `0`,
  and `config_loading.rs`,
  `&&` to `||` in `mixed_command`.
  Parts 2 and 3 had not reached their first mutant yet.
- From 21:21:51 to 21:23:31,
  all four parts reached the limit within 9 seconds of each other:
  `command_reset.rs`,
  `==` to `!=` in `mode_of`;
  `config_values.rs`,
  `<` to `<=` in `safe_integer`;
  `command_status.rs`,
  `||` to `&&` in `parse_status_region`;
  and `real_git_candidate.rs`,
  `!=` to `==` in `identical_content`.

None of the six sits in a loop condition or changes how a loop advances.
The load average of the host was between 27 and 60 whenever it was read during the campaign,
on 16 processors,
with other sessions' containers and an emulator running.

### Rerun of the six files with a timeout

A timeout is not a caught mutant,
so the six files were run again against the same gate image
(image `e56fa45aa948`,
the base image in every manifest of the second campaign and of this rerun):

- `config_loading.rs`,
  `config_values.rs`
  and `command_reset.rs`
  (evidence `native-mutation-ivjwZs`,
  unmutated test time 8 seconds):
  95 mutants,
  90 caught,
  5 unviable,
  0 missed,
  0 timeouts.
- `real_git_candidate.rs`,
  `command_status.rs`
  and `entry.rs`
  (evidence `native-mutation-UUOsDS`,
  unmutated test time 7 seconds):
  94 mutants,
  87 caught,
  7 unviable,
  0 missed,
  0 timeouts.

Both runs exited 0,
and the second reading set nothing aside.
The six mutants that had timed out were each caught,
after 3 to 15 seconds of test time.
The slowest caught mutant of the rerun took 26 seconds.

### Third campaign over every file

After the rerun the load average of the host had fallen below 15,
and the campaign over every file was run once more:
the same 1208 mutants,
the same four parts,
the same gate image.

- Part 0
  (evidence `native-mutation-WPafzn`):
  272 caught,
  30 unviable.
- Part 1
  (evidence `native-mutation-mKw99E`):
  277 caught,
  25 unviable.
- Part 2
  (evidence `native-mutation-js1Bh1`):
  261 caught,
  41 unviable.
- Part 3
  (evidence `native-mutation-uU7heb`):
  264 caught,
  38 unviable.

Together:
1074 caught,
134 unviable,
0 missed
and 0 timeouts.
Every part exited 0,
noticed its five planted controls,
and names image `e56fa45aa948` as its base.
The second reading set no caught mutant aside.
The unmutated test time was 5 seconds in every part,
and the slowest caught mutant took 14 seconds,
against the limit of 90.
The six mutants that timed out in the second campaign were each caught,
after 4 to 14 seconds.

### Result on the final tree

The third campaign is the result:
on the tree of commit `348d94cbe`,
with the two approved exclusions,
1208 mutants,
1074 caught,
134 unviable,
0 missed
and 0 timeouts,
in one run over every file of `src/native/`.

The mutants it ran are exactly the 1208 that the listing task prints.
The second campaign and the rerun agree with it mutant for mutant:
comparing the two reports by mutant name gives the same outcome for 1202,
and the 6 that the second campaign lost to the stalled host,
and the rerun caught,
are caught.
No mutant was excluded beyond the two kinds the human approved,
no test bound was changed,
and the runner's limit of 90 seconds was left as it is.

Three campaigns over every file ran in this delegation.
The two that ran while other sessions loaded the host each lost mutants to a stall,
at the same seconds in every part;
the one on the quieter host lost none.

## Fuzzing

### Target

`package/git-policy/cli.fuzz` has a fourth target,
`wrapper_controls`,
over control removal and the wrapped-command lifecycle.
Its views,
invariants and limits are in that package's `README.md` under "`wrapper_controls`".
The invariants are stated without Git's option tables.
The lifecycle view runs with fixed repository answers at locations that do not exist on disk,
so it starts no Git and reads no configuration.

### Controls

`mise run //package/git-policy/cli.fuzz:test` runs 13 generator controls,
all passing.
The new ones run the removal invariants over every three-token input from an 87-token table,
the separator invariant around every table command,
and the frontier invariants over every two-token input at both locations with every combination of answers.
They count that every way of reading a command region,
every control effect
and every lifecycle ending is reached.

`mise run //package/git-policy/cli.fuzz:test:planted` plants nine defects one at a time in a disposable copy
and requires a control to fail for each
(evidence `package/git-policy/cli.fuzz/target/verification/planted-snVlL5`).
All nine are noticed.
Both tasks were run again on the tree of commit `348d94cbe`,
with the same result:
13 controls pass,
and all nine planted defects are noticed
(evidence `package/git-policy/cli.fuzz/target/verification/planted-z8kyPr`).
Four are new:
a control spelling removed anywhere in a region read without a table,
keep-going removed without being recorded,
a forwarded real commit,
and a publishing push that skips the manual-push gate.
One existing plant was re-anchored,
because the line it replaced was in a rewritten loop.

A control failed while it was being written,
and the failure was a wrong expectation in the control:
recorded under "Status-format options make a commit a dry run".

### Smoke campaign

`mise run //package/git-policy/cli.fuzz:smoke` on the tree of commit `d8adb6015`:
AddressSanitizer targets,
30 seconds each,
in the bounded container.
Every target exited 0
(evidence `package/git-policy/cli.fuzz/target/verification/campaign-PeWOk0`):

- `global_arguments`: 834,636 executions.
- `config_loading`: 268,902 executions.
- `config_schema`: 29,924 executions.
- `wrapper_controls`: 41,774 executions.

The same task on the tree of commit `348d94cbe`,
the final tree,
on a quieter host
(evidence `package/git-policy/cli.fuzz/target/verification/campaign-ma0PAO`),
again with every target exiting 0:

- `global_arguments`: 1,448,046 executions.
- `config_loading`: 511,433 executions.
- `config_schema`: 60,003 executions.
- `wrapper_controls`: 105,604 executions.

The new target is slower per execution than the argument targets
because each execution runs the whole lifecycle twice and removal three times.
Thirty seconds is a smoke run,
not a campaign.

## Commits

In order:

- `6b14f1249`: wrapper controls removed by position, one hatch mechanism.
- `99646d0c6`: the stage, triggers, events and the bounded fix loop.
- `329de1b97`: the three configuration decisions.
- `a9010f435`: repository facts, rule cores, the shipped-policy adapter and the fixed transforms.
- `9cc45dc2c`: tests pinning the linked sequencer head file.
- `d86b7463c`: the two mutant exclusions and the listing task.
- `d1c02273a`: the three loop rewrites.
- `3836299a7`: the lifecycle, the refusal frontier and direct commands.
- `d8adb6015`: the fuzz target and the status-format commit control.
- `4bef275e6`: removal of a helper only tests called.
- `24cfeaaee`: the classifier test that kills the surviving mutant.
- `e0042aeda`: the command word and region read in one place, and the tool-cache test.
- `37d362484`: single-body platform functions, the loop-free bounded read,
  the shared command split and the Windows type-check task.
- `4133d2900`: a task that runs the repository's Rust linter over the native source.
- `090c147e7`: the `policy-incomplete` failure code.
- `08847be01`: option scanners that say whether they took the next token,
  and the status scan that visits each token once.
- `66a3e9b9c`: removal of a wrapper-flag lookup only tests called.
- `52cc1494b`: the test of the shortest answer the sequencer path reader accepts.
- `5e5972f64`: removal of a length comparison the typo check already makes.
- `348d94cbe`: the stash words without a dash,
  and removal of the second detector of the commit-only hatch.

Commits that only change this document are not listed.

## Superseded passages elsewhere

[`cli-git-native-foundation.md`](cli-git-native-foundation.md) still says that the executable
stops every repository-changing command,
in its sections "Authority and scope",
"What the executable does today"
and "Not ported".
Its sections "Survivors left" and "Timeouts" list 10 missed mutants and 17 timeouts;
"Survivors removed with their code" and "The three rewritten loops" record why none of them exists any more.
This document supersedes those passages.
They were left as written,
because the permission to edit that file covered only the three configuration decisions.

## Module map

All files are in `package/git-policy/cli/src/native/`;
each has a sibling `*_tests.rs`.

### Controls

- `wrapper_controls.rs`:
  the 14 general control spellings,
  their meanings,
  the `Controls` record,
  and removal before the subcommand.
- `wrapper_invocation.rs`:
  removal of every control from one invocation by position,
  how the command region was read,
  and the command word and region of the result.
- `command_worktree.rs`:
  whether a `git worktree` invocation creates or moves a worktree.
- `escape_hatch.rs`:
  three hatch spellings.

### Engine

- `policy_trigger.rs`:
  triggers,
  the trigger set of every shipped policy,
  and which lifecycles are ported.
- `policy_events.rs`:
  events and their JSON Lines rendering.
- `policy_engine.rs`:
  one stage.
- `policy_pass.rs`:
  one pass of three stages.
- `policy_convergence.rs`:
  the bounded fix loop.
- `policy_checks.rs`:
  the shipped policies over rule cores and facts.
- `policy_transforms.rs`:
  atomic-push,
  commit-only
  and status-hints,
  in order.

### Facts and rules

- `repository_location.rs`,
  `repository_facts.rs`:
  the location query and the facts interface.
- `rule_linked_worktree.rs`,
  `rule_branch_worktree.rs`,
  `rule_add_explicit.rs`:
  pure decisions of three built-in policies.
  `rule_require_root.rs` and the commit-only cores come from the command-parser delegation.

### Lifecycle and frontier

- `wrapped_command.rs`:
  the lifecycle of a wrapped command.
- `unported.rs`:
  the list of unported work and the refusal notice.
- `refusal_frontier.rs`:
  commits,
  worktree copies,
  aliases
  and leases.
- `pending_state.rs`:
  leftover transaction and worktree-copy state.
- `git_builtins.rs`:
  the built-in command names of Git 2.56.0.

### Changed foundation modules

- `entry.rs`:
  control removal,
  then management,
  then the lifecycle.
- `action.rs`:
  `Action::Forward` carries the arguments to forward and warning events.
- `management.rs`:
  direct commands run a pass.
- `invocation_config.rs`:
  configuration is loaded from the measured identity.
- `config_loading.rs`,
  `global_arguments.rs`,
  `management_arguments.rs`:
  the three loop rewrites.
  `global_arguments.rs` also gained `command_tokens`.
- `forwarding.rs`,
  `git_metadata.rs`,
  `real_git_candidate.rs`:
  one body per platform-specific function,
  and the bounded read without a loop.
- `diagnostics.rs`:
  the failure code `policy-incomplete`.

### Changed command parser modules

- `command_options.rs`,
  `command_options_long.rs`,
  `command_options_short.rs`:
  the scanners return whether the next token was a value,
  and `parse_options` visits each token once.
- `command_status.rs`:
  the status-hints scan visits each token once.
- `command_options_query.rs`:
  `has_wrapper_flag` removed.
- `rule_commit_only.rs`:
  `has_commit_only_escape_hatch` removed.

### Binary-level tests

- `binary_controls_tests.rs`:
  hazards 1,
  2
  and 6.
- `binary_frontier_tests.rs`:
  hazards 3
  and 4.
- `binary_builtin_tests.rs`:
  the four pre-forward built-in policies,
  keep-going,
  a `warn` severity,
  and commit-only.
- `binary_policy_tests.rs`:
  configuration errors and hazard 5.
- `binary_management_tests.rs`:
  direct commands.

## What remains

- The commit path:
  transactions,
  hooks,
  locks,
  replay,
  recovery,
  the post-commit lifecycle
  and auto-push.
  Nothing of it is started,
  and it is the reason for the rewrite.
- Candidate content for `git add`,
  direct commands
  and manual push,
  and with it every content policy,
  including the built-in `final-newline`.
- The manual-push lifecycle.
- Worktree copy,
  alias resolution,
  lease validation,
  index-writer coordination,
  and recovery of leftover state.
- Post-command output,
  which needs forwarding that waits for Git instead of replacing the process.
- Fix application for direct fix.
- Scope validation for direct commands.
- A path that reports `policy-incomplete`:
  the code and its spelling test exist,
  and no shipped policy ported so far has machinery of its own that can fail.
- The 263 `require-rustdoc` findings of the repository's Rust linter,
  if the wrapper is to pass that linter.
- A decision on each item under "Choices open to veto" and "Forwarded with a known omission".
