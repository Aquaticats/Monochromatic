# Podman run of a systemd init under timeout never returns

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 29391 to 29412 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### A run that waited on a container from 20:42:55 to 22:14:46

`first-commit-after-an-install-does-not-wait-for-the-old-service-1` issued
`timeout 300 podman run --rm ... --systemd=always ... /sbin/init | head -5 &` followed by `wait`.
The command never returned:
`timeout` signals `podman run` with `SIGTERM`,
podman forwards it to the container's init,
and systemd as init does not exit on that signal.
The run's feed was silent for 83 minutes while its unit showed as running.
The coordinator confirmed the container (`cff7f606d26e`) by its creation time
and by its `conmon` process inside the run's own unit,
ended it with `podman kill` at 2026-11-11T22:14:46-05:00,
and sent the run a note
(`note-why-the-container-command-did-not-return.md`:
start detached under a name,
end by name with `podman kill` or `podman stop --time`).
A "running" unit is no proof of work:
the hourly pass now reads each run's feed age
(`~/temp/agent/scratch-scripts/turns-of-running-pi-runs.ts`;
its turn count is wrong,
its feed age is right).
(scratch,
deleted on 2026-11-27;
not kept).
