# cli-git Rust open decisions

A decision brief for the owner of this repository.
It takes every choice that the `Open questions` section of the
[behavior ledger](cli-git-rust-behavior-ledger.md) left undetermined for the Rust rewrite of the Git policy wrapper
(`package/git-policy/cli`),
checks each against the incumbent TypeScript,
the planning documents,
the native Rust code as it stands,
and current outside precedent,
and sorts it into one of these verdicts:
settled by evidence,
or needing the owner.

## Purpose and how to respond

The later phases of the rewrite
(optional policies;
then transactions,
hooks,
locks,
replay and recovery;
then cutover)
cannot start cleanly while these choices are open.
This document changes no code and decides nothing.

What to inspect:

- `Verdict summary` lists every item with its verdict and,
  for settled items,
  the answer in one line.
- Each item section states the choice in plain words,
  the evidence with its source,
  and for an item needing the owner the options with a ranking.
- `Question batches` holds a suggested wording and short option labels for each question,
  grouped by the phase the question blocks.

How to respond:
answer the questions in `Question batches`,
and veto any item under `Settled by evidence` whose answer is not the one wanted.
A settled item is adopted without a question unless it is vetoed.

## Outcome

Every question in this brief is now decided,
so none of them is open any more,
even where an item section below still reads "Needs the owner".
The answers are recorded in [`cli-git-rust-implementation.md`](../handover/cli-git-rust-implementation.md),
section `User decisions 2026-10-05`:
the owner answered the optional-policy batch and four transactions-phase questions
(signals, the exit status of a signaled Git, and the hook entries on Unix and on Windows),
and asked on 2026-10-06 that a clearly dominant option be adopted without a question for that session.
Every other item was adopted on that basis,
each with its reason and open to the owner's veto,
in the subsection `Adopted without a question`.
That subsection departs from this brief's ranking once:
the whole native test suite runs on hosted macOS and Windows runners.
Items under `Settled by evidence` stand as written,
except recovery before read-only commands,
whose adopted form keeps commands that start no Git process exempt.

An implementation reading this brief takes the answer from that subsection,
not from the ranking in the item section.

## Sources and conventions

Source:
a read-only survey on 2026-10-05.

- The incumbent TypeScript under `package/git-policy/cli/src` (outside `src/native`)
  and `package/git-policy/cli/SPEC.md` are cited as of commit `cf2bf70d2`,
  the last commit that changed them.
- The native Rust files under `package/git-policy/cli/src/native` are cited as of commit `4bef275e6`.
- The candidate and scanner layer is cited from branch `feat/cli-git-native-candidates` at `1519fd577`,
  which was not merged into `main` when it was read.
  Its handover is `doc/handover/cli-git-native-candidates.md` on that branch.
- Handover documents are cited by section name,
  because they change while work continues.

Path convention,
the same as the ledger's:
`SPEC.md`,
`README.md`,
and paths starting with `src/`,
`e2e/`,
or `perf/` are relative to `package/git-policy/cli/`.
Every other path is repository-relative.

Planning citations:
"implementation plan" is `doc/planning/cli-git-rust-implementation.md`,
"rewrite scope" is `doc/planning/cli-git-rust-rewrite.md`,
and "ledger" is `doc/planning/cli-git-rust-behavior-ledger.md`.

Evidence labels:

- "Measured" means a command was run in this checkout during the survey,
  and the command or its result is given.
- "By reading" means the claim was derived from the cited source lines and was not run.
- Outside precedent was fetched on 2026-10-05 from the named release tag,
  or from the project's default branch where no tag is named.

Option letters are local to the heading they appear under.

## Decisions already made

The owner made these decisions on 2026-10-05.
They are recorded in `doc/handover/cli-git-rust-implementation.md`,
section `User decisions 2026-10-05`,
and are not reopened here.

- The four optional policies
  (`security/forbidden-strings`,
  `mono/forbidden-root-context`,
  `mono/dependent-version-bump`,
  `markdown/autofix`)
  run only when `cli-git.config.jsonc` lists them.
  An empty `{}` file and no file behave identically.
  This repository's translated configuration must list each of them.
- The repository root is the top level Git reports,
  for configuration lookup and for the require-root policy,
  honoring `--git-dir`,
  `--work-tree`,
  and their environment forms.
- A legacy `cli-git.config.ts` beside the JSONC file is reported only by `git cli-git check`.
  A legacy file without a JSONC file stays a migration error.

The same section records ideas the owner rejected,
which nothing here proposes again:
a parity rule in `AGENTS.md`,
and a check mode for file-enforcer.
Its sibling section `User correction: no vetting decision gate` records that a technology owner is chosen
by measurement,
not by a decision gate for the owner.
The crate notes in this document follow that:
they report footprint and leave the pick to measurement.

These decisions close these ledger items in full or in part:
the defaults half of "Fixed policy names and optional-policy defaults",
the root half of "Configuration root",
and "Legacy configuration during the rollback window".
The remaining halves have their own sections.

## Verdict summary

### Settled by evidence

- Policy names:
  keep the namespaced IDs this repository's own configuration defined.
- Policy options other than the rules file:
  `builtinRules`,
  `rules`,
  and `exclude` stay;
  `executable` and `command` are rejected.
- Bare repository:
  no configuration file is read and the defaults apply.
- Retired trust commands,
  stream:
  the explanation goes to standard error,
  and `--help` output to standard output.
- Retired trust commands,
  `status`:
  it is retired with `trust` and `untrust` and gets no other purpose in this rewrite.
- Event sequence numbering:
  once per invocation,
  from `0`,
  across every batch the invocation writes.
- Non-UTF-8 paths in journals:
  `captured.json` keeps one code point per path byte,
  the form the incumbent reads.
- Tool-cache allowlist:
  it stays compiled in,
  with uv's documented Windows location added.
- Verbose switch,
  the `--verbose` argument:
  a Git argument no longer turns wrapper diagnostics on.
- Markdown policy without the linter:
  an engine failure with exit status `2`,
  as the incumbent reports a command it cannot start.
- Found outside the ledger's list,
  manual-push gate without a configuration file:
  the gate applies whenever an enabled policy declares it.
- Found outside the ledger's list,
  recovery before read-only commands:
  it runs there too once recovery is ported.
- Found outside the ledger's list,
  a wrapper failure before Git runs:
  exit status `2`,
  as the spec's exit contract says.

### Needs the owner

Optional policies phase:

- How the forbidden-strings rules file is named (`FORBIDDEN_STRINGS_RULES`).
- Which failure code reports a shipped policy that could not complete.
- Whether the dependent-version bump keeps a second implementation in TypeScript.

Transactions phase:

- Whether a signal sent to a waiting wrapper reaches real Git.
- Which exit status reports real Git ended by a signal.
- How an event carries a path that is not UTF-8.
- The form of a hook entry file on Unix.
- The form of a hook entry file on Windows.
- How far locks and journals interoperate between the incumbent and the native wrapper.
- What happens to a legacy single-journal directory.
- Whether the release executable carries the test phase markers.
- Found outside the ledger's list:
  whether an alias for `commit` stays outside the commit guard.

Cutover:

- The exit status of the retired trust commands.
- When and how an unsupported Git is detected.
- Which Git releases count as supported.
- The wrapper's line after `git --version`.
- The wrapper's note after `git status`.
- Which native tests run on macOS and Windows.
- What happens to the hk cleanup utility.
- Which switch turns verbose diagnostics on.
- Which numbers gate performance acceptance.
- Found in the rewrite scope:
  whether the reported slow commit is an acceptance workload.

## Fixed policy names

### The choice

What the four optional policies are called.
The name is what a person writes as a key in `cli-git.config.jsonc`,
what appears as `policyId` and as the prefix of `code` in every event,
and what follows `--no-enforce-` to skip that policy for one command.

### Evidence

- Under the incumbent a policy ID from a plugin is `<namespace>/<policy-name>`,
  and each repository's configuration picks the namespace (`SPEC.md:68`,
  `347-349`).
- This repository's configuration picked `markdown`,
  `mono`,
  and `security` (`cli-git.config.ts:22-26`).
  The policy-local names are `autofix` (`package/git-policy/markdown-lint/src/index.ts:96`),
  `forbidden-root-context` (`package/git-policy/repository/src/index.ts:114`),
  `dependent-version-bump` (`package/git-policy/repository/src/dependent-version-bump-policy.ts:247`),
  and `forbidden-strings` (`package/git-policy/forbidden-strings/src/index.ts:115`).
- The implementation plan makes unknown policy IDs errors (lines 55 to 61)
  and preserves JSONL events (lines 107 to 112).
- The native registry fixes the four names to this repository's IDs
  (`POLICY_REGISTRY`,
  `src/native/policy_registry.rs:101-174`).
  This is a provisional choice:
  `doc/handover/cli-git-native-foundation.md`,
  section `Policy names`,
  lists it as open to veto and names package-derived or flat names as alternatives.
- The decision record of 2026-10-05 itself names `mono/dependent-version-bump`.

### Verdict

Settled by evidence:
keep `markdown/autofix`,
`mono/forbidden-root-context`,
`mono/dependent-version-bump`,
and `security/forbidden-strings`.
The determining sources are the owner's own configuration,
which defined these IDs,
and the implementation plan lines 107 to 112,
which preserve the events that carry them.
A rename would change every event,
every escape flag,
and every document that quotes one,
and no document asks for it.

A consequence to know:
a namespace used to be each repository's choice and is now fixed for every repository.

## Policy option surface

### The choice

This item holds separate decisions.

Which options a policy accepts in `cli-git.config.jsonc`.

How the wrapper learns which private rules file the bundled forbidden-strings scanner loads
on top of its built-in rules.
Today an environment variable,
`FORBIDDEN_STRINGS_RULES`,
names that file.
The answer decides whether the scan a commit gets depends on the environment `git` was started from,
or only on the repository's configuration.

### Evidence

- The rewrite scope lines 126 to 131 forbid `executable` and `command`:
  repository JSONC must not select a program.
- Native `main` accepts `builtinRules` for `security/forbidden-strings`
  and `rules` and `exclude` for `markdown/autofix`,
  and rejects every other key
  (`src/native/config_schema.rs:76-96`;
  `forbidden_strings_options`,
  `src/native/config_policies.rs:109-151`).
  Those are the incumbent's options minus the forbidden ones
  (`src/optional/forbidden-strings/index.ts:40-49`;
  ledger,
  "Markdown autofix").
- Under the incumbent the policy passes no rules path.
  The scanner child process reads the variable itself:
  `--rules`,
  then `FORBIDDEN_STRINGS_RULES`,
  then `forbidden-strings.local.txt` in its working directory
  (`package/forbidden-strings/cli/src/lib.rs:628-649`).
  A file that is named and missing is an error;
  a missing default file is tolerated when the built-in rules are on
  (comment at `package/forbidden-strings/cli/src/lib.rs:619-621`).
- The policy declares the variable and the file as inputs (`src/optional/forbidden-strings/index.ts:78-101`).
- This repository sets the variable in the root `mise.toml:1323-1329`
  to `{{config_root}}/.cache/forbidden-strings.rules.txt`.
  file-enforcer generates that file from a committed appendix and a gitignored private one,
  and no rules file exists at the repository root (`file-enforcer.config.ts:784-805`).
- The scanner library takes the path from its caller:
  `Scanner::load(runtime_rules_path, builtin_rules, explicit)`
  (`package/forbidden-strings/cli/src/scanner.rs:64`).
- Native `main` reads no variable and has no rules option.
  The candidate branch made a provisional choice:
  `rules_source` restates the variable-then-default precedence
  (`src/native/scanner_selection.rs:71-87` on `feat/cli-git-native-candidates`;
  its handover,
  section `Rules file and eligibility`).
- `doc/handover/cli-git-native-foundation.md`,
  section `Options`,
  says the rules-file naming "is not decided here".
- By reading:
  the wrapper and the variable reach a shell through the same `mise` activation
  (ledger,
  "Executable entry":
  the root `mise.toml` puts `node_modules/.bin` on `PATH`),
  so a wrapper without the variable needs an unusual launch,
  such as calling the launcher by path from a process whose environment was reduced.

### Verdict

The accepted options are settled by evidence:
`builtinRules`,
`rules`,
and `exclude` stay,
and `executable` and `command` are rejected with a message that says they are retired.
The determining source is the rewrite scope lines 126 to 131 together with incumbent parity for the rest.

The rules-file naming needs the owner.

### Options: naming the rules file

#### Option A: keep the environment variable

`FORBIDDEN_STRINGS_RULES` names the file,
and without it the scanner looks for `forbidden-strings.local.txt` at the repository top level.
Nothing is added to JSONC.

```toml
# mise.toml
FORBIDDEN_STRINGS_RULES = "{{config_root}}/.cache/forbidden-strings.rules.txt"
```

For:
no cutover edit;
the wrapper,
the standalone scanner and CI keep one shared name for the file;
the path of a gitignored scratch file stays out of committed configuration.

Against:
which rules guard a commit depends on something outside the repository's configuration.
A wrapper started without the variable scans with the built-in rules only and says nothing,
because the fallback file does not exist in this repository (by reading).

#### Option B: a JSONC option only

A repository-relative `rulesFile` option names the file,
and the wrapper ignores the variable.

```jsonc
// cli-git.config.jsonc
{
  "policies": {
    "security/forbidden-strings": ["error", { "rulesFile": ".cache/forbidden-strings.rules.txt" }]
  }
}
```

For:
the file is named explicitly on every run,
so a missing file is an error wherever `git` was started from;
the configuration alone describes the policy.

Against:
the standalone scanner and the CI workflow still need the variable,
so the path is written in more than one place;
the option needs a rule for paths that leave the repository;
a repository that wants no private rules must say so instead of relying on an absent default file.

#### Option C: the JSONC option first, then the variable

`rulesFile` wins when the configuration has it;
otherwise the variable;
otherwise the default file.

For:
this repository can pin the path in configuration,
and a repository that sets only the variable keeps working.

Against:
more than one source for the same fact,
and a precedence rule to test and to explain in diagnostics.

#### Ranking

A over C,
because the silent fallback that C guards against needs a wrapper started without the `mise` environment,
which also supplies the wrapper itself,
while C adds a second source and a precedence rule.
C over B,
because B alone leaves the standalone scanner and CI on the variable anyway,
so the path is named in more than one place with no fallback between them.

## Configuration root in a bare repository

### The choice

What configuration applies when `git` runs inside a bare repository,
which has no worktree and therefore no top level.

### Evidence

- The decision of 2026-10-05 makes the root the top level Git reports.
- Git reports no top level for a bare repository,
  for the inside of `.git`,
  and outside a repository.
  The native loader then reads no file and returns the defaults
  (`load_identity_config`,
  `src/native/invocation_config.rs:34-47`).
- The incumbent walks up to the nearest directory holding a Git marker
  (`src/trust/config-discovery.ts:110-128`).
  By reading,
  an ordinary bare repository has no such marker in any ancestor directory,
  so the incumbent also finds no configuration there.

### Verdict

Settled by evidence:
no file is read and the defaults apply.
The determining source is the decision of 2026-10-05.
A consequence to know,
by reading:
in a bare repository the optional policies are off,
so a push from one is not scanned by `security/forbidden-strings`.

## Retired trust commands

### The choice

`git cli-git trust`,
`git cli-git untrust`,
and `git cli-git status` existed to approve,
revoke,
and inspect permission to run a repository's configuration code.
JSONC configuration is data,
so nothing is left to approve.
The open points are what a script or person observes when one of these commands is still typed:
the exit status,
the stream the explanation is written to,
and whether `status` gets another job.

### Evidence

- The implementation plan line 444:
  "Retired trust commands explain that JSONC no longer requires code-execution approval."
- The incumbent wrote one JSON object to standard output for each of them,
  and help text to standard output with exit status `0` (`SPEC.md:799-812`,
  `817`,
  `839-842`).
- The stream contract puts machine objects on standard output and human disclosures on standard error
  (`SPEC.md:1579-1586`),
  and reserves exit status `2` for usage,
  trust,
  configuration and engine failures (`SPEC.md:1588-1598`).
- Native provisional choice:
  `plan_management` exits `0`,
  with the explanation on standard error for an attempted action and on standard output for `--help`
  (`src/native/management.rs:299-313`;
  text at `99-107`).
  `status` is parsed as a retired command like the others (`src/native/management_arguments.rs:65-73`).
  `doc/handover/cli-git-native-foundation.md`,
  section `Retired trust commands`,
  lists exit `0` as open to veto and exit `2` as the alternative.
- Measured:
  `rg --count-matches 'cli-git (trust|untrust|status)'` over the repository,
  outside documentation and the wrapper package,
  matches `.github/workflows/cli-git-trust.yml` and `package/forbidden-strings/cli/README.md` only.
  The workflow retires with the trust subsystem (ledger,
  "CI workflows").
  No script outside the package depends on the commands' output.

### Precedent

- husky 9.1.7 keeps a harmless retired command working and fails the ones that no longer do anything:
  `install` prints `husky - install command is DEPRECATED` on standard error and continues with exit `0`,
  while `add`,
  `set`,
  and `uninstall` print the same line and exit `1`
  (`bin.js` in [husky][husky]).
- Git 2.56.0 refuses its own retiring command:
  `git whatchanged` prints an explanation on standard error and dies unless `--i-still-use-this` is given
  (`builtin/log.c:550-557`,
  `usage.c:381-405` in [Git][git-src]).

### Verdict

The stream is settled by evidence:
the explanation is human prose,
so it goes to standard error,
and `--help` output goes to standard output.
The determining source is `SPEC.md:817` and `1579-1586`,
with the measurement that no consumer parses these commands' output.

`status` is settled by evidence:
it is retired with the others.
The determining source is the implementation plan line 444 with `SPEC.md:799-812`,
which lists `status` among the trust commands.
A new job for the name would be a new feature,
and the rewrite scope lines 116 to 119 keep the first release to current coverage.

The exit status needs the owner.

### Options: exit status of a retired trust command

#### Option A: exit `0`

```text
$ git cli-git trust --yes
git cli-git trust is retired. cli-git now reads cli-git.config.jsonc as data and runs no
repository-supplied code, so there is no code execution to approve, revoke or report.
Nothing was changed; existing trust records are left in place and are no longer read.
$ echo $?
0
```

For:
old instructions and scripts of the form `git cli-git trust --yes && git commit ...` keep working
under both wrappers during the rollback period;
nothing failed,
because the approval the caller asked for is no longer needed.

Against:
a script never learns that it calls a command that does nothing,
so stale instructions stay.

#### Option B: exit `2`

The same explanation,
with the status the contract uses for usage failures.

For:
automation notices at once and gets cleaned up;
it matches how Git treats its own retiring command.

Against:
during the rollback period one instruction cannot serve both wrappers,
because the incumbent requires `trust` where the native wrapper would fail it.

#### Option C: exit `0` for `trust`, exit `2` for `untrust` and `status`

`trust` asked for a state that now always holds,
so it succeeds.
`untrust` and `status` asked for something that cannot be given
(a revocation,
a trust report),
so they fail.

For:
the common setup step keeps working,
and a caller that expected a report or a revocation is told it got none.
This is husky's split.

Against:
commands that retire together behave differently,
which needs explaining.

#### Ranking

A over C,
because the measurement found no consumer of `untrust` or `status` output to protect,
so C's split buys nothing here and costs an explanation.
C over B,
because B breaks the one instruction that must work under both wrappers while rollback is possible.

## Signals

### The choice

Pure forwarding is not affected:
on Unix the native wrapper replaces itself with real Git for a command it adds nothing to,
so Git receives every signal directly
(`replace_process_with_real_git`,
`src/native/forwarding.rs:161-170`).

The choice concerns the commands where the wrapper must outlive Git:
a commit transaction,
a push behind the manual-push gate,
a worktree creation that copies ignored files,
and every command on Windows,
which cannot replace a process.
There the wrapper waits for real Git as a child.
This item holds separate decisions.

What happens when something signals the waiting wrapper,
for example an agent harness that cancels a tool call,
`timeout`,
or `kill <pid>`:
does real Git,
with its hooks and editor,
stop too,
or keep running without the wrapper that owns the transaction.

Which exit status the wrapper reports when real Git itself was ended by a signal.

### Evidence

- The incumbent installs no signal handler
  (ledger,
  "Forwarding, standard streams, signals, and exit codes",
  with the `rg` that finds none).
  By reading,
  a signal to the wrapper ends the wrapper,
  and nothing signals the Git child.
- The incumbent maps a signaled Git to exit `1` (`src/bin.ts:454-456`).
  The ledger lists that under "Incumbent defects and stale comments".
- The spec says "When real Git runs, preserve its exit code" (`SPEC.md:1600-1601`)
  and is silent on a Git that has none.
- The spec says policy cancellation uses a signal object (`SPEC.md:383-385`);
  nothing ever aborts it (ledger,
  "Spec and code disagreements",
  "Policy cancellation").
- The implementation plan ports "signals" (lines 84 to 91) and preserves "cancellation" (lines 222 to 224)
  without a contract,
  and requires tests of "process termination and restart" (lines 340 to 347).
- Native provisional choices,
  both in `src/native/forwarding.rs`:
  `run_real_git` installs no handler (`120-149`),
  and `exit_code` reports a signaled child as `128 + N` (`77-89`).
  `doc/handover/cli-git-native-foundation.md`,
  section `Forwarding`,
  records both and cites Git's convention for the second.
- `doc/handover/cli-git-rust-implementation.md`,
  section `Native wrapper foundation`,
  records that forwarding a signal from a waiting wrapper needs signal handling the standard library lacks,
  and calls the crate "a technology choice for the transaction phase".
- A terminal sends Ctrl-C to every process in the foreground group,
  so the wrapper and Git both receive it without any forwarding
  (the reason [sudo][sudo-man] gives for not relaying such a signal).

### Precedent

- Git 2.56.0 reports its own signaled child as `128 + N`,
  "so that code & 0xff mimics the exit code that a POSIX shell would report"
  (`run-command.c:574-583` in [Git][git-src]).
- [tini][tini] forwards every signal except `SIGCHLD` to its child and exits `128 + N` for a signaled child
  (`src/tini.c:520-540`,
  `574-590`).
- [sudo][sudo-man] relays signals to its command,
  but `SIGINT` and `SIGQUIT` "are only relayed when the command is being run in a new pty
  or when the signal was sent by a user process, not the kernel.
  This prevents the command from receiving SIGINT twice each time the user enters control-C."
- [cargo][cargo] replaces its process on Unix,
  and on Windows installs a console handler that ignores Ctrl-C in the parent,
  so the child decides,
  then waits (`crates/cargo-util/src/process_builder.rs:320-330`,
  `664-690`,
  `752-777`).
- [foreground-child][foreground-child],
  the helper npm test tools use,
  proxies fatal signals to the child and,
  when the child was ended by a signal,
  ends the parent with the same signal:
  "this parent process will exit in the same way".

### Crate footprint

Any option that catches a signal needs an interface the Rust standard library lacks.
This is not an owner question;
the pick follows the behavior chosen.
Measured facts for that pick:

- The wrapper's lockfile on `main` holds 2 packages (`package/git-policy/cli/Cargo.lock`).
  With the scanner linked it holds 155 (the same file on `feat/cli-git-native-candidates`),
  among them `libc`,
  `rustix`,
  `windows-sys`,
  and `tracing`
  (versions 0.2.186,
  1.1.4,
  0.61.2,
  and 0.1.44 in `package/forbidden-strings/cli/Cargo.lock`).
  It holds none of `signal-hook`,
  `nix`,
  or `ctrlc`.
- Manifests in this repository that name such a crate directly:
  `libc` in `package/wg/cli-quicker-exempt/Cargo.toml:36`
  and `package/music-player/desktop-app/Cargo.toml:188`;
  `windows` in `package/music-player/desktop-app/Cargo.toml:203`
  and `package/file-manager/app/Cargo.toml:41`.
  No manifest names `signal-hook`,
  `ctrlc`,
  `nix`,
  `rustix`,
  or `windows-sys`.
- [signal-hook][signal-hook] 0.4.5 depends on `libc` and `signal-hook-registry`,
  and describes its Windows support as limited.
- [ctrlc][ctrlc] 3.5.2 handles `SIGINT`,
  and with a feature also `SIGTERM` and `SIGHUP`;
  its handler closure takes no argument,
  so it cannot tell which signal arrived,
  and its manifest marks it passively maintained.
- [nix][nix] 0.31.3 covers Unix-like systems only.
- [rustix][rustix] 1.1.5 can send a signal (`src/process/kill.rs`);
  its handler interface (`kernel_sigaction`) sits in a module the crate hides,
  renames periodically,
  and reserves for implementers of a C library (`src/lib.rs:308-323`).
- `libc` and `windows-sys` are what [cargo][cargo] uses for the same job.

### Verdict

Both decisions need the owner.

### Options: a signal sent to the waiting wrapper

#### Option A: no handling

As today.
The wrapper ends at once.
Real Git,
its hooks,
and an editor keep running unless the same signal reached them too.
The next wrapper invocation finds a transaction whose owner is gone and recovers it.

For:
incumbent parity;
no new dependency;
the recovery path it relies on is required and tested anyway.

Against:
cancelling the wrapper alone,
which is how `timeout` and `kill <pid>` work,
leaves Git and a hook running while recovery already treats the transaction as abandoned (by reading);
locks and the private repository stay behind until another command runs.

#### Option B: catch, relay when the wrapper alone was signaled, wait, clean up

The wrapper handles `SIGINT`,
`SIGTERM`,
`SIGHUP`,
and `SIGQUIT`.
It passes the signal on to Git only when the sender was a process,
not the terminal,
which is sudo's rule.
It then waits for Git,
releases its locks,
removes the transaction,
and exits with the status chosen under `Options: exit status when real Git was ended by a signal`.
On Windows it ignores console Ctrl-C the way cargo does and lets Git decide.

For:
cancelling the wrapper cancels Git;
a cancelled commit leaves nothing for recovery;
Ctrl-C reaches Git once.

Against:
reading the sender needs `sigaction` with signal details,
so `libc` or `signal-hook`,
and `unsafe` code for the handler;
if Git ignores the signal,
for instance while an editor is open,
the wrapper keeps waiting,
which is what Git alone would do.

#### Option C: catch and relay every signal

As Option B without the sender check,
which is what tini does.

For:
no sender check,
so no signal details have to be read.

Against:
Ctrl-C at a terminal reaches Git and its hooks twice,
once from the terminal and once relayed,
the case sudo's manual describes;
a hook that cleans up on the first `SIGINT` is ended by the second.

A further design was considered and dropped:
starting Git in its own process group and relaying everything,
as a supervisor does.
Git,
an editor,
and a credential prompt need the terminal's foreground group,
and a background group that reads the terminal is stopped.

#### Ranking

B over A,
because A leaves Git and hooks running after the process that owns the transaction is gone,
in the very case an agent produces when its tool call is cancelled.
A over C,
because C changes what interactive Ctrl-C does to hooks,
while A keeps today's behavior,
which recovery already has to cover.

### Options: exit status when real Git was ended by a signal

The example is real Git ended by `SIGTERM`,
signal number 15,
during a commit.

#### Option A: exit `1`

```text
$ git commit --message=message -- a.txt ; echo $?
1
```

For:
incumbent parity.

Against:
the caller cannot tell a signal from an ordinary failure;
the ledger lists this as a defect.

#### Option B: exit `128 + N`

```text
$ git commit --message=message -- a.txt ; echo $?
143
```

For:
a shell shows the same number as for real Git run directly;
it is Git's own convention for its children;
it needs no signal interface;
the native code already does it.

Against:
a caller that is not a shell sees an exit code where real Git run directly would show a signal death,
so the commands the wrapper waits on differ from the ones it replaces itself for.

#### Option C: end the wrapper with the same signal

After cleanup the wrapper resets the handler and sends itself the signal.
A shell still shows `143`;
a program that inspects the status sees termination by `SIGTERM`.

For:
every caller sees what real Git run directly would show,
on the waiting path and the replacing path alike.

Against:
it needs the signal interface even if Option A is chosen for forwarding;
Windows has no equivalent,
and foreground-child's own documentation says the status there may come out as `1`.

#### Ranking

B over C,
because shell callers,
which is what agents and scripts here are,
cannot tell them apart,
and B needs no signal interface.
C over A,
because A hides that Git was killed,
which the ledger already records as a defect.

## Event sequence numbering

### The choice

Every JSONL event carries a `sequence` number.
One wrapper invocation can write events at more than one moment:
before forwarding,
at the manual-push gate,
and after a commit.
Either the numbers run on across those moments,
or they restart at `0` for each.

### Evidence

- The spec:
  "`sequence` starts at `0` for each cli-git invocation and increments by one in emission order"
  (`SPEC.md:1306`),
  and configuration warnings share the "invocation-local sequence" (`SPEC.md:1365`).
- The incumbent code numbers each engine run from `0`
  (`src/policy-engine/engine.ts:298`;
  `src/policy-engine/post-commit-lifecycle.ts:178`,
  `188`),
  which the ledger records by reading under "Spec and code disagreements".
- The incumbent's own end-to-end suite enforces the spec.
  `extractPolicyEvents` reports every event whose `sequence` differs from its position
  in one invocation's standard error (`e2e/jsonl-event-fixture.ts:159-164`),
  and the suite applies it to every wrapper attempt
  (`e2e/observation-attempt-fixture.ts:364-365`,
  `467-476`).
- The implementation plan reuses those scenarios against the Rust executable (lines 289 to 291)
  and makes "the accepted behavior and named regressions" the oracle,
  not an incumbent bug (lines 315 to 316).
- Native provisional choice:
  `render_policy_events` numbers a list from a given first number
  (`src/native/policy_events.rs:342-350`),
  and `run_direct_command` continues after the legacy notices it printed
  (`src/native/management.rs:227-243`).

### Verdict

Settled by evidence:
number once per invocation,
from `0`,
across every batch.
The determining sources are `SPEC.md:1306` and the end-to-end check the plan reuses.
The lifecycles not ported yet
(post-commit,
manual push,
transaction events)
must start from the count already written.

```jsonc
// wrapper standard error for one commit: a warning before forwarding, then a post-commit failure
{"schemaVersion":1,"sequence":0,"type":"finding","trigger":"pre-forward","policyId":"final-newline", ...}
{"schemaVersion":1,"sequence":1,"type":"engine-failure","code":"content-unavailable", ...}
{"schemaVersion":1,"sequence":2,"type":"commit-landed", ...}
```

## Non-UTF-8 paths in events and journals

### The choice

Git allows a file name that is any sequence of bytes.
A JSON string can only hold text.
This item holds separate decisions:
how an event names such a file to the agent or script reading it,
and how a recovery journal stores such a name.

### Evidence

- The implementation plan lines 47 to 49:
  "do not reconstruct commands as shell strings or require every path to be UTF-8".
  Lines 304 to 305 require tests with names that are not UTF-8.
- Events carry a path as a JSON string (`SPEC.md:103`,
  `1337`,
  `1435`,
  `1468`).
- The incumbent decodes Git's path listings with a decoder that throws on invalid UTF-8
  (`src/policy-engine/commit-transaction-candidates.ts:33-36`;
  `src/policy-engine/commit-transaction-index-paths.ts:11-14`,
  `57`).
  By reading,
  a commit that touches such a path fails inside the wrapper,
  so the incumbent never emits an event for one.
  This was not run.
- Journals:
  `captured.json` stores "Latin-1 decoded Git path bytes" (`SPEC.md:3138-3139`),
  and replay decodes bytes and paths as Latin-1 "so comparisons are byte-exact and paths encode back unchanged"
  (`SPEC.md:2941-2942`;
  `src/policy-engine/commit-replay-subsumption.ts:256`).
  Each byte becomes one code point,
  which loses nothing.
- Native provisional state:
  an event path is `Option<String>` and changed paths are `Vec<String>`
  (`src/native/policy_events.rs:71-72`,
  `127-128`);
  `json_string` takes text (`src/native/diagnostics.rs:94`);
  wrapper-made messages replace undecodable bytes
  (`src/native/unported.rs:119-130`;
  `src/native/invocation_config.rs:83-84`).
  No ported policy emits a repository path yet.
  The candidate branch keeps a path as bytes
  (`Candidate.path: Vec<u8>`,
  handover section `Public API for the optional-policy phase`).
- The bundled scanner already has a display form:
  bytes that are not UTF-8 become `\xNN`,
  and a backslash becomes `\\`
  (`safe_component`,
  `package/forbidden-strings/cli/src/path_name_bytes.rs:78-99`).
- Measured:
  of 11,296 tracked paths in this repository,
  0 are not UTF-8
  (`git ls-files -z` filtered through a byte-level UTF-8 pattern,
  with a planted invalid name as the positive control).

### Precedent

- [ripgrep][ripgrep] emits a path as `{"text": "..."}` when it is UTF-8
  and as `{"bytes": "<base64>"}` otherwise (`crates/printer/src/json.rs:163-202`).
- Git prints such a path in a quoted form with octal escapes for bytes larger than `0x80`
  (`core.quotePath`,
  `Documentation/config/core.adoc:152-164` in [Git][git-src]).

### Verdict

Journals are settled by evidence:
keep one code point per byte in `captured.json`.
It is lossless and it is the only form the incumbent reads.
The determining sources are `SPEC.md:3138-3139` and the implementation plan lines 228 to 229.

Events need the owner.
The plan rules out the incumbent's behavior of failing the command.

### Options: a path that is not UTF-8 in an event

The example file name is the bytes `63 61 66 e9 2e 74 78 74`,
"café.txt" in Latin-1.

#### Option A: replace undecodable bytes

```jsonc
// wrapper standard error
{"schemaVersion":1,"sequence":0,"type":"finding", ..., "path":"caf�.txt", ...}
```

For:
nothing new in the schema;
no escaping rule for a reader to learn.

Against:
different names can print the same,
and a reader cannot get back to the file.

#### Option B: escape in place

Use the scanner's display spelling,
or Git's quoted one,
inside the same `path` string.

```jsonc
// wrapper standard error
{"schemaVersion":1,"sequence":0,"type":"finding", ..., "path":"caf\\xe9.txt", ...}
```

For:
exact and reversible;
one spelling across the wrapper and the scanner it bundles.

Against:
a backslash has to be escaped too,
so an ordinary name that contains one changes its spelling;
every reader must unescape before using the path,
and one that does not gets a name that matches no file.

#### Option C: a readable path plus the exact bytes

`path` stays text with replacement characters.
An optional `pathBytes` field,
base64 as in ripgrep's format,
appears only when the name is not UTF-8.

```jsonc
// wrapper standard error
{"schemaVersion":1,"sequence":0,"type":"finding", ..., "path":"caf�.txt","pathBytes":"Y2Fm6S50eHQ=", ...}
```

For:
exact;
ordinary paths are untouched;
a reader that ignores the new field behaves as under Option A;
`SPEC.md:1284-1291` allows a new optional field under schema version 1.

Against:
a new field to specify and test;
a reader has to opt in to get the exact name.

#### Ranking

C over B,
because B changes ordinary names that contain a backslash and breaks any reader that does not unescape,
while C leaves `path` as it is today and only adds.
B over A,
because A cannot tell different files apart.

## Engine failure codes that lose their source

### The choice

When a policy cannot finish,
for example the scanner cannot load its rules or the Markdown linter cannot start,
the wrapper stops with exit status `2` and one `engine-failure` event.
The event's `code` tells an agent or script what kind of failure it was.
The incumbent's codes for this are named after plugins,
which no longer exist.
The choice is which code the native wrapper prints.

### Evidence

- The schema lists `plugin-threw` and `policy-incomplete` among the failure codes (`SPEC.md:1445-1460`).
- The incumbent emits `plugin-threw` whenever any policy's check throws,
  built-in ones included (`src/policy-engine/policy-stage.ts:162-169`),
  and `policy-incomplete` when a completed check returned an invalid finding (`233-240`).
  `doc/troubleshooting/cli-git-tag-push-eagain.md`,
  line 18,
  shows a recorded `plugin-threw` for the built-in `final-newline`.
- The spec ties the names to plugins:
  "A thrown plugin callback emits `plugin-threw` and exits `2`;
  invalid completed plugin output emits `policy-incomplete`" (`SPEC.md:4056-4057`).
  It also names `content-unavailable` for required content that cannot be determined (`SPEC.md:4041`).
- New failure codes are a backward-compatible addition,
  and consumers must ignore codes they do not know (`SPEC.md:1284-1291`).
- Native provisional choices:
  `EngineFailureCode` drops `plugin-threw`,
  `policy-incomplete`,
  and the trust codes,
  and its comment leaves this question to the engine (`src/native/diagnostics.rs:23-53`).
  The engine reports a policy whose repository fact could not be read as `content-unavailable`
  (`src/native/policy_engine.rs:254-263`).
  `doc/handover/cli-git-native-policy-engine.md`,
  section `Engine failure codes`,
  lists that as open to veto.
- The candidate branch's handover,
  section `Gaps in the scanner's embedding API`,
  notes that the scanner returns an untyped load error,
  so the wrapper cannot yet tell a missing rules file from an invalid rule.
- Measured:
  outside documentation,
  `rg --files-with-matches 'plugin-threw|policy-incomplete'` finds only the wrapper's own source,
  tests,
  and fixtures.
  No consumer outside the package matches on either code.

### Verdict

Needs the owner.
Incumbent parity points at `plugin-threw`,
a name that would describe something the tool no longer has.

### Options: the code for a shipped policy that could not complete

The example is the scanner failing to load its rules file before a commit.

#### Option A: keep `plugin-threw`

```jsonc
// wrapper standard error; the trigger and policyId fields that follow are the same under every option
{"schemaVersion":1,"sequence":0,"type":"engine-failure","code":"plugin-threw","message":"...", ...}
```

For:
the same line as today,
comparable with every recorded incident.

Against:
it names a plugin where there is none,
which misleads anyone who reads the event without the history.

#### Option B: `policy-incomplete` for every such failure

```jsonc
// wrapper standard error
{"schemaVersion":1,"sequence":0,"type":"engine-failure","code":"policy-incomplete","message":"...", ...}
```

For:
the code already exists in the schema and its name fits;
no addition.

Against:
its meaning widens from "returned an invalid result" to "could not finish";
the same failure prints a different code than the incumbent did.

#### Option C: by cause

`content-unavailable` when a candidate's bytes or a repository fact could not be read,
and `policy-incomplete` when the policy's own machinery failed
(rules file,
linter start,
an internal error).

For:
the reader learns whether the repository or the installation is at fault;
both codes exist,
and `SPEC.md:4041` already uses the first for unreadable content;
the native engine already separates the first case.

Against:
each failure needs a rule for which side it falls on,
and the scanner's untyped load error makes that harder until it is typed.

#### Option D: a new `policy-failed`

For:
a name with no history.

Against:
it adds a code while leaving unused ones in the schema,
and gains nothing over Option B.

#### Ranking

C over B,
because a reader acts differently on unreadable content than on a broken installation,
and the native engine already keeps them apart.
B over A,
because no consumer outside the package matches `plugin-threw`,
and the name would be wrong from the first native release on.
A over D,
because A at least keeps old records comparable,
while D breaks parity and grows the schema.

## Hook dispatcher and generated hooks without Node

### The choice

During a commit the wrapper points Git at a private hooks directory.
The files there,
one per hook name,
are what Git starts instead of the repository's own hooks;
each hands over to the wrapper's dispatcher,
which takes the hook lock and then runs the repository's hooks.
A pre-push file of the same kind captures what a push would send.
Today each file is a Node program whose first line names the Node executable.
The native wrapper has no Node,
so the files need a new form.
A maintainer cares because the form decides what can go wrong on an unusual install path,
and which outside program a commit depends on.
Unix and Windows are separate decisions,
because Git starts a hook differently on each.

### Evidence

- The incumbent writes `pre-commit`,
  `prepare-commit-msg`,
  and `commit-msg` with first line `#!<process.execPath>`
  (`hookEntryProgram`,
  `src/hook-dispatch/hook-dispatch-program.ts:223-238`;
  `src/hook-dispatch/hook-shim-writer.ts:107-128`),
  and a `pre-push` file of the same kind (`src/policy-engine/manual-push-hook.ts:187-203`).
- The spec chose its form on purpose:
  "The shim is a runtime-generated Node program, not a shell script" (`SPEC.md:2417-2418`).
- The spec leaves open "the shebang form for Windows, where Git for Windows parses shebangs itself,
  and for a `process.execPath` containing spaces" (`SPEC.md:2460-2462`).
  The incumbent refuses only a line break in that path (`src/policy-engine/manual-push-hook.ts:188-192`).
- The transaction silences the repository's own config-based hooks with `hook.<event>.enabled=false`
  and its hookdir hooks with `core.hooksPath` (`SPEC.md:2410-2415`).
- The implementation plan lines 425 to 426:
  the installed launcher resolves directly to the native executable,
  not to a Node process.
- The native executable is built as `cli-git-native` and gets the name `git` only by installation wiring
  (`package/git-policy/cli/Cargo.toml`,
  the `[[bin]]` entry and its comment).
- Native state:
  absent.
  No provisional choice exists.

### Precedent

Read from source on 2026-10-05.

- Linux reads at most 256 bytes of a first line (`BINPRM_BUF_SIZE`,
  `include/uapi/linux/binfmts.h:19` in [Linux][linux-src]).
  The interpreter path ends at the first space or tab,
  and whatever follows is passed as a single argument (`fs/binfmt_script.c`).
  A path with a space therefore cannot be named there.
- On Windows,
  Git looks for `hooks/<name>` and then `hooks/<name>.exe`
  (`find_hook`,
  `hook.c:26-45`;
  `STRIP_EXTENSION`,
  `config.mak.uname:541` and `735` in [Git][git-src]).
  It starts an `.exe` directly.
  For any other file it reads the first line,
  keeps only the last component of the interpreter path,
  and searches `PATH` for that name
  (`parse_interpreter`,
  `compat/mingw.c:1674-1708`;
  `2183-2186`).
  Git for Windows 2.56.0 has the same function with a longer buffer
  (`compat/mingw.c:1869` in [Git for Windows][gfw-src]).
  So the directory in the incumbent's first line is ignored on Windows,
  and `node.exe` is whichever one `PATH` yields (by reading).
- Git runs a config-based hook command through a shell (`hook.c:612-619`),
  and `hook.<event>.enabled=false` switches off every config-based hook of that event
  (comment at `hook.c:374`;
  `SPEC.md:2412-2413`).
- Git itself dispatches on the name it was started under:
  started as `git-<name>` it runs that command (`git.c:949-952`).
- Every hook manager surveyed writes a shell script and relies on Git for Windows' bundled `sh` there:
  - [lefthook][lefthook] (latest release 2.1.17):
    `#!/bin/sh`,
    then a search for the `lefthook` executable with an `.exe` suffix on Windows
    (`internal/templates/hook.tmpl`).
  - [husky][husky] 9.1.7:
    `#!/usr/bin/env sh` files in a directory that `core.hooksPath` points at (`index.js`,
    `husky`).
  - [pre-commit][pre-commit] (latest release 4.6.2):
    `#!/usr/bin/env bash`,
    then `exec` of the Python recorded at install time (`pre_commit/resources/hook-tmpl`).
  - [hk][hk] (latest release 2.5.0):
    on Git 2.54 or newer it writes no file and registers `hook.hk-<event>.command`;
    on older Git it writes `#!/bin/sh` files (`src/cli/install.rs`).

### Verdict

Both forms need the owner.

One design was considered and dropped for both systems:
registering the dispatcher as a config-based hook,
with no file at all,
as hk does.
The event-level switch that silences the repository's config-based hooks during a transaction would
silence the dispatcher too,
and the command would pass through a shell.

### Options: the hook entry on Unix

The example wrapper path is `/home/user/project/node_modules/.bin/git`
and the hook is `pre-commit` in a transaction directory `<tx>`.

#### Option A: a first line that names the wrapper

The closest form to today's.
The file holds one line;
the system starts the wrapper with `cli-git`,
the file's own path,
and the hook's arguments,
so the dispatch enters through the management namespace.

```text
#!/home/user/project/node_modules/.bin/git cli-git
```

For:
no other program is involved;
the same idea as the incumbent.

Against:
a wrapper path that holds a space,
or a line longer than the system's limit,
cannot work,
which is the gap `SPEC.md:2460-2462` already records;
the wrapper has to refuse such an install path with a clear message.

#### Option B: a shell script

```sh
#!/bin/sh
# <tx>/hooks/pre-commit
exec '/home/user/project/node_modules/.bin/git' cli-git hook-dispatch pre-commit "$@"
```

For:
any install path works when quoted correctly;
it is the form every surveyed hook manager uses.

Against:
the spec chose "not a shell script";
the path crosses a shell's quoting rules,
which needs adversarial tests for quotes and line breaks;
every commit then depends on `/bin/sh`.

#### Option C: a link named after the hook

`<tx>/hooks/pre-commit` is a symbolic link to the wrapper executable,
with no file contents.
Started under a hook's name beside a `plan.json`,
the wrapper acts as the dispatcher,
the way Git acts on the name it is started under.

```text
<tx>/hooks/pre-commit -> /home/user/project/node_modules/.bin/git
```

For:
nothing to encode,
no length limit,
no interpreter,
any install path.

Against:
it needs a file system that supports links,
with a hard link or a copy as the fallback;
the wrapper gains a second way in,
by name,
that must be recognized only inside a transaction's hooks directory.
This was not run.

#### Ranking

C over A,
because A fails on an install path with a space or an over-long line,
a gap the spec already carries,
while a link has no text to get wrong.
A over B,
because the spec rejected a shell script on purpose,
and B adds a quoting boundary and a dependency on `/bin/sh` that A does not have.

### Options: the hook entry on Windows

Nothing in this section was run on Windows.

#### Option A: an executable named after the hook

`<tx>\hooks\pre-commit.exe` is a hard link to the wrapper executable,
or a copy when the Git directory is on another volume.
Git finds it through its `.exe` lookup and starts it directly.
The wrapper dispatches on its name,
as in Option C for Unix.

For:
no interpreter and no `PATH` lookup;
the same mechanism as the Unix link form.

Against:
a copy for each hook on each commit when a hard link is not possible;
the name-based way in has to exist.

#### Option B: a first line that names the wrapper

For:
the same file as on Unix under Option A.

Against:
Git on Windows keeps only the last component of the interpreter path and searches `PATH` for it.
A wrapper installed as `git.exe` would be looked up as `git.exe`,
and inside a hook that finds Git's own executable first (by reading).
It could only work if the wrapper were also installed on `PATH` under a second,
distinct name.

#### Option C: a shell script

The Unix shell script,
started by the `sh` that Git for Windows ships.

For:
the form lefthook,
husky,
pre-commit,
and hk's fallback use on Windows.

Against:
a Windows path has to be quoted for a POSIX shell;
it depends on the bundled `sh` being present,
which was not checked for reduced Git distributions.

#### Ranking

A over C,
because A needs no shell and no quoting and shares its mechanism with the Unix link form,
while C depends on a bundled shell.
C over B,
because C is what the surveyed tools already rely on,
while B depends on the order of `PATH` inside a hook.

## Lock and journal interoperability across versions

### The choice

Both wrappers keep state inside the repository's Git directory:
locks that say which process owns the right to land a commit or run hooks,
and journals that let a later command finish or undo an interrupted commit.
While rollback is possible,
both executables exist on this machine,
and several sessions commit in this repository at once.
The choice is what happens at the moment `git` switches from one executable to the other,
in either direction,
while the other one's state is still there:
can each judge and recover the other's state,
or must the repository be quiet first.

### Evidence

- The implementation plan lines 228 to 233:
  keep the existing journal and lock formats "where practical during the first release,
  with cross-version fixture verification";
  never discard old state silently;
  "Do not intentionally switch wrapper versions during a live transaction";
  fail with a recovery diagnostic when a state cannot be read safely.
  Line 446 keeps the previous executable for rollback.
- A lock owner counts as alive only when the birth identity computed now for its PID
  equals the recorded string
  (`ownerLockHolderIsAlive`,
  `src/owner-lock/owner-lock-record.ts:165-171`;
  the hook dispatcher repeats it at `src/hook-dispatch/hook-dispatch-program.ts:107-111`).
  A lock whose owner is judged dead is retired (`156-160`).
  So a reader that computes a different string for a live owner takes its lock.
- The strings (`src/policy-engine/commit-transaction-process-identity.ts:36-75`,
  `132-159`):
  on Linux `linux:` plus field 22 of `/proc/<pid>/stat`;
  on macOS `darwin:` plus the output of `ps -o lstart= -p <pid>`;
  on Windows `win32:` plus the tick count PowerShell reports for the process start time.
- The incumbent rejects any owner record whose schema version is not 1
  (`src/owner-lock/owner-lock-record.ts:98-99`)
  and any journal record that is not schema version 2
  (`src/policy-engine/commit-transaction-journal-parse.ts:93`),
  so it fails closed on a format it does not know (by reading).
- Measured context:
  this repository's registry is rarely empty.
  `doc/handover/cli-git-native-policy-engine.md`,
  section `Leftover durable state`,
  records two live transactions,
  staging directories several days old,
  and a reservation lock at one inspection.
- Native state:
  absent.
  The development executable refuses every guarded command beside any such state
  (`pending_state`,
  `src/native/pending_state.rs:189-222`),
  which is a stand-in,
  not the final behavior.
- By reading,
  not run:
  the macOS string is local-time text with 1 s resolution,
  so a different time zone in the checking process would change it for the same process;
  the Linux tick count restarts with each boot.
  Keeping the strings keeps these properties.

### Precedent

- Linux:
  field 22 of `/proc/<pid>/stat` is "the time the process started after system boot",
  in clock ticks ([proc_pid_stat(5)][proc-stat]).
- Windows:
  `GetProcessTimes` returns the creation time as a count of 100-nanosecond units since 1601
  ([Microsoft documentation][getprocesstimes]),
  and the tick count the incumbent prints counts the same units since year 1
  ([Microsoft documentation][datetime-ticks]).
  The incumbent's string can therefore be produced by adding a constant,
  without starting PowerShell.
  That both start times are the same value is an inference,
  to be proven by a fixture on Windows.
- [psutil][psutil] identifies a process by the same pair:
  "two instances are equal if they have the same PID and creation time" (`docs/api.rst:1111-1114`).

### Verdict

Needs the owner.
Under Option A the Linux string is not a choice:
it has to match byte for byte,
or each version takes the other's live locks.

### Options: interoperability between the incumbent and the native wrapper

#### Option A: both directions, while both are in use

The native wrapper reads and writes the incumbent's formats,
including identical birth-identity strings on every platform.
Either executable can judge the other's locks and recover the other's journals.
Switching needs no quiet moment,
though a single transaction is never handed from one version to the other.

```jsonc
// <git-common-dir>/cli-git/hook.lock/owner.json, written by either version
{"schemaVersion":1,"token":"...","ownerPid":4242,"ownerBirthIdentity":"linux:8423337"}
```

For:
cutover and rollback work while other sessions keep committing;
it is the plan's stated default.

Against:
every record needs a fixture in both directions;
the macOS string has to reproduce `ps` output,
by running `ps` as the incumbent does or by formatting the same text;
the identity scheme's weaknesses stay.

#### Option B: the native wrapper reads the old formats and writes new ones

The native wrapper recovers and respects everything the incumbent left,
and writes its own records under a new schema version with an identity that also records the boot.
The incumbent cannot read those and fails closed.

```jsonc
// <git-common-dir>/cli-git/hook.lock/owner.json, written by the native wrapper
{"schemaVersion":2,"token":"...","ownerPid":4242,"ownerBirth":{"platform":"linux","bootId":"...","startTicks":8423337}}
```

For:
cutover needs no quiet moment;
the identity can be improved now.

Against:
rollback works only once no native state is left,
so it needs a check and a way to drain;
until then the incumbent stops every guarded command with a recovery error.

#### Option C: separate state, a quiet repository both ways

Each version keeps its own directories and formats,
and each refuses to run while the other's state exists.

For:
no reader for the other version's records and no cross-version fixture is needed.

Against:
both cutover and rollback need every session stopped,
which this repository rarely is;
the incumbent would have to learn to look for the native state,
or the two could land commits at once without excluding each other.

#### Ranking

A over B,
because sessions here commit concurrently,
so only a switch that needs no quiet moment in either direction keeps rollback usable in practice.
B over C,
because B at least lets the native wrapper finish whatever the incumbent left,
while C stops at both switches.

## Legacy single-journal directory

### The choice

Before 2026-09-25 the wrapper kept one journal directory,
`cli-git-transaction`,
per Git directory.
If a commit was interrupted back then and never recovered,
that directory may still exist in some clone.
The choice is whether the native wrapper can recover it,
or stops and says how to recover it with the previous executable.

### Evidence

- The spec:
  "The legacy single-journal `<git-dir>/cli-git-transaction` directory is recovered read-only
  until no retained legacy directory can exist" (`SPEC.md:2145-2146`).
- The incumbent's recovery for it is a module of its own
  (`src/policy-engine/commit-transaction-recovery-journaled.ts`).
- The per-transaction registry replaced the single journal in commit `9f53f7857`,
  dated 2026-09-25
  (`git log -S'cli-git-transactions'` on `src/policy-engine/commit-transaction-registry.ts`).
- The implementation plan lines 230 to 233 forbid discarding old state silently
  and allow failing with an actionable recovery diagnostic.
- Measured:
  `find .git -maxdepth 3 -name 'cli-git-transaction*'` in this checkout lists the plural registry
  in the main Git directory and in linked worktrees,
  and no `cli-git-transaction`.
  Other clones and other machines were not inspected.
- Native provisional state:
  the development executable treats the directory's presence as a reason to refuse
  (`pending_transactions`,
  `src/native/pending_state.rs:143-156`).

### Verdict

Needs the owner,
because whether another clone can still hold such a directory cannot be measured from here.
Ignoring the directory is not an option:
the plan forbids it.

### Options: a legacy single-journal directory

#### Option A: port its recovery

For:
any clone recovers by itself,
with no need for the previous executable.

Against:
a second recovery path to port and test for a format with no known instance.

#### Option B: detect it and stop with instructions

```text
cli-git: <git-dir>/cli-git-transaction holds a commit journal in a format this executable does not
recover. Nothing was changed. Run any git command once with the previous cli-git executable to
recover it, then run this command again.
```

For:
fails closed;
nothing to port;
it follows the plan's rule for state that cannot be read safely.

Against:
recovery depends on the previous executable,
which is kept only until the old implementation is retired.

#### Ranking

B over A,
because no instance was found here and the format has been superseded since 2026-09-25,
while the previous executable stays available through the rollback period for the rare clone that has one.

## Test phase markers

### The choice

The crash tests need to stop a commit at exact inner steps,
such as after the branch moved and before the index was updated.
The incumbent does this with an environment variable,
`CLI_GIT_TEST_ONLY_PHASE_SIGNAL`,
that makes the wrapper kill itself or pause at a named step.
The choice is whether the executable people install contains that mechanism,
or only a separate build made for tests.

### Evidence

- The incumbent's production code reads the variable
  (`src/policy-engine/commit-transaction-test-phase.ts:37`,
  `47-55`),
  and the spec documents it:
  "Only this explicitly test-named variable arms a marker;
  a malformed value fails the invocation" (`SPEC.md:3954-3972`).
- The implementation plan reuses the end-to-end scenarios (lines 289 to 291),
  which reach the landing steps only through these markers (ledger,
  "Test phase markers"),
  and requires container tests to "exercise the installed Rust binaries" (lines 326 to 331).
- Native state:
  absent.

### Precedent

- Git reads `GIT_TEST_*` variables in its ordinary executable,
  for example `GIT_TEST_SPLIT_INDEX` (`read-cache.c:3342`;
  `t/README` in [Git][git-src]).
- [fail-rs][fail-rs],
  the fail-point library from TiKV,
  compiles its points out unless a Cargo feature is on:
  "Fail points generation by this macro is disabled by default,
  and can be enabled where relevant with the `failpoints` Cargo feature."

### Verdict

Needs the owner.
The incumbent ships the markers,
but a single JavaScript artifact left it no other way;
Rust can compile the markers out behind a Cargo feature,
so parity does not show a preference.

### Options: where the test phase markers live

#### Option A: in the release executable

For:
the crash tests run against the exact executable that ships;
incumbent parity;
Git does the same.

Against:
an environment that carries the variable makes an installed wrapper kill itself or wait at that step.
Whoever controls the wrapper's environment already controls `PATH`,
so the variable opens no path that control of `PATH` does not already open,
but it is test-only behavior in a tool that guards commits.

#### Option B: only in a build made for tests

A Cargo feature compiles the markers in;
the release build has none.

For:
no test hook in what people run.

Against:
the executable under crash test is not the one that ships,
so the inner steps are never exercised on the release build,
and a second build has to be kept in step with it.

#### Ranking

A over B,
because the landing steps can be reached only through the markers,
so under B the riskiest part of the wrapper would go untested on the artifact that ships.

## Unsupported Git detection

### The choice

The native wrapper is written for one Git release,
2.56.0,
and has no fallbacks for older ones.
This item holds separate decisions.

When,
and by what test,
the wrapper notices that the real Git behind it is not suitable.

Which releases count as suitable:
only the one it was built for,
or that one and newer.

A person feels this as the difference between one clear message and a silent loss of protection,
and as whether a system update of Git stops every commit until a new wrapper is built.

### Evidence

- The implementation plan lines 22 to 25:
  support "the latest stable Git release only",
  record the exact release in build and test evidence,
  and never query the network at run time.
  Lines 235 to 237:
  report missing required behavior "as an unsupported-environment failure".
- The incumbent has no version gate and detects each feature "by what Git does rather than by its version string"
  (`SPEC.md:3553-3563`).
  Some features fail silently on an older Git:
  it ignores `core.lockfilePid` and writes no PID file,
  and it runs no config-based hooks (`SPEC.md:3590-3605`).
- The native option tables are compiled from Git 2.56.0 (`src/native/lib.rs`,
  module comments).
  An option the tables do not know makes a region "Git refuses":
  the command is forwarded for Git to judge,
  except `commit`,
  `push`,
  and `stash`,
  which the wrapper refuses
  (`doc/handover/cli-git-native-policy-engine.md`,
  section `Hazard 6, a region Git refuses`).
- The native read-only path starts at most one Git query before forwarding
  (same handover,
  section `Hazard 5, the read-only fast path`).
- Native state:
  no check exists.
- Measured:
  `/usr/bin/git --version` on this host prints `git version 2.55.0`.
  The newest upstream tag is `v2.56.0`,
  and the newest Git for Windows release is `v2.56.0.windows.1`
  (GitHub tag and release listings on 2026-10-05).
  Under any run-time check,
  this host needs a newer Git before cutover.

### Precedent

- [hk][hk] compares versions:
  it runs `git --version` once,
  parses the third word,
  and checks for at least 2.54 before using config-based hooks
  (`git_version` and `git_at_least`,
  `src/git_util.rs`).

### Verdict

Both decisions need the owner.

### Options: when and how the check runs

#### Option A: compare the version on every command that reaches real Git

For:
one message,
early,
for every command.

Against:
one more Git process for every command,
read-only ones included,
which the engine otherwise keeps to at most one query;
commands that behave the same on any Git are stopped too.

#### Option B: compare the version once, before work that depends on newer Git

The wrapper asks for the version only when it is about to start a commit transaction,
dispatch hooks,
or rely on lock PID files.
Read-only and plainly forwarded commands are never checked.

For:
no cost on the common path;
the commands that could lose protection silently are all covered.

Against:
`git status` works on an unsupported Git and a commit does not,
which can surprise.

#### Option C: probe behavior at the point of use

As the incumbent does,
with a failure where it had a fallback.

For:
no version parsing;
a vendor build that carries the needed feature under another version number still works.

Against:
the features that fail silently cannot be seen by a probe at the point of use,
so each needs its own advance probe;
a failure can surface in the middle of a transaction.

#### Option D: check at installation and in `git cli-git check` only

For:
no run-time cost at all.

Against:
a Git that changes after installation is never noticed.

#### Ranking

B over A,
because A charges every command a Git process for a fact that matters to few of them.
A over C,
because required behavior fails silently on older Git,
which a probe at the point of use cannot see.
C over D,
because D stops looking after installation.

### Options: which releases count as supported

#### Option A: only the release the wrapper was built for

Any other minor release is unsupported.
The rule still needs to say how patch releases and vendor suffixes such as `2.56.0.windows.1` count.

For:
the compiled option tables are only ever applied to the Git they were checked against.

Against:
every system update of Git stops every guarded command until a new wrapper is built and installed.

#### Option B: that release or newer

For:
a newer Git keeps working;
an option newer than the tables is handled by the rule the engine already has for a region Git refuses.

Against:
under that rule a `commit`,
`push`,
or `stash` that uses an option newer than the tables is refused until the tables are updated;
behavior on a newer Git is not covered by the recorded test evidence.

#### Ranking

B over A,
because distributions move Git on their own schedule,
as this host's own Git shows,
and under A each such move stops work until someone rebuilds the wrapper.

## Post-command output

### The choice

The incumbent adds text of its own after certain Git commands.
After `git --version` it prints a line that lists what the wrapper enforces.
After a plain `git status` it prints a note about the staging and branching rules,
having switched off Git's own hints,
which would suggest commands the wrapper rejects.
These are separate decisions.
Each matters to a person as a visible sign that the wrapper is active or of what it allows,
and to a script as extra output where it expects Git's alone.
Each also decides whether the wrapper can simply become Git for that command,
or has to stay alive behind it.

### Evidence

- `printPostCommandOutput` (`src/post-command-output.ts:38-103`):
  the version line at `88-94`,
  the status note at `95-102`,
  suppressed for machine-readable output and when the caller set `advice.statusHints` (`72-87`).
  Both go to standard output.
- No `SPEC.md` section states either (ledger,
  "Post-command output").
- The version line's list is maintained by hand and omits `final-newline` (ledger,
  "Incumbent defects and stale comments").
- Measured on this host.
  The wrapper's line is one line of output,
  shown here on two:

  ```text
  $ git --version
  git version 2.55.0
  cli-git wrapper (require-root, linked-worktree-only, branch-worktree-only, add-explicit,
  atomic-push, commit-only, status-hints-off, auto-push)
  ```

- Native state:
  neither is printed,
  "forwarding replaces the process, so nothing runs after Git"
  (`doc/handover/cli-git-native-policy-engine.md`,
  section `Post-command output`).
  The status-hints transform is applied,
  so plain `git status` shows neither Git's hints nor the note.
  The native entry already writes warning events to standard error before it becomes Git
  (`run_process`,
  `src/native/entry.rs:216-227`).

### Precedent

- [hub][hub],
  GitHub's former Git wrapper,
  does what the incumbent does:
  `hub --version` runs `git version` and then prints `hub version <n>` (`commands/version.go`).
- [hk][hk] reads the third word of `git --version` output (`src/git_util.rs`),
  so a line added after Git's does not disturb it.

### Verdict

Both need the owner.

### Options: the wrapper's line after `git --version`

#### Option A: keep it on standard output, after Git's line

For:
incumbent parity;
it answers "is the wrapper active" where people already look.

Against:
standard output differs from Git's for any script that reads all of it;
the wrapper must wait for Git instead of becoming it;
the list has to be generated to stay true.

#### Option B: print it on standard error, before becoming Git

```text
$ git --version
cli-git wrapper (require-root, ...)
git version 2.56.0
```

For:
standard output is exactly Git's;
a person still sees the line;
the wrapper does not outlive Git.

Against:
the order and the stream change from today's.

#### Option C: drop it

`git cli-git --help` is the way to see that the wrapper is active.

For:
`git --version` is exactly Git's.

Against:
the check people use today for an active wrapper is gone.

#### Ranking

B over A,
because standard output stays byte-identical to Git's and the wrapper need not stay alive for it.
A over C,
because A keeps a visible answer in the place people look,
which C removes.

### Options: the wrapper's note after `git status`

#### Option A: keep it on standard output, after Git's output

For:
incumbent parity;
the rules are in front of every reader of `git status`.

Against:
the wrapper waits behind every plain `git status`,
so signals and the exit status pass through it instead of being Git's own;
agents read the same sentence on every status call.

#### Option B: print it on standard error, before becoming Git

For:
the note still reaches every reader;
`git status` is handed to Git whole.

Against:
it is printed before the status instead of after it,
and a pager does not capture it.

#### Option C: drop it

The rules stay in `git cli-git --help` and in each rejection message.

For:
`git status` output is Git's alone,
minus the hints the transform removes.

Against:
with Git's hints off and no note,
nothing tells a newcomer the rules until a command is rejected.

#### Ranking

B over A,
because the note is kept while the most frequent command stays a plain hand-over to Git.
A over C,
because C leaves `git status` with neither Git's hints nor the wrapper's rules.

## Tool-cache allowlist

### The choice

The policy that keeps destructive commands such as `git reset --hard` out of a main worktree
exempts repositories inside a tool's own cache,
so that a tool which runs such commands in clones it owns keeps working.
Today the list holds uv's cache and is compiled into the wrapper.
The open points are whether the list stays compiled in or becomes configurable,
and whether it learns where the cache is on other systems.

### Evidence

- `resolveUvCacheDir` derives `UV_CACHE_DIR`,
  then `XDG_CACHE_HOME/uv`,
  then `<home>/.cache/uv`,
  with no platform branch (`src/allowed-worktree-dirs.ts:60-88`).
  Its comment:
  "Baked into the binary because the set is a property of the machine's tooling, not of any repository" (`95-99`).
- The native code repeats the same derivation and the same reasoning
  (`uv_cache_dir` and `default_allowed_worktree_dirs`,
  `src/native/effective_target.rs:57-105`).
- By reading:
  configuration is loaded from the repository a command runs in
  (`load_identity_config`,
  `src/native/invocation_config.rs:34-47`).
  The repositories to exempt are a tool's clones of other projects,
  which carry no `cli-git.config.jsonc`,
  so a key in this repository's JSONC could not reach them.
- A machine-wide settings file is excluded:
  "Do not introduce a replacement trust registry or global policy layer" (rewrite scope line 124).
- The implementation plan lines 300 to 303 keep Linux,
  macOS,
  and Windows coverage.

### Precedent

- [uv][uv] documents its cache location as `$XDG_CACHE_HOME/uv` or `$HOME/.cache/uv` on Unix
  and `%LOCALAPPDATA%\uv\cache` on Windows,
  after `--cache-dir`,
  `UV_CACHE_DIR`,
  and a `tool.uv.cache-dir` setting (`docs/concepts/cache.md`,
  section "Cache directory").
  The compiled derivation therefore matches uv on Linux and macOS and misses it on Windows.

### Verdict

Settled by evidence:
the list stays compiled in,
and the derivation gains uv's documented Windows location.
The determining sources are incumbent parity for where the list lives,
the mechanism that rules out repository JSONC,
the rewrite scope line 124 that rules out a machine-wide file,
and uv's documentation for the Windows location.
The Windows addition corrects a case where the incumbent's exemption never matched;
it changes nothing on Linux or macOS.
An environment variable that adds directories would be a new feature and is not proposed.

## macOS and Windows coverage

### The choice

Which tests of the native wrapper run on macOS and Windows,
and where.
Today the only jobs on those systems test the trust subsystem,
which is being removed,
so keeping "existing coverage" would keep nothing.
The wrapper's platform-specific code has never been compiled there.
The answer decides how soon a break on those systems is noticed,
and what it costs in runner time or in manual work.

### Evidence

- The implementation plan lines 300 to 303 keep the existing Linux,
  macOS,
  and Windows consumer coverage,
  and line 338 says native macOS and Windows jobs "remain necessary for their filesystem and process contracts".
- The only wrapper jobs on those systems run trust and filesystem-identity tests
  (`.github/workflows/cli-git-trust.yml:189-232`).
- `doc/handover/cli-git-rust-implementation.md`,
  section `Native wrapper foundation`:
  "Windows and macOS code paths were never compiled."
- The code that differs by platform is confined to named areas:
  process identity,
  lock-holder evidence,
  shadow links,
  hook entries,
  and real-Git lookup (ledger,
  "Platform-specific behavior").
- The wrapper's gate runs in a Linux container on an audited Git 2.56.0 image
  (`doc/handover/cli-git-native-policy-engine.md`,
  section `Gate results`).
  A hosted macOS or Windows runner has whatever Git its image carries.
- Other workflows here already use hosted macOS and Windows runners
  (`.github/workflows/cargo-publish.yml`,
  `.github/workflows/fs-id.yml`,
  `.github/workflows/readonly-semantic-bridge.yml`).
- The scanner's Windows verification ran in a local virtual machine
  (`doc/handover/cli-git-rust-implementation.md`,
  section `Scanner Windows-native verification`).
- Measured:
  `gh repo view` reports this repository as public.

### Precedent

- GitHub:
  "GitHub Actions usage is free for self-hosted runners and for public repositories
  that use standard GitHub-hosted runners" ([billing documentation][actions-billing]).

### Verdict

Needs the owner.

### Options: native tests on macOS and Windows

#### Option A: the whole native test suite on hosted runners, on every wrapper change

For:
the widest coverage,
at no runner cost for a public repository.

Against:
tests that compare with real Git need Git 2.56.0 installed on each runner;
every change waits for the slowest system.

#### Option B: the platform-specific suites on hosted runners, the whole suite on Linux

Process identity,
lock-holder evidence,
shadow links,
hook entries,
real-Git lookup,
and one ordinary commit run on each system.

For:
it covers what actually differs;
the suites that do not depend on the platform run once,
on Linux.

Against:
someone has to keep the list of platform suites complete;
a platform difference in code believed neutral goes unseen.

#### Option C: local virtual machines and hosts, before a release

For:
no CI wiring;
the exact Git version is under control.

Against:
it runs only when someone starts it;
the macOS host is limited
(repository rule HRM in `CLAUDE.md`:
16 GiB of memory and a fragile internal disk).

#### Option D: compile only

CI builds the wrapper for each system and runs nothing.

For:
a build step and nothing else;
it ends "never compiled".

Against:
no behavior is ever checked.

#### Ranking

B over A,
because the platform-specific code sits in named areas,
so running those covers the differences without making every change wait on every system.
A over C,
because hosted runners are free here and run on every change,
while C depends on someone remembering.
C over D,
because D never runs the code.

## TypeScript utilities beside the wrapper

### The choice

Tools written in TypeScript sit beside the wrapper and share code or a package with it.
This item holds separate decisions.

The dependent-version bump:
when a package's version is raised,
packages that depend on it get a patch bump.
The wrapper does this at commit time as a policy,
and a standalone task does it in the release workflow.
Both use the same TypeScript planning code today.
Once the policy is in Rust,
either the task keeps the TypeScript code,
so the plan exists in both languages,
or the task calls the native wrapper.

The hk cleanup:
a one-time maintenance task that removes leftover Git configuration from a retired hook manager.
Its source lives inside the wrapper package,
whose TypeScript is deleted at the end of the rewrite.

### Evidence

- The implementation plan line 125 puts dependent-version propagation in the first native registry,
  lines 33 to 35 forbid a permanent TypeScript bridge,
  and lines 296 to 297 delete retired implementations only after their consumers have moved.
- The standalone task `//package/git-policy/repository:bump:dependents` runs `node src/bump-dependents.ts`
  (`package/git-policy/repository/mise.toml:42-44`).
  The root `changeset:version` task calls it (`mise.toml:1193-1197`),
  and the release workflow calls that (`.github/workflows/npm-release.yml:127`).
  That workflow installs Node and pnpm only.
- The task compares manifests in the worktree with a base revision,
  `HEAD` by default (`package/git-policy/repository/src/bump-dependents.ts:41-49`).
- The policy already has a second way to apply the same bumps:
  its `direct-fix` trigger "lets `git cli-git fix` apply the ripple `git cli-git check` reports
  (owner decision 2026-09-15)"
  (`package/git-policy/repository/src/dependent-version-bump-policy.ts:252-257`).
  Whether `git cli-git fix` and the task produce the same result on the release workflow's input
  was not compared.
- Measured:
  the planning code and its entry points are 2,034 lines in `package/git-policy/repository/src`
  (`wc --lines` over the non-test files that the policy and the task use).
- The hk cleanup is `src/maintenance/hk-config-cleanup.ts` with a command entry,
  run by the root task `cleanup:hk-git-config` (`mise.toml:655-661`)
  and described in `doc/runbook/remove-retired-hk-git-config.md`.
  The ledger keeps it "as a repository task outside the wrapper executable".
- Measured:
  `git config --show-scope --get-regexp '^hook\.hk-'` on this host exits `1` with no output,
  so nothing is left to clean here.
  Other machines were not inspected.

### Verdict

Both need the owner.
The dependent-version question belongs to the optional-policies phase,
because the port of that policy decides what stays shared.
The hk question belongs to cutover.

### Options: the dependent-version bump after the policy is in Rust

In both options the TypeScript task stays until the native executable is available to the release workflow;
the options differ in the end state.

#### Option A: keep the TypeScript task and its planning code

The plan exists in Rust for commits and in TypeScript for releases,
held together by shared fixtures both must pass.

For:
the release workflow stays on Node alone.

Against:
a bump planned at commit time and one planned at release time come from different code and can disagree;
every change to the rules is made twice.

#### Option B: one implementation, in the native wrapper

`changeset:version` runs `git cli-git fix --all --policy mono/dependent-version-bump`,
and the TypeScript planning code is deleted at cutover.

For:
one plan;
it uses the path the owner decision of 2026-09-15 already opened.

Against:
the release workflow has to build or fetch the native executable;
the equivalence of both entry points has to be proven first.

#### Ranking

B over A,
because the same rule computed by separate implementations is the kind of drift the plan's ban on a
permanent bridge is meant to prevent,
and the cost of B is a build step,
paid once.

### Options: the hk cleanup utility

#### Option A: keep it in TypeScript and move it

At cutover the files move to a TypeScript package that survives,
for example beside the Git resolver they already import.

For:
no behavior change;
the runbook keeps working.

Against:
a one-time migration tool is carried along indefinitely.

#### Option B: retire the task and its runbook at cutover

For:
less code to keep.

Against:
a machine that still has the old keys loses the supported way to remove them.

#### Option C: port it into the native wrapper as a command

For:
no TypeScript left beside the wrapper.

Against:
a permanent native command for a one-time migration,
in a release that otherwise adds no commands.

#### Ranking

A over B,
because whether another machine still has the keys cannot be measured from here,
and keeping the task costs only a move.
B over C,
because C adds lasting code for a job that is nearly done.

## Verbose diagnostics switch

### The choice

How a person turns on the wrapper's detailed diagnostics when a command behaves unexpectedly.
Today it is the environment variable `MONOCHROMATIC_VERBOSE=true`,
or any `--verbose` among the arguments.
The native wrapper has no diagnostics yet and no switch.
The switch matters beyond its name:
the wrapper's standard error also carries the JSONL events agents parse,
and a variable is inherited by every program a hook starts.

### Evidence

- The incumbent's logger turns verbose on for `MONOCHROMATIC_VERBOSE=true` or an argument `--verbose`
  (`package/logger/core/src/sink/console.ts:55-77`).
- The ledger lists the argument form as a defect:
  by reading,
  `git commit --verbose` also turns on wrapper debug lines.
  Every wrapper-only control has a `--cli-git-` or `--no-enforce-` spelling (ledger,
  "Wrapper controls and escape hatches").
- The spec:
  "Debug logs must not corrupt the selected JSONL event stream" (`SPEC.md:1586`).
- Sibling Rust tools here use `tracing` with `RUST_LOG`
  (`package/forbidden-strings/cli/Cargo.toml:82-87`;
  `package/nested-wayland-session/cli/Cargo.toml`).
  The scanner the wrapper links brings `tracing` with it.
- Measured:
  the native wrapper's non-test sources hold no `eprintln!`,
  `tracing::`,
  or `log::` call.

### Precedent

- Git's own switch is `GIT_TRACE`:
  `1`,
  `2`,
  or `true` write to standard error,
  and an absolute path writes to that file (`Documentation/git.adoc:794-808` in [Git][git-src]).
- lefthook and husky each use one variable of their own
  (`LEFTHOOK_VERBOSE`,
  `HUSKY=2`;
  the hook templates cited under "Hook dispatcher and generated hooks without Node").

### Verdict

The argument form is settled by evidence:
it is not ported.
The determining source is the ledger's defect entry with the spelling rule for wrapper controls.

The environment switch needs the owner.

### Options: the environment switch for verbose diagnostics

#### Option A: `MONOCHROMATIC_VERBOSE=true`

```sh
MONOCHROMATIC_VERBOSE=true git commit --message=message -- a.txt
```

For:
the switch people and documents already use;
one variable for every Monochromatic tool.

Against:
it also turns on every other Monochromatic tool that a hook starts;
it writes to standard error beside the events.

#### Option B: `RUST_LOG`

```sh
RUST_LOG=git_policy_cli=debug git commit --message=message -- a.txt
```

For:
the convention of the sibling Rust tools;
filtering by module;
the linked scanner's own events appear through the same switch.

Against:
every Rust program a hook starts inherits it;
the sibling tools log at `info` by default,
which the wrapper cannot do without mixing lines into its event stream.

#### Option C: a variable of the wrapper's own, with Git's grammar

```sh
CLI_GIT_TRACE=/tmp/cli-git.trace git commit --message=message -- a.txt
```

For:
it reaches this tool only;
a file destination keeps standard error clean for the events;
Git users know the grammar.

Against:
a third convention in this repository,
with its own reader to write.

#### Ranking

A over C,
because the switch already in use keeps working and no new name has to be learned.
C over B,
because `RUST_LOG` leaks into every Rust program a hook starts and has no file destination.

## Performance acceptance

### The choice

The rewrite was asked for because the wrapper felt slow.
Before the native wrapper replaces the incumbent,
a measurement decides whether it is fast enough.
This item holds separate decisions.

Which numbers that measurement is held to.

Whether the slow commit the owner reported is part of the measured workload.

### Evidence

- The rewrite scope lines 73 to 96:
  the owner wants faster execution throughout;
  "No numeric acceptance budget is settled for the new design";
  the incumbent's budgets are "synthetic-fixture overhead budgets".
- The reported observation:
  staging and committing one Markdown file took 8.9 seconds,
  which "establishes an end-to-end workload to investigate, not its cause" (rewrite scope lines 80 to 89).
- The incumbent's budgets:
  a ceiling of 2,000 ms,
  described in the source as user-required (`perf/lifecycle-latency-contracts.ts:210-212`;
  `SPEC.md:4038`),
  and one budget per scenario,
  each "twice measured maximum rounded up to next 25 milliseconds" (`215-231`).
- The spec:
  "A performance budget is accepted only after at least one measured baseline on each enforced operating system"
  (`SPEC.md:4251`).
- The implementation plan lines 318 to 322:
  measure the release artifact at the real command interface,
  with positive controls,
  local processing apart from push latency,
  and without restarting the canceled incumbent profiling.
- Recorded incumbent baseline (rewrite scope lines 270 to 280):
  wrapper-added medians of 409.953 and 423.590 ms for a one-file commit with local push,
  and medians of 2,336.066 and 582.359 ms for the same 256-path commit in two unchanged runs.
  The 256-path medians differ by a factor of about 4 between unchanged runs.
- Native state:
  no measurement exists.

### Verdict

Both need the owner.
The workload decision is not in the ledger's list;
it comes from the rewrite scope lines 80 to 89.

### Options: the numbers that gate the native wrapper

#### Option A: the incumbent's budgets as ceilings

Each retained scenario stays under its existing budget and under 2,000 ms.

For:
the numbers exist and the harness enforces them already.

Against:
each budget is twice the incumbent's measured maximum,
so a native wrapper slower than the incumbent would pass.

#### Option B: faster than the incumbent

On the same fixture and the same container bounds,
the native wrapper-added median must be lower than the recorded incumbent median
by more than the spread between unchanged runs,
for each scenario.

For:
it tests the reason for the rewrite.

Against:
the spread has to be measured first,
and where it is as large as in the 256-path scenario the test cannot decide;
it relies on the recorded incumbent runs,
since the plan forbids restarting that campaign.

#### Option C: fresh budgets from the native wrapper's own first measurement

The incumbent's method applied to the native wrapper,
with the 2,000 ms ceiling kept.

For:
budgets close to the native wrapper's own times,
which guard against regressions after cutover.

Against:
the thing being judged sets its own bar,
so it says nothing about whether the rewrite helped.

#### Option D: report only

Numbers are published and nothing is gated at the first release.

For:
no risk of holding the cutover on a noisy number.

Against:
the goal of the rewrite is never checked.

#### Ranking

B over A,
because A accepts a result slower than today's.
A over C,
because A binds the native wrapper to numbers that exist independently of it.
C over D,
because D gates nothing.

### Options: the reported slow commit as a workload

#### Option A: the synthetic scenarios only

For:
the harness exists and is reproducible in a bounded container.

Against:
it uses a synthetic scanner and no Markdown policy,
so by the rewrite scope's own account it "cannot establish or dismiss the cause of the reported 8.9 seconds".

#### Option B: add a repository-shaped scenario and gate on it

One Markdown file committed with this repository's real policies and rules,
measured under the gate chosen in "Options: the numbers that gate the native wrapper".

For:
it measures the experience that prompted the rewrite.

Against:
a new scenario to build,
with private rules that cannot be baked into a shared image as they are.

#### Option C: measure it once, report only

For:
the owner sees the number without a new gate.

Against:
a later regression in that workload is not caught.

#### Ranking

B over C,
because the reported commit is the one concrete symptom on record,
and only a gate keeps it from returning.
C over A,
because A never looks at it.

## Markdown policy without the linter

### The choice

The Markdown policy will call the native linter that is installed together with the wrapper.
The choice is what a commit of a Markdown file does when that linter is missing or cannot start:
stop,
warn and continue,
or skip the policy.

### Evidence

- The implementation plan lines 181 to 189:
  the policy selects the coordinated installation's own linter,
  and repository JSONC cannot choose the executable.
- The incumbent treats a command that cannot start as a failure:
  "markdown-lint could not be started." is thrown
  (`src/optional/markdown-lint/rewrite-candidates.ts:335-336`),
  and a thrown check becomes an engine failure with exit status `2`,
  whatever the policy's severity (`src/policy-engine/policy-stage.ts:162-176`).
- The stream and exit contract gives engine failures exit status `2` (`SPEC.md:1588-1598`).
- Native state:
  absent.

### Verdict

Settled by evidence:
an engine failure with exit status `2`,
and the command is not forwarded.
The determining source is incumbent parity.
A missing linter is a broken installation,
not a property of the commit,
and skipping would stop the normalization without anyone noticing.
The event's `code` follows the answer under "Engine failure codes that lose their source".

## Choices found outside the ledger's list

The native handovers list further choices as open to veto.
Most are implementation details their delegate owns,
recorded in `doc/handover/cli-git-native-foundation.md`
(sections `Configuration choices open to veto`,
`Forward-target marker, open to veto`,
and `Diagnostics choices open to veto`)
and in `doc/handover/cli-git-native-policy-engine.md`
(sections `Choices open to veto` and `Forwarded with a known omission`).
The ones in this section change what a person using `git` observes and were not in the ledger's list.

### The manual-push gate without a configuration file

The incumbent runs its pre-push checks only when a configuration was loaded (`src/bin.ts:314`).
With a configuration that lists nothing,
it still runs them,
because the built-in `final-newline` declares the `manual-push` trigger
(`doc/troubleshooting/cli-git-tag-push-eagain.md`,
line 18,
shows such a run).
The native engine applies the gate whenever an enabled policy declares that trigger,
with or without a file
(`doc/handover/cli-git-native-policy-engine.md`,
section `The manual-push gate applies without a configuration file`).

Settled by evidence:
the gate applies.
The determining source is the decision of 2026-10-05 that an empty file and no file behave identically,
together with what the incumbent does for an empty configuration.
A consequence to know:
in a repository without a configuration file,
each real push now gains the gate's dry-run negotiation with the remote,
for a policy that only warns by default.

### Recovery before read-only commands

The incumbent recovers interrupted transactions at startup,
before read-only commands too (`src/bin.ts:199-204`;
`SPEC.md:3499-3503`).
The native development executable forwards a read-only command without looking
(`doc/handover/cli-git-native-policy-engine.md`,
section `Recovery on the read-only path`).

Settled by evidence:
once recovery is ported,
it runs before read-only commands as well.
The determining source is incumbent parity with `SPEC.md:3499-3503`,
and the ledger's own recovery test,
which uses `git status` as the command that triggers recovery.
Without it,
`git status` after a crash could show an index that recovery was about to repair.

This overrides a choice the engine delegate made on purpose and recorded as open to veto:
that a read-only command changes nothing a recovery would protect,
so the read-only path need not look.

### A wrapper failure before Git runs

When the wrapper itself cannot proceed,
for example it finds no real Git,
the incumbent prints the message and exits `1` (`src/bin.ts:457-460`),
the status the exit contract reserves for error findings.
The contract gives usage,
configuration,
transaction,
and engine failures status `2` when real Git does not run (`SPEC.md:1588-1598`).
The native executable exits `2` for a missing real Git,
a malformed management invocation,
an invalid configuration,
and a repository that cannot be inspected
(`doc/handover/cli-git-native-foundation.md`,
section `Exit status 2 for wrapper failures`,
which lists it as open to veto).

Settled by evidence:
exit status `2`.
The determining source is `SPEC.md:1588-1598` with the implementation plan lines 315 to 316,
which make the accepted behavior the oracle where the incumbent differs from it.
A caller that tested for `1` in these cases sees `2`.

### An alias for `commit`

#### The choice

Whether a Git alias that expands to `commit` is guarded like `commit` itself.
An alias can be defined for one command with `-c`,
so this is also a way around the commit rules that needs no escape flag.

#### Evidence

- The incumbent applies the commit transaction only when the command word is literally `commit`
  (`src/policy-engine/commit-transaction.ts:109-110`),
  and the post-commit lifecycle likewise (`src/bin.ts:380-383`).
  It resolves aliases only for index-writer coordination and worktree copies (ledger,
  "Alias resolution and built-in command table").
- No `SPEC.md` section says whether an aliased commit is guarded.
- Measured on 2026-10-05 with the installed incumbent,
  in a disposable repository without a configuration file,
  on Git 2.55.0:
  `git commit -a -m direct` was rejected with `commit-only/all-flag` and exit status `1`,
  and `git -c alias.c=commit c -a -m aliased` created the commit with exit status `0`.
- The native executable reproduces this from the main worktree
  (`doc/handover/cli-git-native-policy-engine.md`,
  section `An aliased commit`),
  and that section notes that refusing every unknown command word would close the path
  at the cost of refusing every alias and external command.
- Repository rule CLG in `CLAUDE.md` tells agents not to bypass the wrapper's guards.

#### Verdict

Needs the owner.
Parity keeps the gap;
the plan's rule against preserving an incumbent bug applies only if this is one,
and no document says.

#### Option A: keep it

An alias that expands to `commit` runs as a plain Git commit:
no transaction,
no commit-time policies,
no automatic push.

For:
incumbent parity;
nothing to build.

Against:
the commit rules can be sidestepped by one `-c alias...` argument,
without any of the flags meant for that.

#### Option B: resolve the alias first

The wrapper resolves the command word through Git's alias rules,
which it must port anyway for worktree copies,
and treats an alias that expands to `commit` as `commit`.
A shell alias,
one starting with `!`,
stays unresolved,
as today.

For:
the commit rules hold for every ordinary spelling of a commit.

Against:
one more Git query on commands whose word Git does not build in;
a shell alias that calls `git commit` by an absolute path to real Git still gets around it,
as does calling real Git directly.

#### Ranking

B over A,
because the alias path defeats the rules with a single argument that looks harmless,
and the resolution it needs is already on the list of things to port.

## Disagreements between the ledger, the plan, and the native code

- The ledger's native citations predate later commits.
  `src/native/policy_registry.rs:95-160` is now `101-174`,
  `src/native/config_schema.rs:75-93` is now `76-96`,
  and root `mise.toml:1321-1327` is now `1323-1329`.
- The ledger's `Open questions` section still lists the items decided on 2026-10-05,
  and says the native registry defaults the optional policies to `Off` and reports a legacy file on every load.
  Commit `329de1b97` implements the decisions.
- The ledger marks forwarding,
  events,
  wrapper controls,
  and management commands as absent or in progress.
  Library code and an executable for them now exist
  (`doc/handover/cli-git-native-policy-engine.md`,
  section `What the executable does today`).
- A signaled Git child:
  the incumbent exits `1`,
  the ledger calls that a defect,
  the plan states no contract,
  and the native code returns `128 + N` (`src/native/forwarding.rs:77-89`).
- Event numbering:
  the ledger records a disagreement between the spec and the incumbent code by reading.
  The incumbent's own end-to-end fixture enforces the spec,
  and the native code follows the spec.
- Paths that are not UTF-8:
  the plan forbids requiring UTF-8,
  the incumbent fails on such a path by reading,
  native events hold paths as text,
  and the candidate branch holds them as bytes.
- The fast path:
  the plan lines 76 to 77 keep it,
  and the ledger's test for it expects no extra work.
  The engine handover's section `Hazard 5, the read-only fast path` records that the requirement
  "as written is not met" together with the root decision,
  because the root now comes from one Git query.
- Recovery:
  the ledger's recovery test triggers recovery with `git status`,
  and the native fast path does not look at leftover state for read-only commands.
- Post-command output:
  the ledger marks it retained,
  and the native executable prints none.
- Failure codes:
  the spec and the incumbent have `plugin-threw` and `policy-incomplete`;
  the native enumeration has neither and uses `content-unavailable`.
- The rules file:
  native `main` reads no variable and accepts no option;
  the candidate branch reads `FORBIDDEN_STRINGS_RULES`.
- The manual-push gate:
  the incumbent applies it only with a loaded configuration,
  the native engine whenever an enabled policy declares it.
- The legacy single-journal directory:
  the spec says it is recovered,
  and the native development executable refuses beside it.
- Supported Git:
  the plan supports the latest stable release only,
  the native code has no check,
  and this host's real Git is 2.55.0 while the tables target 2.56.0.
- The tool-cache allowlist:
  the ledger lists the missing platform branch as a defect,
  and the native code repeats the derivation unchanged.
- A wrapper failure before Git runs:
  the spec's exit contract says `2`,
  the incumbent exits `1` for an error it does not classify,
  and the native executable exits `2`.

## Not grounded

- Nothing was run on macOS or Windows.
  Every statement about Windows hook entries comes from reading Git and Git for Windows source,
  and every statement about macOS process identity from reading the incumbent.
- That the Windows process start time PowerShell reports equals the `GetProcessTimes` creation time
  is an inference from the documented units of each.
- Whether reduced Git for Windows distributions ship `sh` was not checked.
- The incumbent's behavior on a path that is not UTF-8,
  and on a signal sent to the wrapper alone,
  was read,
  not run.
- How agent harnesses cancel a command
  (which signal,
  to the process or to its group)
  was not measured.
- Whether `git cli-git fix --all --policy mono/dependent-version-bump` equals the standalone bump task
  on the release workflow's input was not compared.
- Whether any other clone holds a legacy single-journal directory,
  and whether any other machine still has `hook.hk-` configuration,
  cannot be measured from this checkout.
- The page-fetch tool reached its session limit during the survey.
  Crate versions come from the crates.io interface,
  and crate capabilities from each project's repository on its default branch
  (rustix from its `v1.1.5` tag),
  not from rendered documentation pages.
- For lefthook,
  husky,
  pre-commit,
  hk,
  cargo,
  tini,
  foreground-child,
  ripgrep,
  hub,
  uv,
  psutil,
  signal-hook,
  ctrlc,
  nix,
  fail-rs,
  and the Linux kernel,
  the files were read from the default branch on 2026-10-05;
  the release numbers given are the latest release at that time.

## Question batches

Each question restates its context so it can be asked without the item section.
Labels are listed in ranked order,
the first being the one ranked highest in the item section.
A batch holds at most four questions.

### Optional policies phase

#### Rules file for the forbidden-strings scanner

Suggested wording:
"The wrapper will run the forbidden-strings scanner inside itself.
The scanner needs to know where this repository's private rules file is.
Today the environment variable `FORBIDDEN_STRINGS_RULES`,
set by `mise`,
names it;
a `git` started without that variable scans with the built-in rules only and says nothing.
How should the rules file be named?"

- `Keep the variable`:
  nothing changes;
  the scan depends on the environment.
- `Config first, then variable`:
  a `rulesFile` option in `cli-git.config.jsonc` wins,
  and the variable is the fallback.
- `Config only`:
  only the `rulesFile` option;
  the standalone scanner and CI keep using the variable separately.

Detail:
"Policy option surface".

#### Failure code for a policy that could not finish

Suggested wording:
"When a policy cannot finish,
for example the scanner cannot load its rules,
the wrapper stops and prints one event with a `code`.
Today that code is `plugin-threw`,
but plugins no longer exist.
No script outside the wrapper matches on it.
Which code should the native wrapper print?"

- `By cause`:
  `content-unavailable` when repository content could not be read,
  `policy-incomplete` when the policy's own machinery failed.
- `policy-incomplete`:
  one existing code for every such failure.
- `Keep plugin-threw`:
  identical to today,
  with a name that no longer fits.
- `New policy-failed`:
  a new code.

Detail:
"Engine failure codes that lose their source".

#### A second implementation of the dependent-version bump

Suggested wording:
"Raising a package's version also bumps the packages that depend on it.
The wrapper does this at commit time,
and a separate TypeScript task does it in the release workflow,
from the same planning code.
Once the wrapper's side is in Rust,
should the release task keep its TypeScript copy of the plan,
or call the native wrapper?"

- `One implementation`:
  the release task runs `git cli-git fix` for that policy;
  the release workflow needs the native executable.
- `Keep both`:
  the release workflow stays on Node;
  the plan exists twice.

Detail:
"TypeScript utilities beside the wrapper".

### Transactions phase, first batch

#### A signal sent to the wrapper during a commit

Suggested wording:
"During a commit the wrapper waits for real Git.
If something stops the wrapper alone,
for example an agent's cancelled tool call or `timeout`,
real Git and its hooks keep running today,
and the next command recovers the abandoned commit.
What should the native wrapper do?"

- `Relay and clean up`:
  pass the signal to Git when the wrapper alone was signaled,
  wait,
  release locks.
- `As today`:
  the wrapper ends;
  recovery deals with the rest.
- `Relay everything`:
  no sender check,
  but Ctrl-C reaches Git and hooks twice.

Detail:
"Signals".

#### Exit status when real Git was killed

Suggested wording:
"When real Git is ended by a signal during a commit,
the wrapper today exits with `1`,
the same as an ordinary failure.
Which status should the native wrapper report?"

- `128 plus the signal number`:
  what a shell shows for real Git,
  for example `143` for `SIGTERM`.
- `Die by the same signal`:
  the wrapper ends itself with that signal,
  so programs see a signal death.
- `Keep 1`:
  as today.

Detail:
"Signals".

#### Hook entry files on Unix

Suggested wording:
"During a commit the wrapper gives Git entry files that hand each hook over to the wrapper.
Today they are Node programs.
Without Node,
which form should they take on Linux and macOS?"

- `Link named after the hook`:
  a symbolic link to the wrapper,
  which acts on the name it was started under;
  works with any install path.
- `First line naming the wrapper`:
  closest to today;
  fails when the install path has a space or is very long.
- `Shell script`:
  what other hook tools do;
  the spec avoided it on purpose.

Detail:
"Hook dispatcher and generated hooks without Node".

#### Hook entry files on Windows

Suggested wording:
"The same entry files on Windows,
where Git starts hooks differently.
Nothing was run on Windows;
this is from reading Git's source.
Which form should they take there?"

- `Executable named after the hook`:
  `pre-commit.exe` as a hard link or copy of the wrapper;
  no shell.
- `Shell script`:
  relies on the `sh` Git for Windows ships,
  as other hook tools do.
- `First line naming the wrapper`:
  works only if the wrapper is also on `PATH` under a second name.

Detail:
"Hook dispatcher and generated hooks without Node".

### Transactions phase, second batch

#### Locks and journals shared with the incumbent

Suggested wording:
"Both wrappers keep locks and recovery journals in the Git directory.
While rollback is possible both exist,
and several sessions commit here at once.
When `git` switches from one wrapper to the other while the other's locks or journals are still there,
what must work?"

- `Both directions, live`:
  each reads and writes the same formats;
  switching needs no quiet moment;
  every record needs a fixture in both directions.
- `Native reads old, writes new`:
  cutover is free,
  but rollback needs every native lock and journal gone first.
- `Separate state`:
  both cutover and rollback need every session stopped.

Detail:
"Lock and journal interoperability across versions".

#### An old-format journal directory

Suggested wording:
"Before 2026-09-25 the wrapper kept interrupted commits in a directory named `cli-git-transaction`.
None exists in this checkout.
If the native wrapper meets one in some other clone,
what should it do?"

- `Stop with instructions`:
  say that the previous wrapper must be run once to recover it.
- `Port the old recovery`:
  the native wrapper recovers it by itself.

Detail:
"Legacy single-journal directory".

#### Test-only crash switches in the installed wrapper

Suggested wording:
"Crash tests stop a commit at exact inner steps through an environment variable,
`CLI_GIT_TEST_ONLY_PHASE_SIGNAL`.
The current wrapper carries that switch in what everyone runs.
Should the native wrapper's release executable carry it too?"

- `In the release executable`:
  crash tests run on exactly what ships.
- `Test build only`:
  nothing test-only in what people run,
  but the inner steps are never tested on the shipped executable.

Detail:
"Test phase markers".

#### File names that are not UTF-8 in events

Suggested wording:
"Events name files in a `path` field,
which is JSON text.
Git allows file names that are not valid text;
this repository has none.
The current wrapper fails on such a file;
the plan says the native one must not.
How should an event name such a file?"

- `Readable path plus exact bytes`:
  `path` with replacement characters,
  and a new optional `pathBytes` field.
- `Escape in place`:
  `\xNN` inside `path`;
  exact,
  but readers must unescape.
- `Replace bytes`:
  no new field and no escaping;
  the file cannot be identified from the event.

Detail:
"Non-UTF-8 paths in events and journals".

### Transactions phase, third batch

#### An alias for commit

Suggested wording:
"The commit rules apply only when the command word is literally `commit`.
Measured today:
`git commit -a` is rejected,
while `git -c alias.c=commit c -a` commits.
Should the native wrapper keep that,
or treat an alias for `commit` as a commit?"

- `Resolve the alias`:
  an alias that expands to `commit` gets the commit rules.
- `Keep as today`:
  aliases stay outside the commit rules.

Detail:
"Choices found outside the ledger's list",
"An alias for `commit`".

### Cutover, first batch

#### Noticing an unsupported Git

Suggested wording:
"The native wrapper is written for Git 2.56.0 and has no fallbacks for older Git.
Some protections fail silently on older Git.
This machine's Git is 2.55.0 today.
When should the wrapper check the Git behind it?"

- `Before work that depends on it`:
  check the version once before a commit,
  hooks,
  or locks;
  read-only commands are never checked.
- `On every command`:
  one message everywhere,
  at the price of one more Git process each time.
- `Probe behavior`:
  test features where they are used;
  silent failures need extra probes.
- `At install and in check only`:
  no run-time cost;
  a later change of Git goes unnoticed.

Detail:
"Unsupported Git detection".

#### Which Git releases are supported

Suggested wording:
"Whatever check is used,
what counts as a supported Git?"

- `2.56.0 or newer`:
  a system update of Git keeps working;
  a commit that uses an option newer than the wrapper knows is refused until the wrapper is updated.
- `Exactly the release it was built for`:
  every Git update stops commits until a new wrapper is built.

Detail:
"Unsupported Git detection".

#### Exit status of the retired trust commands

Suggested wording:
"`git cli-git trust`,
`untrust`,
and `status` no longer have anything to do,
because configuration is data now.
Each will print an explanation.
No script outside the wrapper's own retiring workflow calls them.
Which exit status should they return?"

- `0 for all`:
  old instructions keep working under both wrappers during rollback.
- `0 for trust, 2 for the others`:
  the setup step keeps working;
  a caller expecting a report or a revocation is told it got none.
- `2 for all`:
  automation notices at once,
  but one instruction cannot serve both wrappers.

Detail:
"Retired trust commands".

#### Tests on macOS and Windows

Suggested wording:
"The only wrapper tests on macOS and Windows today cover the trust subsystem,
which is being removed,
and the native wrapper has never been compiled there.
Hosted runners are free for this public repository.
Which native tests should run on those systems?"

- `Platform suites in CI`:
  process identity,
  locks,
  hook entries,
  Git lookup,
  and one commit on each system;
  everything on Linux.
- `Everything in CI`:
  the whole suite on each system,
  on every wrapper change.
- `Local machines before a release`:
  no CI wiring;
  runs only when someone starts it.
- `Compile only`:
  builds on each system,
  runs nothing.

Detail:
"macOS and Windows coverage".

### Cutover, second batch

#### The wrapper's line after `git --version`

Suggested wording:
"Today `git --version` prints Git's version and then a line listing what the wrapper enforces.
To print anything after Git,
the wrapper has to stay alive behind it.
What should the native wrapper do?"

- `Print it first, on standard error`:
  still visible;
  standard output is exactly Git's.
- `Keep it after Git's line`:
  as today.
- `Drop it`:
  `git cli-git --help` shows that the wrapper is active.

Detail:
"Post-command output".

#### The wrapper's note after `git status`

Suggested wording:
"Today a plain `git status` ends with a note about the staging and branching rules,
and Git's own hints are switched off because they suggest commands the wrapper rejects.
What should the native wrapper do with the note?"

- `Print it first, on standard error`:
  the note stays;
  `git status` is handed to Git whole.
- `Keep it after Git's output`:
  as today;
  the wrapper waits behind every status.
- `Drop it`:
  neither Git's hints nor the note.

Detail:
"Post-command output".

#### The switch for verbose diagnostics

Suggested wording:
"Today `MONOCHROMATIC_VERBOSE=true` turns on the wrapper's detailed diagnostics.
The native wrapper has none yet.
Which switch should it use?"

- `MONOCHROMATIC_VERBOSE`:
  the existing switch,
  shared with the TypeScript tools.
- `A variable of its own`:
  for example `CLI_GIT_TRACE`,
  which can also write to a file and keep the event stream clean.
- `RUST_LOG`:
  the convention of the other Rust tools here;
  inherited by every Rust program a hook starts.

Detail:
"Verbose diagnostics switch".

#### The hk cleanup utility

Suggested wording:
"A TypeScript task removes leftover Git settings from the retired hk hook manager.
It lives inside the wrapper package,
whose TypeScript is deleted at the end of the rewrite.
This machine has no such settings left.
Do other machines still need it?"

- `Keep and move it`:
  it moves to another TypeScript package.
- `Retire it`:
  the task and its runbook are deleted at cutover.
- `Port it`:
  it becomes a command of the native wrapper.

Detail:
"TypeScript utilities beside the wrapper".

### Cutover, third batch

#### The performance bar

Suggested wording:
"The rewrite was asked for because the wrapper felt slow,
and no number has been agreed for the native one.
The current budgets are twice the incumbent's own measured maximum,
under a 2,000 ms ceiling.
Which bar should the native wrapper clear before cutover?"

- `Faster than the incumbent`:
  lower than the recorded incumbent medians by more than the noise,
  per scenario.
- `The existing budgets`:
  under today's ceilings;
  a slower result than today's could pass.
- `Fresh budgets`:
  set from the native wrapper's own first measurement.
- `Report only`:
  no gate at the first release.

Detail:
"Performance acceptance".

#### The reported slow commit

Suggested wording:
"You reported 8.9 seconds for staging and committing one Markdown file.
The existing benchmark uses a synthetic scanner and no Markdown policy,
so it cannot show that case.
Should that commit become a measured scenario?"

- `Add it and gate on it`:
  a scenario with this repository's real policies.
- `Measure it once`:
  a reported number,
  no gate.
- `Synthetic scenarios only`:
  no new scenario.

Detail:
"Performance acceptance".

[git-src]: https://github.com/git/git/tree/v2.56.0
[gfw-src]: https://github.com/git-for-windows/git/tree/v2.56.0.windows.1
[linux-src]: https://github.com/torvalds/linux
[lefthook]: https://github.com/evilmartians/lefthook
[husky]: https://github.com/typicode/husky
[pre-commit]: https://github.com/pre-commit/pre-commit
[hk]: https://github.com/jdx/hk
[cargo]: https://github.com/rust-lang/cargo
[tini]: https://github.com/krallin/tini
[foreground-child]: https://github.com/tapjs/foreground-child
[sudo-man]: https://www.sudo.ws/docs/man/sudo.man/
[signal-hook]: https://crates.io/crates/signal-hook
[ctrlc]: https://crates.io/crates/ctrlc
[nix]: https://crates.io/crates/nix
[rustix]: https://crates.io/crates/rustix
[psutil]: https://github.com/giampaolo/psutil
[proc-stat]: https://man7.org/linux/man-pages/man5/proc_pid_stat.5.html
[getprocesstimes]: https://learn.microsoft.com/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes
[datetime-ticks]: https://learn.microsoft.com/en-us/dotnet/api/system.datetime.ticks
[ripgrep]: https://github.com/BurntSushi/ripgrep
[hub]: https://github.com/mislav/hub
[uv]: https://github.com/astral-sh/uv
[fail-rs]: https://github.com/tikv/fail-rs
[actions-billing]: https://docs.github.com/en/billing/concepts/product-billing/github-actions
