# A system manager request that needs authorization opens a password dialog on the owners desktop

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 43604 to 43616 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  their gate review starts together and does not wait for the other eight.
  A DESIGN RUN PUT A PASSWORD DIALOG ON THE
  OWNER'S DESKTOP at 2026-11-22T17:24:06-05:00 ("Authentication is required to send a UNIX signal to the processes of
  'systemd-random-seed.service'";
  the owner asked "Is this yours?").
  It was the forced-removal design run:
  its feed holds
  `systemctl kill --signal=KILL systemd-random-seed.service`,
  run as a probe of what an account's request to signal a
  system unit answers,
  which ended after 25.2 s with "Failed to kill unit systemd-random-seed.service:
  Connection timed
  out";
  the journal has the agent's request at 17:24:06 and its cancelling at 17:24:31,
  and `polkitd` wrote that the
  session "FAILED to authenticate" for `org.freedesktop.systemd1.manage-units`.
  Nothing was signalled,
  and the unit's
  `MainPID` read 0 just before.
  The briefs said never `sudo` and never start a service manager,
  and said nothing about
  asking the system manager to act.
  All ten design runs were sent the rule at once (no `systemctl` verb that acts without
  `--user` on a unit the run started,
  no call to `org.freedesktop.systemd1`,
  `login1` or `PolicyKit1` on the system bus,
  no
  `pkexec`,
  `--no-ask-password` on every `systemctl`,
  no signal to a process the run did not start;
  the answer comes from
  the source or is owed to a guest),
  the shared brief text carries it,
  and the skill `pi-as-subagent` now tells every
  brief to say it.
  The build rounds going were not sent it:
  their briefs put root-only work in guests.
  THE OWNER'S ANSWERS
