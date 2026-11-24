# Reviewers told to find issues return issues so the coordinator weighs them

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 49504 to 49540 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Cross-vendor review findings (2026-11-09T13:16:01-05:00):
  the owner wrote
  "If you ask GPT to find issues,
  it will find issues.
  Be sure to dismiss them via your own judgement at some point."
  and,
  on a coordinator line that blamed the asking,
  "I'm not asking you to ask differently.
  I'm asking you to use your judgement on GPT's results.
  Sometimes they just nitpick."
  and,
  on pasting that last sentence into the preamble,
  "don't.
  Phrase smartly.
  If you tell GPT to find issues,
  they will find issues no matter what,
  even if the issues aren't really good issues.
  That's why I said to use your own judgement."
  and,
  later,
  "assign a code to the rule and put it in AGENTS.md .
  Remove the "a Claude subagent is a second pass" line."
  `AGENTS.md` rules `XVW`,
  `PFW` and `WGH` carry it,
  split in three because `RLM` caps a tagged rule at 200 characters and the command alone takes most of that.
  The review request stays as it was.
  The coordinator's working bar,
  a revisable suggestion and not an owner rule:
  fix what ordinary use or a killed process can cause,
  and dismiss what needs someone to deliberately set up the situation.
  First dismissals under it:
  Pi's CLI path passed as a data argument to an unrelated Node program
  (`package/wg/cli-quicker-exempt/src/update_entry_points.rs:183`,
  review `~/temp/agent/reviews/wg-quicker-providers-round-1-sol.txt`),
  and a FIFO replacing a launcher record between `metadata` and `read_to_string`
  (`repair/package_repair_toolchain.rs:136-143`,
  review `~/temp/agent/reviews/tokio-repair-round-2-sol.txt`).
