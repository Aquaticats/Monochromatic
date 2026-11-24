# Pi subagent driver owner answers 2026 11 15 to 2026 11 18

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 45830 to 45840 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **Keeping a Pi run out of the main checkout**: "Protect, containers via manager (Recommended)".
  The checkout stays read-only for a run through systemd's path properties;
  a run's containers are started outside the run by the account's service manager.
  The owner was told, and accepted by choosing it:
  a container started that way can mount the checkout writable, one more stated hole beside the transient-service one,
  since the protection guards against accidents and not against a run that tries to get out;
  `link` and `rename` across the run's folders end `EXDEV` and that stays a stated limit;
  and `podman` had not yet been run through the route when the question was asked.
  So `EXDEV` no longer stops the driver branch as a finding it brings.
  The options not taken: hold the branch for a protection without mounts (Landlock, unmeasured);
  the protection with no containers in a run; no protection.

From lines 45947 to 45954 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


The Pi driver and reviews:

- A review that does not cover its change: **"Restart once by itself ON A DIFFERENT MODEL"**, and which model: **"The other of the pair; Sonnet while Qwen is dry (Recommended)"**.
- A review's unit and a read-only worktree: **"Authors only"** (against the session's recommendation).
- A worktree at a run's name with no proof of who made it: **"Remove after a bound"** (against the session's recommendation).
- A wrapper process in a run's worktree: **"Do not count the commit service (Recommended)"**.
- A run kept because its scan cannot be read: **"A shorter limit, then end as failed (Recommended)"**.

From lines 45989 to 45990 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Pi driver, the queue of runs: **"Folder, one file per entry (Recommended)"**. The lock file over one JSON file is set aside; the rule on lock-free concurrency decides it.
  The scratch queue of this session is rewritten with it at the changeover.

From lines 46064 to 46064 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  The owner's earlier answer for a thin review ("Restart once by itself ON A DIFFERENT MODEL", "The other of the pair; Sonnet while Qwen is dry") has no other model left to go to while Sonnet is the only reviewer:

From lines 46078 to 46079 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The owner's two answers (2026-11-16, about 10:17).** A thin Sonnet review: **"Kimi for these only"**: the Kimi pool is reopened for one command for each thin review (`PI_RUNS_POOL_LIMITS=neuralwatt-kimi-flex=20` on that start alone); the pool stays closed for everything else, and the adoption recorded on 2026-11-15T23:42 (hold a thin review) is replaced by this.
  The open LFS audit of `0.6.1-alpha.14`: **"Sonnet, current code (Recommended)"**: it is run as a Sonnet audit of the LFS path in today's `main`.

From lines 46508 to 46508 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **How much Sonnet review: "All queued, three at once (Recommended)".** Asked with the measurement beside it (four reviews used 223,867 to 582,762 subagent tokens, mean about 372,000, 17 to 27 minutes each; about fifty tips queued, on the order of 19 million tokens).

From lines 46516 to 46518 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

One more answer, at about 11:34 on 2026-11-16, asked with the morning's measurements beside it (reviews finish at up to about 7 an hour at three places, author runs end at about 6 to 7 an hour, so 49 queued reviews do not shrink):

- **How the 18 agent places are split: "5 reviews, 13 Pi runs (Recommended)".** Five Sonnet reviews at once; the unit that fills Pi places runs at level 13 since 11:34:59 (no going run is stopped). The cap of 18 is unchanged.

From lines 45697 to 45698 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

After a "Pi agent finished" notification came a minute before the coordinator woke, the owner asked "What even is the
idle window for?" and then "And shouldn't it be defaulted to 0?". Adopted: `--idle-seconds` becomes optional with

From lines 45769 to 45771 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **Agent cap:** "The cap shouldn't be 20. The 20 cap was set because my old disk was slow, and we're on the new disk now.",
  then, after the session read that as no cap, "I didn't tell you to outright remove the cap. You're meant to bisect for it.",
  then "Start from 20 instead." The search on the new drive and its readings are in `doc/handover/agents-at-once-on-this-machine.md`.

From lines 45772 to 45776 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **Git for agents:** "A: Build wrapper by cargo now (Recommended)": no Pi run could start, because the driver's launcher refuses
  a session whose `git` is not the wrapper and the wrapper's link comes from the held install.
  The session first linked it for every shell in the checkout and had told the owner their own shell would keep `/usr/bin/git`,
  which was wrong: `.config/mise/conf.d/20-env.toml` puts `.cache/monochromatic-git-policy-cli-bin` on PATH for every shell there.
  The session took the link out and made an agent-only folder; the owner then wrote "Do the owner-wide change anyway."

From lines 46035 to 46039 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

Reviews and audits move from Kimi to Claude Sonnet (the owner's message of about 23:27 on 2026-11-15, written here at 2026-11-15T23:36:02-05:00; not asked, the owner wrote it unprompted):

- The owner: "I just checked my bills and found that Kimi K3.1 Pro is almost prohibitively expensive. Please use Sonnet for reviews and audits from now on.
  Close the Kimi pool for now, but let the already running runs finish. Anyway, I'll probably figure out something better in the next week. That's it, going back to sleep now."
  And, a few minutes later: "Do the AGENTS.md edit too. Yes this decision is probably temporary, but AGENTS.md is expected to change often too."
