# Qwen token plan individual terms fit pi runs

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 45386 to 45394 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


**For the owner (terms of the Qwen plan, found 2026-11-19 on that same FAQ page):** "Token Plan Personal Edition is for
individual interactive development through officially designated tools (such as Cursor, Claude Code, and Windsurf).
Using the API key for production automated services, batch scripts, or scheduled background tasks is not allowed." It
also recommends 6 to 8 concurrent agents on the Pro tier. The reviewer pool puts `qwen-token-plan-individual` first (the
owner's word of 2026-11-18), and the coordinator's Pi runs are scripted and run in systemd units. Whether that use fits
those terms is the owner's call; no new run is started on the Qwen plan until then (the benchmark already going keeps
its Qwen control arm). Answered 2026-11-19 in the morning: the use fits, and the hold is lifted (entry "The Qwen plan's
terms fit the Pi runs on it" under "Owner decisions in force").

From lines 44770 to 44772 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

The owner, plain text, on return in the morning: "Regarding the Qwen plan's terms: We talked about this, and we
determined we're fine because our traffic shape is approved and it is indeed agentic coding and you're monitoring
them." So the reviewer order keeps `qwen-token-plan-individual` first, the hold on new Qwen-plan runs written on
