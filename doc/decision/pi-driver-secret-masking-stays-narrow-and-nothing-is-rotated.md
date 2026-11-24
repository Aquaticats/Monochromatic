# Pi driver secret masking stays narrow and nothing is rotated

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 45890 to 45907 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


### Answers of the evening, the long sitting (2026-11-15, asked from about 19:22 after the owner's "Ask me all the questions now.", written here at 19:44:14-05:00)

Eleven rounds through the question tool.
Each answer is quoted as chosen or as typed;
what follows a quotation is the session's reading,
to be vetoed.
Not yet acted on unless a line says so.

Credentials and masking:

- The string after `PNPR_SECRET=` in a transcript of 2026-09-15:
  **"Bad question.
  You can check the repo for it."**
  A measurable fact was asked as a preference.
  The session checks it itself,
  in process,
  printing no value.
- Runs started without the values in reach:
  **"Masking only,
  and keep the masking work narrow.
  There are gotta be more and more ways through always,
  and we shouldn't fight that.
  We'll accept the risk and rotate.
  And Neuralwatt is ZDR."**
- Where narrow ends:
  **"Plain case and no damage (Recommended)"**:
  masking is merged when a review that builds and runs shows that every declared value printed as it is gets masked,
  and that masking never damages a session file or another run.
  Every other way through becomes a stated limit and is not fixed.
- When to rotate:
  **"After narrow masking is merged (Recommended)"**.
  This replaces "Keep waiting for masking" as the bar.
- Encoded containers,
  wrapped base64,
  values under eight characters:
  **"Accept all three as stated (Recommended)"**.

From lines 46009 to 46020 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


Round nineteen (asked and answered at about 22:24,
written here at 2026-11-15T22:26-05:00),
on the review of the credential masking at `2f303f1bc9`:

- The review passed eleven arrangements of store and environment (143 checks over tool results,
  session files,
  transcripts,
  feeds and kept files),
  a kill at every step of a rewrite,
  and two runs at once beside a foreign file;
  it reproduced six things that each need an unusual set-up,
  and called both points of the narrow line "not shown" on them.
  Asked whether those six hold the merge,
  the owner chose **"Merge now;
  fix three after (Recommended)"**.
  So:
  the branch is merged,
  a canary run is started,
  the names-only scan is made,
  and the list of names goes to the owner for the rotation.
  The author then fixes three in one more run:
  a value under eight characters in the environment for a name whose store value is long leaves the long one unmasked;
  a declared name ending in `_FILE` with different long values in store and environment leaves the environment's unmasked;
  a session file that Pi itself reports is masked without a proof that this run made it.
  The other three become stated limits in the package's docs:
  a tool result whose `details` throws when read keeps its text;
  a result with a cycle is withheld while its full-output file keeps the value;
  a full-output file in a folder that cannot be written stays as it was.

From lines 46021 to 46028 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


Round twenty (asked and answered at about 22:47,
written here at 2026-11-15T22:50-05:00),
on when the credentials are rotated now that the masking is in `main`:

- Seventeen Pi runs that started before the merge go on unmasked until they end,
  and a value rotated while they run can be printed by them.
  Asked when the rotation is done,
  the owner chose **"When the last old run ends (Recommended)"**:
  this session watches those runs and tells the owner the moment the last of them has ended;
  the owner then rotates all 38 once.
  The watch is the unit `wait-until-every-pi-unit-started-before-the-masking-merge-has-ended-…` (at most eighteen hours;
  the units' names are kept in
  `~/temp/agent/rehearsal-of-the-masking-merge-2026-11-15/keep/units-that-became-active-before-the-masking-merge.txt`).
  Until the owner says the values are rotated,
  the sessions the scan names stay untouched and are not resumed.

From lines 46135 to 46135 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  By the owner's "keep the masking work narrow" these are not answered one by one:
  the ways through go on the limits page,
  and the page's false sentences and the stray files are one small correction for the author.

From lines 46319 to 46319 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The masking's small correction is queued as a fresh run from `main` (14:25:06;
  the author's branch no longer exists since its merge):
  the limits page says what the code does (the output files of Pi's bash tool that are not rewritten,
  `--session=<id>` handed over unmasked,
  the feed cutting a line before it masks,
  two false sentences about which file a run may rewrite,
  three statements the code does not do),
  and the driver's tests stop leaving `stand-in-session-<uuid>.jsonl` in the home folder.
  By "keep the masking work narrow" it closes none of the ways through.
  The files already in the home folder are left for this session to remove once the change is in `main`.

From lines 46514 to 46514 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The rotation of the credential values:
  "Ask again after the merge (Recommended)".**
  Put to the owner once the second masking merge is in `main` and a resume canary and a fresh canary have both passed through `main`.

From lines 46522 to 46522 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **When the credential values are rotated:
  "Rotate now,
  runs going (Recommended)".**
  The owner rotates the values and edits the store.
  This session's part:
  the unit that fills run places was stopped at 11:43:25 (it was starting nothing,
  fourteen Pi runs going against its level of thirteen);

From lines 46525 to 46533 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


The rotation is withdrawn by the owner (2026-11-16,
between 11:47 and 11:53):

- The owner wrote,
  while looking at the list of places:
  "I would not consider anything sent to Neuralwatt as compromised.
  So I believe a rotate is not needed."
- This session put the scan's record of 2026-11-14 beside that before acting on it:
  13 Claude Code transcripts hold a real value and went to that harness's provider;
  of 23 Pi session files with a real value six were read for their provider (Neuralwatt and opencode-go) and 17 older ones never were;
  two of the exposed names control more than a provider's bill (a GitHub token,
  which also stands in clear in `~/.bashrc`,
  and a Hetzner token).
- Asked with those facts and four courses,
  the owner answered:
  **"Nothing is rotated"**.
  The exposure is accepted as it stands.
  No value is rotated,
  the answer of 11:42 ("Rotate now,
  runs going") is superseded,
  and the sessions the scan names may be resumed again.
  What does not change:
  no session,
  no brief and no script prints a credential's value,
  and the scan-named session files are not opened or printed by this session,
  because the values in them are still valid.
- The unit that fills run places,
  stopped at 11:43:25 for the rotation,
  runs again at level 13 since 11:50:02.
  No run was started in those seven minutes;
  none was stopped.

From lines 46642 to 46642 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  "Keep waiting for masking".

From lines 15205 to 15212 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **What does not change**:
  the owner's answers of the morning stand
  (one rotation once the masking is merged and shown to hold;
  runs go on starting;
  the affected session files stay untouched).
  The rotation is now at least one more author round and one more review away,
  and until then every Pi run can still send a value it prints to its provider.
  That is said to the owner in this round's message.

From lines 46083 to 46083 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  Asked at about 10:23 how to rotate,
  with the qualification of round twenty in the question itself (a run going at the rotation does not know the new values) and four courses (let the going runs end first,
  stop them,
  rotate with them going,
  not now),
  the owner answered **"Not now"**.

From lines 49352 to 49363 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **When the owner rotates**:
  "All after masking".
  One rotation,
  once the fix that masks credential values before a tool result reaches a model is merged and shown to hold.
  This session tells the owner when that is so,
  through the question tool,
  with what the fix was shown to hold against.
- **Whether runs keep starting meanwhile**:
  "Keep starting (Recommended)".
  The queue goes on filling at the cap.
- **The six session files that hold the values in clear**:
  "Leave until rotated (Recommended)".
