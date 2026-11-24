# Owner instructions of 2026 11 13 after the restart the cap of four and the scratch rules

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 2151 to 2181 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### After the restart: the owner's cleanup, a cap of four runs raised by bisecting, scratch left uncleaned (2026-11-13T10:27:50-05:00)

#### What the owner did and said

The machine came up at 08:56:31.
The owner wrote in the terminal:

- "I rebooted.
  I did a podman system reset -f and deleted some more of my personal data
  and deleted also anything in ~/temp/agent/ last modified before Thursday.
  I put sqfs+zstd archive of them as ~/temp/agent*.zstd ,
  and uploaded them all to pcloud,
  and deleted the older archives locally."
- "I believe this is partially your mess-up.
  I had no idea we had so much stuff in ~/temp/agent/ .
  I believe we need to strengthen our auto-cleanup rules."
- "I've realized both of the disks installed right now would just go boom on a long mv,
  from either direction,
  though that might be an effect of my config.
  I've since set bfq io idle wait to 1 and vm.swappiness to 1,
  which could help with disk somewhat.
  I bought an enterprise SSD which is arriving on Sunday which is supposedly going to help a lot."
- "In the meantime,
  you set your max concurrent pi+sonnet subagents starting from 4,
  then bisecting up until we hit this disk's ceiling.
  We'll re-find the ceiling once the new disk arrives.
  If the new disk still doesn't solve the problems sufficiently,
  we'll look at Sail Research sandboxes and more."

So,
as the project's own rules until the owner changes them:

- This session runs at most four agents at once,
  Pi runs and Sonnet subagents counted together,
  whatever the provider pools would take.
  The pool limits in `package/pi-shared/subagent-driver/src/pool-limits.ts` stay what a provider answers at once;
  they are no longer a number to fill to,
  and "you have 62 slots" in the section "Scope of the work:
  the Git wrapper,
  and the driver,
  mpm and Cargo it is worked on with" is void while the cap stands.
- The cap is raised step by step until this disk's ceiling is found,
  and found again after the new drive is in
  (expected 2026-11-17;
  the owner said on 2026-11-14 at about 14:39 that the arrival slipped from 2026-11-15).
- The scratch folder is to be cleaned by rule,
  not by the owner.
- `podman system reset` replaces the image prune of the wind-down section;
  that step is closed.
- No long move or copy between the two drives in either direction.

From lines 2223 to 2243 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### The cap and how the ceiling is looked for

- Until the launcher carries the cap,
  this session counts by hand before every start:
  `pi-*` units of this work plus Sonnet subagents going,
  at most four.
  The hourly loop is not set up again before whatever fills free slots reads the cap.
- Steps:
  four,
  then double (eight,
  sixteen,
  ...) until a level fails,
  then halve the gap between the last level that held and the one that failed.
  Only real queued work is run;
  no load is made for the test.
- The owner's answers through the question tool,
  2026-11-13 after 10:34:
  - Whose runs count:
    "Every agent on the machine (Recommended)".
    So the four are every Pi run on the machine,
    whoever started it,
    plus this session's Sonnet subagents;
    with one run of another session going,
    this session starts three.
  - What counts as hitting the ceiling:
    "Measured symptoms,
    but \"a commit through the wrapper takes longer than your rule\"
    is obviously a bad \"symptom\" since that might just say the wrapper is bad."
    So a level fails on the first of two things:
    a Pi run gets no answer to its first prompt in 120 s or ends on a disk error;
    or a 4 KiB write-and-sync probe on either drive takes over 5 s twice in a row (the 5 s is this session's figure,
    offered in the question and not objected to).
    A level is held at least an hour and until its runs have each built once.
    The wrapper's commit time is written beside each reading and fails no level;
    nor do the kernel's stall figures.
  - When a run's scratch is deleted:
    "At run end,
    all but keep/ .
    Oh and I forgot to say all the dir names were extremely cryptic and shouldn't have been."
    So the launcher gives each run one scratch folder and deletes it when the run ends,
    except a `keep/` subfolder for its report,
    logs and measurements;
    a subagent's brief names one folder the same way and its starter deletes it on return;
    and every scratch folder's name says what is in it to someone who was not there.
- At every reading the count of all agent runs on the machine is written beside the figures,
  because the ceiling is the disk's.

From lines 2302 to 2310 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The rules `1SF`,
  `SFN` and `SZR` stand as written;
  they concern the scratch folder wherever it is.
  (Note of 2026-11-22:
  `SZR` no longer stands.
  The owner removed it that day,
  "Please remove the SZR rule.
  It unnecessarily uses tons and tons of compute.",
  so no pass measures the scratch folder's size any more.
  `1SF` and `SFN` stand.)
  A rule for what agent work leaves on the home drive (container images above all) waits for the measurement.
