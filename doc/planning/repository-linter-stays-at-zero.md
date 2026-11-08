# Keeping the repository linter at zero findings

This is a proposal for the repository owner.
Nothing in it is built,
and no option is adopted until the owner chooses.

The owner's words that frame it:
"Principle 1:
Lint issues need to be taken care of asap."
and "Layout + linting fixes are the biggest priority."

## What happened

A repository-wide `monochromatic-lint` run on `main` at `136a40fe69` reported 174,916 findings.
By rule,
`markdown/semantic-line-breaks` made up 171,262 of them,
`markdown/no-bare-urls` 1,961,
`rust/require-rustdoc` 999 (731 errors and 268 warnings),
`rust/avoidable-closure` 516 (508 warnings and 8 errors),
the `shell/*` rules about 100,
`rust/max-lines` 18,
`markdown/fenced-code-language` 11,
`rust/tests-in-sidecars` 10,
and a few other Markdown rules.
The branch `fix/repository-passes-monochromatic-lint` records what each fix commit changed and what was left.

The semantic line break rule has been in `monochromatic-lint.config.jsonc` at severity `error`
since `0b49ae109e` (2026-10-06T18:38:11-04:00),
the commit that moved the repository from the earlier Markdown and Rust linters to `monochromatic-lint`.
That commit's own cutover handover,
`doc/handover/unified-linter-cutover.md`,
recorded the Markdown and Rust counts at that time,
and no commit after it burned them down.

## Why nothing caught it

Four separate gaps each let the count grow,
and any one of them alone would have been enough.

### The commit policy runs one Markdown rule

The Git wrapper's `markdown/autofix` policy is configured in `monochromatic-git-policy-cli.config.jsonc`
as `["warn", { "rules": ["lfs-image-url"], "exclude": ["package/ssg/"] }]`.
In `package/git-policy/cli/src/native/config/config_schema.rs` the type `MarkdownRule` has one variant,
`LfsImageUrl`,
and `markdown_rule_from_name` accepts only `lfs-image-url`;
the configuration reader rejects any other name with "the shipped rule is \"lfs-image-url\"".
`0b49ae109e` wired the policy with the defaults `['monochromatic-lint']` and `['markdown/lfs-image-url']`,
so from the first day the commit path rewrote LFS image links and checked nothing else.
The policy's severity is `warn`,
and it covers staged Markdown only:
no commit path runs any Rust rule or any workflow shell rule.

### The CI step cannot run

`.github/workflows/advisory-core.yml` has a step `Static changed paths`
that runs `task-run lint:monochromatic-lint -- <changed .md, .mdx and .rs files>`.
Three facts make it empty:

- The binary is not installed.
  `.github/actions/repo-tools-setup/action.yml` installs `file-enforcer`,
  `mpm-shadow` and `cargo-shadow`,
  and the job installs nothing else that provides `monochromatic-lint`.
  In the four runs of the last 400 where the job ran with changed static files
  (run numbers 42349671262,
  42349787203,
  42350454505 and 42351528780,
  all on 2026-11-07),
  the step log says `task-run: line 1: monochromatic-lint: command not found`.
  The step has `continue-on-error: true`,
  so GitHub reports its `conclusion` as success
  and only its `outcome` as failure,
  which a later step turns into a failed run.
- The job almost never runs.
  The `gate` job admits at most 2 runs per minute,
  4 per hour and 8 per day,
  and skips the rest.
  Of the 400 most recent runs of the workflow (2026-11-04 to 2026-11-07),
  394 skipped the `advisory core` job.
- The workflow is advisory.
  Its summary says "results are advisory:
  no required checks and no branch-protection change",
  and pushes reach `main` straight from auto-push,
  so a red run arrives after the commit is already public.

### The full lint is a manual task

`mise run lint` is the only place that runs every rule together,
and nothing starts it.
The Rust closure check inside it runs Dylint once per Cargo package,
which is expensive:
on this machine a run restricted to one Rust file still took 89.65 seconds because it checked the file's whole package.
The measured cost of the Markdown rules is small:
`monochromatic-lint doc` over the documentation tree took 3.84 seconds.

### Ratchets are not enforced either

`.github/workflows/monochromatic-lint.config.jsonc` holds a ratchet per workflow
(the worst step each file may have,
"raising a number needs the repository owner's approval").
Two steps were over their file's limit on `main` at `b8e5bf3b0c`:
the version detection step of `cargo-publish.yml` (38 code lines against a ratchet of 37,
and four `&&` against three),
which commit `7c5b49ac23` (2026-11-07T09:55:37-05:00) introduced
(the same file at its parent commit passes,
checked with the ratchet configuration as it is now),
and the cli executable step of `npm-release.yml` (3 lines against the default of 2).
Nothing failed when they landed,
because the same gaps apply.

### Two linter behaviors that make a clean run harder

The fixer was run over every documentation family.
It is not a pure whitespace tool,
and these behaviors showed up in the files it was refused for:

- It breaks `///` doc comments inside fenced Rust listings,
  which adds `///` markers to quoted upstream source in troubleshooting records.
  `doc/handover/unified-linter-cutover.md` already recorded this as a case where "the new linter is wrong".
- It wraps a URL that sits inside a code span as `` `<https://example.com/`> ``,
  and `markdown/no-bare-urls` also reports a URL inside a code span.
- It converts a pipe table to an HTML table,
  which rule MD5 also forbids.

These belong in the linter package,
and are listed in the report of the branch that fixed the findings.

## Options

Each option is a separate choice,
and combinations are reachable.

### Option A: the commit policy runs every Markdown rule as a check

`markdown/autofix` gains the other Markdown rules in its `rules` list,
at `error` severity,
in report-only form.
A commit that stages a Markdown file with a finding is refused with the finding's text,
and the author runs `mise run format:monochromatic-lint -- <paths>` to fix it.

Pros:

- The finding is reported at the moment of decision,
  on the files being committed,
  in the same tool that already runs.
- It never rewrites a commit's content on its own,
  so a fixer defect cannot silently change a record.
- Markdown rules are cheap,
  as measured above.

Cons:

- It needs a change to `package/git-policy/cli`,
  which other branches are changing.
- It does not cover Rust or workflow shell steps,
  because the policy sees Markdown only.
- A commit made with `--no-enforce-markdown/autofix`,
  a web edit or another clone still gets through.

### Option B: the commit policy applies the fixer to staged Markdown

Same place as option A,
but the policy runs the fixer on the staged files,
as it does for `lfs-image-url` today.

Pros:

- The author does nothing:
  the commit already contains the fixed text.
- It is the shortest path from the owner's "asap" to a clean tree.

Cons:

- The fixer changed rendered text in 58 of the 498 files it changed in the documentation families,
  so a blind apply would rewrite records that the owner treats as quoted evidence,
  with no proof step.
- The proof used here needs `pandoc`,
  which the wrapper does not embed.
- It has option A's reach limits and option A's package cost.

### Option C: a required CI workflow runs the whole linter

A new workflow `monochromatic-lint.yml` (or a repaired `advisory-core.yml` step) installs the pinned `monochromatic-lint`
from `toolchain.lock.json` the way `repo-tools-setup` installs the other tools,
and runs `monochromatic-lint` over the whole tree on every push and pull request,
outside the advisory run budget,
with the closure check included or run by a second job.
The owner marks it required in branch protection.

Pros:

- It is the only option that covers Rust rules,
  workflow shell rules and every way around the commit wrapper.
- A whole-tree run keeps the guarantee that the count is zero,
  not only that changed files are clean.
- It repairs a gap that exists today:
  the current step cannot start.

Cons:

- It detects after a push,
  because auto-push puts commits on `main` before CI runs;
  only the required status check on pull requests prevents.
- The closure check needs every package's dependencies and costs minutes,
  so the job needs a measured time limit.
- Marking a check required is the owner's branch-protection setting,
  which no file in the repository can set.

### Option D: a daily run that opens an issue

The daily update the repository already schedules (`feat/monorepo-task-daily-update-opens-fix-forward-issues`)
runs the whole linter and opens or updates one issue with the finding counts by rule.

Pros:

- No friction for authors,
  and it reuses the issue mechanism that is being built.
- It measures the count over time.

Cons:

- Findings stay for up to a day,
  which does not meet "asap".
- It depends on the daily update branch landing first.

### Option E: only repair the existing advisory step

Install the binary in `advisory-core.yml` and leave everything else as it is.

Pros:

- The smallest change,
  and it removes a step that fails whenever it runs.

Cons:

- The job still skips nearly every run,
  and the check stays advisory.
- It checks changed files only,
  so a file that becomes dirty without being touched is never seen.

## Ranking

Ranking:
A > C > B > D > E,
because:

- A over C:
  A reports at the commit,
  where the author can fix it in the same minute,
  and C reports after the commit is public;
  C is still the only option that reaches Rust and workflow shell rules,
  so it follows directly.
- C over B:
  C needs no change to a package under active work and covers more rules,
  while B would rewrite quoted records without a proof step until the fixer defects under
  Two linter behaviors that make a clean run harder are fixed in the linter.
- B over D:
  B at least puts a fixed file into the commit,
  while D only counts what is already wrong.
- D over E:
  D adds a whole-tree measurement and an issue to act on,
  while E leaves a mostly skipped advisory check.

"A,
then C" is reachable and is the recommendation:
A closes the Markdown gap at the commit,
which is where almost the whole original count was,
and C closes the Rust,
workflow and bypass gaps.
Option B becomes worth reconsidering after the linter stops changing quoted rustdoc,
code spans and pipe tables.

## Open questions for the owner

- Whether the whole-tree run in option C includes the closure check in the required job or in a second,
  slower job.
- Whether the required check's name goes into branch protection,
  which only the owner can set.
- Whether the fuzz seeds under `package/linter/monochromatic-lint.fuzz/seed/` get an ignore entry:
  they hold rule violations on purpose,
  and four findings stay until the owner approves one (rule LN7).
