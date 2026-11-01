# Translation repair history 2026-08-19 to 2026-08-21: the scope correction that removed the full corpus pass

The owner's instruction of 2026-08-20 that there is no full corpus pass,
the rule that replaces it of sizing a run to its question,
what the earlier stops cost,
and the two-entry validation that started.
The sections its warning says size their work against a corpus-wide pass are not in this file;
the ones that gate or time a full pass or its relaunch are in these files of this group:
`translation-repair-history-2026-08-19-to-21-corpus-pass-stopped-on-purpose.md` for the stop and the relaunch gates,
`translation-repair-history-2026-08-19-to-21-reading-stage-re-ask-settled-entry-and-census.md` for the gate on a pass
and the cost figure to read before planning one,
and `translation-repair-history-2026-08-19-to-21-window-in-repair-lane-and-verify-instruments.md` for the fix that lands
before the 92-entry relaunch.

Part of [Translation repair history: 2026-08-19 to 2026-08-21](translation-repair-history-2026-08-19-to-21.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## SCOPE CORRECTION 2026-08-20: there is no full corpus pass, ever

Owner instruction,
verbatim:
"We don't need 'the full 92-entry pass',
ever."

READ THIS BEFORE PLANNING ANY RUN.
Several sections above size their work against a
corpus-wide pass and treat it as the thing standing between the pipeline and the release.
It
is not,
and it never was:
the deliverable is THE PIPELINE PACKAGE,
and "production ready for
at least this corpus" means the pipeline handles the corpus rather than that the corpus gets
processed end to end.

WHAT THAT DELETES from the plan:
about four days of wall clock that sat on the critical path,
the sharding question,
the argument about measuring a superseded pipeline,
and the pressure
that made an eight-hour verify feel cheap by comparison.

THE RULE THAT REPLACES IT:
SIZE THE RUN TO THE QUESTION.
A pipeline change is validated on
the entries that can actually show it,
and no others.

`#107`'s window is the worked example.
Its two concrete pieces of damage live in two entries,
`lintong` for the duplicated farewell and `saurikissa` for the severed sentence,
so its
validation is those TWO entries and roughly two hours.
The five-entry version was sized by
the flag list rather than by the question,
and the extra three would have bought a slightly
wider replacement-rate reading for six more hours.

### What this cost before the instruction arrived

One 92-entry launch,
killed after four minutes with zero settled artifacts.
One five-entry
verify,
killed after forty minutes with zero settled artifacts.
Both stops were right on
their own reasoning at the time,
so the waste is about forty minutes of one entry.

### What is running now

`--only lintong,saurikissa` into `~/temp/agent/win107-min-20260820`,
on the build carrying
`#126`'s key fix,
detached.
Gates read with
`node ~/temp/agent/window-gates.mjs --new ~/temp/agent/win107-min-20260820/artifacts`.

`#126` LANDED FIRST AND ON PURPOSE,
which is the one piece of sequencing that still mattered
under the new scope:
it re-keys every windowed slice,
so any run started before it would have
had its slice cache invalidated the moment it landed.
