# Policy and candidate route findings on `main`

Status:
audit,
2026-11-09.
Audited revision:
`main` at `4fcd560cea`,
in the worktree
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-policy-and-candidate-route-findings-on-main`
(fast-forwarded to `main` before writing;
every file this page cites is byte-identical to the tip).
No production code was changed and no test was committed.
The scratch scripts are in
`${HOME}/temp/agent/policy-candidate-audit`,
and each quoted result names the script that produced it.
Linux only,
Git 2.56.0 only.

Scope:
the judgement half of the check path:
which content each policy judges at which lifecycle point,
how the engine decides what runs and what stops,
and what a person is told.
The route is

- `package/git-policy/cli/src/native/policy/` (22 production files,
  6,864 lines,
  test sidecars excluded from every count on this page),
- `package/git-policy/cli/src/native/candidate/` (23 production files,
  6,841 lines),
- the passes that run them
  (`invocation/wrapped_command.rs:475`,
  `invocation/management_direct_commands.rs:237`,
  `landing/service_prepare.rs:286-300`,
  `commit/post_commit.rs:161-183`),
- the selection a commit's candidates come from
  (`commit/commit_transaction.rs:213`,
  `commit/commit_index.rs:132`,
  `commit/commit_index_selection.rs:72`),
- and the `add-explicit` decision,
  which is a policy over `git add`
  (`command/command_add.rs`,
  `command/rule_add_explicit.rs`).

Purpose:
the wrapper exists to judge content,
and `doc/handover/cli-git-rust-implementation.md`
("Open review findings",
recorded 2026-11-09)
lists eleven open findings and three route audits
(the copy,
the push and the discarding routes),
each closing with the same sentence:
no review covers the next route.
This page covers the policies and the candidates:
what each policy reads,
at which lifecycle point,
what the engine does with an answer,
and what a person sees.
Findings 1 and 2 are reproduced;
both are ways a policy that is configured,
ported and tested still judges nothing or judges the wrong set.

## What this page does not repeat

`doc/planning/cli-git-commit-checks-route-findings-on-main.md`
(2026-11-09,
`main` at `cf63cacad8`)
already audited these two folders for
what a killed process leaves,
which waits have no ceiling,
which locks exist,
and what two concurrent commands do to one repository,
and it lists seven findings there.
Every one of those findings is still open at this revision;
none of them is about what a policy decides.
This page therefore says nothing new about the direct fix's install,
the candidate reader's size,
the private index's permissions,
the check's waits,
or the lock inventory,
and cites that page where the two meet.
What is left,
and what this page covers,
is the answer to one question per policy:
which bytes or names reach it,
at which lifecycle point,
and what happens to its answer.

## The route, step by step

1.  Five lifecycle points exist,
    and every event names one
    (`policy/policy_trigger.rs:49-70`):
    `pre-forward`,
    `post-commit`,
    `manual-push`,
    `direct-check`,
    `direct-fix`.
    Every one of them is ported,
    so a stage never answers "unavailable" for a whole lifecycle
    (`:181-192`).
2.  A wrapped command runs the pre-forward pass once,
    after the settings load and before Git
    (`invocation/wrapped_command.rs:475`),
    except for a commit:
    a real `git commit` enters the commit transaction first
    (`:458`),
    and the transaction runs its own pass over the private index
    (`landing/service_prepare.rs:286-300`).
    A wrapped `git push` that is not a dry run adds a manual-push stage
    (`:481-520`).
3.  A landed commit that a commit command made is judged again by the
    post-commit gate
    (`commit/commit_command.rs:256`),
    against the landed tree,
    and a blocked gate keeps the commit local and exits `2`
    (`commit/post_commit.rs:49`).
    `git merge`,
    `git cherry-pick`,
    `git revert`,
    `git am` and `git rebase` land commits without this gate:
    the only trigger in the whole codebase for a landed commit is
    `commit/post_commit.rs:178`,
    reached from the commit command alone.
4.  A direct `check` or `fix` prepares its own candidates from the
    worktree,
    by explicit pathspecs or `--all`
    (`invocation/management_direct_commands.rs:237`,
    `:279-297`),
    and runs every policy the trigger selects
    (`policy/policy_trigger.rs:120-158`).
5.  Inside a pass,
    three stages run in a fixed order:
    the built-in policies,
    then the fixed transforms of a forwarded command,
    then the optional policies
    (`policy/policy_pass.rs:137-198`).
    A stage that does not complete ends the pass with the arguments it was
    given,
    and nothing after it runs.
6.  The stage asks four questions about each policy,
    in this order
    (`policy/policy_engine.rs:270-289`):
    does the policy declare this trigger,
    does a `--policy` filter select it,
    is it escaped for this invocation,
    and is its effective severity `off`.
    Only then is the policy run.
7.  A policy answers one of three things
    (`policy/policy_engine.rs:104-140`):
    findings,
    "unavailable" and what it needs,
    or "failed" with an engine-failure code.
    No shipped policy answers "unavailable" at this revision
    (the only producer of `PolicyOutcome::Unavailable` is the engine's own
    match arm,
    `policy/policy_engine.rs:344`).
8.  An error finding stops the pass unless `--cli-git-keep-going` was given
    (`:407-412`);
    any policy that offers a fix ends the stage as "proposed"
    whatever its severity,
    and the fixing loops apply the corrections and run the whole order again
    (`:398-406`).
9.  The pass's exit code is `1` for an error finding,
    a transform rejection or a pending correction,
    `2` for an engine failure or an unavailable policy,
    and `0` otherwise
    (`policy/policy_engine.rs:435-451`).
10. What a content policy reads comes from one prepared state
    (`policy/policy_content.rs`):
    the candidates of this lifecycle,
    prepared on the first read and shared by every later policy,
    with the corrections of a fixing loop laid over them.
    Nothing in this route reads the live worktree:
    candidates come from the private index or from objects,
    which the commit-check audit already recorded as sound.

## The registry, as it decides

Ten policies exist,
in this order
(`policy/policy_registry.rs:147-235`).
The names are the configuration keys and the `policyId` of every event.

1.  `require-root`,
    error by default,
    not warn-safe,
    pre-forward and direct-check.
2.  `linked-worktree-only`,
    error,
    not warn-safe,
    pre-forward.
3.  `branch-worktree-only`,
    error,
    warn-safe,
    pre-forward.
4.  `add-explicit`,
    error,
    not warn-safe,
    pre-forward.
5.  `final-newline`,
    warn,
    warn-safe,
    every trigger.
6.  `markdown/autofix`,
    warn,
    warn-safe,
    off unless listed,
    every trigger.
7.  `mono/forbidden-root-context`,
    error,
    warn-safe,
    off unless listed,
    pre-forward and direct-check.
8.  `mono/dependent-version-bump`,
    error,
    not warn-safe,
    off unless listed,
    pre-forward,
    direct-check and direct-fix.
9.  `mono/catalog-specifiers`,
    error,
    not warn-safe,
    off unless listed,
    pre-forward,
    post-commit,
    manual-push and direct-check.
10. `security/forbidden-strings`,
    error,
    not warn-safe,
    off unless listed,
    pre-forward,
    post-commit,
    manual-push and direct-check.

The trigger sets are the table the engine consults
(`policy/policy_trigger.rs:120-158`),
and it matches `SPEC.md` "Policy order and stopping"
and the per-policy sections that follow it.
Two consequences matter for the findings:

- the three policies that can protect a commit from content that is wrong
  rather than dangerous
  (`final-newline`,
  `markdown/autofix`,
  `security/forbidden-strings`,
  and `mono/catalog-specifiers`)
  are judged again after a landing,
  so a hole in the commit's own pass still reaches the person,
  late;
- `mono/forbidden-root-context` and `mono/dependent-version-bump`
  have no post-commit trigger,
  so a hole in the commit's own pass is silent for both.

## What each content policy reads

A "candidate" is a path this lifecycle changes,
with a mode,
a change kind,
and the object its bytes come from
(`candidate/candidate_version.rs`).
The six content policies differ only in how much of it they read.

- `final-newline` reads every added or modified regular or executable
  candidate's bytes,
  skips the two generated-output families and paths whose `text` attribute
  is unset,
  and asks Git for that attribute only for the files it would otherwise
  change
  (`policy/policy_final_newline.rs:87-114`
  and `:172-206`,
  `:227-258`).
  A correction is offered wherever a caller can apply patches
  (`:271`),
  which is the commit transaction and the direct fix.
- `markdown/autofix` reads Markdown candidates through the linked linter and
  offers link corrections
  (`policy/policy_checks.rs:518-526`,
  `markdown/policy_markdown.rs:444`).
- `mono/forbidden-root-context` reads the pathname alone
  (`policy/policy_root_context.rs`).
- `mono/dependent-version-bump` reads the whole candidate state,
  including files that are not candidates,
  through `store_with_corrections`
  (`policy/policy_content.rs:677-690`).
- `mono/catalog-specifiers` reads the bytes of a candidate named
  `package.json` and the name of every other candidate
  (`policy/policy_catalog_specifiers.rs:147-180`).
- `security/forbidden-strings` reads the bytes and the pathname of every
  scannable candidate,
  leaves out deletions,
  the rules files and the scanner's own rule sources
  (`candidate/scanner_selection.rs:313-330`),
  and skips the bytes of a candidate whose blob a parent tree already holds,
  checking the pathname only,
  which the commit-check audit recorded as sound
  (`candidate/candidate_known_content.rs:271-320`).

The rules file is chosen configuration first,
then `FORBIDDEN_STRINGS_RULES`,
then `forbidden-strings.local.txt`
(`candidate/scanner_selection.rs:124-176`),
and a configured or named file that is missing fails the policy closed
while only the default file may be absent
(`:96-110`).

## What a process killed at each step leaves behind

Nothing durable is new in this route.
Nothing in `policy/` or `candidate/` writes a file except
the direct fix's install
(`policy/direct_fix_install.rs:161-231`)
and the private index's own directory
(`candidate/candidate_private_index.rs:287`),
and both are the commit-check audit's findings 5 and 6.
A kill inside a pass loses findings,
corrections held in memory,
and the private index,
which that page already says.
No event is written to disk by this route,
so a killed command leaves no half-written record of what it decided.

## Waits and their ceilings

- Nothing in `policy/` or `candidate/` sleeps,
  polls,
  or retries on a clock:
  `rg 'sleep|Instant|Duration|retry|poll'` over both folders,
  test sidecars excluded,
  finds one `SystemTime` read that decides whether fd and device identities
  are reused
  (`policy/policy_checks.rs:317-330`)
  and the child-start retry it borrows from the process layer
  (`candidate/candidate_reader.rs:223`).
- Every wait in the route is a Git child,
  bounded by the same quiet windows and ceilings every other child of the
  wrapper gets,
  and the commit-check audit's finding 3 records the one place the route
  waits with no ceiling of its own.
- The scanner is linked in,
  so a scan is a call and not a process,
  and the Markdown linter's own ceiling is that audit's list too.
- No policy waits on the network,
  starts a package manager,
  or reads a user file outside the repository:
  the rules files and the saved copies are the only named files the route
  opens,
  both through the paths the configuration or Git gave.

## Locks and why each exists

There is no repository lock in either folder:
`rg 'flock|lockf|fcntl|Mutex|RwLock|Condvar'` over both,
test sidecars excluded,
finds one `static Mutex`,
a one-entry cache of the last commit's tree records
(`candidate/candidate_store_commit_listings.rs:55`).
Nothing waits on it:
a poisoned or cold cache only costs a listing.
The private index is the only shared state,
and the commit-check audit recorded that a candidate version invalidated
by a correction is refused rather than read
(`candidate/candidate_store.rs:542-550`).

## Reproductions

All ran against the debug binary built from this revision
(`mise run //package/git-policy/cli:native:build`,
2 minutes 19 seconds),
whose version line is
`monochromatic-git-policy-cli 0.6.1-alpha.5, shadowing git at /usr/bin/git`,
on fixtures under
`${HOME}/temp/agent/policy-candidate-audit/runs`,
each a repository with `git init`,
every command through a `bin/git` link to that binary,
with `HOME`,
`GIT_CONFIG_NOSYSTEM`,
`GIT_CONFIG_GLOBAL` and the commit identity set,
and the commit fixtures reaching the build's own commit service through the
user service manager.
The scripts are `probe-*.ts` beside a shared `probe-lib.ts`.
Nothing touched the installed wrapper or its running service.

### A staged bad final newline is corrected, unless literal pathspec mode is on

Script:
`probe-8-literal-mode.ts`.
Each fixture stages a two-LF file through the escape hatch
(`git add --no-enforce-final-newline -- tracked.txt`,
which is the only way to stage it at `error` severity),
then commits with its path named.
The control:

```text
commit: code 0
  stdout "[main ce34997] control\n"
  stderr "{\"schemaVersion\":1,...\"type\":\"fix-summary\",\"trigger\":\"pre-forward\",\"passes\":1,\"changedPaths\":[\"tracked.txt\"]}\n"
  HEAD bytes: "two\n"
```

The same commit,
written `git --literal-pathspecs commit ...`:

```text
commit: code 2
  stdout "[main 7f11c30] literal flag\n"
  stderr "...\"type\":\"finding\",\"trigger\":\"post-commit\",\"policyId\":\"final-newline\",\"severity\":\"error\",...\"path\":\"tracked.txt\"...\n
          ...\"type\":\"commit-landed\",\"oid\":\"7f11c308...\",\"outcome\":\"post-commit-blocked\"...\n"
  HEAD bytes: "two\n\n"
```

The same commit with `GIT_LITERAL_PATHSPECS=1` in its environment produced
the same result and the same bytes.
The commit's own pass produced no event at all,
the correction never happened,
and the only notice came after the content had landed.

### A forbidden string blocks the commit, unless literal pathspec mode is on

Script:
`probe-2-commit-pathspec.ts`.
The fixture lists `security/forbidden-strings` at `error`
with a private rules file whose one rule is the literal `audit-canary-9f2`,
commits a clean base,
then adds a file carrying the canary.
The `git add` is refused in both runs:

```text
add leak: code 1
  stderr "...\"policyId\":\"security/forbidden-strings\",\"severity\":\"error\",...\"path\":\"leak.txt\"...\n"
```

Without literal mode,
the commit that names the same file is refused too,
and nothing lands:

```text
commit leak by path: code 1
  stderr "...\"trigger\":\"pre-forward\"...\"path\":\"leak.txt\"...\n"
HEAD: "base\n"
```

With `GIT_LITERAL_PATHSPECS=1` in the environment,
the same commit lands the secret:

```text
commit leak by path: code 2
  stdout "[main 563038e] leak\n"
  stderr "...\"trigger\":\"post-commit\"...\"path\":\"leak.txt\"...\n
          ...\"type\":\"commit-landed\",\"outcome\":\"post-commit-blocked\"...\n"
content in HEAD: "token = audit-canary-9f2\n"
```

The canary never appears in an event in either run:
the scanner's message names the line and the rule,
never the matched text.

### A root `CONTEXT.md` lands with no event at all under literal pathspec mode

Script:
`probe-13-root-context-hole.ts`.
The fixture lists `mono/forbidden-root-context` at `error`,
stages `CONTEXT.md`,
and commits it with `--literal-pathspecs`:

```text
commit: code 0
  stdout "[main c37452a] context\n"
  any event: true
HEAD subject: "c37452a context\n"
check --all afterwards: code 1
  stdout "...\"policyId\":\"mono/forbidden-root-context\"...\"path\":\"CONTEXT.md\"...\n"
```

The commit's whole output is the branch line:
no finding,
no engine failure,
no warning.
The same fixture without literal mode refuses the commit at the staged
`git add` and again at the commit (`code 1`,
`pre-forward`,
`root-context-forbidden`).
`check --all` is the only command that would have said anything,
and the policy has no post-commit trigger,
so nothing runs at landing
(`policy/policy_trigger.rs:120-158`).

### Five whole-tree pathspec spellings pass `add-explicit`

Script:
`probe-12-spelling-effect.ts`.
Each spelling runs in its own fixture with two untracked files,
`top.txt` and `sub/below.txt`,
and the staged set is read afterwards.
Rejected,
with `bulk-add-rejected`:

```text
["add","."]                 code 1  bulk-add-rejected: true
["add","./"]                code 1  bulk-add-rejected: true
["add","-A"]                code 1  bulk-add-rejected: true
["add","-u"]                code 1  bulk-add-rejected: true
["add",":/"]                code 1  bulk-add-rejected: true
```

Not rejected,
with the whole repository staged:

```text
["add",":/*"]        code 0  staged "A  sub/below.txt\nA  top.txt\n"
["add",":/**"]       code 0  staged "A  sub/below.txt\nA  top.txt\n"
["add",":(top)"]     code 0  staged "A  sub/below.txt\nA  top.txt\n"
["add","**"]         code 0  staged "A  sub/below.txt\nA  top.txt\n"
["add",":(glob)**"]  code 0  staged "A  sub/below.txt\nA  top.txt\n"
```

The positive control is the same fixture with `admin` scope:
`["add","sub"]`,
code `0`,
stages `sub/below.txt` and leaves `top.txt` untracked.

### `:(top)/*` fails inside Git

The same script's `["add",":(top)/*"]` case:
Git 2.56.0 dies in its own pathspec preparation and the wrapper reports
the prediction as unavailable:

```text
["add",":(top)/*"]           code 2  staged "?? sub/\n?? top.txt\n"
    "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"content-unavailable\",
      \"message\":\"monochromatic-git-policy-cli could not predict what git add stages: git add failed: fatal: oops in prep_exclude\"...}"
```

Nothing was staged,
the wrapper fails closed,
and the spelling cannot be used at all.
The message names Git's own text,
so a person cannot tell that the pathspec was the trigger.

### The control that keeps the rest honest

Script:
`probe-10-commit-all.ts`.
The `-a` route
(`git commit -a --no-enforce-only`),
which stages every tracked change into the private index,
is corrected at `error` and at `warn` severity,
the corrected bytes are installed into the worktree,
and `git status` is empty afterwards:

```text
commit -a: code 0
  stdout "[main 5ac27dd] all-error\n"
  stderr "...\"type\":\"fix-summary\",\"trigger\":\"pre-forward\",\"passes\":1,\"changedPaths\":[\"tracked.txt\"]}\n"
  HEAD bytes: "two\n"
  status: ""
```

So the corrections do reach a commit and do reach the worktree,
and the finding above is about the candidate listing,
not about the correction or the install.

## Findings ranked by what they risk losing

### 1. Literal pathspec mode empties a commit's candidate list, so every content policy passes **reproduced**

Sequence.
A person or a script sets `GIT_LITERAL_PATHSPECS=1`,
or writes `git --literal-pathspecs commit ...`.
The commit command enters the transaction,
the private index is built for the selection,
the paths named on the command line are staged into it with their own
spellings
(`commit/commit_index_selection.rs:72-90`,
`commit/commit_index.rs:340-357`),
and the candidate request is built from the paths the private index changes,
each written as `:(top,literal)<path>`
(`commit/commit_transaction.rs:303-315`).
The candidate store that lists them is given the caller's global options
without a mode argument
(`candidate/candidate_prediction.rs:534-543`).
Literal pathspec mode turns `:(top,literal)<path>` into a name that matches
no file,
so the listing returns no entries,
`staged_scope` answers "nothing selected"
(`candidate/candidate_prediction.rs:351-362`),
and every content policy sees zero candidates.

Wrong result.
`final-newline`,
`markdown/autofix`,
`mono/dependent-version-bump`,
`mono/catalog-specifiers`,
`mono/forbidden-root-context` and
`security/forbidden-strings` all report nothing for that commit,
so

- a commit that would have been refused for a secret lands it instead
  (`probe-2-commit-pathspec.ts`:
  `code 1` and no commit without the variable,
  `code 2` with the secret in `HEAD` under it);
- a commit that would have been corrected lands its uncorrected bytes
  (`probe-8-literal-mode.ts`:
  `two\n` against `two\n\n`),
  and the post-commit gate reports it afterwards;
- a root `CONTEXT.md` lands with no event at all,
  because that policy has no post-commit trigger
  (`probe-13-root-context-hole.ts`);
- the dependent bump a commit is required to make never happens and is
  never mentioned,
  which is the route
  `doc/planning/cli-git-config-and-dependent-version-route-findings-on-main.md`
  covers from the other end.

Why it is not caught.
The `Add` and `Direct` arms of the same function both push
`--no-literal-pathspecs` into the store's prefix
(`candidate/candidate_prediction.rs:492-495`,
`:512-520`),
and the function's own documentation names the hazard:
literal mode "would turn into names that match nothing,
leaving every change unchecked"
(`:397-407`).
The `Direct` arm got that line in commit `1899e041ef`
(`fix(git-policy-cli): direct candidate prediction reads pathspecs with magic under literal mode`,
2026-10-10),
which is the same defect for the direct route,
and the `Staged` and `StagedAtBase` arms were not changed.
The only control near this is a unit test over the request's shape
(`commit/commit_transaction_tests.rs:73-90`),
which asserts the `:(top,literal)` spelling and never runs it against Git.

Where else the same spelling breaks.
The worktree records a commit installs corrections into use the same magic
(`commit/commit_worktree_records.rs:107`),
and so does the landing's follow-up
(`landing/service_land_follow.rs:115`,
`:172-176`).
Under literal mode a correction that did happen would find no record and
install nothing.

Suggested fix.
Build the `Staged` and `StagedAtBase` stores with the add and direct arms'
magic-enabled prefix,
so the listing agrees with the pathspecs it is given,
and add the commit-route controls the direct route already has
(`literal_pathspecs_check_a_named_manifest_like_plain_mode` and
`literal_pathspecs_check_all_like_plain_mode`,
`package/git-policy/cli/src/native/binary/binary_dependent_version_tests.rs`)
for a commit that stages a bad final newline and for one that stages a
secret,
each run under the variable and under the flag.
A listing that comes back empty for a scope the index provably changes is
also worth an engine failure rather than a silent "nothing to judge",
since that is the shape a future mode could take.

### 2. `add-explicit` judges four fixed spellings, and five that select the whole tree pass **reproduced**

Sequence.
`add-explicit` decides from the arguments alone
(`command/rule_add_explicit.rs:84-120`).
It reads Git's option table for `-A`,
`-u` and their negations
(`command/command_add.rs:170-245`),
and then compares every positional pathspec against four byte strings:
`.`,
`./`,
`*` and `:/`
(`command/command_add.rs:113`,
`:288-296`).
Anything else is an ordinary path.

Wrong result.
Five spellings that Git resolves to the whole tree are ordinary paths by
that list,
and each was reproduced staging every file while the policy said nothing
(`probe-12-spelling-effect.ts`,
the block under "Five whole-tree pathspec spellings pass `add-explicit`").
`:/*`,
`:/**`,
`:(top)`,
`**` and `:(glob)**`
all stage `top.txt` and `sub/below.txt` and exit `0`.
`:(top)` is the one most likely to be written by accident:
it names the top of the worktree,
which is the whole repository,
and a person adding it expects a scope rather than a sweep.
The policy's own statement of why it exists is that a sweep leaves the index
"in a state that does not match a single logical change"
(`command/rule_add_explicit.rs:1-20`),
and the rejection text offers the per-command escape.

A sixth spelling,
`:(top)/*`,
never reaches the policy's list:
Git 2.56.0 fails in `prep_exclude` while the wrapper predicts what the
`git add` would stage,
and the wrapper reports `content-unavailable` and exits `2`.
So one pathspec a person can reasonably write is refused as an engine
failure that names Git's internal text,
and another four that select the same set are allowed.

Limits of the fix.
A fixed list of spellings cannot be complete,
because Git's pathspec grammar has magic,
globs,
and exclude forms,
so a later reader will find a seventh.
Checking what a pathspec selects needs Git to answer,
which costs one process per `git add`,
and the route deliberately keeps this decision argument-only so it starts no
process before Git does.
Both remedies are honest and neither is obviously the owner's:
extending the list with the five forms and naming them in the message keeps
the route process-free and catches the accidents,
while resolving the pathspec asks Git and catches everything at the cost of a
start and a behavior that no longer follows from the arguments alone.

## What I checked and found sound

- The registry's order,
  severities,
  warn-safe flags,
  off-unless-listed flags and trigger sets match `SPEC.md`
  "Policy order and stopping" and the per-policy sections
  (`policy/policy_registry.rs:147-235`,
  `policy/policy_trigger.rs:120-158`).
- The engine asks trigger,
  `--policy`,
  escape and severity in that order,
  and the four answers decide exactly as the specification says
  (`policy/policy_engine.rs:270-289`).
- A `--policy` selection that would check nothing is refused with `2` and one
  `config-invalid` event naming the setting to change
  (`probe-14-policy-refusal.ts`:
  `check --all --policy mono/catalog-specifiers` with a configuration that
  does not list it,
  `code 2`;
  the same command naming `final-newline`,
  `code 0`),
  and the sentences that name a trigger instead are pinned by
  `invocation/management_off_policy_tests.rs:150-172`.
- The `-a` and explicit-path commit routes both correct a bad final newline,
  install the corrected bytes into the worktree,
  and leave `git status` empty
  (`probe-10-commit-all.ts`,
  `probe-8-literal-mode.ts` control).
- An unfixable error blocks the commit before it lands when literal mode is
  off:
  `code 1`,
  no commit,
  the finding at `pre-forward` naming the path
  (`probe-2-commit-pathspec.ts`).
- The post-commit gate runs after a commit command lands,
  reports against the landed tree,
  emits `commit-landed` after the causal events,
  blocks the push and exits `2` (`:49`),
  and lets a warning-only result through with `0`
  (`probe-4-matrix.ts` case D).
- A matched secret never reaches an event:
  the message names the line and the rule,
  and the pathname of a name match is `[REDACTED]`
  (`probe-1-judgement.ts`,
  `policy/policy_forbidden_strings.rs:217-240`,
  `:421-500`).
- The rules file precedence is configuration,
  then `FORBIDDEN_STRINGS_RULES`,
  then the default,
  and the variable reached every lifecycle I exercised:
  the `git add`,
  the commit,
  and `check --all`
  (`probe-9-rules-variable.ts`:
  refused with the variable,
  clean without it).
- A rules file inside the repository is left out of the scan,
  including when the variable names it,
  and an unrepresentable pathname,
  a missing required rules file and a pathname with a line break all end the
  policy as engine failures rather than as clean
  (`candidate/scanner_selection.rs:96-110`,
  `:124-176`,
  `candidate/scanner_failure_code.rs:60-160`;
  the commit-check audit recorded the same).
- The known-content skip compares blob object names against parent trees,
  never against the worktree or the index,
  and excludes a file the same invocation corrected
  (`candidate/candidate_known_content.rs:190-210`,
  `:250-320`).
- A content policy that cannot read its bytes,
  or cannot get a fact it asked for,
  answers "failed" and never "clean"
  (`policy/policy_checks.rs:213-230`,
  `policy/policy_content.rs:249-262`).
- The convergence loop is bounded at eight changed passes,
  detects a repeated non-adjacent state,
  and reports both as engine failures
  (`policy/policy_convergence.rs:30`,
  `:182-211`;
  the commit-check audit recorded the same).
- The attribute lookup reads the state the candidates came from,
  refuses a record Git printed for another path,
  and treats only the exact value `unset` as a reason to keep bytes
  (`candidate/candidate_attribute.rs:161-235`,
  `:388-410`),
  which matches `SPEC.md` "Final newline".
- Nothing in the route reads the worktree:
  every read is an index listing or an object,
  which the commit-check audit recorded;
  the two probes above that stage through the escape hatch confirm the
  correction works on index bytes and lands in the worktree only after the
  landing.

## What I could not determine

- How the installed release behaves here.
  Every probe used the debug binary from this revision,
  and the fixture used the build's own commit service;
  the installed wrapper at
  `/var/mnt/encrypted/repository/Monochromatic/.cache/monochromatic-git-policy-cli-bin/git`
  was not exercised.
- Why `git add ':(top)/*'` fails in Git.
  The message is Git's (`oops in prep_exclude`,
  Git 2.56.0),
  the wrapper only reports it,
  and I did not build a Git with assertions to read the dying branch.
  Whether Git upstream considers that pathspec valid is unknown here.
- Whether the literal-mode hole reaches a `--pathspec-from-file` commit.
  The entries of a pathspec file are literal by their own grammar,
  and the candidate listing would still be built from `:(top,literal)`
  pathspecs,
  so the hole should be identical,
  but no fixture ran it.
- Whether a person in this repository ever sets the variable.
  `rg` over `.config/`,
  `.github/` and `mise.toml`
  finds no `GIT_LITERAL_PATHSPECS`,
  and `doc/handover/cli-git-native-dependent-version.md`
  records the same variable as a past defect for the direct route,
  which suggests it was met in a probe rather than in daily use.
  What other tools on a person's machine set is outside this reading.
- Whether two services of one repository can answer the one-entry tree cache
  with different replace refs,
  which the commit-check audit also left open.
- Windows and macOS.
  Every probe is Linux,
  and the attribute and pathname conversions are platform-specific by
  construction
  (`candidate/candidate_attribute.rs:300-330`).

## What to inspect and how to respond

Inspect,
in this order:

1.  the two arms without the guard
    (`candidate/candidate_prediction.rs:528-543`)
    beside the two with it
    (`:492-495`,
    `:512-520`),
    and the commit request that feeds them
    (`commit/commit_transaction.rs:303-315`);
    the reproductions are
    `probe-2-commit-pathspec.ts`,
    `probe-8-literal-mode.ts` and
    `probe-13-root-context-hole.ts`,
    and the direct route's controls are the model for the regression tests.
2.  the four spellings `add-explicit` names
    (`command/command_add.rs:113`)
    against the five that stage everything
    (`probe-12-spelling-effect.ts`),
    and choose between extending the list and asking Git what a pathspec
    selects.
3.  the policy table for the two policies with no post-commit trigger
    (`policy/policy_trigger.rs:120-158`):
    every hole before landing is silent for
    `mono/forbidden-root-context` and `mono/dependent-version-bump`.

Nothing here needs the owner's decision to be reported;
finding 1 has one remedy that follows the decision the direct route already
took,
and finding 2 carries the choice this page is asking for.
