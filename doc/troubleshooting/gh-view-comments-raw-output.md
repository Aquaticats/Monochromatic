# GitHub CLI 2.101.0 piped `--comments` views print only comments, and nothing when no comment is displayable

## Symptom

`gh issue view <url> --comments` and `gh pr view <url> --comments` behave differently depending on whether
standard output is a terminal.
Every agent harness,
pipe,
and command substitution reads them in non-terminal mode,
so the piped behavior is the one that matters here.

Three distinct surprises appear in non-terminal mode.

First,
the title,
state,
metadata lines,
and body disappear.
Only raw comment blocks print:

```text
$ gh issue view https://github.com/cli/cli/issues/14530 --comments | wc -c
477
$ gh issue view https://github.com/cli/cli/issues/14530 --comments | head -3
author:	github-actions
association:	contributor
edited:	false
```

The same URL without `--comments` prints metadata and the body instead,
with no comments:

```text
$ gh issue view https://github.com/cli/cli/issues/14530 | wc -c
1715
$ gh issue view https://github.com/cli/cli/issues/14530 | head -3
title:	brew install gh
state:	CLOSED
author:	zakariasarghini40-stack (Zakaria Sarghini)
```

Second,
the command can exit zero while printing zero bytes,
which reads like success with no content and gives a caller nothing to distinguish
"no comments" from "comments exist but were not rendered":

```text
$ gh issue view https://github.com/cli/cli/issues/1 --comments | wc -c
0
$ gh issue view https://github.com/cli/cli/issues/1 --comments; echo "exit=$?"
exit=0
```

That URL is a pull request number whose two comments are both minimized.
An ordinary issue with zero comments behaves the same way.

Third,
in a terminal the same command prints metadata,
body,
and comments together,
so the omission is invisible to anyone reproducing by hand:

```text
$ gh issue view https://github.com/cli/cli/issues/14530 --comments   # attached terminal
title:	brew install gh
state:	CLOSED
...
```

## Root cause

Source citations are from tag `v2.101.0` of `cli/cli`,
commit `0cf1092493af067646fc5f3db9421c6a6ec9c938`.

### Non-terminal output takes a comments-only branch and returns early

`pkg/cmd/issue/view/view.go:190-199`:

```go
	if opts.IO.IsStdoutTTY() {
		return printHumanIssuePreview(opts, baseRepo, issue)
	}

	if opts.Comments {
		fmt.Fprint(opts.IO.Out, prShared.RawCommentList(issue.Comments, api.PullRequestReviews{}))
		return nil
	}

	return printRawIssuePreview(opts.IO.Out, issue)
```

The terminal branch at line 191 calls `printHumanIssuePreview`,
which prints metadata,
body,
and comments.
The non-terminal branch at line 194 prints only `RawCommentList` and returns at line 196,
so `printRawIssuePreview` at line 199,
the function emitting `title:`,
`state:`,
and the body,
never runs when `--comments` is set.

### `--comments` also swaps which comment field the query requests

`pkg/cmd/issue/view/view.go:120-122`:

```go
		if opts.Comments {
			lookupFields.Add("comments")
			lookupFields.Remove("lastComment")
		}
```

Without the flag,
`defaultFields` at `pkg/cmd/issue/view/view.go:96-100` requests `lastComment`,
which is why the metadata line `comments:	2` still reports a count while no comment text is printed.

### The pull request view repeats the same branch

`pkg/cmd/pr/view/view.go:138-141`:

```go
	if opts.Comments {
		fmt.Fprint(opts.IO.Out, shared.RawCommentList(pr.Comments, pr.DisplayableReviews()))
		return nil
	}
```

### Zero bytes come from comments that render as empty strings

`pkg/cmd/pr/shared/comments.go:29-41`:

```go
func RawCommentList(comments api.Comments, reviews api.PullRequestReviews) string {
	sortedComments := sortComments(comments, reviews)
	var b strings.Builder
	for _, comment := range sortedComments {
		fmt.Fprint(&b, formatRawComment(comment))
	}
	return b.String()
}

func formatRawComment(comment Comment) string {
	if comment.IsHidden() {
		return ""
	}
```

`RawCommentList` builds its whole output from `formatRawComment`,
which returns an empty string for a hidden or minimized comment.
Two minimized comments therefore produce an empty string,
and an issue with no comments produces an empty string too.
Neither path signals anything to the caller,
so the exit code stays zero.

Querying the same node directly shows both comments minimized,
which is the state that produced the zero-byte output for `cli/cli#1`:

```text
$ gh api graphql -f query='query { repository(owner: "cli", name: "cli") { issueOrPullRequest(number: 1) { __typename ... on PullRequest { comments(first: 100) { totalCount nodes { isMinimized author { login } } } } } } }' --jq '.data.repository.issueOrPullRequest'
{"comments":{"nodes":[{"author":{"login":"Vadim0695"},"isMinimized":true},{"author":{"login":"Vadim0695"},"isMinimized":true}],"totalCount":2},"__typename":"PullRequest"}
```

### Upstream states this is intended behavior

`cli/cli#13322`,
"`gh issue view --comments` does not show the issue body when stdout is not a TTY",
was closed on 2026-05-12 with state reason `NOT_PLANNED`.

`cli/cli#13452`,
"gh issue view <N> --comments silently emits 0 bytes to non-TTY pipe (regression in 2.92.0)",
drew this maintainer reply from `BagToad` on 2026-06-05 before it closed:

```text
This is actually intended behavior. You are seeing zero bytes because neither issues has any
comments, and without TTY when you use `--comments` we only show comments. If you run this on an
issue with comments you'll see more than zero bytes.

The "comments only on non-TTY" decision was deliberate when this was first built in cli/cli#2462.
```

The quoted design discussion in `cli/cli#2462` records the assumption and the reviewer's agreement:

```text
When invoked in non-tty, I made the assumption that if the `--comments` flag is provided the user
would like to just output comments information and skip the issue metadata and body.

That's a good call! For now, I think it's best if scripts fetch issue bodies and its comment thread
separately.
```

That last sentence is the workaround this repository adopted.

## Verification

Version under test:

```text
$ gh --version
gh version 2.101.0 (2026-09-15)
https://github.com/cli/cli/releases/tag/v2.101.0
```

Source under test:
`gh repo clone cli/cli <dir> -- --depth 1 --branch v2.101.0`,
`git rev-parse HEAD` reporting `0cf1092493af067646fc5f3db9421c6a6ec9c938`.

Runnable harness.
Every line runs against a pipe,
which is the mode that shows the behavior:

```bash
gh issue view https://github.com/cli/cli/issues/14530 | wc -c
gh issue view https://github.com/cli/cli/issues/14530 --comments | wc -c
gh issue view https://github.com/cli/cli/issues/1 --comments | wc -c
gh issue view https://github.com/cli/cli/issues/13118 --comments | wc -c
gh pr view https://github.com/cli/cli/pull/14517 | wc -c
gh pr view https://github.com/cli/cli/pull/14517 --comments | wc -c
```

Observed byte counts on this host,
all with exit code zero:

- `issue view 14530` without the flag:
   1715 bytes,
   metadata and body,
   no comments.
- `issue view 14530 --comments`:
   477 bytes,
   comments only.
- `issue view 1 --comments`:
   0 bytes,
   both comments minimized.
- `issue view 13118 --comments`:
   9109 bytes,
   comments only for an issue with a large body.
- `pr view 14517` without the flag:
   4676 bytes,
   metadata and body.
- `pr view 14517 --comments`:
   1059 bytes,
   comments and reviews only.

### Patterns that carry the body

- `gh issue view <url>` piped:
   metadata lines plus body.
- `gh pr view <url>` piped:
   metadata lines plus body.
- `gh issue view <url> --json title,state,body,comments`:
   every requested field in one call.
- Any of the above attached to a terminal,
   which takes the human preview branch.

### Patterns that drop the body

- `gh issue view <url> --comments` piped.
- `gh pr view <url> --comments` piped.

### Patterns that print nothing at all

- `gh issue view <url> --comments` piped,
   where the issue has no comments.
- `gh issue view <url> --comments` piped,
   where every comment is minimized.
- `gh pr view <url> --comments` piped,
   where the pull request has no comments or reviews.

## Verified workarounds

### Fetch the body and the comment thread as two calls

This is the workaround `package/search-fetch/pi-plugin` adopted,
and the one the upstream reviewer recommended in `cli/cli#2462`.

```bash
gh issue view "$URL" &
gh issue view "$URL" --comments &
wait
```

The planner in `package/search-fetch/pi-plugin/src/gh-invocation-plan.ts` emits both invocations as one
attempt,
marking the body read required and the comment read optional,
and `package/search-fetch/pi-plugin/src/gh-client.ts` runs them concurrently,
drops empty parts,
and joins the rest with a blank line.

Tradeoffs:

- two GraphQL round trips instead of one.
   Measured on this host against `cli/cli#13118`,
   three samples each on one unchanged build:
   sequential 2869 ms,
   3024 ms,
   2850 ms;
   concurrent 1846 ms,
   1571 ms,
   1630 ms.
   The two bands do not overlap,
   so concurrency roughly halves the wait.
- the comment read must be treated as optional.
   A zero-byte comment read is normal for an issue with no comments,
   so failing the whole attempt on it would lose the body.
- a minimized-only thread still yields no comment text.
   The body read carries `comments:	N`,
   so a caller can tell that comments exist but were not rendered.

### Ask for JSON fields instead

```bash
gh issue view "$URL" --json title,state,body,author,labels,comments
```

Tradeoffs:

- one call returns everything,
   including minimized comment state.
- markdown bodies arrive JSON-escaped,
   which costs tokens and reads worse for a model than gh's raw text rendering.
- `--comments` cannot be combined with `--json` on gh 2.101.0.
   The pair is rejected before any request,
   measured as exit code 1 with `specify only one of --comments or --json`,
   and enforced by `cmdutil.MutuallyExclusive` at `pkg/cmd/issue/view/view.go:60-63`.
   `cli/cli#14214` described the older silently-ignored behavior.

### Allocate a pseudo-terminal

Running gh under a pty takes the terminal branch and prints metadata,
body,
and comments together.

Tradeoffs:

- output arrives with ANSI color and pager behavior that a caller must strip.
- it needs a pty driver in the calling process,
   which a Pi extension fetching a page should not carry.
- the human preview branch truncates and re-wraps text to terminal width,
   so content is lossy in a different way.

Rejected for this repository.

## What does not work

- Reading `--comments` output as a full page.
   It never contains the title or body in non-terminal mode.
- Treating exit code zero as evidence of content.
   Zero bytes with exit zero is the normal result for a thread with no displayable comment.
- Passing `--comments` together with `--json`.
   gh 2.101.0 rejects the combination with exit code 1 and `specify only one of --comments or --json`,
   rather than silently ignoring one flag as `cli/cli#14214` described for an older release.
- Expecting `gh release view <url>` to accept a URL the way `issue view`,
   `pr view`,
   `pr diff`,
   and `gist view` do.
   It does not,
   and the failure reaches implicit repository resolution from the working directory;
   see `doc/troubleshooting/gh-implicit-repository-git-wrapper.md`.

## Upstream filing decision

Default policy is not to file.
Walking the six constraints:

1. **Is it really upstream's fault?**
   No.
   The behavior is deliberate and documented in the design discussion behind `cli/cli#2462`,
   restated by a maintainer in `cli/cli#13452`,
   and `cli/cli#13322` closed as `NOT_PLANNED`.
   This is a design decision,
   not a defect.
2. **Can upstream fix it?**
   Yes technically,
   by printing metadata and body before the comment list on the non-terminal branch.
   This constraint does not decide the outcome.
3. **Are they supporting this use case?**
   Yes.
   The maintainer reply names the supported pattern for scripts:
   fetch bodies and comment threads separately.
4. **Would the repo welcome our contribution?**
   Unlikely for this behavior.
   `.github/CONTRIBUTING.md` at `v2.101.0` routes bug reports and enhancement requests through the
   issue templates in `.github/ISSUE_TEMPLATE`,
   and no ban on AI-assisted reports was found in that file or those templates.
   The `github-actions[bot]` triage messages in `cli/cli#13452` state the team is not looking for
   external contributions on enhancement-labeled issues without a `help wanted` label.
5. **Will they likely fix it?**
   No.
   `cli/cli#13322` closed `NOT_PLANNED`,
   and `cli/cli#13452` closed after a maintainer declared the behavior intended.
   Both are direct signals against a change.
6. **Have we prototyped a minimal fix?**
   Not needed,
   because constraints 1 and 5 already fail,
   and the consumer-side workaround is in place and measured.

Duplicate search performed with `gh search issues --repo cli/cli` over the phrases
`issue view --comments`,
`issue view comments body missing`,
and `RawCommentList minimized`.
The first two returned `cli/cli#13322`,
`cli/cli#13452`,
`cli/cli#14214`,
and `cli/cli#12606`,
all closed.
The third returned nothing.

Decision:
file nothing.
Both observations already have closed upstream reports carrying an explicit intended-behavior ruling,
so a new report or comment would only add noise.
