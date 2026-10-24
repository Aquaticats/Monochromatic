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

Options that were not built, best first:

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
