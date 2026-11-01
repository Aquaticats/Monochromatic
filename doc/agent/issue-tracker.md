# Issue tracker: GitHub

Issues and PRDs for this repo live as GitHub issues.
 Use the `gh` CLI for all operations.

## Conventions

- **Create an issue**:
   `gh issue create --title "..." --body "..."`.
   Use a heredoc for multi-line bodies.
- **Read an issue**:
   `gh issue view <number> --comments`,
   filtering comments by `jq` and also fetching labels.
- **List issues**:
   `gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'`
with appropriate `--label` and `--state` filters.
- **Comment on an issue**:
   `gh issue comment <number> --body "..."`
- **Apply / remove labels**:
   `gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **Close**:
   `gh issue close <number> --comment "..."`

Infer the repo from `git remote -v`;
 `gh` does this automatically when run inside a clone.

## When a skill says "publish to the issue tracker"

Create a GitHub issue.

## When a skill says "fetch the relevant ticket"

Run `gh issue view <number> --comments`.

## Resolution workflow

When the user says "resolve issue N" (or links to a specific issue),
 the verb authorizes the full workflow:
 fix and commit.
Put a `Closes #N` line in the commit body.
Auto-push (the `APG` rule) lands the commit on the default branch,
 so GitHub closes the issue automatically once the push is processed.
Reference what changed from the commit message so a reader landing on the closed issue can find it.

Close manually with `gh issue close <N> --comment "..."` only when no fix commit carries the closing keyword:
 a wontfix,
 duplicate,
 or already-fixed issue,
 or work that landed on a non-default branch GitHub will not scan.
