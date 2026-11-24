# Podman image prune fails with no space left on a full drive

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 25264 to 25276 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- `podman image prune --force` (dangling images) was started at about 16:05 and `podman image prune --all --force` at about 16:10,
  each in a capped scope.
  Read at 16:52:07: the first one failed after about forty-seven minutes and freed nothing:
  "write /var/home/user/.local/share/containers/storage/overlay-images/.tmp-images.json...: no space left on device".
  `podman` has to rewrite its own image list to remove an image, and the drive had no room for that.
  The second was still running at 16:45:40 and faces the same.
  `/var/home` read 2138 MiB free at 16:43:15 and 1 MiB at 16:45:40: something was still writing to it; what, is not established.
  After the restart the drive needs some room made by other means before a prune can work.
  Read at 17:05:22: the second prune ended with the same two errors (on `overlay-images` and `overlay-layers`),
  and `/var/home` then read 43 GiB free, 98% used.
  Whether that room came from images the prune did remove before it failed or from the owner's side is not established.
  With room there, the prune the owner chose can be run again; it was not run again before the restart,
  so as not to have a storage change under way when the machine goes down.
