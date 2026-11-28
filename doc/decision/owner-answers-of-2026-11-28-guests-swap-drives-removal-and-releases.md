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
Since 2026-11-28T05:44-05:00 the figure is six,
by the owner's answer in the section
"The guests' memory:
the unit's cap,
builds outside the guest,
and where more memory comes from".
Nine was never set against the limit of the unit the guests run in.

## The guests' memory: the unit's cap, builds outside the guest, and where more memory comes from

Asked through the question tool after 2026-11-28T05:41:48-05:00
and answered before 2026-11-28T05:44:12-05:00;
the fourth question was asked after that and answered before 05:45:26.
What the questions rested on is recorded in
`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
"The guests' unit at its own memory cap,
2026-11-28":
eight guests that had touched 58.2 GiB sat in a unit `mvm` limits to half the machine's memory,
30.8 GiB,
and the unit swapped against that limit while the machine had memory free.

### The unit's cap

The question's own words on what was at stake:
"What should hold?
Ranking:
keep half with four guests > three quarters with six > no cap."
The session recommended keeping half.
The owner chose "Three quarters,
six guests",
whose text was:

> How:
> the divisor in mvm's session-daemon-memory.ts changes so the unit's MemoryMax becomes 46.3 GiB;
> it applies when mvm next starts the unit,
> and I bound guests alive to six.
> Gives:
> six guest rounds at once.
> Gives up:
> host builds,
> the gate (its unit may take 20 GiB) and the desktop share 15.4 GiB,
> so they swap instead.
> Second because it moves the squeeze from guests to builds.

This replaces the project's choice of 2026-11-21 for issue 6338,
"half of total".

### When the new cap takes effect

The chosen text said the cap applies when `mvm` next starts the unit,
which had been up since 2026-11-27T20:03:51-05:00 and starts again only once every guest is gone.
So the session asked when the 46.3 GiB takes effect,
and the owner chose "Now,
on the running unit (Recommended)":

> How:
> I run `systemctl --user set-property --runtime mvm-libvirt-session.service MemoryMax=47364M`.
> systemd writes a drop-in under /run/user/1000/systemd/user.control/ (gone at the next reboot)
> and sets the group's limit at once;
> no guest restarts.
> The eight guests alive have touched 58.2 GiB,
> so they fill 46.3 GiB and still keep about 12 GiB in swap until two end;
> everything else drops to 15.4 GiB at once,
> where it had 7 to 13 GiB to spare,
> so host work swaps and the gate waits longer for its 8 GB.
> Undo:
> the same command with MemoryMax=31576M.
> mvm's code changes too,
> so later starts agree.

What was done:
the command ran at 2026-11-28T05:45:26-05:00 and exited 0.
The unit's `memory.max` went from 33,109,835,776 to 49,664,753,664 bytes,
the eight guests stayed up,
and the unit stayed active.
One thing differs from the option's text:
the unit is a transient one,
so systemd wrote the drop-in to
`/run/user/1000/systemd/transient/mvm-libvirt-session.service.d/50-MemoryMax.conf`,
not under `user.control/`.
It is gone at the next reboot all the same.

### Building the test binaries outside the guest

The owner shelved `doc/planning/mvm-guest-memory-held-for-a-rounds-life.md` on 2026-11-26 until the crate split.
Asked whether that night's reading reopens its one question on where the crate's test binaries are built,
the owner chose "Reopen that one question (Recommended)":

> How:
> one round measures the route on the ZFS recipe's guest:
> the test binaries are built once per commit in a container whose glibc is the guest's (Ubuntu 24.04),
> at the same absolute path,
> then pushed into a 4 GiB guest that runs no compiler.
> It returns as a measured design for you to accept;
> no driver changes before that.
> Gives:
> possibly eight small guests under the cap.
> Gives up:
> one round's work now;
> the container build cost 289 s and a 9.0 GiB host peak in its one measurement;
> the crate split will move the figures,
> which is why you shelved it.

The proposal's other questions stay shelved.

### Where more memory comes from

The owner had written,
earlier that night,
"If you think me giving more resources to you,
hardware,
or through a provider,
would make you work better,
tell me."
The session answered that memory for guests would,
and offered three things of which several could be chosen:
two more 32 GiB modules
(the board has two of four slots empty and its firmware reports a ceiling of 128 GiB),
guests on OVHcloud,
and neither.
The owner chose "Guests on OVHcloud" alone:

> How:
> guest rounds rent a machine by the second through mvm's new ovh backend,
> so a guest uses none of this machine's memory.
> Not working yet:
> neither billed run produced a machine that answered,
> the sizes design came back from its gate read not ready (five High points),
> and a third run needs your yes.
> I would finish that design and bring its questions to you.

So no memory is bought,
the sizes design
(`doc/planning/mvm-create-takes-vcpus-and-memory-and-picks-a-size.md`)
is revised and its questions go to the owner,
and no billed run is made before the owner says so.

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

### How the swap partition is encrypted, and what a yes covers

The option the owner chose said "an encrypted swap partition" and nothing about how.
The session then built it as plain dm-crypt,
`aes-xts-plain64`,
with a 512-bit key read from `/dev/urandom` each time the mapping opens,
the setup the zram writeback partition on the same drive already has,
and switched it on at 2026-11-28T03:40:32-05:00.
The owner,
seeing it reported:

> Holdup,
> encrypted swap?
> Encrypted with what?

Told how it works,
and asked whether it stays a random key each boot,
becomes a stored key,
or loses its encryption,
the owner answered:

> Yes,
> random key each boot.
> My objection was not encryption itself - my objection was you didn't confirm how exactly the
> encryption would work with me before doing it.
> Now that you've made it clear,
> I approve.

So the partition stays as built.

Offered a rule about encryption,
keys and credentials,
the owner answered:

> Too narrow.
> This pattern isn't about enc/key/cred/whatever..

Offered the general reading
(an option named a mechanism by a label only,
the yes covered the label,
and the session then picked the mechanism itself),
the owner chose "A yes covers what was written (Recommended)".
Two rules went into `AGENTS.md` for it,
`OHW` under "Present options with pros,
cons,
and a personal ranking"
and `YSC` under "Match action scope to the request verb".
Not chosen:
the same two with a third rule that stops the session before every lasting change to the machine.

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
