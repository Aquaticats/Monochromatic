# Config and dependent-version route findings on `main`

Status:
audit,
2026-11-09.
Audited revision:
`main` at `70b67c6ef2`,
in the worktree
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-config-and-dependent-version-route-findings-on-main`
(fast-forwarded to `main` before writing;
every file this page cites is byte-identical to the tip).
No production code was changed and no test was committed.
The throwaway scripts are in
`${HOME}/temp/agent/pi-config-dependent-version-audit`,
and each quoted result names the script that produced it.
Linux only.

Scope:
what the wrapper reads before it does anything else,
and what it rewrites in a commit on someone's behalf:

- `package/git-policy/cli/src/native/config/` (13 production files,
  3,869 lines,
  test sidecars excluded from the count),
- the settings entry points that call it
  (`invocation/wrapped_command.rs`,
  `invocation/management_direct_commands.rs`,
  `invocation/entry_forward_action_planning.rs`),
- the Git configuration snapshot the commit path reads
  (`config/config_snapshot.rs`,
  `config/config_fsync.rs`),
- the forbidden-strings rules selection the configuration names
  (`candidate/scanner_selection.rs`,
  `policy/policy_forbidden_strings.rs`,
  `candidate/scanner_adapter.rs`),
- `package/git-policy/cli/src/native/dependent_version/` (20 production files,
  4,633 lines,
  test sidecars excluded),
  the policy arm that calls it
  (`policy/policy_checks.rs:527-534`),
  and the worktree install its corrections take
  (`policy/direct_fix_install.rs`).

Purpose:
`doc/handover/cli-git-rust-implementation.md`
("Open review findings",
recorded 2026-11-09)
lists eleven open findings,
the copy route's audit and the push route's audit,
and records no review of either the settings route or the dependent-version step.
This page answers the route's own questions,
with a sequence and a wrong result for each finding:

- can the settings file stop every commit,
  and does the person learn why,
- does a rule that cannot be read fail open or closed,
- can the dependent-version step edit a file the person did not stage,
  lose an edit,
  or leave a manifest half written after a kill,
- what can it wait on,
  and which locks exist.

## The route, step by step

1.  Which invocations load the settings at all:
    `config/config_loading.rs:19` (`ConfigLoading`),
    the inspection command list at `:27`,
    and the classifier at `:221`.
    Read-only forms skip the file and run on defaults.
2.  A guarded command loads it once,
    before anything else:
    `invocation/wrapped_command.rs:444-452` decides,
    `:291` calls `invocation_config.rs:46`
    (`load_identity_config`),
    which asks `config/config_file.rs:364` (`load_repository_config`)
    for the settings of the worktree Git reported.
    `invocation/entry_forward_action_planning.rs:106`
    makes the same call for the fast forwarding path,
    for index writers only.
3.  `load_repository_config` resolves the name:
    the current file when it exists,
    else the earlier name,
    else a legacy executable file is a refusal
    (`config_file.rs:364-420`).
    The person is told once per process about the earlier name
    (`invocation_config.rs:59`,
    `:71`,
    an `AtomicBool`,
    not a lock).
4.  The read is `config_file.rs:262` (`read_config_text`):
    `symlink_metadata` first (`:270`),
    a symbolic link is refused (`:278`),
    a non-regular file is refused (`:285-293`),
    at most `MAX_CONFIG_BYTES` (`:89`),
    read through `take(MAX_CONFIG_BYTES + 1)` (`:320`),
    an oversized file refused by name (`:329`),
    then strict UTF-8 (`:336-346`).
5.  The text is parsed by `config/config_parse.rs:194` (`parse_config`):
    JSONC (`:203`),
    a syntax error names its byte offset (`:206-212`),
    then the top-level keys,
    the policy section (`config/config_policies.rs`),
    each policy's options,
    the retired concurrency keys (`config_parse.rs:117-176`),
    and the discarded-copies limits.
    Duplicate keys are refused (`config/config_values.rs:174`,
    the repeated-key refusal at `:213`).
6.  A refusal ends the command with status 2 and one
    `config-invalid` engine event naming the file and the cause
    (`wrapped_command.rs:291-299`;
    `management_direct_commands.rs:311`,
    whose event goes to standard output on `check` and `fix`).
    Nothing has been written at that point,
    so a refused commit leaves `HEAD`,
    the index and the worktree as they were.
7.  Git's own configuration is read once by the commit path
    through `config/config_snapshot.rs:612` (`read_config_snapshot`)
    as `git config --list -z --show-scope` (`:647`).
    A failure,
    a non-zero exit,
    or a listing of an unexpected shape returns `None`,
    and each consumer then starts Git for its own key
    (`:66-110`),
    so a snapshot problem never becomes a wrong value:
    `Lookup::Unhandled` is "ask Git".
    `core.fsync` is read from the same snapshot
    (`config/config_fsync.rs:129`),
    and decides the flushes of the wrapper's own in-process writes.
8.  The rules files are chosen after the settings load,
    when the forbidden-strings policy runs:
    `candidate/scanner_selection.rs:124` (`rules_source`)
    takes the configured `rulesFile` entries first,
    then `FORBIDDEN_STRINGS_RULES`,
    then `forbidden-strings.local.txt` (`:68`).
    A named file that is missing,
    unless its entry is `optional`,
    fails the load (`candidate/scanner_adapter.rs:213-243`),
    and the policy reports it as an engine failure
    (`policy/policy_forbidden_strings.rs:460-476`).
    When no candidate would be scanned,
    the rules are not loaded at all (`:450-458`),
    so a commit that changes nothing scannable cannot be stopped by a missing rules file.
9.  The dependent-version step runs only where its trigger table says:
    `policy/policy_trigger.rs:131-136` gives it
    `pre-forward`,
    `direct-check` and `direct-fix`,
    and not `post-commit` or `manual-push`.
    On `pre-forward` it returns before reading anything
    (`dependent_version/policy_dependent_version.rs:309`),
    and the request it builds for every other trigger sets
    `forwards_commit: false` (`:320`),
    which `dependent_version/dependent_version_policy.rs:166` turns into "plan nothing".
    This is finding 2.
10. Planning itself starts at `dependent_version_plan.rs:222`
    (`plan_workspace_bumps`):
    every workspace manifest at both states
    (`dependent_version_states.rs:105`),
    the registry configuration only when a version was raised
    (`dependent_version_plan.rs:141-155`),
    then the publish set
    (`dependent_version_publishable.rs:107`),
    the bundled-development-import walk
    (`dependent_version_bundled.rs:109`),
    and the plan with exact bytes before and after each bump
    (`dependent_version_plan.rs:159-180`).
    This is finding 1.
11. A correction exists only on `direct-fix`
    (`policy_dependent_version.rs:355-363`),
    and is installed by `policy/direct_fix_install.rs:396`
    (`install_corrections`):
    the real index is read (`:399`),
    every destination must still hold the bytes the fix read
    (`:273-278`),
    each replacement is written beside its file (`:283-300`),
    renamed over it (`:332-341`),
    and the real index re-read to prove it did not move (`:344-350`),
    with rollback on any failure (`:180-205`).

## What a process killed at each step leaves behind

1.  During the settings read (`config_file.rs:262-347`):
    nothing.
    Only a read happens;
    the kernel closes the descriptor when the process dies.
    No notice is owed,
    and no later command can see that the read was cut short.
2.  During the Git configuration snapshot (`config_snapshot.rs:612`,
    `:662-693`):
    nothing.
    The child is watched (`transaction/transaction_git_step_bounds.rs:52`),
    killed with the process's group,
    and a listing is never cached on disk.
3.  During the rules load (`scanner_adapter.rs:213`):
    nothing in the worktree.
    The scanner's compiled-rules cache is a private temporary published by rename,
    and a killed compile leaves a temporary no reader opens
    (already recorded in
    `doc/planning/cli-git-commit-checks-route-findings-on-main.md`,
    "What I checked and found sound").
4.  During dependent-version planning:
    nothing.
    `dependent_version_lifecycle.rs` reads through the candidate store
    (`:364-420`) and writes no file;
    the plan is held in memory.
5.  After a `direct-fix` correction is prepared and before the renames
    (`direct_fix_install.rs:283-300`):
    a `.cli-git-direct-fix-<suffix>.old` copy of every file the fix read,
    and a `.new` beside it,
    stay in the worktree,
    and nothing ever removes them.
    The next `fix` stages them as content
    (`candidate/candidate_prediction.rs:317-336`,
    `git add --all --`),
    which is already recorded as that route's finding 5.
6.  Between the renames (`direct_fix_install.rs:332-341`):
    some dependents carry the bumped version and some do not,
    and no record says which.
    Each file is a complete document,
    so nothing is half written,
    but the workspace is inconsistent until the fix is run again,
    and the next run refuses the files already renamed
    (their worktree bytes no longer equal the bytes it read,
    `:263-279`) unless it is `fix --all`,
    which recomputes from the worktree.
    Read from the code;
    see "What I could not determine".
7.  During a commit's policy pass:
    the dependent-version step has written nothing to lose,
    because it planned nothing.
    The private index and scratch directory a killed commit leaves are the
    commit route's,
    not this one's.

Work cannot be lost by this route before item 5;
from item 5 on,
the residue is the direct-fix install's,
whose loss window (a write that arrives between the read and the rename) is
already recorded in
`doc/planning/cli-git-commit-checks-route-findings-on-main.md`
as its first finding.
No state in this route can be applied to the wrong place:
the settings file is one fixed name at the worktree's top level,
and the planner names files by their bytes from the candidate state.

## Waits and their ceilings

- The settings read itself (`config_file.rs:262-347`)
  is an ordinary file read with **no ceiling**.
  `symlink_metadata`,
  `File::open` and `read_to_end` block for as long as the
  filesystem takes,
  and no thread the wrapper owns can end them.
  On a filesystem that stops answering
  (a suspended encrypted volume,
  a FUSE mount whose daemon is gone,
  a removed network filesystem)
  every index-writing command blocks with no output.
  This is the one wait in the route that the owner's
  "nothing should ever wait forever or indefinitely" cannot reach;
  see "What I could not determine".
- The Git configuration snapshot is a quick step:
  `config` is in `PLUMBING_SUBCOMMANDS` (`push/child_class.rs:104-118`),
  so it is ended after 60 seconds without sign of work
  (`:69`,
  `QUERY` quiet window)
  and at the plumbing ceiling of two minutes
  (`service/wait_ceilings.rs:378`,
  applied by `transaction_git_step_bounds.rs:52-70`).
- Candidate and object reads of the planner are bounded the same way:
  `git cat-file --batch` runs under a `ChildWatch` at the object ceiling of ten minutes
  (`candidate/candidate_reader.rs:252`,
  `service/wait_ceilings.rs:395`)
  with a five-minute quiet window (`push/child_class.rs:76`).
- The rules load reads files through the scanner library with no ceiling of its own,
  inside the bounded forbidden-strings policy pass;
  the policy has no wait and no retry.
- Nothing in this route waits on the network,
  starts a package manager,
  sleeps,
  polls,
  or retries.
  `rg 'sleep|retry|Instant|Duration'` over `config/` and `dependent_version/`
  finds no production hit.

## Locks and why each exists

There is no lock in either folder:
`rg 'flock|lockf|fcntl|Mutex|RwLock|Condvar'` over
`config/` and `dependent_version/`,
test sidecars excluded,
finds nothing.
The two things that look like shared state are not locks:

- `invocation_config.rs:71` (`EARLIER_NAME_SAID`) is an `AtomicBool`
  that makes the earlier-name notice say once per process.
  Nothing waits on it and two readers may both say it after a race.
- The settings file is read without any exclusion,
  so a writer replacing it during a read is not serialized against.
  What that costs is in "What I checked and found sound".

The install of a fix takes no lock either:
exclusive creation of the sibling files
(`direct_fix_install.rs:210-231`,
`create_new`),
rename over each destination,
and a byte comparison of the real index before and after.
An interleaved `git add` therefore makes the fix roll back
rather than lose an entry.

## Reproductions

All ran against the debug binary built from this revision
(`mise run //package/git-policy/cli:native:build`,
two minutes fifty seconds),
on fixtures under
`${HOME}/temp/agent/pi-config-dependent-version-audit/runs`,
each a repository with `git init`,
a settings file,
and one workspace of `package/module/a` and `package/module/b`.
Every command ran under `timeout`,
through a `bin/git` link to that binary,
with `HOME`,
`GIT_CONFIG_NOSYSTEM`,
`GIT_CONFIG_GLOBAL` and the commit identity set,
and the commit fixtures started the build's own service process
(`monochromatic-git-policy-cli service run --path <socket>`),
so nothing touched the installed wrapper or its running service.

### A wrapped commit does not bump the dependent

Script:
`repro-1-commit-does-not-bump.ts`.
`@s/a` is raised from `1.0.0` to `1.1.0`,
staged,
and committed through the wrapper with the policy at `error`:

```text
check --all: code 1
{"schemaVersion":1,"sequence":0,"type":"finding","trigger":"direct-check",\
"policyId":"mono/dependent-version-bump","severity":"error",\
"code":"mono/dependent-version-bump/dependent-version-stale",\
"message":"@s/b reaches a package bumped in this commit (@s/a); bump it from 2.0.0 to 2.0.1 in the same commit.",\
"path":"package/module/b/package.json","fix":"available"}

wrapped commit: {"code":0,"stdout":"[main 8defe7a] bump-a\n","stderr":""}
HEAD subject and files: "bump-a\n\npackage/module/a/package.json\n"
@s/b in the commit: version 2.0.0
@s/b worktree copy: version 2.0.0
check --all after the commit: {"code":0,"stdout":"","stderr":""}
fix --all: {"code":0,"stdout":"","stderr":""}
@s/b after fix --all: version 2.0.0
```

The commit lands with one file,
the dependent keeps its version in both the commit and the worktree,
no event names it,
and after the commit there is nothing left to notice:
the raise is in `HEAD` now,
so `check` is silent and `fix` changes nothing.
The control in the same fixture:
`check --all` before the commit exits 1 and names the dependent with a fix available.

### The publish set can be empty, and the ripple then reports success

Script:
`repro-2-publishable-list.ts`.
Three independent fixtures,
each with `@s/a` raised and `@s/b` depending on it:
a `packages:` list narrower than the publish set above it,
a single list holding the publish set,
and no registry configuration at all.

```text
--- a narrower list above the publish set
the first packages: line is "          packages:"
check --all: {"code":0,"stdout":"","stderr":""}
fix --all: {"code":0,"stdout":"","stderr":""}
@s/b after the fix: version 2.0.0

--- one list, the publish set
the first packages: line is "    packages:"
check --all: code 1, one dependent-version-stale finding, fix "available"
fix --all: code 0
{"schemaVersion":1,"sequence":0,"type":"fix-summary","trigger":"direct-fix","passes":1,\
"changedPaths":["package/module/b/package.json"]}
@s/b after the fix: version 2.0.1

--- no registry configuration at all
the first packages: line is "(no file)"
check --all: {"code":0,"stdout":"","stderr":""}
fix --all: {"code":0,"stdout":"","stderr":""}
@s/b after the fix: version 2.0.0
```

`fix` and `check` both exit 0 and print nothing,
which is the same output a clean workspace gives.

### The settings refusals, and which commands they stop

Script:
`repro-3-config-refusals.ts` (eight cases,
each its own fixture) and
`repro-5-config-reach.ts` (the reach).
Each case is one commit of one staged edit,
with one difference in the settings file,
and the order below is the order that script prints.

```text
a symbolic link to a valid file: code 2
{"type":"engine-failure","code":"config-invalid","message":"Configuration path must not be a \
symbolic link: <repo>/monochromatic-git-policy-cli.config.jsonc. Replace it with a regular file."}

a file larger than the one mebibyte cap: code 2
"Configuration file <repo>/monochromatic-git-policy-cli.config.jsonc is larger than 1048576 bytes."

a file that is not UTF-8: code 2
"Configuration file <repo>/monochromatic-git-policy-cli.config.jsonc is not UTF-8 text: \
invalid utf-8 sequence of 1 bytes from index 2."

an empty file: code 2
"<repo>/monochromatic-git-policy-cli.config.jsonc: JSONC syntax error at byte 0: \
JSONC document root must be an object or array"

a file holding only a comment: code 2
"<repo>/...config.jsonc: JSONC syntax error at byte 15: JSONC document root must be an object or array"

an unknown policy ID: code 2
"<repo>/...config.jsonc: Unknown policy ID: mono/no-such-policy. Shipped policies: require-root, ..."

a required rulesFile that is missing: code 2
{"type":"engine-failure","code":"policy-incomplete","trigger":"pre-forward",\
"policyId":"security/forbidden-strings","message":"monochromatic-git-policy-cli could not load the \
forbidden-strings rules, so no file was scanned: read rules <repo>/rules-that-are-not-there.txt: \
No such file or directory (os error 2)."}

the same fixture with --no-enforce-security/forbidden-strings: code 0
{"code":0,"stdout":"[main 6c09667] change\n","stderr":""}

HEAD is still: "base\n"   (every refused case)
```

Reach,
with the settings file a symbolic link in a repository with one uncommitted edit:

```text
git status: code 0, " M a.txt" and the two untracked files
git diff: code 0, "a.txt | 1 +"
git log: code 0
git add: code 2, config-invalid, the symbolic-link message
git commit: code 2, config-invalid, the symbolic-link message
staged after the refused add: ""
```

So an unusable settings file stops exactly the index writers,
names the file and the cause every time,
and leaves the repository untouched.

### A manifest that declares `version` twice is refused, not half edited

Script:
`repro-6-duplicate-version.ts`.
`@s/b` declares `"version": "2.0.0"` twice,
which `JSON.parse` reads as the last one.

```text
check --all before the fix: code 1, dependent-version-stale, 2.0.0 to 2.0.1
fix --all: code 2
{"type":"engine-failure","code":"policy-incomplete","trigger":"direct-fix",\
"message":"mono/dependent-version-bump could not finish: package/module/b/package.json declares \
version \"2.0.1\", expected \"2.0.0\". Correct the file and retry, or pass \
--no-enforce-mono/dependent-version-bump to skip this policy for one command."}
@s/b after the fix: unchanged, both keys still "2.0.0"
fix --all again: the same refusal
```

The convergence loop runs a second pass over the corrected bytes,
where the first key is `2.0.1` and the last is still `2.0.0`,
so the rewrite is refused whole rather than written as a no-op.

### The release task's ripple command, as spelled

Script:
`repro-4-release-command.ts`.
The root task `changeset:version` runs
`monochromatic-git-policy-cli monochromatic-git-policy-cli fix --all --policy mono/dependent-version-bump`
(`mise.toml:1449-1453`).
The repeated word is not a mistake:
run by its own name,
the program reads the first word as the command word,
and the short form `fix --all` is forwarded to real Git instead
(`git: 'fix' is not a git command`).

```text
as the task spells it: code 0
{"type":"fix-summary","trigger":"direct-fix","passes":1,"changedPaths":["package/module/b/package.json"]}
@s/b after the task command: version 2.0.1
without the repeated word: code 1, "git: 'fix' is not a git command."
```

## Findings ranked by what they risk losing

### 1. The ripple reads whichever `packages:` line comes first, and an empty publish set reads as success

`dependent_version/dependent_version_publishable.rs:107-132`
(`read_publishable_names`) walks the registry configuration for the **first**
line whose trimmed text is exactly `packages:`,
then takes the single-quoted items after it and stops at the first line that is not one.
`dependent_version_plan.rs:229-231` turns "no registry configuration" into an empty plan,
and `plan_dependent_bumps`
(`dependent_version_graph.rs:182-215`) bumps only names in that list.

Sequence:

1.  `package/config/pnpr/config.yaml` is generated
    (its header names `@monochromatic-dev/config-pnpr`).
    Today the first `packages:` line is the CI workload's publish list
    (`package/config/pnpr/config.yaml:29`),
    and the second (`:207`) is a glob map.
2.  A second workload block,
    or a change of the generator's block order,
    or a change of the list's quoting style,
    puts another `packages:` list above the publish set,
    or none at all.
3.  `git monochromatic-git-policy-cli fix --all`,
    which the release task runs,
    exits 0 with no output;
    `check --all` exits 0 with no output too.
4.  The dependents that the raised package reaches keep their old versions,
    and both commands report the same silence a clean workspace produces.

Wrong result:
a publishable dependent's version does not move with the dependency it
bundles,
and the two commands that exist to notice are the ones that stay silent.
This ranks first because it defeats the only signal a person has:
finding 2's commit does not notify either,
so with an empty publish set nothing in the wrapper ever mentions the stale dependent.
Reproduced above,
with the control that names exactly what changed (the first `packages:` line).
Owner decision touched:
`doc/decision/private-npm-registry.md:158-164`
("A cli-git commit policy bumps dependents by patch,
transitively,
in the same commit as a hand bump")
and `doc/decision/cli-git-policies-platform.md:744-756`
("patch-bumps every publishable dependent ... adding their manifests to the
commit through tracked targets").
Suggestion (suggestion):
read the list the generator writes by its structure rather than by position,
or have the generator emit a marker line the reader requires,
and refuse the ripple with a named failure
(`policy-incomplete`,
naming the file and the line it read)
when the publish set comes out empty while a workspace version was raised.
The second half alone would have turned both reproductions into a message.

### 2. A wrapped commit never plans a dependent bump, and nothing says so

`dependent_version/policy_dependent_version.rs:309-313` returns "clean"
for every `pre-forward` trigger,
`:320` sets `forwards_commit: false` for the request it builds elsewhere,
and `dependent_version_policy.rs:166` then makes `should_plan` false.
The trigger table (`policy/policy_trigger.rs:131-136`) does not include
`post-commit`,
so a landed commit is never checked for a stale dependent either.
The commit path's own stage request is `Trigger::PreForward`
(`commit/commit_command.rs:207-212`),
which is the trigger that returns early.

Sequence:

1.  A person raises `@s/a`'s version by hand and commits,
    naming that manifest,
    as `add-explicit` and `commit-only` require.
2.  `commit/commit_command.rs:215` hands the prepared commit to the service.
    The policy pass in `commit/commit_convergence.rs:259-274`
    runs with `can_apply_patches = true`,
    but the dependent-version arm has already returned.
3.  The commit lands with one file.
    No finding,
    warning or note names `@s/b`.
4.  After the landing the raise equals `HEAD`,
    so `check --all` and `fix --all` are silent as well
    (reproduced above).
5.  Only the release task's `fix --all`
    (`mise.toml:1449-1453`) bumps it before publishing,
    if that task runs.

Wrong result:
the committed tree pins the old dependency,
the local check cannot see it any more,
and the owner's decided behavior
(in the same commit as the hand bump)
does not happen at all.
The person is told nothing,
which is the part that turns a design gap into a silent one.
Owner decisions and records touched:

- `doc/decision/private-npm-registry.md:158-177`:
  "A cli-git commit policy bumps dependents by patch,
  transitively,
  in the same commit as a hand bump",
  and
  "Owner decision (2026-09-15):
  extend the engine so a policy pass can add paths
  to the commit and write those fixes to the worktree when the worktree copy
  still matches `HEAD`".
- `doc/decision/cli-git-policies-platform.md:744-756`:
  "`dependent-version-bump` runs before commit ... adding their manifests to
  the commit through tracked targets.
  It is the first policy that adds paths."
- `package/git-policy/cli/SPEC.md:207-208` says the opposite:
  "A commit adds no paths of its own:
  only the paths it selected reach the landing."
- `doc/handover/cli-git-native-dependent-version.md:1028-1037` records the state
  at port time:
  "`git commit` cannot reach the planner yet ... Commit planning changes both
  together,
  and belongs to the commit transaction port."
  The transaction landed
  (`doc/handover/cli-git-native-transactions.md`,
  slices 6 and 7);
  the pair did not.
- `package/git-policy/cli/SPEC.md:110` lists the policy and says nothing else
  about it anywhere,
  so the contract file neither states the decided behavior nor records that it
  was dropped.

Reading from the code and from those records,
which disagree with each other:
the owner has to say which stands.
Suggestion (suggestion):
if the 2026-09-15 decision stands,
carry the decision's two halves into the commit path
(a policy pass that may add a path,
and the worktree install of its rewrites),
and make `forwards_commit` true for the commit's stage request;
if the SPEC's "no added paths" is the newer decision,
amend both decision records and have the commit report the stale dependent
as a finding,
so the silence ends even when the bump is left to the release task.

Resolved (2026-11-10):
the owner answered the question with
"Build what the decision says (Recommended)",
whose option read
"A commit adds the dependents' manifests and updates them in the worktree when that copy still matches `HEAD`".
The commit path now plans and applies the ripple,
`SPEC.md` states the exception and quotes that answer,
and every dependent the commit may not touch is named in a note instead of being
silently left behind.
Branch `feat/git-policy-cli-commit-adds-the-dependents-it-bumps`
(commits `947920acf0` and `f638a19a7e`).

## What I checked and found sound

- **An unusable settings file fails closed and stops only index writers.**
  Reproduced above with six causes,
  plus the reach test:
  `status`,
  `diff` and `log` ran while `add` and `commit` refused,
  and the refused add left the index empty.
  The verdict is the same for a mapped `check`/`fix`
  (`management_direct_commands.rs:311-330`),
  where the event goes to standard output as the direct-command contract says.
- **A rules file that cannot be read fails closed and names it.**
  Reproduced:
  `policy-incomplete` naming the configured path and the system's reason,
  and the same commit landing once
  `--no-enforce-security/forbidden-strings` is passed.
  The precedence is configuration first,
  then the variable,
  then the default file (`scanner_selection.rs:124-176`),
  which matches the owner's decision of 2026-10-05 recorded in that module.
  The refusal message names the file but not the hatch,
  so the way past a broken rules file is documented rather than shown.
- **A concurrent rewrite of the settings file fails closed.**
  The two states a truncating editor passes through are refused by name
  (an empty file,
  a comment-only file:
  reproduced).
  A partial read can only be accepted as a whole document when a prefix of the
  new content is itself a complete JSONC object,
  and a JSON object is only complete at its own end,
  so a torn read cannot silently drop the policies that follow the tear.
  What a concurrent rewrite does cost is a refused command for the length of
  the write,
  with the syntax error's byte offset as the message.
  Read from the code,
  with the two tear states reproduced.
- **Git's own configuration never becomes a guessed value.**
  A value the snapshot does not model is `Lookup::Unhandled`,
  and the consumer starts Git for that key
  (`config_snapshot.rs:66-110`);
  a snapshot that cannot be read or parsed returns `None`
  and every consumer asks Git instead (`:612`,
  `:662-693`).
  This is the same conclusion the commit-checks audit reached for the snapshot
  and it holds at this revision.
- **A bump never rewrites a manifest whose worktree copy differs from `HEAD`.**
  `policy_dependent_version.rs:355` proposes a correction only on `direct-fix`;
  `correction_of` (`:233-296`) admits an unselected file only when `HEAD`,
  the private index and the worktree agree
  (`policy_added_path.rs:106` and `:159`),
  and the installer repeats the worktree half of that check
  (`direct_fix_install.rs:273-278`)
  and refuses the whole fix when the real index moved (`:344-350`).
  The binary controls in
  `binary/binary_dependent_version_tests.rs:171-222`
  cover the refusal and the admitted case.
  The remaining loss window (a write between that check and the rename) is the
  commit-checks audit's first finding,
  not a new one.
- **A duplicated `version` key cannot be half edited.**
  Reproduced:
  the second convergence pass sees the disagreement and the fix
  refuses the file by name.
- **The planner reads candidates,
  never the live worktree.**
  `dependent_version_lifecycle.rs:364-420` answers every read from the
  candidate store,
  so an edit made while `fix` runs cannot change what it planned;
  an edit made before the install stops the install
  (`direct_fix_install.rs:273-278`).
- **The release task's ripple command is correct as spelled.**
  Reproduced above:
  the doubled word works,
  and the shorter form is forwarded to real Git and fails.
  Nobody should "fix" the task by deleting a word.
- **No lock,
  and every wait but a plain file read has a ceiling.**
  Listed under "Waits and their ceilings";
  the only unbounded wait is the settings and rules file read itself.

## What I could not determine

- **A kill between two renames of one fix.**
  `direct_fix_install.rs:332-341` renames each prepared file in a loop with no
  journal and no phase marker,
  and the throwaway harness has no seam there
  (`transaction/transaction_phase.rs` arms phases only for an isolated merge and
  a commit headed for the service).
  The partial ripple in "What a process killed at each step leaves behind"
  is therefore read from the code,
  not observed.
  What the next `fix --all` does with a half-bumped workspace is also read only.
- **Whether the settings file can block a command on a hung filesystem.**
  The read has no ceiling by construction,
  and this host has no filesystem that stops answering to test it with:
  a FUSE mount would need a fixture whose daemon I stop mid-read,
  which is a probe I did not build.
- **Whether the commit path ever fills a dependent in another shape.**
  `commit/commit_worktree_records.rs:1-9` and `:139-190` record only
  final-newline corrections for the post-landing worktree install,
  and no other corrected-dependent path exists in the commit modules
  (`rg 'dependent' commit/` finds nothing).
  That is a reading of the whole commit folder,
  not a live two-command race.
- **What the installed release does.**
  Every probe used the debug binary built from this revision,
  by path,
  through a fixture `git` link,
  with a service started from that binary.
  The installed wrapper and its running service were not touched,
  so the shipped release was not exercised.
- **Whether the generator can emit a second `packages:` list today.**
  The generator package was not read;
  finding 1 rests on the reader's rule (first matching line wins)
  and on the shape of the file it reads now,
  and the reproduction shows what the reader does when such a list exists.
- **The reserved-code rule of the local appendix.**
  `forbidden-strings.append.local.txt` is gitignored and was not read,
  so whether a word in this page matches a local rule is unknown until a commit
  is scanned.

## What to inspect and how to respond

Inspect,
in this order:

1.  finding 2's four records against
    `dependent_version/policy_dependent_version.rs:309-363`,
    `policy/policy_trigger.rs:131-136`
    and `commit/commit_command.rs:207-212`,
    then decide which of the two decisions stands;
2.  finding 1 against
    `dependent_version/dependent_version_publishable.rs:107-132`,
    with `package/config/pnpr/config.yaml:29` and `:207` beside it;
3.  the reproductions in
    `${HOME}/temp/agent/pi-config-dependent-version-audit`
    (`repro-1` to `repro-6`).

Respond with a decision on finding 2
(keep the 2026-09-15 decision and build the commit-time bump,
or amend the decision records to match the SPEC),
and with whether finding 1 should be fixed now
or only made loud.
The audit changed no production code,
so no other action is pending.
