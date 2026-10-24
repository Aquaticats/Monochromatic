# TypeScript 7.1.0-beta answers `textDocument/inlayHint` with hints that stand outside the requested range

`tsc --lsp --stdio` from `typescript@7.1.0-beta` answers a range request with the hints of every syntax node that
reaches into the range,
and a node's hints stand where the node puts them,
not where the range is.
A statement that begins above the range and has a body that runs through it answers with the hints of its first line,
although that line is not in the range.

The IDE (`package/desktop-app/ide`) asks for the lines around the view and keeps hints per line
(`Annotations::merge_labels` in `src/annotation/blocks.rs`).
Before the fix recorded here it treated every hint of an answer as part of that answer,
so each answer for lines past a long statement added that statement's hints to its first line again.

## Symptom

Opening `package/build-tool/css/src/build.unit.test.ts` (286 lines) and clicking around the minimap:

- The block of virtual rows above line 96 (`const integrationChildren = integrationFixtures.map(...)`)
  held the same four hints twice,
  three times,
  and up to five times,
  so every line below it moved down by one more block each time.
- The minimap's picture,
  which is drawn from the same blocks,
  showed the repeated rows as a band of overdrawn text,
  and the box and the picture jumped as the repeated rows came and went.
- The repeated rows went away for the lines an answer asked about,
  so after clicking back to line 96 the hints were doubled until the next answer covered it.

With the window's own log (`RUST_LOG=ide_app=debug`),
the count of hints on line 96 after each answer,
measured before the fix with a probe line in `merge_labels`,
was:

```text
first=0   last=154  answered=41  line96=4
first=137 last=287  answered=68  line96=8
first=70  last=287  answered=86  line96=4
first=146 last=287  answered=62  line96=8
```

An answer that asks for line 96 replaces its hints (4);
an answer that does not ask for it adds 4 more.
Answers that follow one another without a covering answer between them add 4 each.

Frames of the code view recorded in the nested compositor
(1080 by 1880 logical px at scale 2)
while the pointer clicked the minimap where line 96 is in view,
then far away,
then back,
with a real TypeScript 7.1.0-beta server on a disposable copy of the file:

- Installed build of 2026-10-22:
  the distance from the line number 95 to the line number 96 was 130 px in 5 recorded frames
  (about 0.2 s at the 22.5 frames per second the recorder kept),
  then 82 px in the remaining 30 frames.
  The block of line 96 held 6 hint rows for those frames instead of 3.
- Build with the fix:
  82 px in all 34 frames after the click.

## Probe

The server was started on the project's own TypeScript
(`node node_modules/typescript/bin/tsc --lsp --stdio` in `package/build-tool/css`),
initialized with the `typescript.inlayHints` settings of `src/language/config/typescript.rs`,
and asked for hints over line ranges of the file
(zero-based,
end exclusive,
as `Annotations` is told them).
Hints on lines outside each range:

```text
range [0, 95)    asked: 21 hints,  0 outside
range [0, 100)   asked: 27 hints,  0 outside
range [100, 200) asked: 64 hints,  4 outside: 96:25 `: TestDescriptor<DescribeResult>[]`, 96:52 `callbackfn:`,
                                              96:68 `: { readonly label: ... }`, 96:70 `: TestDescriptor<DescribeResult>`
range [150, 250) asked: 61 hints,  8 outside: the four of line 96, `opts:` of line 117, `opts:` of line 145,
                                              `: Promise<void>` of line 147, `options:` of line 149
range [200, 286) asked: 32 hints,  4 outside: lines 181, 184, 189, 191
range [95, 96)   asked:  4 hints,  0 outside
```

A request that covers line 96 returns its four hints once.
A request that starts after line 96 and before the end of the statement returns them too.

## Cause

A range request returns the hints of the nodes that intersect the range.
It does not drop a hint whose own position is outside it.
The protocol text was not read for this record,
so whether it permits that is unverified here;
the client has to do the limiting either way,
because its store replaces whole lines.

## Fix

`merge_labels` keeps only the hints of a snapshot that stand on the lines the snapshot asked about
(`first_line` to `last_line`),
which are the lines whose old hints it removes.
A line the answer did not ask about keeps what the answer that asked about it gave,
and a debug record names how many hints were left out.

The tests are `hints_outside_the_asked_lines_are_not_part_of_an_answer` in `tests/annotation.rs` and
`an_answer_for_other_lines_adds_no_hint_to_a_line_it_did_not_ask_about` in
`src/native/annotation_range_tests.rs`,
and the guards `hints-asked-lines` and `hints-asked-lines-window` in `bin/inspect-source-guards.ts`
remove the rule in a disposable copy and watch those tests fail.

A hint of an unasked line is not lost:
the next answer that asks for that line returns it,
and the store takes it then.

## Residual first-visit difference of the minimap: cause measured

After the fix,
the minimap crop of a first click at the bottom of the file (y=1700 of the 1080 by 1880 logical window at scale 2)
differed by 994 px (device rows 2500 to 3238) from the crop after a visit to another place.
The earlier guess was that the server's first answer is incomplete (68 hints against 72).
Measured on 2026-10-24, that guess does not hold, and the picture is not what differs.

- The server's first answer is complete.
  On a disposable project with the project's own server and the IDE's settings,
  five answers in a row for the ranges the IDE asks ([0, 154), [137, 287), [137, 287), [0, 154), [137, 287))
  were identical hint by hint for equal ranges (68 and 68 and 68, 41 and 41),
  after delays of 0, 100, 500, 2000 and 6000 ms between opening the file and the first request.
  For eight ranges the hints standing on the asked lines equal the same lines of the full-file answer (90 hints),
  with 0 missing and 0 extra.
  The 72 of the earlier note was not reproduced;
  68 is the whole answer for [137, 287) of which 61 stand on the asked lines.
- The picture is the same.
  The 994 px are the outline of the minimap's viewport box, drawn about 120 device rows lower in the warm crop (top edge at row 2501 against 2622),
  and a few scattered pixels.
  A first click lower (y=1870) and the click at y=1700 after a visit elsewhere give crops that differ by 0.43 px,
  and the second click gives the same crop whether it follows the first visit or the visit elsewhere.
- The code view shows why the box differs.
  The first click showed lines 215 to 273 (the view's top line reported to the hint request is 214),
  the same click after a visit elsewhere showed lines 224 to 287 (top line 223).
  The minimap request posted after the first click was for a layout of 8688 logical px,
  because only the hints of the first answer ([0, 154), 41 hints) were in the layout when the click was mapped to a scroll offset;
  the answer for [137, 287) arrived 20 ms after the visible lines were reported and made the layout 9522 px.
  After a visit elsewhere every line already holds its hints,
  the layout is 9522 px from the click on, and the same click lands at the end of the file.
- A window that is tall enough for the first request to cover the whole file is the control.
  At 1080 by 3760 logical px the first request asked for [0, 287), the answer held all 90 hints,
  the layout was 9522 px at once, and the first visit against the visit after another place differed by 0 px in all three pairs.

So the IDE does what it is built to do:
a click on the minimap is mapped through the layout the window has at that moment,
and a line that was never asked about holds no hint rows yet.
The first visit to a place that no earlier request covered therefore lands where the incomplete layout puts it,
and the box moves to the place the final layout gives only on the next click.
No picture is left stale, and no answer is lost.

Options, best first (the owner chose the first on 2026-10-24, and it was built the same day:
see the section "Re-aim once of a minimap press: built" below):

- Re-aim once: when an answer changes the layout height while the view still stands where the last minimap click put it,
  scroll to the place the same click gives in the new layout.
  Pro: the click means the same place on the first and on every later visit, with no extra requests.
  Con: a second scroll step shortly after the first, and a rule for when the view counts as untouched.
- Ask for the hints of the whole file once the visible lines have settled, for files up to a size bound.
  Pro: the layout is complete after the first answer, so no click is ever mapped through an incomplete layout.
  Con: the server does work for lines nobody looks at, and a bound has to be chosen; a file over the bound still shows the difference.
- Leave it.
  Pro: no new behavior.
  Con: the first click at a far place lands about nine lines from where the next click lands.

Ranking: re-aim once > ask for the whole file > leave it,
because the first fixes the cause (the click is mapped through a layout that is about to change) for files of every size,
the second removes the cause only for files under a bound and costs server work,
and the third keeps a visible difference that has a known cause.

Probes: `~/temp/agent/mm2/cold.ts`, `ranges.ts`, `exp6.ts` (scratch, not kept in the repository).

## Re-aim once of a minimap press: built

Built on 2026-10-24 after the owner chose "Re-aim once" through the question tool.
The code is `src/native/minimap/aim.rs` in `package/desktop-app/ide`,
with the calls in `src/native/rows.rs` (`settle`), `src/native/viewport.rs` (`place`),
and `src/native/minimap/pointer.rs` (`jump`).

### The rule

A press on the minimap outside the box is kept as three things:
its place on the track (the pointer's distance from the top and from the left edge, and whether Shift was held),
the share of the scroll range the view stood at before the press
(the picture slides by that share of its overhang, so the share is part of what a press means),
and what the press left behind
(the scroll offset, the anchor and head and side of the caret, the stamp of the text, and the size of the view).

When a layout is installed (`rows::settle`, after the new map replaced the old one),
the remembered press is mapped through the new layout by the same code that maps a press,
with the old share applied to the new scroll range,
when the offset the window shows, the caret, the text stamp, and the view size all still equal what the press left.
The view and the caret then go to the new place in the same call that keeps the view still across new rows,
so the correction is part of the frame that brings the rows,
and what the press left is replaced by the new place,
so the next change of the layout is judged by the same rule.

The press is forgotten as soon as anything else happens:

- `viewport::place`, through which every placement of the view passes,
  forgets it when the offset is not the one the press left.
  This covers a wheel step, a key that scrolls, a drag that moved the view, a page step, a find jump,
  and a tree click.
  It also covers a wheel that comes back to the very offset of the press,
  which the comparison of offsets alone would not see.
- The comparison in the new layout's step catches what the placement cannot:
  a scroll step the toolkit has made but the window has not yet been told of
  (rows that arrive below the view do not wait for scrolling to stop,
  and would otherwise take the view away from the step),
  a caret that moved without scrolling (a key with the caret in view),
  another text (an edit, a reload, another file),
  and another size of the view.
- A press that is still held has not been followed.
  The drag it started takes the aimed offset as its start, and then follows the live layout as before;
  a drag that moved the view has followed the press, and the next change keeps the view still.

### What was measured

The disposable project is `/tmp/ide-mm-proj-8MEEkc`
(a copy of `package/build-tool/css/src/build.unit.test.ts`,
with stubs for its two workspace imports and the TypeScript 7.1.0-beta server),
the window is 1080 by 1880 logical px, and the sessions run in the nested compositor with the real server.
Both binaries were built from the same tree, before and after the change.

- The first click of a session at y=1700, from the debug log.
  Before: the press took offset 6840 px (layout 8688 px);
  the answer for lines 137 to 287 (68 hints) made the layout 9522 px;
  the rule that keeps the view still moved the offset to 7310 px, with the same top line, 214,
  which stayed the last one reported.
  After: the press took 6840 px, and the same layout change aimed it again, 6840 to 7674 px (line 286, row 0);
  the next report for hints names top line 223, 223 ms later.
- The same click after a click at y=1000 (the visit elsewhere): 7674 px, top line 223, in both binaries.
  So the first visit and the later visit land on the same first line after the change (223 and 223),
  where they landed on 214 and 223 before.
- The minimap crops of the first visit and of the later visit differ by 994.01 px before the change (the box outline,
  120 device rows apart)
  and by 0.38 px after it.
- The click at y=1000 is aimed again as well: 4884 to 4878 px, from line 178 to line 171 (row 1) of the caret,
  seven lines, about 260 ms after the press, with the answer for the rows,
  as a later press at that place does in the complete layout.
  The caret moves with the view because the press names a place in the layout and the caret goes to the place it names.

### Frames around the correction

The recording at scale 2 captured 37 frames in 1.97 s (18.8 per second),
because 2160 by 3760 px per frame is more than the capture pipeline reads back in a tick,
so the frames were taken at scale 1 (1080 by 1880 px) as uncompressed BMP.
Three recordings of each binary, alternating, 0.4 s before the click to 1.5 s after it,
53.0 to 56.1 frames per second captured,
0 dropped and 0 failed in all six:

- Every recording shows the same three states of the code view in the same order, for both binaries:
  before the press, after the press, and after the rows arrived.
  The change from the second to the third state is one step with no frame between,
  in the one frame that also shows the hint rows.
  The press state is held 14 to 15 frames (about 270 ms)
  after the change in the three recordings of the new binary and 13 to 16 frames in the old.
- After the change the minimap shows, in order, the box at its new place (371 px differ from the press state),
  the picture rebuilt for the new layout (2083 px differ), and a step of 0.01 px that is noise of the render.
  Before the change the minimap passed through two states after the code step, and after it through three,
  the last 0.01 px from the one before.
- The view is not corrected twice: after the correction the layout does not change again, and the view stays.

### Costs

- One more request for hints on the first visit.
  The correction puts the view on other lines,
  so the hints of those lines are asked for after the settling time (`HINT_SETTLE`, 200 ms).
  Counted in the debug log of the first click: one report of visible lines before the change (top line 214, 68 hints),
  two after it (top line 214, then top line 223, 62 hints).
  A press after a visit elsewhere costs one report for the press, in both binaries.
  The second range is within the first answered range,
  so a rule in the hint request that skips a window the last answer already covers
  would remove the extra request; it was not built, because it was not asked for.
  (Built on 2026-10-24 after the owner asked for it: see the section "Skip a hint request that stored answers cover: built".)
- The view moves about a quarter of a second after the press, once, by the difference the missing rows made
  (834 px for this file at the bottom).
  That is the correction the owner asked for, and it is the one visible step.

### Proof that the tests and the guards see the rule

- The first seven tests in `src/native/minimap_aim_tests.rs` all failed on the commit that added them,
  before the rule existed
  (the view stayed at 21162 px where the same press lands at 13374 px in the layout after the hints,
  and in the second change at 22436 against 16624),
  and all eight pass after it (the eighth, an unreported scroll step, starts from the same aimed press).
  The tests of an input after the press start from a press that was aimed again,
  so each fails without the rule rather than passing for lack of it.
- Ten cases in `bin/inspect-source-guards.ts` (`minimap-aim-remembered`, `minimap-aim-share`,
  `minimap-aim-after-install`,
  `minimap-aim-placed-away`, `minimap-aim-place-forgets`, `minimap-aim-unreported-step`, `minimap-aim-caret`,
  `minimap-aim-text`, `minimap-aim-view`, `minimap-aim-drag-start`) remove one line each in a disposable copy.
  All ten tests fail with the line removed, with the expected assertion message, and pass again with it restored:
  28 runs (8 baseline, 10 removed, 10 restored), all accepted.
- The gates on the final tree: `lint` 0, `test:native` 299 passed and 5 skipped in one run,
  `test` 804 passed and 1 skipped, `test:bin` 71 passed, `inspect:no-software-renderer` 0.

Probes and recordings: `~/temp/agent/reaim/` (`arm.ts`, `compare.ts`, `rec.ts`, `runs.ts`; scratch,
not kept in the repository).

## Skip a hint request that stored answers cover: built

Built on 2026-10-24 after the owner chose,
through the question tool,
a rule that skips a hint request
when the last answer already covers the window.
The code is `Annotations::answered_covers` (`src/annotation/coverage.rs`,
`src/annotation.rs`),
which `merge_labels` in `src/annotation/blocks.rs` feeds,
and the call in `hints` in `src/native/language/sync.rs` (`HINT_SETTLE` is still 200 ms).

### The rule

A request for the visible lines asks one view height above the first visible line and two below it,
clipped to the text (`request_lines` in `src/language/hints.rs`).
That request is not sent when its range lies inside the union of the ranges that hint snapshots,
stored for the same text revision,
asked about:

- Only a stored answer adds lines.
  A request that is pending,
  failed,
  or cancelled never reaches the store,
  so it covers nothing,
  and the IDE holds the hints of the covered lines because the same snapshots put them in its per-line store.
- The lines belong to one text revision (file generation and revision).
  The first stored answer for another revision starts the record afresh,
  and a query for another revision is not covered,
  also while no answer for it has arrived.
  So an edit,
  a reload,
  and another file reset it.
  Opening the file in the worker again (an explicit action that restarts failed servers) forgets all lines too,
  because the new worker holds no earlier request.
- Ranges that overlap or touch are joined,
  so two answers can cover a range together,
  and a gap between answers is not covered.
  An empty range is never covered.
- A skipped view is recorded as the last handled one (the debug line is
  `skipped the hint request: stored answers already cover its lines`),
  so the view is not examined again on every tick.
  The worker keeps the window it was last told.

### What was measured

The disposable project and binaries are those of the section "Re-aim once of a minimap press:
built":
a copy of `build.unit.test.ts` (287 lines) with the TypeScript 7.1.0-beta server,
a window of 1080 by 1880 logical px at scale 2,
the nested compositor,
and the debug log.
The base is the tree before the rule,
the fix is the tree with it.
Each scenario ran three times per binary,
alternating base and fix,
under the load the host had
(other sessions were building throughout;
load average 6 to 19).
Requests are the lines `reported visible lines for inlay hints`,
skips the lines `skipped the hint request`.

- The first click at y=1700 of a session:
  base 2 requests in 3 of 3 runs (top line 214,
  then 223),
  fix 1 request and 1 skip in 3 of 3 runs.
  The answer of the first request was stored 250 to 255 ms after the click in base and 254 to 269 ms in fix;
  the second answer,
  498 to 510 ms in base,
  is the one that is gone.
  The second range,
  lines 146 to 287,
  lies inside the first,
  lines 137 to 287.
- A click at y=1000 and then the click at y=1700:
  base 1 request for each click (3 of 3 runs),
  fix 1 request for the click at y=1000 and none for the click at y=1700 (1 skip,
  3 of 3 runs).
  This differs from the expectation that the later press still costs one request:
  the answer to the first click asked lines 70 to 288 (top line 147 plus two view heights clips to the end of the 288 lines),
  so it already asked every line the press at y=1700 needs,
  and all of their hints are stored (86 in that answer).
  In a file where the first click's answer does not reach the end the later press costs what it did:
  with a text of 843 lines (the header of the file,
  then its body three times as functions),
  the click at y=1000 cost 2 requests in base (the minimap re-aim reports a second view) and 1 request and 1 skip in fix,
  and the click at y=1700 after it 2 requests in both binaries (3 of 3 runs each).
- Ordinary scrolling still asks when the view leaves the covered lines.
  In the 843-line text after a click at y=1700 (3 requests in both binaries),
  a wheel of 40 notches up sent 1 request in both binaries (3 of 3 runs):
  the answer was stored 414,
  427,
  and 428 ms after the wheel in base and 392,
  403,
  and 421 ms in fix.
  The spread of the same binary is 14 ms in base and 29 ms in fix,
  and the two binaries differ by less than that spread
  at the median (427 against 403 ms),
  so the hints appear within the same time.
  The three smaller steps that followed,
  one notch up,
  one notch down,
  and 40 notches down to the first view,
  sent 1 request each in base (3 of 3 runs) and none in fix (3 skips each,
  3 of 3 runs):
  their lines were inside what the two earlier answers had asked,
  and their hints were stored.

### Known costs

- A skipped view leaves the worker's window at the last view it was told about.
  The worker asks again from that window after a reload (and when a second server starts late),
  so such an ask names the lines of an earlier view.
  Measured in `an_edit_forgets_the_lines_the_earlier_revision_asked_about` with the view left on a skipped place
  (first requests start at lines 0,
  173,
  and 200;
  the skipped view would start at line 186):
  an external edit sent 2 requests,
  the first starting at line 200 (the window the worker was told)
  and then one starting at line 186 (the view);
  a tree without the rule was not measured for this.
  The answer for the earlier window is dropped as "no longer wanted" when the current window differs.
- With several servers one stored answer covers the lines for all of them,
  although a server that started later has answered fewer lines.
- A later press that lies wholly inside what an earlier visit asked costs no request,
  as the second measurement shows;
  the owner expected "still one" there,
  and the hints of those lines are stored,
  so nothing is missing from the view.

### Proof that the tests and the guards see the rule

- `src/native/language/hint_skip_tests.rs` has six tests that count the requests the scripted server receives
  (its report file),
  and `tests/annotation.rs` has four tests of the covered lines.
- On a tree whose skip is disabled (`if false && covered`),
  4 of the 6 native tests fail and 2 pass:
  `a_view_inside_the_answered_lines_sends_no_request`,
  and the three tests that first show a covered view
  being skipped (`a_view_leaving_the_answered_lines_sends_a_request`,
  `an_edit_forgets_the_lines_the_earlier_revision_asked_about`,
  `opening_the_file_again_forgets_the_lines_asked_about`),
  all with the message
  `a view whose request asks only answered lines sent a request`.
  `a_pending_request_does_not_cover_its_lines` and `a_failed_request_does_not_cover_its_lines` pass on such a tree,
  because the request they wait for is sent anyway;
  they fail against the guard cases `hint-skip-pending` and `hint-skip-failed`,
  which record a request as covering its lines when it is sent.
- Eleven cases `hint-skip-*` in `bin/inspect-source-guards.ts` change one line each in a disposable copy
  (`hint-skip-covered`,
  `hint-skip-noted`,
  `hint-skip-revision-reset`,
  `hint-skip-edit`,
  `hint-skip-revision-query`,
  `hint-skip-empty-range`,
  `hint-skip-ends`,
  `hint-skip-touching`,
  `hint-skip-pending`,
  `hint-skip-failed`,
  `hint-skip-reopen`):
  all eleven tests fail with the line changed,
  with the expected message,
  and pass again with it restored.
  One line is not guarded:
  the record of a skipped view as handled
  (`language.hints.sent = Some((stamp, wanted))` in the skip branch);
  without it only the debug line repeats on every tick.

Probes and scripts:
`~/temp/agent/cr/` (`measure.ts`,
`runs.ts`,
`restart.ts`,
`big.ts`;
scratch,
not kept in the repository).
