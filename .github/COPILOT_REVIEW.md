# Copilot review workaround — remote stub-branch flow

This document describes a simple,
 low-friction workflow to get AI review (for example,
 GitHub Copilot’s PR review or a Copilot CLI run) over commits without changing how developers push or merge.
 Use this when you want manual control over when changes are sent for automated review.

Summary

- Create a remote-only branch called `stub-copilot-review`.
- Continue pushing to `main` as usual.
- Open a pull request from `main` -> `stub-copilot-review` when you want the AI to review a batch of commits.
- Update `main` with additional commits;
   the PR updates automatically.
   Merge (with bypass) the PR when done.

Why this works

- Pull requests are the natural integration point for Copilot’s automatic reviews and other CI/Checks.
- A PR from your active branch (head) into a dedicated “stub” base branch gives a visible,
   auditable place to request AI review while preserving the team’s existing push/merge workflow.
- The approach is manual,
   avoiding changes to branch protection or developer habits
   while still enabling automated or manual review tools on the PR.

Benefits

- No change to developer push/merge workflow.
- Controlled,
   on-demand AI review windows.
- Easy to batch commits for a single AI review session.

Limitations

- Manual step to open a PR.
- Not a substitute for enforced branch protection or mandatory automated reviews.
- Requires repository-level configuration (optional) to get the best automated review UX.

Examples

- Create stub branch from `main`:
  - git push origin main:
    stub-copilot-review
- Create PR with `gh`:
  - gh pr create --base stub-copilot-review --head main --title "AI review:
     weekly batch" --body "Run Copilot review for latest commits"

Appendix:
 Common troubleshooting

- PR not updating:
   ensure the PR’s head is `main` and not a fork;
   pushing to `main` updates the PR head.
