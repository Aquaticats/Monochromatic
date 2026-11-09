# Health-route findings on `main`

## Scope and evidence at 2026-11-09T06:59:33-05:00

Audited revision:
`main` at `baf3d03fe668432f127e024275edeb3b9a668281`.
The worktree started at that revision on `docs/planning-cli-git-health-audit`.
This audit changes no production code and commits no tests.
It covers `package/git-policy/cli/src/native/health/`,
its callers,
and the readers and migration helpers it invokes.
Linux only;
Windows and macOS remain outside the owner's requested scope.
No unmerged implementation branch was used.

The report follows [the copy-route audit](cli-git-copy-route-findings-on-main.md).
The handover's `Open review findings` section was read at the audited revision
(`doc/handover/cli-git-rust-implementation.md:5703`).
It does not record these health-route findings.
The settings audit already records unbounded settings-file reads
([settings and dependent-version audit](cli-git-config-and-dependent-version-route-findings-on-main.md),
`Waits`);
this report names the additional health entry points rather than claiming that settings defect as new.

## What the route does and who calls it

`git monochromatic-git-policy-cli doctor` is an on-request inventory and explanation,
not a proof that every later command will succeed.
`package/git-policy/cli/src/native/invocation/management.rs:329` selects the real Git,
retains the caller's global options and environment overlay,
and runs `doctor_action_with_section` on the runtime.
The latter locates and scans hook directories,
recognizes exact Git LFS shims,
reads configured external hooks and retired settings,
reports fsmonitor,
counts saved conflict copies,
and appends the filesystem section
(`package/git-policy/cli/src/native/health/doctor.rs:477`).
Management then adds notes about earlier batches and removed-worktree service units.

`package/git-policy/cli/SPEC.md:442` defines the exit status narrowly:
`0` means no reported unsupported hook,
`1` means one was found,
and `2` means the repository or directory could not be examined.
Fsmonitor and filesystem text do not change that status.
A broken JSONC settings file is deliberately not diagnosed by `config_findings`
(`package/git-policy/cli/src/native/health/doctor_config.rs:100`).
Consequently,
an exit of `0` beside that file is not ranked here as a promise of globally sound configuration.

Commit preparation calls the warning writers only on its first preparation,
from the configuration snapshot it just captured
(`package/git-policy/cli/src/native/landing/service_prepare.rs:106`).
The traditional warning is triggered by a configured non-empty `core.hooksPath`,
not a directory scan.
The external warning is triggered by configured hook names not judged disabled.
Neither warning runs a hook or changes the result by returning an error.
The warning decisions use only the supplied snapshot;
their writers synchronously write to standard error.

### Owner decisions that govern the findings

The audit brief supplies these requirements verbatim:

- "Commits should never fail."
- "never discard user data"
- "nothing should ever wait forever or indefinitely"

The last requirement also has the owner's explicit boundary behavior:
"Return with an error" and "No, keep it for a retry (Recommended)"
(`doc/handover/cli-git-rust-implementation.md:7283`).
The saved work stays saved;
nothing should land later by itself because the waiting command gave up.

External hooks are intentionally unsupported.
The owner's words were "Stop supporting them and port anything in."
and the selected option was "B: Only hooks we declare, natively"
(`doc/handover/cli-git-rust-implementation.md:8165`).
The older dispatcher and hook-lock paragraphs in
`doc/decision/cli-git-concurrent-commits.md:50` describe the superseded design,
not a reason to restore external hooks.
A missed warning in this report is a missed disclosure,
not a request to run those hooks.

For fsmonitor,
the owner said:
"If git's fsmonitor is fundamentally broken and cannot be easily or meaningfully worked around,
disable git's fsmonitor globally entirely and make doctor say so."
The record is `doc/handover/cli-git-rust-implementation.md:8066`.
Accurately saying whether it is off is the issue here;
none of the probes started a monitor daemon.

For retained names,
the owner answered "rename them."
(`doc/planning/git-policy-cli-program-rename.md:115`).
That plan's handling at `:130` moves wrapper-owned state when first needed,
while the doctor's contract says it "changes nothing"
(`package/git-policy/cli/SPEC.md:527`).
The selected-repository migration finding records that conflict explicitly,
not an invented ban on all migrations.

## How the reproductions ran

Scratch files are inside this worktree at
`package/git-policy/cli/target/health-route-audit/`:

- `probe.ts` and `results.json` contain the initial disposable-repository differential run.
- `followups.ts` and `followup-results.json` contain raw-Git reader controls,
  the enabled-hook positive control,
  and unusual Git-directory layouts.
- `warning-seams.patch` preserves the temporary sidecar tests.
  Their additions were removed with exact edits after the run;
  neither sidecar is part of the submitted diff.

`mise run //package/git-policy/cli:native:build` built the audited executable successfully.
The oracle reported `git version 2.56.0`.
The wrapper reported `monochromatic-git-policy-cli 0.6.1-alpha.5`.
Both were invoked as `git`:
the oracle with `PATH=/usr/bin:/bin`,
the wrapper with a disposable `bin/git` link to the just-built executable before those directories.
No installed-wrapper or real-service behavior is evidence for these findings.

The fixtures supply their own `HOME`,
`XDG_RUNTIME_DIR`,
author and committer identities,
`GIT_CONFIG_NOSYSTEM=1`,
and `GIT_CONFIG_GLOBAL=/dev/null`.
Every Git step has a 20-second deadline.
The build and nextest runs have a 15-minute outer deadline,
4-CPU quota,
6 GiB memory cap,
192-task cap,
and I/O weight 10.
The differential harness has a five-minute deadline,
2-CPU quota,
1 GiB memory cap,
96-task cap,
and I/O weight 10;
the follow-up harness uses the same resource caps and a three-minute deadline.
No mount or synthetic host load was generated.

In output excerpts,
`<fixture>` replaces the initial run's absolute disposable root,
and `<followup>` replaces the follow-up root.
Escaped `\n` inside quoted pathname text denotes a byte in that pathname,
not a directory separator.
Repeated informational fsmonitor and filesystem paragraphs are omitted where not relevant.
These are excerpts,
not claims that omitted stderr was empty.

## Findings ranked by risk

### Path answers select and mutate an unrelated directory

Status:
**reproduced** with the built executable and real Git,
including a control with `CLI_GIT_IN_PROCESS_READS=off`.
This ranks first because it crosses the selected repository's boundary and moves unrelated state.
No deletion or lost file content was observed.

`package/git-policy/cli/src/native/health/doctor.rs:252` decodes all path answers as UTF-8.
`locate_hooks` asks for the common and private Git directories together,
then takes the first lines as separate paths (`:274` to `:285`).
A newline inside a valid pathname is therefore mistaken for the answer separator.
The resulting `common_directory` goes both to the scan (`:487`)
and to `settle_common_directory_names` (`:504`).
It is not checked against the repository Git actually selected.

Exact sequence:

1.  Create a disposable directory `<fixture>/newline-decoy`.
    Put an executable `hooks/pre-commit` under it,
    and put `cli-git/do-not-move.txt` there with `unrelated-content`.
    This directory is not a repository.
2.  Use real `git init --quiet --initial-branch=main` to create a separate repository whose name is
    `<fixture>/newline-decoy\nactual`.
    Its own `.git/hooks` holds only Git's samples.
3.  In that repository,
    run native `git hook run --ignore-missing pre-commit` and
    `git rev-parse --path-format=absolute --git-common-dir --absolute-git-dir`.
4.  Run the built wrapper's `git monochromatic-git-policy-cli doctor` from the same directory.
    Inspect the unrelated directory afterwards.

Observed excerpts:

```text
# package/git-policy/cli/target/health-route-audit/results.json
native hook: exit 0, stdout "", stderr ""
rev-parse stdout, escaped:
"<fixture>/newline-decoy\nactual/.git\n<fixture>/newline-decoy\nactual/.git\n"
doctor: exit 1
stdout: "<fixture>/newline-decoy/hooks/pre-commit\n"
stderr: "... Delete each script, or express what it checked as a policy ..."
decoyEarlierExists: false
decoyCurrentExists: true
```

Wrong result:
doctor lists a script outside the selected repository,
tells the person to delete it,
and renames the unrelated directory's `cli-git` folder to `monochromatic-git-policy-cli`.
This is a deterministic pathname parse error,
not a race with another program.
It bears on "never discard user data" by directing action at somebody else's data;
the reproduction proves a move,
not data loss.

The same path-as-text assumption has a second reproduced symptom.
Create an external hooks directory whose final byte is `ff`,
put an executable `pre-commit` there,
and write its raw pathname bytes as `core.hooksPath` in the fixture's `.git/config`.
Git runs it;
doctor refuses the valid pathname:

```text
# package/git-policy/cli/target/health-route-audit/results.json
native hook: exit 0, stderr "HOOK-RAN\n"
doctor: exit 2, stdout ""
stderr: "git monochromatic-git-policy-cli doctor: git config --type=path --get core.hooksPath printed bytes that are not UTF-8\n"
```

Suggested fix:
preserve operating-system path bytes through the query boundary,
obtain path answers separately rather than splitting multiple answers on pathname-valid bytes,
and remove only each answer's protocol terminator.
Do not guess a common directory from a partially decoded report.
This is a suggestion,
not an implemented change.

### Doctor migrates saved state despite its read-only contract

Status:
**reproduced** in a repository with an ordinary UTF-8 name,
independently of the wrong-directory finding.

`package/git-policy/cli/src/native/health/doctor.rs:504` always settles common-directory names
before counting saved copies.
`package/git-policy/cli/src/native/foundation/kept_names_in_the_git_directories.rs:184`
renames old-only names and folds names that exist under both spellings.
The doctor's documented promise is broader than "does not write configuration":
`package/git-policy/cli/SPEC.md:527` says it takes no lock and "changes nothing".

Exact sequence:

1.  Initialize a disposable repository with real Git.
2.  Create `.git/cli-git/health-audit-kept.txt` containing `saved-content\n`.
    Leave `.git/monochromatic-git-policy-cli` absent.
3.  List `.git`,
    run doctor,
    and list `.git` again.
    Read the saved content through its current name.

Observed excerpts:

```text
# package/git-policy/cli/target/health-route-audit/results.json
before includes: "cli-git"
doctor: exit 0, stdout ""
after includes: "monochromatic-git-policy-cli"
earlierExists: false
currentContent: "saved-content\n"
```

Wrong result:
a report command moves a saved-state directory even when it reports no unsupported hook.
The bytes survived this probe.
The risk is an unexpected state change during inspection,
not a demonstrated discard.
The owner's "rename them." decision supports migration generally;
it does not make the documented statement that doctor changes nothing true.
The later rename-plan explanation also calls doctor's earlier-batch note read-only
(`doc/planning/git-policy-cli-program-rename.md:684`).

Suggested fix:
use the existing read-only retained-name lookup for reports,
and leave migration to a state-changing command.
Alternatively,
revise and explicitly accept a narrower doctor contract before relying on it as read-only.
Neither alternative is adopted by this audit.

### Filesystem reads and warning writes escape the wait bounds

Status:
**read from the code only**.
No unresponsive mount or blocked output sink was created.
This ranks ahead of reporting inaccuracies because the warning writer is on commit preparation's waiting path.

The doctor performs synchronous `read_dir` and metadata calls
in `package/git-policy/cli/src/native/health/doctor.rs:154` to `:174`.
For regular LFS-named files,
`is_lfs_shim` opens and reads synchronously (`:360` to `:381`);
its byte cap is not a time bound.
The filesystem section calls `filesystem_kind` and subvolume metadata synchronously
(`package/git-policy/cli/src/native/health/doctor_filesystem.rs:218`),
which reaches `libc::statfs`
(`package/git-policy/cli/src/native/subvolume/fs_kind.rs:97`).
`doctor_action_with_section` calls these functions directly,
with no outer deadline (`doctor.rs:487` to `:506`).
`run_on_runtime` waits for the future's result via `block_on`,
not a report deadline
(`package/git-policy/cli/src/native/foundation/lazy_runtime.rs:297`).

Concrete filesystem sequence:
configure `core.hooksPath` to an existing directory on a mount,
let Git's path queries complete,
then have that mount leave the doctor's directory read unanswered.
The command is stuck in `find_traditional_hooks`;
no timer in this route returns an error to the person.
The same result follows from a regular LFS shim's stalled read,
or the filesystem section's stalled `statfs` after hook scanning completes.
These are conditional code traces,
not a measured assertion about this host's mounted filesystems.

The warning route has a separate I/O instance of the same missing boundary:

- `package/git-policy/cli/src/native/health/traditional_hook_warning.rs:169` calls
  `write_all`,
  then `flush`,
  on its supplied writer.
- `package/git-policy/cli/src/native/health/external_hooks.rs:233` does the same.
- `package/git-policy/cli/src/native/landing/service_prepare.rs:106` calls them synchronously
  before continuing preparation.

Concrete warning sequence:
a caller supplies a full blocking pipe as standard error,
keeps its read end open without draining it,
and starts a commit that reaches preparation with a warning-triggering configuration.
The warning's `write_all` cannot complete until the reader drains or closes the pipe.
No warning deadline lets preparation move on.
Logging a returned write error does not handle a write that never returns.
The pipe can be full before the wrapper starts;
this sequence does not depend on the wrapper having emitted a long warning first.

This conflicts with "nothing should ever wait forever or indefinitely"
and the SPEC's "The warning never blocks the commit"
(`package/git-policy/cli/SPEC.md:5688`).
It is distinct from a hook program hanging:
the wrapper deliberately runs none.

Suggested fix:
put potentially blocking filesystem probes behind a bounded operation boundary
that can return even when the operation does not,
and make optional warning delivery bounded without changing the command's success result.
A timer around a future that directly blocks inside a synchronous syscall is not sufficient by itself.
Choose the isolation and diagnostic policy in a separate design;
this audit supplies no arbitrary shipped timeout.

### Lexical parent removal scans a different hooks directory from Git

Status:
**reproduced** with real Git,
with in-process readers enabled and disabled.

`package/git-policy/cli/src/native/health/doctor.rs:96` normalizes each scan path lexically.
`package/git-policy/cli/src/native/foundation/lexical_path.rs:31` removes the component before `..`
without resolving a symlink.
This changes the directory to examine,
not just its display name or duplicate key.

Exact sequence:

1.  Initialize `<fixture>/symlink-parent` with real Git.
2.  Create `<fixture>/external/child` and `<fixture>/external/hooks`.
    Put an executable `pre-commit` in the latter.
3.  Symlink `<fixture>/symlink-parent/redirect` to `<fixture>/external/child`.
4.  Set `core.hooksPath` to `redirect/../hooks` with native `git config`.
5.  Run native `git hook run pre-commit`,
    then doctor from the same repository.

Observed:

```text
# package/git-policy/cli/target/health-route-audit/results.json
native hook: exit 0, stdout "", stderr "HOOK-RAN\n"
doctor: exit 0, stdout ""
```

Wrong result:
Git resolves the path to `<fixture>/external/hooks` and runs the hook,
but doctor normalizes it to `<fixture>/symlink-parent/hooks` and lists nothing.
The ordinary direct-symlink positive control does list the hook and exit `1`;
this is specifically the lexical `..` rewrite.
The doctor contract says to scan the directory the configured path names
(`package/git-policy/cli/SPEC.md:453`).
The hook still never runs through the wrapper,
by the owner's external-hook decision.

Suggested fix:
scan the path with its filesystem meaning intact.
Deduplicate by an identity established without changing the path being scanned,
or tolerate duplicate report entries when identity cannot be established.

### A user-owned shadow-shaped path is treated as the wrapper's override

Status:
**reproduced through the warning decision seam**,
with a real-Git hook positive control for the pathname shape.
The full wrapped-commit boundary was not reproduced.

`package/git-policy/cli/src/native/health/traditional_hook_warning.rs:66` recognizes
`/dev/null/cli-git-no-hooks` specially,
then accepts any path ending in `shadow/<canonical UUID>/no-hooks`.
At `:87`,
it checks only the `shadow` component,
not the wrapper's data directory,
the selected repository,
or whether the value came from its own command override.
`configured_hooks_path` suppresses the warning at `:119`.

Exact sequence:

1.  In a disposable repository,
    create an executable hook under
    `shadow/12345678-1234-4234-8234-123456789abc/no-hooks/pre-commit`.
2.  Configure `core.hooksPath` to that directory's absolute pathname.
    Native `git hook run pre-commit` prints `HOOK-RAN`.
    Doctor lists the same path and exits `1`.
3.  Supply the configuration snapshot seam with local
    `core.hookspath=/project/shadow/12345678-1234-4234-8234-123456789abc/no-hooks`.
    Assert that `traditional_hooks_warning` returns a line.

The temporary nextest test fails:

```text
# package/git-policy/cli/target/health-route-audit/warning-seams.patch
FAIL health::traditional_hook_warning::tests::health_audit_unrelated_shadow_directory_still_warns
thread 'health::traditional_hook_warning::tests::health_audit_unrelated_shadow_directory_still_warns' panicked:
a user-owned /project/shadow directory is not a wrapper override
```

Wrong result:
the snapshot's user configuration is silently treated as an internal hooks-off override.
The SPEC excludes the wrapper's own transaction overrides,
not unrelated paths sharing a suffix
(`package/git-policy/cli/SPEC.md:5693`).
This hides the disclosure that the owner's unsupported-hook decision requires;
it does not make the wrapper run the hook.

Suggested fix:
carry enough provenance or repository context to recognize the actual internal override,
without adding a directory scan to each commit.
A filename suffix alone is not ownership.

### Partial boolean parsing invents fsmonitor and external-hook faults

Status:
**reproduced** by differential doctor runs and a failing warning-selection sidecar test.

`package/git-policy/cli/src/native/config/config_snapshot.rs:145` deliberately models only plain decimal numbers.
`canonical_bool` returns `None` for other numeric spellings (`:206`).
That means "ask Git" to snapshot callers,
not "this is certainly a hook path" or "this is enabled".
The health consumers instead decide:

- `package/git-policy/cli/src/native/health/doctor_fsmonitor.rs:220` passes that result to
  a branch that calls unmodelled text a hook and says fsmonitor is on (`:240`).
- `package/git-policy/cli/src/native/health/external_hooks.rs:143` drops only `Lookup::Found(false)`.
  An unhandled false value keeps the external-hook report and warning.

Exact fsmonitor sequence:
initialize a disposable repository,
set `core.fsmonitor` to `00`,
and compare native `git config --type=bool --get core.fsmonitor`,
native `git status --porcelain`,
and doctor.
The ordinary `false` spelling is the control.

Observed excerpts:

```text
# package/git-policy/cli/target/health-route-audit/results.json
core.fsmonitor=false: native bool "false\n"; doctor says "Git's fsmonitor is off"
core.fsmonitor=00: native bool "false\n"; native status exit 0, stdout "", stderr ""
doctor exit 0; stderr contains:
"Git's fsmonitor is on, against the owner's decision of 2026-11-03 to keep it off: core.fsmonitor names the hook 00 in file:.git/config, local scope."
```

`0x0`,
`-0`,
`+0` and `0k` produced the same disagreement.
Git's `fsmonitor-settings.c:119` first asks its boolean reader;
it uses the hook-path interpretation only when that reader rejects the value.
The inspected Git 2.56.0 source is in the read-only local checkout
`${HOME}/temp/agent/git-2.56.0-20261103`;
`parse.c:192` accepts integers through `git_parse_int`.
The executable's own output,
not that checkout's build,
is the differential oracle.

Exact external-hook sequence:
set `hook.audit.event=pre-commit`,
`hook.audit.command` to a command printing `EXTERNAL-RAN`,
and enable the hook with `true` as a positive control.
Native `git hook run --ignore-missing pre-commit` prints the marker.
Replace the enabled value with `00` and run both again.

```text
# package/git-policy/cli/target/health-route-audit/followup-results.json
enabled=true: native bool "true\n"; native hook stderr "EXTERNAL-RAN\n"
enabled=00: native bool "false\n"; native hook exit 0, stdout "", stderr ""
doctor with enabled=00: exit 1, stdout "hook.audit (events: pre-commit)\n"
```

The direct warning-selection seam also fails:

```text
# package/git-policy/cli/target/health-route-audit/warning-seams.patch
FAIL health::external_hooks::tests::health_audit_numeric_zero_disables_configured_hook
Git disables this hook, but the warning retains [ConfiguredHook { name: "audit", events: ["pre-commit"], has_command: true }]
```

Wrong result:
doctor alleges an owner's-fsmonitor violation that does not exist,
and reports an already-disabled external hook as needing removal.
The warning selection has the same disabled-hook error.
These do not demonstrate a failed commit or a running daemon.

Suggested fix:
retain the snapshot's unhandled state and ask Git when health needs the answer,
or implement and differentially test Git's complete boolean grammar in the shared owner.
Do not reinterpret parser uncertainty as enabled state.

### The filesystem report mistakes a directory name for checkout identity

Status:
**reproduced** with real Git's separate-Git-directory and bare layouts.

`package/git-policy/cli/src/native/health/doctor_filesystem.rs:385` accepts any common directory
whose basename is `.git`,
then uses its parent as the main checkout.
It does not establish that the repository is non-bare or that this parent is its worktree.
`doctor.rs:506` passes only the common directory to that section.

Exact separate-directory sequence:

1.  Create disposable directories `<followup>/real-worktree` and `<followup>/metadata`.
2.  From `real-worktree`,
    run native `git init --quiet --separate-git-dir=<followup>/metadata/.git`.
3.  Run native `git rev-parse --show-toplevel`,
    then doctor from `real-worktree`.

Observed excerpts:

```text
# package/git-policy/cli/target/health-route-audit/followup-results.json
native top level: "<followup>/real-worktree\n"
doctor exit 0; stderr contains:
"git monochromatic-git-policy-cli doctor: filesystem: <followup>/metadata is on btrfs but is an ordinary directory, not a subvolume."
"    git monochromatic-git-policy-cli migrate-to-subvolume"
```

The bare variant is also concrete:
initialize a bare repository at `<followup>/bare-parent/.git` and run doctor there.
Native `rev-parse --is-bare-repository` prints `true`;
`rev-parse --show-toplevel` exits `128` with
`fatal: this operation must be run in a work tree`.
Doctor nevertheless emits the btrfs checkout and migration paragraphs for `bare-parent`.
An ordinary `bare.git` control emits no filesystem section.

Wrong result:
the filesystem advice names a metadata directory or a bare repository's parent as the checkout.
The migration suggestion is based on that wrong directory's facts.
The probe used one filesystem,
so it proves the wrong named target,
not a measured difference between two filesystem types.

Suggested fix:
pass an established main-checkout identity to the section,
and emit no checkout advice when that identity cannot be established.
A basename is not a repository-layout proof.

## Verification and limits at 2026-11-09T06:59:33-05:00

The native build passed.
The temporary warning tests ran through nextest,
not an ad hoc Rust runner:

```text
# mise run //package/git-policy/cli:native:test:host -- health_audit
Starting 2 tests across 1 binary (5093 tests skipped)
Summary: 2 tests run: 0 passed, 2 failed, 5093 skipped
```

Each failure is quoted with its finding.
After removing the temporary additions,
the existing health suite passed:

```text
# mise run //package/git-policy/cli:native:test:host -- health::
Starting 70 tests across 1 binary (5023 tests skipped)
Summary: 70 tests run: 70 passed, 5023 skipped
```

The passing names include `the_report_is_built_on_the_runtime_it_is_awaited_on`,
`the_host_section_names_the_main_checkout_and_is_empty_for_anything_else`,
`the_wrappers_own_overrides_do_not_warn`,
and `a_refusing_stream_does_not_panic`.
Those controls run;
they do not cover the adverse path shapes,
parser uncertainty,
or a writer that blocks instead of returning an error.
No full-crate test suite or mutation campaign was run.
Package lint and the report's format,
lint and rendered checks are still being recorded at this checkpoint.

### What could not be determined

- No full wrapped-commit warning boundary was exercised.
  The disposable environment lacks a user service manager,
  so wrapped commits stopped before preparation with:
  `This computer cannot start the commit service on demand, so this commit was not made.`
  No attempt used or changed the installed service.
  The warning findings rely on the sidecar seams and the cited caller,
  not those stopped commands.
- No stopped mount,
  stalled regular-file read,
  blocked warning pipe,
  or kill during a migration fold was exercised.
  The wait finding is explicitly code-only;
  the ordinary rename probe does not prove fold crash safety.
- The filesystem wording controls cover btrfs,
  overlayfs,
  other filesystems,
  mount options and command quoting,
  but this audit did not run remounts,
  kernel-argument edits,
  conversion,
  or cross-filesystem separate-Git-directory fixtures.
- No claim is made about a hook's own execution being safe.
  Git ran only disposable marker hooks.
  The wrapper's intentional hook suppression is not a compatibility defect in this audit.

## What was checked and found sound

- A normal linked worktree uses the shared common hooks directory.
  Real Git ran the marker hook;
  doctor listed `<fixture>/worktree-parent/.git/hooks/pre-commit` and exited `1`.
  It described the main checkout in its filesystem section,
  not the linked worktree's administrative directory.
- A normal bare repository with `core.hooksPath=custom` resolves it against the bare Git directory.
  Real Git ran the hook;
  doctor listed `<fixture>/bare.git/custom/pre-commit`,
  exited `1`,
  and emitted no checkout filesystem advice.
- A direct symlinked hooks folder pointing outside the repository works for both Git and doctor.
  The reproduced symlink defect requires parent-component normalization,
  not a symlink alone.
- The inventory's regular-file,
  executable-bit,
  sample-suffix and symlink rules match its written SPEC and its passing controls.
  This inventory is intentionally not identical to Git's runnable-hook lookup:
  executable FIFOs and directories are omitted,
  whereas native `git hook run pre-commit` failed with
  `fatal: cannot exec '.git/hooks/pre-commit': Permission denied` for those fixtures.
  Dangling symlinks count in the inventory even though Git cannot execute their absent targets.
  Those explicit inventory choices are recorded as limits,
  not additional accidental findings.
- LFS recognition checks the name and non-symlink regular-file type before opening,
  limits bytes read,
  and compares the exact shim for that hook's name
  (`package/git-policy/cli/src/native/health/doctor.rs:360`).
  A static FIFO or symlink cannot reach that open through the scan;
  a slow regular file still has the wait limitation recorded with the wait finding.
- The existing tests verified exact LFS shims separately from modified,
  misnamed and linked shims;
  shims alone leave the exit status at `0`,
  while a foreign hook beside them yields `1`.
- Fsmonitor reporting reads configuration without writing it or starting or stopping its daemon.
  Last-entry precedence,
  ordinary false words,
  command scope and origin reporting have passing controls.
  The numeric false spellings in the boolean finding are outside those controls.
- The warning decision functions neither start a process nor inspect a hook directory.
  A missing snapshot gives no warning.
  A returned write error is ignored after diagnostic logging,
  as the refusing-writer tests verify;
  it does not propagate a commit failure from these functions.
