# Git wrapper open issues, third pass

What:
 the open issues that name the Git wrapper in the folders no other agent is changing,
 read against today's code,
 with what held,
 what each fix changed,
 and what is left for someone else.

Why:
 the first pass
 ([first pass](git-wrapper-open-issues-against-main.md))
 classified the whole list and left the module-doc citation family for one decision,
 and the second pass
 ([second pass](git-wrapper-open-issues-round-2.md))
 took the behaviour and untested-arm reports.
This pass works the folders nobody else holds,
so its commits do not collide with the branches another agent will merge.

## Method

The list came from one command:

`gh issue list --repo Aquaticats/Monochromatic --state open --limit 5000 --search 'git-policy in:title,body' --json number,title,body`

It answered with 306 open issues.
The numbers the two earlier passes already named were read out of their own text,
123 of them,
which left 243 candidates.

A mechanical pass then resolved every backticked path in each candidate's title and body
by file name against today's tree,
because the crate moved its modules into subsystem folders and the issues name the flat layout.
That pass sorted the 243 by the folder the code has now.
The folders this branch may touch held 27 issues:
26 of them from the deleted-TypeScript citation family,
and one factual module-doc claim.

Two more came from the package's own specification:
their evidence names `package/git-policy/cli/SPEC.md` or the constant in `config/`,
and the defect is a doc sentence rather than code,
so the task's folder rule counts them inside this branch's work.

## Fixes

### `#4511` the ref-file module doc misdescribed the chain Git follows

Held:
`src/native/repo_read/ref_files.rs:5` and `:6` said:

```text
//!  `git symbolic-ref --quiet HEAD` only reads the worktree's `HEAD` file and,
//!  for a branch,
//! checks that the branch is not itself a symbolic ref.
```

Git does neither.
Its own reader refuses a symbolic branch file for exactly the reason the doc denied:
Git answers with the end of the chain,
so a first-hop answer would differ.

Measured on this host with `git --version` 2.56.0 in a disposable repository under
`${HOME}/temp/agent`:

- `HEAD -> refs/heads/alias -> refs/heads/main`:
 `git symbolic-ref --quiet HEAD` printed `refs/heads/main`,
 exit status 0.
- a chain whose end does not exist:
 printed `refs/heads/nowhere`,
 exit status 0.
- a cycle:
 `fatal: No such ref: HEAD`,
 exit status 128.

`loose_branch_is_plain`'s own `Why` carried the same error ("fails on a broken one"),
so both sentences now say what the probe showed.
The change is a doc comment and one new control;
no production code moved.

Fixed in `29180a2a69`,
with the new control
`repo_read::ref_files::tests::a_chain_ending_at_a_missing_branch_is_left_to_git`.
The control states Git's answer for a missing chain end against real Git,
and the same fixture answers a plain `HEAD`,
so the refusal cannot pass because the fixture itself is broken.
Shown live by expecting the first hop instead:

```text
assertion `left == right` failed: the oracle prints the end of the chain, missing or not
  left: Ok(Branch("refs/heads/nowhere"))
 right: Ok(Branch("refs/heads/alias"))
```

### The deleted-TypeScript citations in the six folders (26 issues)

Held:
every one of the 26 docs cites a file the TypeScript wrapper carried as if it were in the tree.
The cited file is gone,
the line range cannot be opened,
and the sentence reads as present tense.

Each issue names the remedy itself,
and it is the one `WR6` asks for:
keep the original text and put an in-place note beside it naming where the cited file reads.
The note gives the deleting commit,
its date,
and a `git show <commit>^:<path>` that reads the file.
Every one of the 15 distinct cited paths was checked to exist at the commit named,
and every cited line range was checked to fit inside the file it names.

The commits,
one per issue,
all of them doc comments only.
Each path below is under `package/git-policy/cli/src/native/`:

- `#4493` `f02e072b9d`,
 `dependent_version/policy_added_path.rs`.
- `#4520` `751c74e279`,
 `command/rule_atomic_push.rs`.
- `#4521` `6533cd9b93`,
 `command/rule_atomic_push_tests.rs`.
- `#4539` `d8dab1f0f7`,
 `command/command_commit_tests.rs`.
- `#4540` `ea10d6b7ee`,
 `command/command_options_tests.rs`.
- `#4556` `4fcb0301b5`,
 `dependent_version/dependent_version_bundled.rs`.
- `#4557` `032797325a`,
 `dependent_version/dependent_version_content.rs`.
- `#4558` `06aa115596`,
 `dependent_version/dependent_version_graph.rs`.
- `#4559` `c988d7c908`,
 `dependent_version/dependent_version_imports.rs`.
- `#4560` `f7c3a3b23a`,
 `dependent_version/dependent_version_imports_tests.rs`.
- `#4561` `13a66eebee`,
 `dependent_version/dependent_version_graph_tests.rs`.
- `#4562` `5c9cdd4cee`,
 `dependent_version/dependent_version_manifest.rs`.
- `#4564` `2e0ecdb703`,
 `dependent_version/dependent_version_manifest_tests.rs`.
- `#4565` `7561ae091f`,
 `dependent_version/dependent_version_paths.rs`.
- `#4566` `a673ab39a5`,
 `dependent_version/dependent_version_paths_tests.rs`.
- `#4567` `5d5389ea52`,
 `dependent_version/dependent_version_plan.rs`.
- `#4568` `4072e4f7a8`,
 `dependent_version/dependent_version_plan_tests.rs`.
- `#4569` `b334066317`,
 `dependent_version/dependent_version_policy.rs`.
- `#4570` `be483c32db`,
 `dependent_version/dependent_version_policy_tests.rs`.
- `#4571` `d76f388aec`,
 `dependent_version/dependent_version_publishable.rs`.
- `#4572` `e77c1e378c`,
 `dependent_version/dependent_version_publishable_tests.rs`.
- `#4573` `b712780037`,
 `dependent_version/dependent_version_release.rs`.
- `#4574` `aa96a59095`,
 `dependent_version/dependent_version_release_tests.rs`.
- `#4575` `dd28c334ee`,
 `dependent_version/dependent_version_states.rs`.
- `#4577` `22b34ffaab`,
 `dependent_version/dependent_version_text.rs`.
- `#4578` `c7e5b266c8`,
 `dependent_version/dependent_version_text_tests.rs`.

The source files were deleted in `642b6d72cb` (2026-10-13,
"delete the TypeScript Git policy wrapper"),
and the differential case files in `a2b56c548c` the same day,
so the two notes differ only in that commit and its path.

The note shape,
beside the citation and before the pseudocode block:

```text
//! `manifest-text.ts` was deleted on 2026-10-13 in `642b6d72cb` with the TypeScript wrapper,
//!  and `git show 642b6d72cb^:package/git-policy/cli/src/optional/repository-policy/manifest-text.ts` reads it there.
```

The in-place note over deleting the citation is the remedy each of these issues names and the
one `WR6` states.
It is a flagged choice,
open to the owner's veto:
the other remedy round 1 listed,
deleting the citation and stating the behaviour in the module's own words,
reverts these 26 commits one file at a time.

Commit `f02e072b9d` and the 25 after it change doc comments and nothing else,
so no control covers them and none can.

### `#4581` the slash predicate cited a rule that no longer exists

Held:
`src/native/config/config_rules_file.rs:97` said the named predicate for `split` exists
"because the repository bans anonymous functions".
That syntax-only rule,
`rust/no-anonymous-functions`,
was removed repo-wide on 2026-10-18 by the owner's decision
(`doc/decision/rust-closure-lint-dylint.md`,
"Removal of `rust/no-anonymous-functions`"),
and a configuration that still names it fails as an unknown built-in rule.
What stands in its place,
`rust/avoidable-closure`,
reports a closure argument that a named function could replace,
which asks for this named predicate,
and `monochromatic-lint.config.jsonc` enables it for `**/*.rs`.

Fixed in `b0301ec217`:
the `Why` names the current rule and records when the ban it cited went.
No code changed.

### `#5722` the starvation guard's default was recorded wrong

Held:
`doc/planning/cli-git-concurrent-commits.md` said "K defaults to 2,
is tunable,
and the
concurrent-commit benchmark confirms the default".
Three things there are not what the tree says:

- the accepted default is 1,
 changed 2026-09-26 (veto open),
 in `doc/decision/cli-git-concurrent-commits.md`,
 which `package/git-policy/cli/SPEC.md` states as `1` for `landing.reserveAfterLostRaces`,
 and `DEFAULT_RESERVE_AFTER_LOST_RACES` in
 `src/native/config/config_schema.rs` is 1;
- no benchmark recorded that confirmation:
 `SPEC.md` still lists the sweep of `landing.reserveAfterLostRaces` among the scenarios to
 measure;
- the key reserves nothing today:
 `SPEC.md` says it sets nothing and that `doctor` asks a file to drop it,
 and no module outside `config/` reads it.

Fixed in `6e23015b97`:
the bullet keeps its text and gains a dated note with those four facts and their sources
(`WR6`).
The pass of this document used
`mise run //:format:monochromatic-lint -- doc/planning/cli-git-concurrent-commits.md`
and then `mise run //:lint:monochromatic-lint -- <that file>`:
both clean.

### `#5719` the status still described the design as unimplemented

Held:
the same record says "design in progress;
nothing implemented" and closes with
"No implementation before confirmation",
while `doc/decision/cli-git-concurrent-commits.md` records the design accepted on 2026-09-25,
and `cefcee148e` (2026-09-25),
`a50d674d7d` (2026-09-26) and `f7e501a685` (2026-09-26) implemented preparation in a shadow
repository,
serial landing,
and the replay of a commit that lost a landing race.
`SPEC.md` states the landing order and the replay refusal,
and the crate carries the modules as `src/native/shadow/replay_*.rs` and
`src/native/landing/*.rs`.

Fixed in `b3d57a3536`:
both places keep their text and gain a dated note (`WR6`).
Format and lint of that file:
clean.

## Not reached

- `#3024` and `#1768`:
 left as instructed;
 both need the owner.
- `#5926`,
  `#5788`,
  `#3191`,
  `#3275`,
  `#5765`,
  `#5766`,
  `#6101`,
  `#4357`:
 the sentence each names lives in `doc/decision/`,
 which this branch may not edit.
- `#2648`,
  `#4858` (`binary/`,
  `invocation/`),
 `#1672` (half its sites are in `foundation/`),
 `#4522` and `#4523` (`rule_commit_only` moved into `commit/`),
 and the duplicate-scan reports `#5987`,
  `#5988`,
  `#5989` (`shadow/`):
 each needs a folder another agent holds.
- The other 213 candidates name no path in the six folders,
 the package's scripts or its docs,
 so they belong to the folders this branch keeps out of.
- `#6101` names `package/git-policy/cli/package.json`,
 which is neither a script nor a document of the package,
 and its claim is about a decision record's line numbers.

## What was run

- `mise run //package/git-policy/cli:native:test:host -- ref_files`:
 14 passed,
 including the new `a_chain_ending_at_a_missing_branch_is_left_to_git`.
- The same task filtered to `dependent_version` (96 passed),
 `command::rule_atomic_push` (5 passed),
 `command::command_commit` (19 passed),
 `command::command_options` (38 passed),
 and `config::config_rules_file` (3 passed).
- `mise run //package/git-policy/cli:native:clippy` after the last doc edit:
 no finding.
- `mise run //package/git-policy/cli:lint:monochromatic-lint`:
 no finding.
- `mise run //:format:monochromatic-lint` and `mise run //:lint:monochromatic-lint` on this
 file.
- The three probes of `git symbolic-ref --quiet HEAD` above,
 in a disposable repository under `${HOME}/temp/agent`.
- The 15 `git show <commit>^:<path>` reads that back every in-place note,
 and `wc -l` of each cited file at its commit to confirm each quoted line range is inside it.

## What this branch does not do

- It closes,
 comments on and edits no issue;
 that is the coordinator's step after review.
 No commit carries `Closes`,
  only `Refs`.
- It changes nothing in `merge/`,
 `conflict/`,
 `landing/`,
 `commit/`,
 `push/`,
 `index/`,
 `worktree_copying/`,
 `worktree/`,
 `subvolume/`,
 `policy/`,
 `candidate/`,
 `binary/`,
 `invocation/`,
 `process/`,
 `foundation/`,
 `shadow/`,
 `service/` or `lock/`,
 and nothing in `doc/decision/`.
- It runs no mutation campaign,
 starts no LFS work,
 changes nothing for Windows or macOS,
 and merges nothing into `main`.
