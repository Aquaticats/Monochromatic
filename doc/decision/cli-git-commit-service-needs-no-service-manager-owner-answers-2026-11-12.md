# Cli git commit service needs no service manager owner answers 2026 11 12

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 25974 to 25993 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### The owner adds to the systemd answer (typed after 12:11:18, while these were being recorded)

- "Also regarding systemd answer: We're also supposed to workaround systemd issues."
- It adds to the section
  "The owner: "Remove systemd as a hard dependency. Bazel doesn't need it."":
  besides needing no service manager and using one only as an enhancement,
  the wrapper works around the defects of systemd it meets.
- This session's reading, to be corrected if wrong:
  a defect of systemd is never a reason for a wrapper command to fail, to wait, or to tell a person to act;
  the wrapper detects the state and gets its own work done another way.
  The cases on record are those of issue 6286
  (a manager-wide reload that holds every start for seconds or minutes,
  and systemd v262.1 reading an expired reply as a lost bus name, `src/core/unit.c:3576`, `src/core/service.c:5975`),
  a start request that is not answered within a bound,
  and a unit left in a failed or stale state by an earlier release.
- Sent as a note to the two agents on that path,
  `commit-service-is-started-by-the-wrapper-itself-and-needs-no-service-manager-1` (round 297)
  and `reload-fix-brought-onto-main-where-a-program-link-already-is-1` (round 301):
  `~/temp/agent/notes-to-send/2026-11-12T1214-the-wrapper-also-works-around-defects-of-the-service-manager.md`.

From lines 26930 to 27046 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### The owner: "Remove systemd as a hard dependency. Bazel doesn't need it." (2026-11-12, about 10:40; round 297 started 10:41:51)

Two messages of the owner,
typed while the answers of 10:35:07 were being acted on.

The first:
"Is systemd the problem?"

What this session answered from the record
(`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`,
`doc/planning/git-wrapper-reload-remedy.md`):

- For the memory kill of this morning,
  no:
  the kernel enforced a cap an agent set,
  and the cause is the wrapper's read of a descriptor record (issue 6287).
- For the loss of the desktop session,
  two things together.
  The wrapper's part:
  it asks the user manager for a reload of every unit from inside commits,
  161 of the 208 reloads in the retained journal
  (median 1,635 ms,
  forty over three seconds,
  twelve over ten,
  the last one 139,770 ms).
  systemd's part,
  read from its source at `v262.1` and reproduced in a container by the other session's investigation:
  a manager that is re-reading its units can pass a unit's start deadline before it reads a reply the bus already sent,
  takes the expiry for a lost bus name,
  and stops a running `Type=dbus` unit;
  KWin is one,
  and the Plasma target is bound to it.
  Upstream `main` and `v262.2` have the same code;
  nothing was filed.
- Underneath both:
  every commit depends on a manager that the desktop depends on too.

The second,
the decision:
"Remove systemd as a hard dependency. Bazel doesn't need it."

What follows from it:

- The commit service is started,
  found,
  replaced and stopped by the wrapper itself,
  as Bazel's client does with its server.
  The wrapper is whole on a machine with no systemd,
  and on one that has it,
  no commit depends on it.
- Round 297,
  started 10:41:51,
  wall twelve hours:
  `commit-service-is-started-by-the-wrapper-itself-and-needs-no-service-manager`,
  on `main`.
  Its order:
  an inventory of what the wrapper takes from systemd today;
  Bazel's answers,
  fetched and cited;
  the design
  (`doc/planning/cli-git-commit-service-without-a-service-manager.md`),
  each part committed as it is written;
  then the build,
  the old way taken out,
  and a proof in a container that has no systemd and no bus.
- What the brief names as lost without a manager,
  for the design to answer:
  the service lives in the cgroup of whatever started it
  (here,
  an agent's shell cgroup with a memory cap,
  which is ended whole under memory pressure),
  nobody restarts it but the next command,
  and its logs need a bounded home.
- Left for the owner,
  put forward by that agent with both sides and no code:
  whether a manager may still be used when one is there
  (a transient unit,
  which needs no unit file and so no reload,
  would give the service a cgroup of its own).
  "Not a hard dependency" allows it;
  whether it is wanted is the owner's.
- That question was answered by the owner at about 10:45,
  before it was asked:
  "Progressively enhance. You can also test on Alpine."
  So:
  the base is the wrapper whole with no manager;
  where a manager is present the wrapper may use it to do better,
  never as a condition;
  the base's behaviour is what happens whenever the manager is absent,
  slow,
  refusing or in error;
  no enhancement writes a unit file or asks for a reload.
  The proof that the base needs no systemd runs in an Alpine container
  (no systemd,
  another service manager,
  musl),
  and anything that does not build or behave on musl is a finding of its own.
  Sent to round 297's agent as a note at 10:46;
  its run was shown working by its feed at 10:45:24.
  The repository's rule for container images
  (`doc/decision/container-base-distro.md`)
  is not changed by this;
  Alpine is for this proof.
- The agent for issue 6286 finishes its narrower change (no manager-wide reload) for the next release.
  A note of 10:41:51 tells it,
  and the agent for a service that does not answer,
  to build nothing new that only a manager can give.
- Option choices of `doc/planning/git-wrapper-reload-remedy.md` (item 2.1 of the list for the owner) are overtaken:
  the unit layout it compares goes away.
  The question that remains from it is only what the next release carries meanwhile,
  and that is the 6286 agent's change.
- Rules that name a service manager stay as they are until the code is in:
  agents still never start one on this machine,
  and tests that need one,
  or need its absence,
  run in a container.
