# The Git wrapper's open issues, fourth pass, 2026-11-10T03:14:11-05:00

What:
 the 15 oldest open issues that name the Git wrapper,
 read in full with their comments and checked against `package/git-policy/cli` at `aca920cdf4`,
 with the verdict for each.

Why:
 the earlier passes took the behaviour reports and the citation family,
 and left behind the feature requests and the sentences that live in `doc/decision/`.
 This pass reads the next 15 in the same order the earlier ones used,
 so the next agent does not read them again,
 and so the owner sees which of them need an answer rather than a patch.

## Method

The list is
`gh issue list --repo Aquaticats/Monochromatic --state open --limit 5000 --search 'git-policy in:title,body' --json number,title,labels`,
which returned 89 issues on 2026-11-10.

Skipped before reading:
`#6265`,
the only issue `doc/handover/questions-for-the-owner.md` names,
and the Windows and macOS work the owner paused ("Ignore Win and mac until I say so").
No issue in the 89 carries a mutation or LFS label.

The 15 oldest whose subject is the wrapper,
and what each says now:

- `#492` post-push policy stage,
- `#574` a deterministic check on branch and worktree names,
- `#630` sharing the e2e scenario fixtures,
- `#1332` the Windows self-shim marker literals,
- `#1698` moved paths cited in decision docs,
- `#1717` prose that points by relative position,
- `#1768` silent loss of the wrapper's protection,
- `#1907` the dependent-bump ripple,
- `#1911` the per-user config doc's trust owner,
- `#3024` the hook rejecting a vendored Ghostty checkout,
- `#3258` the manual-push-candidates citation,
- `#3275` the policies-platform record's superseded parts,
- `#3276` the same record's issue range,
- `#3745` the platform record's missing superseded note,
- `#3796` stale citations in `doc/troubleshooting`.

Every claim below was checked against the tree with `rg`,
`test -e`,
`git log` or a test run,
never from the issue's own text alone.

## The one fix this branch carries

`#1717` and `#3796` name positional pointers in `doc/handover`.
The two that pointed at nothing but a place are now named:

`doc/handover/cli-git-rust-implementation.md`,
in "Still to do",
said two open items were "closed by the section below".
Both are closed by one recorded section,
so both now name it:
"The landing records the stat data it can prove,
and the staleness question walks its entries in parallel"
(2026-11-08T19:20:00-05:00).

Measured before:
`rg -i 'see (above|below)|listed above|described (above|below)|previous section|earlier in this (section|document)|the section (above|below)|written out below' doc/handover/*.md`
found 5 lines in 3 files.
After the edit it finds 3:
`doc/handover/git-policy-cli-on-tokio.md:697` and `:753`,
and a data row in
`doc/handover/two-lane-outcome-vocabulary-rendering-audit-design-population-and-settled-run.md:579`
that points at a later row of its own table.
The two `on-tokio` lines name "the section above" and "the previous section",
which two different sections of that file can satisfy,
so they stay for the owner's judgement rather than a guess.

## Verdicts

### `#492` the post-push policy stage

Does not hold as a patch:
`rg -n -i 'post.?push'` over `package/git-policy/cli/src/native` finds no stage and no event,
and no Worker URL check task or code exists;
the one mention of one is `package/config/lfs-r2-worker/README.md`,
which is the consumer the issue's own triage comment of 2026-10-20 records as missing.

Needs the owner:
the three open design points are choices,
not measurements.
The triage comment's defaults (warning-only findings,
skip dry runs and Markdown-free pushes,
bounded timeouts,
silent offline skip,
an opt-out variable) are recorded as defaults to adopt
once a consumer exists,
and nothing in the tree consumes the stage today.

### `#574` non-descriptive short-form names

Needs the owner:
the mechanism is unchosen.
The five built-in policies are `require-root`,
`linked-worktree-only`,
`branch-worktree-only`,
`add-explicit` and `final-newline`
(`doc/decision/cli-git-policies-platform.md`,
"Policy and transform order"),
and none of them reads a name for its meaning;
`command/rule_branch_worktree.rs` decides where a branch may be created,
never what it is called.

The issue itself names the open questions:
which surface enforces the check,
which tokens count as generally understood,
and whether a violation refuses or only warns.

### `#630` the e2e scenario fixtures

Holds:
`package/git-policy/cli/e2e` still holds 24 `scenario-*-fixture.ts` files among 83 entries.

Not reached:
the option-2 scope is recorded in the issue's comment of 2026-10-20,
and its verification is `mise run //package/git-policy/cli:test:e2e:concurrent:native`,
whose image task pulls the CachyOS base and needs the network
(`test:e2e:concurrent:image`:
"needs a network when that image or the Containerfile is new to this host").
A 15-thousand-line refactor of the suite with no run to prove it keeps its 61 scenario names is not worth the risk.

### `#1332` the Windows self-shim marker literals

Does not hold in `package/git-policy/cli`:
`package/git/executable` is gone.
The module is `package/git-executable/core/src/self-shim.ts`,
the bundled-entry marker pair the issue quotes is gone
(the module doc says that marker "went with it" after `642b6d72cb`),
and one hand-written pair remains at lines 143 and 148,
`GIT_POLICY_CLI_PACKAGE_NAME` and `WINDOWS_GIT_POLICY_CLI_PACKAGE_PATH`.
That half is Windows,
which the owner paused.

### `#1698` moved paths cited in decision docs

Needs the owner:
19 of the 22 sentences the issue lists are in `doc/decision/`,
which this branch may not edit.
`doc/philosophy/vm-dev-environment.md:217` already links `../../package/file-enforcer`,
which exists.
The two remaining sentences are in `doc/migration/no-class.md`,
which cites 16 `package/` paths that no longer resolve;
their successors are under `package-paused/`
(`git log --follow --diff-filter=R` gives `e26035dbb2` and `ece5b7553f` for the editord family,
`64046360d2` for the morph-compact family).
That file is a curated rename table,
not a one-line edit,
so it is left whole.

### `#1717` prose that points by relative position

Partly fixed in `08cdb8979a`:
two of the five remaining pointers now name the section they meant,
and the next commit carries only the line breaks the linter requires of those two sentences.

Needs the owner:
the issue's other named remedy is a markdown lint rule for `doc/` phrase patterns.
A new lint rule is an owner decision,
and this branch adds none.

### `#1768` silent loss of the wrapper's protection

Does not hold in `package/git-policy/cli`.
The issue's triage comment of 2026-10-20 records the decision:
"no change to the wrapper binary.
A process that resolves `git` to `/usr/bin/git` never runs the wrapper,
so the wrapper cannot detect its own absence;
the check belongs to whatever builds the child environment",
and names the owner as "the supervising launcher (auto-mode / Pi RPC),
not `package/git-policy/cli`".
The correction that follows it adds the one wrapper-specific identity signal,
`git cli-git --help`,
which the launcher preflight is meant to use beside the `realpath` comparison.

### `#1907` the dependent-bump ripple

Needs the owner:
the sentence lives in `doc/decision/private-npm-registry.md`,
which this branch may not edit.
The dead paths stand:
`package/git-policy/cli/src/policy-engine/apply-policy-patches.ts` and
`package/git-policy/repository/src/bump-dependents.ts` both do not exist,
and the ripple that replaced them is `src/native/dependent_version/`.

### `#1911` the per-user config doc's trust owner

Needs the owner:
the sentence is `doc/decision/monorepo-manager-per-user-config.md:35`,
which cites `package/git-policy/cli/src/allowed-worktree-dirs.ts`.
That file is gone,
and its subject is now `src/native/invocation/effective_target.rs`.
The edit belongs in a decision record this branch may not touch.

### `#3024` the hook rejecting a vendored Ghostty checkout

Holds,
and needs the owner.
`src/native/invocation/effective_target.rs` exempts a repository only when its Git directory sits under
a tool cache,
and `default_allowed_worktree_dirs` names exactly two:
uv's cache and mise's data directory.
A cargo build directory is not one of them,
so the vendored Ghostty clone `libghostty-vt-sys` makes under `CARGO_TARGET_DIR` is enforced,
and its `git checkout` is refused.
`doc/planning/git-wrapper-open-issues-against-main.md` recommends adding that directory beside the two,
with its own words for why the choice is the owner's:
"This widens an exemption that protects data,
so it needs the owner."

The options,
for the owner's answer:

1. Add the cargo build directory,
   read from `CARGO_BUILD_DIR` and then `CARGO_TARGET_DIR`,
   to `default_allowed_worktree_dirs`,
   and state the exemption's limit in that function's doc.
   Smallest change,
   same shape as uv and mise,
   and it covers every build script that keeps its clones there.
2. Enforce worktree policies only inside a repository whose `.cache/cli-git-bin/git` link belongs to it,
   which is the principled boundary but needs a repository-identity signal the wrapper does not read today.
3. Let a build script opt out through an environment variable.
   Rejected by the recommendation in that record:
   a third-party build script cannot know the variable exists.

### `#3258` the manual-push-candidates citation

Needs the owner:
`doc/decision/prefer-readonly-result-provenance.md:672` and `:981` cite
`package/git-policy/cli/src/policy-engine/manual-push-candidates.ts`,
which is gone,
and the record is one this branch may not edit.

### `#3275` the policies-platform record's superseded parts

Holds,
and needs the owner:
`doc/decision/cli-git-policies-platform.md` records two moves in place
(lines 960 and 966,
the TypeScript entry point and `escape-hatch.ts`),
and still describes the TypeScript authoring surface as current at lines 280 to 283
("Public authoring exports include") and `cli-git.config.mjs` discovery at line 363.
A superseded note is an edit to `doc/decision/`.

### `#3276` the same record's issue range

Holds,
and needs the owner:
line 6 says implementation is tracked by "GitHub issues #342 through #357",
and line 977 says "GitHub issues #341 through #356".
The sentence is in `doc/decision/`.

### `#3745` the platform record's missing superseded note

Holds,
and needs the owner:
`doc/decision/cli-git-policies-platform.md` carries no "superseded" note for the parts the Rust rewrite replaced;
its one use of the word at line 893 is about issues `#143` and `#160`.
The note belongs in `doc/decision/`.

### `#3796` stale citations in `doc/troubleshooting`

Holds,
and needs the owner's answer on the remainder:
the issue's own progress comments record two sweeps,
the second adding one timestamped note to 107 records,
and leave 153 distinct paths whose successors Git never carried,
mostly build output,
placeholders and paths that never existed.
"A note would be a guess" is the recorded reason they stay,
and what to do with a citation whose target never existed in history is a choice,
not a measurement.

## What was run

- `mise run //package/git-policy/cli:native:test:host -- manual_push_range`:
  14 passed,
  5758 skipped,
  including `a_request_list_larger_than_a_pipe_is_written_while_the_answers_are_read`,
  which is the control for the deadlock `#4487` reported.
  That issue therefore does not hold either:
  `run_git_bytes` now writes its request list from a thread with its own poll-based deadline
  and reads the answers on the runtime with `RANGE_QUERY_CEILING`.
- `rg -i 'see (above|below)|...'` over `doc/handover`,
  before and after the edit:
  5 lines,
  then 3.
- `git log --follow --diff-filter=R` for the three moved families `doc/migration/no-class.md` cites.
- `test -e` over every path this record calls gone,
  and `rg --files` for each successor named.

## What this branch does not do

- It comments on and closes no issue.
- It edits nothing in `doc/decision/`,
  which is the owner's.
- It runs no mutation campaign,
  no e2e suite and no release.
- It changes no wrapper behaviour:
  every issue among the 15 except `#630` and `#3024` is a sentence in an owner's record,
  and `#630` needs the e2e image before a refactor of that size can be shown to keep its scenarios.
