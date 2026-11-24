# Systemd path properties give a user unit its own user namespace

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 46450 to 46451 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  **The main-checkout author's ninth run ended whole at `7751ffbaca`** (228 unit tests pass; `main` merged): the three Git commands that exited 0 having recorded nothing now record; a start is refused only when the manager does not answer (21 ms where it was refused); a refusal after the worktree leaves a record; the page lists every hole. **By nine measured variants, any path property of a user unit puts it in a user namespace of its own mapping one uid, so `podman` and `newuidmap` fail inside whatever else is set, and each writable folder is its own mount, so `link` and `rename` across them end `EXDEV`.** It did not choose and laid the arrangements out.
  **A measurement of this session's at 17:29:13 widens them:** a transient service started with `systemd-run --user --wait --pipe` from inside a unit that has `ReadOnlyPaths=` reads `0 0 4294967295` in `/proc/self/uid_map`, where the unit itself reads `1000 1000 1`. That is all that was measured: the namespace of a service the account's manager starts on a run's request. `podman` itself was not run through that route, and a container started that way is outside the confinement and can mount the checkout writable, one more stated hole beside the transient-service one. Whether a run gets containers that way is the author's tenth run (started 17:31; brief `pi-run-cannot-write-into-the-main-checkout-a-run-inside-the-confinement-can-still-run-containers-through-the-accounts-manager-prompt.md`), and whether the project wants that route is put to the owner. What stays a cost of any mount-based protection is `EXDEV` (Git, pnpm and Cargo were measured working).
