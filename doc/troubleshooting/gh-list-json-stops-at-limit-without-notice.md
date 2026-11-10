# gh 2.103.3 list commands with `--json` stop at `--limit` without any notice, so a `--jq` filter never sees older items

## Symptom

On 2026-11-01 in `Aquaticats/Monochromatic`,
this command was meant to confirm that 25 known issues were still open:

```sh
# shell: check 25 known issue numbers against the open list
gh issue list --state open --limit 500 --json number,title \
  --jq '.[] | select(.number as $n | [995,996,998,999,1000,1001,1004,1008,1009,1030,1039,1040,1046,1047,1051,1053,1054,1119,1179,1823,1824,1825,1843,1847,2133] | index($n)) | "\(.number) \(.title)"'
```

It printed nothing and exited 0.
`gh issue view <number> --json number,state,title` then reported all 25 as `OPEN`.
Nothing on standard output or standard error said that the list had been cut short.

The same silence applies whenever `--json` is given and `--limit` is below the number of matching items,
whatever the `--jq` expression is.

## Root cause

1.  `gh issue list` fetches at most `--limit` issues,
    30 by default
    ([`pkg/cmd/issue/list/list.go:113`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmd/issue/list/list.go#L113)):

    ```go
    // pkg/cmd/issue/list/list.go:113
    cmd.Flags().IntVarP(&opts.LimitResults, "limit", "L", 30, "Maximum number of issues to fetch")
    ```

2.  It fetches newest first by creation date,
    and it asks GitHub for the total in the same query
    ([`pkg/cmd/issue/list/http.go:30-34`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmd/issue/list/http.go#L30-L34)):

    ```graphql
    # pkg/cmd/issue/list/http.go:33-34
    issues(first: $limit, after: $endCursor, orderBy: {field: CREATED_AT, direction: DESC}, states: $states, filterBy: {assignee: $assignee, createdBy: $author, mentioned: $mention}) {
        totalCount
    ```

    The loop at `http.go:84-115` pages 100 at a time,
    stops when `len(issues) == limit`,
    and keeps `totalCount` in the result
    (`api.IssuesAndTotalCount{Issues: issues, TotalCount: totalCount}`,
    line 115).

3.  With `--json`,
    the command writes only the issues and returns,
    before both notices the command has
    ([`pkg/cmd/issue/list/list.go:215-226`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmd/issue/list/list.go#L215-L226)):

    ```go
    // pkg/cmd/issue/list/list.go:215-226
    if opts.Exporter != nil {
    	return opts.Exporter.Write(opts.IO, listResult.Issues)
    }

    if listResult.SearchCapped {
    	fmt.Fprintln(opts.IO.ErrOut, "warning: this query uses the Search API which is capped at 1000 results maximum")
    }
    if isTerminal {
    	title := prShared.ListHeader(ghrepo.FullName(baseRepo), "issue", len(listResult.Issues), listResult.TotalCount, !filterOptions.IsDefault())
    	fmt.Fprintf(opts.IO.Out, "\n%s\n\n", title)
    }
    ```

    The only place the total reaches the user is `ListHeader`
    ([`pkg/cmd/pr/shared/display.go:73-83`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmd/pr/shared/display.go#L73-L83)),
    which builds `Showing %d of %s in %s`,
    and it is printed only when standard output is a terminal.

4.  `--jq` runs inside that exporter,
    on the issues it was handed.
    gh embeds gojq,
    not jq
    ([`go.mod:136`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/go.mod#L136):
    `github.com/itchyny/gojq v0.12.19`,
    reached through `github.com/cli/go-gh/v2/pkg/jq`,
    imported at [`pkg/cmdutil/json_flags.go:16`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmdutil/json_flags.go#L16)).
    The filter can only ever see the truncated list.

5.  `pr list` returns from the same exporter branch before its header
    ([`pkg/cmd/pr/list/list.go:214`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmd/pr/list/list.go#L214)),
    and so does `run list`
    ([`pkg/cmd/run/list/list.go:160`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmd/run/list/list.go#L160)).
    Neither was run for this doc.

In the incident the repository had 2759 open issues.
`--limit 500` fetched numbers 2780 to 3282,
and all 25 checked numbers,
995 to 2133,
were older.

### The earlier jq reading was wrong

The first suspicion was that jq's `index` misbehaves on an array,
or that `select` treats index 0 as false.
That was a guess,
and both parts are disproved:

```sh
# shell: gh's embedded gojq and system jq give the same answer
gh api rate_limit --jq '[995,996,2133] | [index(995), index(2133), index(1)]'
# [0,2,null]
jq --null-input --compact-output '[995,996,2133] | [index(995), index(2133), index(1)]'
# [0,2,null]   (jq-1.8.1)
```

In jq,
only `false` and `null` are falsy,
so `select(0)` keeps its input.
The filter in the symptom was correct;
it ran on the wrong list.

## Verification

- gh `gh version 2.103.3 (2026-10-26)`,
  source tag `v2.103.3`,
  commit `63837fb778d6a39f02be0ce76ff89f2bc8d74bdc`.
- System jq `jq-1.8.1`.
- Repository `Aquaticats/Monochromatic`,
  2026-11-01.
- Each permalink in "Root cause" was opened in `agent-browser`,
  and the anchored line of GitHub's full-file text area holds the quoted text.
  Reading the page's rendered text instead gave false negatives for the longer files,
  because GitHub's code view draws only part of a long file.

Harness:

```sh
# shell: compare what --json returns with the total GitHub reports
gh issue list --state open --limit 3 --json number --jq 'length'
# 3
gh api graphql \
  -f query='query{repository(owner:"Aquaticats",name:"Monochromatic"){issues(states:OPEN){totalCount}}}' \
  --jq '.data.repository.issues.totalCount'
# 2814
gh issue list --state open --limit 3 | head --lines=3
# three tab-separated rows and no "Showing 3 of 2814" line, because standard output is a pipe
```

### Patterns that return everything or show the cut

- `gh issue list --state open --limit 5000 --json number --jq 'length'` printed `2759`,
  every open issue at the time,
  because the limit was above the total.
- `gh issue view <number> --json number,state,title` for each known number found all 25.
- `gh issue list` with standard output on a terminal prints the `Showing N of M` header (from source at `list.go:222-224`;
  not run in a terminal for this doc).

### Patterns that cut silently

- `gh issue list --json ... --limit N` with N below the matching total,
  with or without `--jq`:
  the incident and the harness.
- `gh issue list --limit N` piped into another command:
  the harness's last command.
- `gh pr list --json` and `gh run list --json`,
  from source only.

## Verified workarounds

1.  **Ask for the total first.**
    Read `totalCount` with `gh api graphql`
    (the harness),
    then compare it with the length of the `--json` result,
    and fail when the list is shorter.
    Tradeoff:
    one more request,
    and issues created between the two calls make the numbers drift,
    so treat a small difference as a race and re-run.
2.  **Set `--limit` above the measured total.**
    `--limit 5000` returned all 2759.
    Tradeoff:
    gh pages through everything 100 at a time
    (28 requests here),
    which is slow,
    and the total keeps growing,
    so the bound needs the first workaround to stay safe.
3.  **Look up known numbers one by one.**
    `gh issue view <number> --json number,state,title` per number.
    Tradeoff:
    one request per number,
    and it only answers for numbers you already know.
4.  **Narrow on the server.**
    `gh issue list --state all --search "Depot in:title" --limit 200 --json number`
    returned 54 issues,
    including 995.
    Tradeoff:
    it matches text,
    so #2133,
    whose title lacks the word,
    was missed;
    it uses the Search API,
    which caps at 1000 results,
    and with `--json` the cap warning at `list.go:219-221` is skipped too.

## What does not work

- Blaming jq or gojq:
  see "The earlier jq reading was wrong".
- Expecting a warning on standard error under `--json`:
  the command returns at `list.go:216`,
  before both notices.
- Relying on `isTerminal` output in scripts:
  the header only appears on a terminal.

## Upstream filing decision

Checked 2026-11-01 against `cli/cli` at `v2.103.3`.

- `.out-of-scope/`:
  no entry for gh
  (the folder lists `bun-install.md`,
  `cargo-workspace.md`,
  `claude-code-upstream-bugs.md`,
  `codex-harness.md`,
  `jsr.md`,
  `lightningcss.md`,
  `low-impact-typescript-formatting.md`,
  `module-es-monolith.md`,
  `pi-gpt55-long-context.md`,
  `terminal-title-fork-parity-tests.md`
  and `typescript-project-references.md`).
- Duplicate search:
  `gh search issues --repo cli/cli --match title,body` with each of
  "limit json total count",
  "warn results truncated limit",
  "jq limit truncated"
  and "totalCount --json"
  returned no issues.
- Work-in-progress search:
  not run,
  because the decision is not to file.

The six constraints:

1.  **Is it upstream's fault?**
    No.
    `--limit` is documented as "Maximum number of issues to fetch",
    and gh fetched exactly that many.
    The cut is the requested behavior;
    the caller chose a limit below the total without checking it.
    The missing notice under `--json` is consistent with keeping JSON output free of extra text,
    and the terminal path already shows the total.
2.  **Can upstream fix it?**
    Yes,
    if they wanted to:
    a line on standard error before `list.go:216` when `len(listResult.Issues) < listResult.TotalCount` would leave the JSON on standard output unchanged.
3.  **Are they supporting this use case?**
    Scripting with `--json` and `--jq` is a gh feature;
    whether a truncation notice is in scope was not checked.
4.  **Would the repo welcome our contribution?**
    Issues yes,
    pull requests only by invitation:
    `.github/CONTRIBUTING.md:5` says
    "We accept external pull requests only for issues labelled `help wanted` with explicit Acceptance Criteria.
    We encourage issues and discussion posts for all other contributions."
    No policy on AI-assisted reports was searched for.
5.  **Will they likely fix it?**
    Not checked.
6.  **Have we prototyped a fix?**
    No.
    Constraint 1 fails,
    so the auto-prototype rule does not apply.

Decision:
do not file.
The draft below is kept as a feature request a future session could re-evaluate.
Filing is always the poster's decision,
and a feature request against documented behavior,
written with agent help,
risks being closed as by design and counted against the poster by any trust system the tracker runs;
no trust-system search was done.
The readiness check was not run,
because there is no recommendation to file.

~~~md
<!-- do not file as-is -->
Title: `issue list`, `pr list` and `run list` give no sign under `--json` that `--limit` cut the result

Labels: enhancement

`gh issue list --json ... --limit N` returns the newest N items and nothing else,
even though the command already fetched `totalCount`
(`pkg/cmd/issue/list/http.go:34`).
The `Showing N of M` header and the Search API cap warning are both skipped,
because the exporter branch returns first (`pkg/cmd/issue/list/list.go:215-216`).
A script filtering with `--jq` cannot tell a short answer from a complete one.

Reproduction (gh 2.103.3):

    gh issue list --state open --limit 3 --json number --jq 'length'   # 3
    gh api graphql -f query='query{repository(owner:"OWNER",name:"REPO"){issues(states:OPEN){totalCount}}}' --jq '.data.repository.issues.totalCount'   # larger

Suggested change:
when `len(listResult.Issues) < listResult.TotalCount` under `--json`,
print one line to standard error,
for example `warning: showing 3 of 2814 matching issues; raise --limit to fetch more`,
leaving standard output unchanged.
The same applies to `pkg/cmd/pr/list/list.go:214` and `pkg/cmd/run/list/list.go:160`.
~~~
