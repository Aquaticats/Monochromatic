# Cli git host pressure is never an excuse

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 47874 to 47940 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Host pressure is never an excuse

The owner,
2026-11-03 between about 13:05 and 13:17-05:00 (said in the session),
after the coordinator offered disk pressure as the reason commits took 8.99 s,
16.6 s and 35.7 s:
"Disk pressure being high is not an excuse.",
then
"I've said something to the effect of "disk pressure / host / load is not an excuse" like 10 times now.",
then,
on the coordinator's "remove it or move it off the waiting path":
"sometimes the work itself can be optimized.
And we control every part of the wrapper,
including everything it embeds.",
then
"Think optimization can't do much?
No!
Look at our forbidden-strings and forbidden-regex.
Much faster than anything else.
At least 2x faster in every way than official `regex` crate.
And I did it with a much weaker model."
Then,
asked through the question tool how AGENTS.md should carry it,
"Tighten NHR,
add SLW (Recommended)",
followed in the session by
"Host pressure might sometimes be the cause but it's never an excuse.
I also told you if you think the 5s standard cannot be always met even if we put in all the work,
you can ask me to raise it with evidence.",
and
"I told you that a long time ago.
And you never used it."
(that earlier permission was not written into this handover,
SPEC or any planning doc;
a search for it just before the commit that records this entry found nothing).
On the release gate that failed on main on 2026-11-04
(`a_killed_wrapper_with_an_escaped_writer_stops_the_recovery`),
when the coordinator read the failure without naming a cause:
"I don't see any load,
and once again,
load is not an excuse."
Meaning for this effort:
a command slower under load,
disk pressure,
swap or a slow SSD is a defect in the command,
even where the pressure is a cause.
The fix is found by measuring,
under that same pressure,
what the wrapper does beyond native Git,
and then removing it,
making the work itself cheaper (embedded code included),
batching it,
or moving it off the path the person waits on.
Standing permission:
when the evidence shows the 5 s standard ("commits vs native git should not take 5s longer.")
cannot always be met even after all that work,
ask the owner to raise it,
with the measurements;
never accept a miss silently,
and never offer the pressure as the reason instead.
Offering the pressure as the excuse is retracted wherever this handover did it
(the 1996 ms commit was split by step against the load average,
and the slowdown was then attributed to load).
The precedent for how far optimization goes is `package/forbidden-strings/core` and its bench `package/forbidden-strings/core.bench`.
AGENTS.md now carries NHR ("sometimes a cause,
never an excuse or a reason to wait") and SLW.
