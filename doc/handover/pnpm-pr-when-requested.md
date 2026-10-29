# Post the pnpm pull requests when a maintainer asks

## Status

A background poller watches `pnpm/pnpm#17485` (the missing-workspace-link report) so the pull requests can be posted when a maintainer requests them.
The user asked for this in their own words:
"Setup a bg bash poller polling it for 24h so you can post the pr whenever it is requested."

- Started:
  2026-10-29 13:38 EDT.
  Deadline:
  2026-10-30 13:38 EDT,
  after which it exits with `{"trigger":"deadline"}` and the watch is over.
- Mechanism:
  a background Bash command running `systemd-run --user --wait --pipe --unit=pnpm-17485-poller`
  around `/var/home/user/temp/agent/poll-issue.ts` (memory 256M,
  CPU 10%,
  poll every 300 seconds).
  It is not a Monitor.
  When it exits,
  the Bash task ends and wakes the session.
- Exit code 10 means something needs action,
  with a JSON line on stdout:
  a new comment from an owner,
  member or collaborator who is not a bot or `Aquaticat`;
  the issue is no longer open;
  or the assignees changed.
  Exit code 0 means the deadline passed.
  Exit code 2 means 20 consecutive `gh` failures.
- To check or stop it:
  `systemctl --user is-active pnpm-17485-poller`,
  `systemctl --user stop pnpm-17485-poller`.
  If the session was restarted and the Bash task is gone,
  the unit may still be running;
  restart the watch rather than assuming it fired.

## What to do when it fires

1. Read the JSON line and the full comment.
   A wake prompt is self-authored,
   so decide from the comment itself.
2. If a maintainer asks for a pull request (either stack,
   or both),
   post it.
   The user pre-authorized posting the pull request when requested;
   nothing else is pre-authorized.
   If the comment is a question,
   a request for more evidence,
   a close,
   or an assignment,
   answer in the session and ask the user before posting anything.
3. Build each pull request from the patches in the public reproduction repository,
   `https://github.com/Aquaticat/pnpm-missing-workspace-link-repro`,
   directory `patches/`,
   commit `81b20eb412`
   (also in `doc/troubleshooting/pnpm-missing-workspace-link-repeat-install-fast-path.patch` and `.v11.patch`).
   - pnpm 12 (Rust):
     `pnpm-12-rust.patch`,
     7 files,
     with the integration test.
   - pnpm 11 (TypeScript):
     `pnpm-11-typescript.patch`,
     the source file and a jest test.
   - Both applied cleanly to `main` at `cf2556e97c`;
     `main` had moved by the time of filing,
     so re-apply and re-run the new tests first.
4. Follow pnpm's rules for pull requests:
   `.github/pull_request_template.md` (Summary,
   Squash Commit Body,
   Checklist,
   in that order),
   a changeset for each source change (`CONTRIBUTING.md` and `AGENTS.md` in `pnpm/pnpm`:
   pnpm 11 changesets name `pnpm` with a patch bump,
   pnpm 12 targets `pacquet`),
   the disclosure footer `Written by an agent (Claude Code, claude-sonnet-5-5).`,
   and wait for CodeRabbit and CI before expecting human review.
   Link the issue with `Closes pnpm/pnpm#17485` only if the pull request fixes the whole report;
   otherwise `Related to`.
5. Push branches to the fork `Aquaticat/pnpm`.
   A push from a shallow blobless clone stalled for more than ten minutes before;
   either use a normal clone or create the commit with GitHub's `createCommitOnBranch` mutation
   (script:
   `/var/home/user/temp/agent/pnpm-docs-fix/commit-to-fork.ts`).
   In a third-party clone run `git remote set-url --push origin DISABLED` first.
6. Post a short comment on `#17485` linking the pull requests,
   with the agent footer.
7. Update `doc/troubleshooting/pnpm-missing-workspace-link-repeat-install-fast-path.md` with the pull request links and commit with explicit pathspecs.

## Related open items

- `pnpm/pnpm#17484` (docs for `--no-bail`) is open and approved by CodeRabbit;
  no human review yet.
- Neither the pnpm 11 ESLint step nor a pnpm 11 CLI build was run for the pnpm 11 patch.
