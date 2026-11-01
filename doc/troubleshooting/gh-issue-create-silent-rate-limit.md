# gh issue create exits 0 with no output while the secondary rate limit rejects the mutation

`gh` 2.103.3, scripted batch issue creation against GitHub: when GitHub's secondary rate limit
is active, `gh issue create` can exit 0, print nothing to stdout or stderr, and create no
issue, so a loop keyed on the exit code silently skips filings.

## Symptom

A filing loop ran `gh issue create --repo <owner>/<repo> --title <title> --body-file <path>`
plus `--label` flags, capturing stdout as the new issue URL. 491 consecutive creates printed
URLs and created issues (through `#2313`, confirmed the tracker's newest issue at the
boundary). The next 169 creates produced: exit code 0, empty stdout, no stderr, no issue
created. The per-iteration log recorded `{"action":"new","url":""}` for every one:

~~~json
{"id":"w2-i18n-12","action":"new","issue":2313,"url":"https://github.com/<owner>/<repo>/issues/2313"}
{"id":"w2-i18n-13","action":"new","url":""}
{"id":"w2-i18n-15","action":"new","url":""}
~~~

The same run's comment mutations failed loudly instead, with the secondary-limit error on
stderr and a non-zero exit:

~~~text
GraphQL: was submitted too quickly (addComment)
Error: Command failed: gh issue comment 1379 --repo <owner>/<repo> --body-file <path>
~~~

So the limit was demonstrably active in the silent-create window; only the create path failed
quietly. After stopping and restarting the loop, the first retried create reproduced the
silent failure live (four consecutive attempts, each printing only the loop's own backoff
note), proving the state persists across processes and is not a one-off.

## Root cause

Proven at the consumer boundary: the create mutation is rejected while the secondary rate
limit is active, and gh still exits 0 with empty output. The tracker boundary proves the
issues were not created: the newest issue was `#2313`, the last one to print a URL, and 169
logged "successes" beyond it do not exist.

Not yet traced: the internal path in cli/cli that swallows the failure. The files to read
are `pkg/cmd/issue/issueCreate/issueCreate.go` (the `createIssue` run function and how it
prints the result) and the GraphQL client's secondary-limit retry handling under `pkg/api`.
Until that trace lands, this doc claims only the observed boundary behavior, not the
internal mechanism.

Rejected earlier reading: "creates print the `GraphQL: was submitted too quickly` error
too, and the loop ignored it". Disproved by the log: across the whole run there are exactly
4 rate-limit stderr lines and 2 `Command failed` lines, all from `addComment`; the 169 create
failures produced zero stderr lines and zero non-zero exits.

## Verification

- gh under test: `gh version 2.103.3 (2026-10-26)`,
  release tag `v2.103.3`.
- Harness: the sweep's filing loop (scratch `file-issues.ts`), whose create path was:

  ~~~ts
  const output = run(['issue', 'create', '--repo', '<owner>/<repo>',
    '--title', entry.title, '--body-file', bodyPath(entry.id),
    ...entry.labels.flatMap((label) => ['--label', label]),
  ]);
  const url = output.trim();
  const match = /\/issues\/(\d+)/.exec(url);
  record({ id: entry.id, action: 'new', issue: match ? Number(match[1]) : undefined, url });
  ~~~

- Patterns that work cleanly: the same command before the limit: URL on stdout, issue
  created, tracker numbers increase in lockstep with the log.
- Patterns that fail, split by variant:
  - create under the active limit: exit 0, empty stdout, no stderr, nothing created
    (169 observations plus 4 and counting on the live rerun);
  - comment under the active limit: stderr `GraphQL: was submitted too quickly (addComment)`,
    non-zero exit, `Command failed` error raised (2 observations).

## Verified workarounds

Each landed in the filing loop and observed live:

1. Validate the returned URL instead of the exit code; on no URL match, back off 30s times
   the attempt count and retry the same create, capped at 9 attempts, then record a failure.
   Tradeoff: a create that succeeded server-side while losing its URL would duplicate on
   retry. Not observed; rejected mutations do not execute server-side, and the tracker
   boundary check (newest issue equals the last logged URL) is the guard that proves it.
2. Pace every gh call by 1.5s and retry the explicit rate-limit error strings with the same
   backoff. Tradeoff: slower filing; reduces but does not prevent the limit, and the
   secondary limit clears on its own schedule regardless of pacing.
3. Treat log entries carrying an error or an empty URL as not filed when resuming, so a
   restart retries them. Tradeoff: none observed; requires the log to carry the URL field.

## What does not work

- Checking the exit code: the failing creates exit 0.
- Expecting the GraphQL error on stderr: the create variant prints nothing.
- Restarting the loop against the original log: the resume set marked the 169 silent
  failures as already filed and would skip them forever; the log had to be stripped to the
  500 good entries first (backup kept at `filed-log.jsonl.bak-batch2` in the sweep scratch).

## Upstream filing decision

Duplicate search on 2026-10-31 against `cli/cli`, queries and results:

- `issue create rate limit`: no results.
- `issue create no output exit`: no results.
- `secondary rate limit`: closest hits are
  `#4801` (closed, create issue fails with the loud `GraphQL error: was submitted too
  quickly`, the opposite symptom: an error the user can see),
  `#3292` (open, whether the CLI honors throttling headers),
  `#12812` (open, stale rate-limit headers from cached introspection),
  `#9586` (closed, release create secondary-limit option),
  `#10240` (open, 429 when setting an environment variable),
  `#4443` (closed, `--paginate` throttling).
- None describe a silent exit-0 create. No duplicate found.

`.out-of-scope/` in this repository was read on 2026-10-31: 11 exemption files, none
covering the gh CLI or rate limits, so no exemption applies.

The six constraints, walked honestly:

1. Is it upstream's fault? The rate limit is GitHub's; the silent exit 0 on a failed create
   is gh's behavior. Yes for the exit-code defect.
2. Can upstream fix it? Plausible: a failed create should exit non-zero. The mechanism is
   untraced, so the difficulty is unknown. Not yet assessed to a yes.
3. Are they supporting this use case? Non-interactive scripted creation is a documented gh
   use case. Not yet verified by quoting the docs page.
4. Would the repo welcome the report? `CONTRIBUTING.md` and the issue templates were not
   read yet; no AI-contribution ban was found incidentally, but this check has not run.
5. Will they likely fix it? `#4801` shows maintainer engagement with rate-limit reports on
   this command; no won't-fix signal found. Sorta-holds.
6. Minimal prototyped fix? Not yet. The source trace and the disposable-clone prototype are
   the named remaining steps.

Verdict: the audit is incomplete; constraints 2 through 4 and 6 need the source trace, the
policy read, and the prototype. Until then the draft below is an audit artifact marked
**do not file as-is**. Filing also puts the poster's account in front of strangers; the
trust-system search on the target repository has not run, and the risk sits with whoever
posts. The steps to finish the audit: clone cli/cli at v2.103.3 into a fresh private
`${HOME}/temp/agent/` directory, trace the create path, prototype the smallest fix that
propagates the failure (likely surfacing the GraphQL error and exiting non-zero), verify
against the live failing state, then re-run the duplicate search and the trust-system
search on the day of filing.

~~~md
Title: `gh issue create` exits 0 with no output when the secondary rate limit rejects the
creation

Labels: bug

`gh` version: 2.103.3 (2026-10-26).

While batch-creating issues with `gh issue create --repo <repo> --title <t> --body-file <f>
--label <l>` in a loop, GitHub's secondary rate limit became active mid-run. From that
point:

- `gh issue comment` failed loudly: stderr `GraphQL: was submitted too quickly (addComment)`,
  non-zero exit.
- `gh issue create` printed nothing at all: exit 0, empty stdout, empty stderr, and no issue
  was created. 169 consecutive attempts behaved this way; the tracker's newest issue number
  matches the last successful create exactly, so none of the silent attempts executed.

Because the exit code is 0 and there is no output, a script cannot distinguish failure from
success except by validating that stdout contains an issue URL.

Reproduction: run `gh issue create` in a tight loop until the secondary rate limit triggers
(no pacing); observe that once the limit is active the command exits 0 with empty output
while creating nothing, unlike `gh issue comment` under the same limit which fails loudly.

Suggested fix: the create path should surface the mutation failure (print the GraphQL error
to stderr) and exit non-zero when the issue was not created, matching the comment path's
behavior; source locations to check: `pkg/cmd/issue/issueCreate/issueCreate.go` and the
GraphQL client's secondary-limit handling under `pkg/api`.
~~~
