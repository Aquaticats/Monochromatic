# Cli git throwaway clone runs use the private runtime directory as the boundary

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 2311 to 2335 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Issue 6293, filed by the other session at 10:55:20

A throwaway clone run with its own `HOME` and `XDG_RUNTIME_DIR` still got a commit-service unit in the account's real manager,
with its journal under the real `~/.local/state`,
and that unit retries a push to the address `DISABLED` with growing pauses.
`doc/troubleshooting/git-policy-cli-service-units-of-deleted-worktrees.md` says such a run "has no route to a service and refuses every commit";
here the commit was made.
The route as the source reads (posted on the issue at about 11:00,
read and not run):
`src/native/landing/service_route.rs` line 383 reads `DBUS_SESSION_BUS_ADDRESS`,
and `session_bus_path` (`src/native/service/systemd_dbus.rs` line 171)
prefers that variable's `unix:path=` entry over `$XDG_RUNTIME_DIR/bus`,
so a run that changes two variables and inherits the third still speaks to the real manager.
The troubleshooting doc's sentence is about a run "without a bus" and does not cover this case.
Unsetting the third variable alone would,
by that doc,
leave the clone unable to commit at all,
so the skill's sentence cannot simply be extended.
The owner decided both parts through the question tool on 2026-11-13 at about 11:02:

- How a throwaway clone is kept away from the real manager:
  "Private runtime dir is the boundary (Recommended)".
  When `XDG_RUNTIME_DIR` is not the directory the inherited bus belongs to,
  the wrapper does not use that bus and runs its commit service inside the private directory,
  in line with the decision of 2026-11-12 that the service needs no service manager.
  The two alternatives put to the owner and not chosen:
  an explicit throwaway switch (a setting meaning commit,
  never push,
  no unit),
  and correcting the docs only.
- What happens to a push whose address cannot be a remote at all:
  "End the plan at once,
  say so once (Recommended)".
  The plan ends on the first refusal with one line;
  an address that might come back (an unmounted drive,
  a host that is down) keeps its retries.
  Not chosen:
  stop after a few tries,
  and leave it as it is.

The brief is `~/temp/agent/pi-runs/issue-6293-private-runtime-directory-is-the-boundary-and-a-push-to-no-remote-ends-at-once-prompt.md`
(it replaces the investigation brief written before the decision,
which is deleted).
(scratch,
deleted on 2026-11-27;
not kept).
The first part rests on the branch `refactor/git-policy-cli-commit-service-is-started-by-the-wrapper-itself-and-needs-no-service-manager`,
cut off on 2026-11-12 and in the resume queue,
so that resume goes before this run,
and the brief tells the run to build on it and write no second service starter.
