# Scratch expires after a day, and the coordinator keeps what lasts

Recorded 2026-11-26T19:15:00-05:00.
The owner decided this on 2026-11-26.

## What the owner set

After the coordinating session had left 2,419 folders (17.9 GiB) under `~/temp/agent`,
the owner wrote:
"If this keeps up I might have to set ~/temp to auto expire in 24h rolling."
and then:
"I just did this. You've been warned."
with `/etc/tmpfiles.d/temp.conf` reading `d /home/user/temp - - - 24h -`.

`systemd-tmpfiles-clean.timer` runs it daily and 15 minutes after a boot.
The home dataset is mounted `noatime`,
so reading a file does not keep it:
a file survives only if it changed in the last 24 hours
(`tmpfiles.d(5)`, "Age").

When the coordinator began carrying its scratch tools into the repository on a side branch,
the owner wrote:
"Don't promote anything in ~/temp that breaks repo rules or plain broken into the repo.
If you can't move everything before the scheduled cleanup, too bad.
That is for you to learn."
The branch was removed unpushed.

## Who keeps what lasts

Asked whether `AGENTS.md`'s rule that a run keeps only its `keep/` folder should change,
the owner chose no option and wrote:
"Some reports, logs, and measurements are supposed to be temporary.
The coordinator has the responsibility of organizing/rewriting reports, logs, and measurements that are sorta permanent into the main repo."

So a run's `keep/` folder stays the place for its reports,
logs and measurements while they are needed,
and the session that started the run rewrites the lasting ones into the repository's records,
by the repository's rules,
before the folder expires.
`AGENTS.md` carries it as rule `SXD`.

## Why this was needed

The owner had already said on 2026-11-11:
"I don't wanna see you edit pi-runs outside the workspace again."
(`doc/decision/pi-driver-and-provider-owner-answers-2026-11-09-to-2026-11-11.md`).
The coordinator kept writing launchers,
merge lanes and gate runners under `~/temp/agent` after that,
and repository documents came to cite scratch paths as evidence:
801 documents with 3827 citations,
counted on 2026-11-26.
Those citations point at nothing once the cleanup reaches them;
the coordinator's account of the failures is
`doc/planning/pi-round-start-failures-and-what-would-stop-them.md`.
