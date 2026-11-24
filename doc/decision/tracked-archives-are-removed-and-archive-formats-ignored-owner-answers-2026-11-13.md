# Tracked archives are removed and archive formats ignored owner answers 2026 11 13

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 24090 to 24131 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

The owner's answers,
through the question tool at about 17:33,
each asked with the measurements beside it:

- How the data is kept
  (plain files of everything,
  trimmed to what is cited,
  or a store outside the repository):
  "Trim to what is cited.
  It doesn't discard the evidence for good because we are not going to rewrite history."
- Whether history is rewritten too:
  "Leave history (Recommended)".
- What happens to the three Java archives
  (keep all,
  drop the plugin's,
  drop all three):
  "Override allowlist all 3."
  Read as:
  `*.jar` is ignored like the other formats,
  and the three tracked ones stay tracked through exceptions written beside the pattern.

So,
as the project's own:

- the raw bulk leaves the tree and Git history is where each whole archive stays;
  one record lists every removed archive with its path,
  bytes,
  SHA-256,
  the commit that last held it and the command that prints it from history;
- what a document states that a reader can only check by opening the archive is kept beside it as plain text,
  cut to the records that back the statement;
  a document that names an archive only as the place raw material is kept gets nothing but the pointer;
- every file that names a removed archive is edited in place by rule `WR6`,
  and every file that records a digest of an edited file gets the current bytes and SHA-256 beside the recorded ones,
  round after round until nothing changes.
  The coordinating session had adopted the opposite
  (a file a recorded digest pins is not edited)
  and put it to the owner at about 17:38 with three options;
  the owner chose "Edit all,
  re-record digests".
  The brief was changed to that before the run had a run folder;
  whether the run was sent the changed text is checked against its feed when it starts.

From lines 19318 to 19318 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  "Keep as plain text (Recommended)".
