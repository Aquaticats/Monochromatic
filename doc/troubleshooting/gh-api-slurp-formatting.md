# GitHub CLI 2.103.3 rejects `api --slurp` combined with `--jq`

## Symptom

During open pull request enumeration,
`gh api` exited 1 with this diagnostic:

```text
the `--slurp` option is not supported with `--jq` or `--template`
```

## Root cause

The caller combined mutually exclusive output flags.
In source tag `v2.103.3`,
commit `63837fb778d6a39f02be0ce76ff89f2bc8d74bdc`,
the command's `RunE` validates these options and returns the error from
[`pkg/cmd/api/api.go:260`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmd/api/api.go#L260-L268):

```go
// pkg/cmd/api/api.go:260
if opts.Slurp {
    if err := cmdutil.MutuallyExclusive(
        "the `--slurp` option is not supported with `--jq` or `--template`",
        opts.Slurp,
        opts.FilterOutput != "",
        opts.Template != "",
    ); err != nil {
        return err
    }
```

The release source was shallow-cloned and read without modifying it.
The source permalink was opened in `agent-browser`,
and the quoted diagnostic was confirmed in GitHub's full-file text.

## Verification at 2026-11-04T17:19:38-05:00

Installed binary:
`gh version 2.103.3 (2026-10-26)`.

Failing invocation:

```sh
# shell: rejected output flag combination
gh api 'repos/Aquaticats/Monochromatic/pulls?state=open&per_page=100' \
  --paginate --slurp --jq '[.[][] | {number, title, html_url, state, draft}]'
```

Working invocation:

```sh
# shell: enumerate open pull request numbers
gh api 'repos/Aquaticats/Monochromatic/pulls?state=open&per_page=100' \
  --paginate --jq '.[].number'
```

The working invocation exited 0 and returned `6113`,
`3472`,
`3471`,
`3470`,
`3469`,
`3468`,
`3338`,
and `2741`.
The same query returned the same set after comments were posted.
This live collection fit within one page;
this incident did not exercise a subsequent page or GraphQL pagination.
Using `--paginate --slurp` without `--jq` also exited 0.

## Verified workaround

For per-record output,
use the working invocation in "Verification".
Remove `--slurp` and change the query from an outer array of pages to the individual response array.
The tradeoff is separate output values rather than one aggregate JSON document.
Do not infer a collection-wide count from a page-local aggregation.

## What does not work

Combining `--paginate --slurp --jq` fails before the filter can provide any records.
The original assumption that these output flags compose was wrong.
The command's explicit mutual-exclusion check disproves it.

## Upstream filing decision

The upstream search `repo:cli/cli slurp jq` found
[the existing feature request](https://github.com/cli/cli/issues/10459#issue-body-viewer).
Its body and comments were read through the API,
and its body selection was checked in `agent-browser`.
It already records this error and the output-shape concern.
There is nothing additive to post from this invocation mistake.
The `.out-of-scope/` inventory contains no GitHub CLI entry.

- Fault attribution:
  the caller selected an unsupported flag combination;
  this incident establishes no new upstream defect.
- Fixability:
  the exercised consumer invocation resolves this task without an upstream change.
- Supported use case:
  pagination with a per-record `--jq` filter succeeded;
  the combined `--slurp --jq` form was explicitly rejected.
- Contribution eligibility:
  not investigated because no contribution is proposed.
- Maintainer direction:
  the existing discussion considers the feature and its streaming tradeoffs;
  no acceptance or delivery commitment is inferred.
- Prototype:
  no upstream patch was attempted because fault attribution does not pass;
  the consumer correction was exercised.

No upstream issue or comment was filed.
Filing readiness checks for a new contribution were not run.
