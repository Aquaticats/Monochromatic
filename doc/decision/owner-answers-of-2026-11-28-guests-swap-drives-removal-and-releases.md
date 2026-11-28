# The owner's answers of the night of 2026-11-27 to 2026-11-28: guests, swap, drives, the removal and releases

Status:
the owner's answers,
recorded by the coordinating session of the Git wrapper work.
Each answer is quoted as given.
What the session did about each is stated beside it and is unapproved and non-final.

Two answers of the same night have records of their own:
the layout rule's wiring uses
(`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`,
the amendment of 2026-11-28)
and the OVHcloud backend
(`doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md`).

## The rounds that were running, and what bounds a later batch

The machine ran out of memory on the night of 2026-11-27 with nineteen rounds and ten guests alive
(`doc/troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md`).
Asked after 2026-11-27T23:27:29-05:00,
answered before 2026-11-28T01:24:27-05:00.

To "What do I do with the rounds that are running now?"
the owner answered "Let it ride (Recommended)".

To "For later batches,
which of these bound how much I start at once?",
where several could be chosen,
the owner chose "Bound guests alive (Recommended)" alone.

What follows:
a later batch is bounded by the guests alive at once,
and by nothing else the question offered.
Ten guests is a level that failed that night.
At 2026-11-28T01:24-05:00 nine were alive while memory had recovered,
so the session tells a round to create its guest only when fewer than nine are alive;
that figure is the session's pick,
with the veto open,
until the search in `doc/handover/agents-at-once-on-this-machine.md` finds the level.

## Whether the machine and the repository are set up right

Typed between 2026-11-27T23:27-05:00 and 2026-11-28T01:48-05:00:

> Are we still not configuring our system (software),
> how we compile,
> how we layout what's in the repo,
> our system (hardware and remote) correctly?

> I'm not really frustrated.
> If you think me giving more resources to you,
> hardware,
> or through a provider,
> would make you work better,
> tell me.

Asked which of a Hetzner guest trial and a remote gate proposal to act on,
the owner chose neither and wrote:

> Don't we already have gh?

What follows:
no Hetzner trial is authorised.
GitHub's hosted runners,
which the repository already has,
come first.
The session had recommended a paid provider before it had checked them,
and withdrew that.

## The crate split's fourth round

> Oh and right the crate split.
> If you need me to unblock it,
> ask me.

To the question that had waited in `doc/handover/questions-for-the-owner.md` since 2026-11-27T06:40:00-05:00,
"does the crate split's step 4 run now?",
the owner answered "Read the signatures first (Recommended)".

What follows:
the five signatures of round 4 are prepared as a proposal,
read by a gate reader,
and then put to the owner.
Round 4 does not run before that.

## Swap on the old NVMe

> Would cutting out an actual disk swap from the old nvme help?

Asked which swap to make,
the owner answered "128 GiB swap partition (Recommended)".

> Keep in mind we're going to format all the parts (except what's needed for this system)
> on the old nvme after migration anyway.

> The data on the old nvme can be moved to the old sata.

Asked in which order the old NVMe's work goes,
where the first way was to shrink its file system now and move its data afterwards,
the owner answered:

> 1,
> and you can actually unlock the old sata.
> ~/Downloads/old-drives

What follows:
the old NVMe's Btrfs file system is shrunk by 128 GiB for a plain swap partition,
then its data is moved to the 860 drive,
and no partition of the old NVMe is formatted before the owner is asked again.

## The 860 drive

> In fact,
> please migrate the old 860 systemd setup on this system.

And,
naming the note `/home/user/Plain/Text/Notes/860-luks-auto-mount-at-user-login.md`:

> This should also help - or be updated.

What follows:
the unit that unlocks and mounts the 860 drive at login is carried over from the old system,
and the owner's note says what that took on this one.

## The removal's command path, and releases

A round switched `git worktree remove` of a managed ZFS clone from a refusal to the real removal.
The session told the owner that its guest test had never finished,
that most of its new tests were not shown able to fail,
and that one of its test runs had opened `/dev/zfs` on this machine once,
which the host's rule forbids.
Asked how alpha releases of the wrapper go until the removal has run green in a guest,
where the ways shown were to merge and hold alphas,
to merge and release as usual,
and to hold the merge too,
the owner answered,
before 2026-11-28T02:21:18-05:00,
"Merge,
release as usual".

What follows:
the round is on `main`,
and the wrapper's alphas go out unasked as before
(`doc/decision/cli-git-wrapper-owner-answers-2026-11-12.md`,
item 4.9).
The rounds that make the removal's guest test pass and show its controls able to fail run as ordinary work,
and hold no release.
