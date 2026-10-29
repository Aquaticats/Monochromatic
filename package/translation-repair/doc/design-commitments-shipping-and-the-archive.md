# Design commitments: shipping and the archive

The commitments on what a run ships,
what counts as a defect in a page,
and what stays as the archive has it.

Part of [Design commitments](design-commitments.md),
which lists every file of these commitments.

- **A placeholder in the archive is not content the pipeline preserves.**
  `passArchiveText` (`corpus-run/pass-archive.ts`,
  run inside `preparePassEntry`) folds invisible variants and then strips any paragraph
  that is nothing but a stub token (`(To-Do)`,
  `TODO`,
  `TBD`,
  `WIP`,
  one layer of brackets,
  any case) outside front matter,
  HTML comments and code fences,
  logging `archive: stripped stub marker ... at line N`.
  Measured against the 92 English pages of the pinned corpus:
  one such marker,
  XIEPT2's,
  which the pipeline had published over a finished translation.
  HTML comments stay:
  `entry-notes.ts` reads them as editor comments and a reader never sees them.
  Decision:
  `doc/decision/translation-repair-good-result-over-bad-original.md`.
- **The translate lane's two-ballot text ships where the final round cannot seat two judges.**
  Owner,
  2026-10-29 ("Ship the two-ballot text (Recommended)"):
  "When the final round can't seat two judges,
  the translate lane's winner ships if two judges chose it and it passes the publication rule."
  One judge alone never decides,
  a slice with no wording that passes keeps the archive (2026-09-27),
  and a disputed archive is never a candidate or a fallback.
  Every kept archive records why it stayed,
  and the page's `no-valid-wording` defect says so slice by slice.
  Ledger:
  B1020 to B1025 in `audit-ledger-the-two-ballot-translate-text.md`.
- **Whitespace Markdown renders the same is not a defect.**
  Owner,
  2026-09-27:
  "There is no need to eliminate extra newlines,
  because markdown doesn't care."
  A run of blank lines where the archive has one,
  or a wording whose edge newlines differ from its archive span's,
  renders as the archive does,
  so no pass,
  floor or finding acts on it.
  Spacing that changes rendering is another matter:
  a list's items loose or tight (`list-spread-restore.ts`),
  or two paragraphs joined by a single newline.
  Carriage returns are kept apart by a second ruling:
  asked whether this one retires `corpus-run/line-ending-fold.ts`,
  which folds a model's CRLF to LF at page assembly (ledger A3),
  the owner kept the fold on 2026-09-27.
  The corpus reader already folds the archive to LF,
  so the fold keeps each published page on one line-ending convention,
  and a stray CR shows in diffs and tools even where Markdown renders it alike.
- **A run always ships.**
  Owner,
  2026-09-27,
  rejecting a `verify-published` that would refuse a run over a page no artifact records:
  "One of the rules is no matter what,
  we must ship."
  A check reports what it finds and repairs what it can;
  it never withholds a run's pages.
  A page whose artifact a crash lost between the two writes (`publish-fixed.ts`) therefore ships,
  reported as `PUBLISHED AND NOT SETTLED`;
  measured on 2026-09-27,
  no such page exists among the 214 pages in 372 run directories.
  Asked the same day how the rule reaches the two findings that then still exited 1,
  the owner chose repair and a clean exit:
  a pass starting in a runs directory rewrites from its artifact any page that is missing
  or that differs from what the artifact says ships,
  and `verify-published` prints every finding and exits 0,
  keeping its separate exit 2 for a run it could not read at all.
  Following from the same two rules rather than from an answer,
  a decline removes a page an earlier crash left for the entry,
  so the archive ships as the archive's note says it must.
  That question told the owner no page had ever disagreed with its artifact,
  a count that had only checked pages exist (ledger M18).
  Judged by the build of 2026-09-27,
  77 of the 214 stored pages disagree,
  every one written after the publish-time agreement check existed and so agreeing with its own build:
  the would-ship reader applies `restoreTypography` when it reads,
  and two later fixes there (class 181,
  punctuation inside a closing quote;
  class 147,
  a nested quotation curled as a pair) account for 69 of them.
  Told this,
  the owner chose again,
  the same day,
  that a pass rewrites such pages to the running build's reading:
  a page that does not parse stays as it was and is reported.
  A pass meets an older build's pages only on a resume the operator opted into,
  since the build-generation guard refuses one otherwise.
  The entry in flight when the router's every-provider-dry refusal stops a run is not shipped:
  it stays `INCOMPLETE` and resumable with its cache,
  and the entries settled earlier ship.
  That is a ruling of the lead of 2026-10-22,
  open to the owner's veto
  ([the rulings of 2026-10-22](design-commitments-rulings-of-2026-10-22.md),
  ledger B821).
- **A settled page ships with its defects reported.**
  Owner,
  2026-09-27,
  asked whether the publish-time checks keep refusing a settled entry
  (front matter,
  sealed English originals,
  contributor names,
  link destinations,
  merged headings,
  and whether the page parses),
  with the archive's page shipping in its place:
  "Ship with the defect".
  The page ships and each failed check is reported,
  in the run log and on a line a grep over a pass totals;
  a page that does not parse still refuses,
  since it would break the site build.
  Measured before the ruling:
  seven real entries were refused for one dropped link destination each,
  about 10.3 hours of settling discarded (ledger E1).
  The check that a page carries what its artifact says ships joined them by a second answer:
  asked the same day whether a page failing it,
  which is a defect in the assembly code rather than in the text,
  should still refuse with the archive's page shipping in its place,
  the owner chose "Ship with defect reported".
  It had fired in no real run (the 8 run logs naming it were all test suites).
- **A slice with no valid wording keeps the archive's,
  and the page ships.**
  Owner,
  2026-09-27,
  asked whether an entry keeps stopping at once
  when no wording for a slice passes the deterministic rule
  (the lanes',
  the consolidations' and the archive's,
  or the archive has nothing there),
  given that a stopped entry leaves the archive's whole page live,
  that slice included:
  "Keep archive,
  ship".
  The slice keeps exactly what the archive has there,
  nothing where the archive is silent,
  the page ships with every other repair,
  and the slice is reported on the `DEFECTS` line.
  This replaces the "else fail the slice at once" half of the rule of 2026-09-04;
  its first half stands,
  so a valid proposal is still preferred wherever one exists.
  Measured before the ruling:
  18 real entries stopped on this refusal from 2026-09-04 to 2026-09-27,
  about 34.3 hours in all,
  most on terminals later rulings already turned into shipping the slate's choice;
  on the build of the ruling,
  one real stop was of the kind still reachable (yulianNyanner,
  `incumbent-only`,
  0.48 hours).
- **A slice with no archive text and no backed wording ships its source text under a note.**
  Owner,
  2026-10-23,
  asked what a shipped page carries for such a slice,
  after the corpus pass XingZ6020 shipped 88 of its 118 slices and left the other 30,
  31.5 percent of the original and every footnote definition,
  off the page with nothing on it to say so:
  "Source text + note".
  The page carries the source text of the passage,
  preceded by a note saying it has not been translated yet and the original follows,
  footnote definitions included;
  a later run replaces it once a wording is backed.
  Options offered and not chosen:
  the note alone,
  keeping the silent gap,
  and unbacked wording marked as unreviewed,
  which would break the standing rule that one judge alone never decides
  (restated by the owner on 2026-10-25 with the removal of every weight floor:
  "floor of 3 - don't set a floor.",
  "Two ballots decide (Recommended)" and "Every weight floor";
  the full record is in
  [votes, repairs and run-offs](design-commitments-votes-repairs-and-run-offs.md),
  a gate that heard one judge no longer ships a consolidation past its neither on its own authority,
  and a declined round ships a candidate only when two or more judges named it,
  the wording in place staying otherwise).
  The earlier rulings stand:
  a run always ships,
  and a slice with archive text and no valid wording keeps the archive.
  One note stands before each contiguous run of such slices,
  in Canadian English as a block quote on its own lines:
  `> This passage has not been translated yet. The original text follows.`
  The source text follows as the artifact records it:
  each slice's text exactly,
  the slices of one run joined by one blank line,
  and no text between slices (an HTML comment the original holds between two blocks,
  for instance),
  since that text belongs to no slice.
  No page pass,
  floor or footnote guard rewrites it,
  since each of them reads or edits English;
  the helper that lists what each slice contributes to a page leaves the source text out
  unless its caller asks,
  and only the writer of the published page asks.
  The page grammar is strict MDX,
  so a bare `{`,
  an unclosed tag or an unmatched closing tag in the original can make the composed page unparseable.
  The writer of the page therefore ships each run as recorded whenever the composed page parses,
  and otherwise takes up to three steps,
  parsing the page after each:
  first it writes,
  as literal text,
  only the runs that do not parse as a fragment of their own,
  any run that opens an HTML comment which never closes,
  and any run that parses alone and cannot stand beside another
  (a fence mark indented four spaces or more,
  a name an earlier run still declares,
  a fence or display math the run leaves open at its end,
  which the strict grammar would end at a later run's line)
  (a backslash before each `{`,
  `}` and `<` the CommonMark parse of the run reads as text;
  a fence,
  a code span,
  math,
  a link destination,
  a closed comment and an escape the original already wrote stay as they are);
  then it writes every run so;
  then it ships the page without the runs no writing makes parse
  (without every run where that does not help)
  and adds the defect `page-source-unparseable` naming the slices.
  Each step taken writes one warn line naming the slices and the step,
  and the entry never stops on this path.
  The bytes of the original differ from the artifact's only on those paths.
  The second step is taken when the page of the first step does not parse
  (the page as written when the first step changed nothing)
  and writing every run as literal text gives a page unlike the one last tried.
  Two seeded searches of forty thousand pairs of runs each under the real grammar took it for 365 and 403 pairs,
  every one a run nested past the bound beside a run holding a character that step escapes,
  and it made the page parse for none.
  The third step is taken when no writing makes the page parse:
  a run nested past the bound of the plain reading
  (the structural limit,
  with the grammar's source cited in the module comment of `page-check/src/corpus-run/source-ships-encoding.ts`
  and in ledger B832).
  A fence or a `$$` flow that a run opens and no line of it closes is written as literal text with a closing line
  at its end (`page-check/src/corpus-run/source-ships-open-flow.ts`),
  since the strict grammar would end it at the next run's line
  and leave the text after that line outside it as code.
  A carriage return that is not part of a Windows line ending is a line ending to the grammar,
  and the readers of the encoder split lines on the line feed alone.
  A run written as literal text therefore has each such carriage return written as a line feed
  (`foldLoneCarriageReturns` in `page-check/src/corpus-run/source-ships-readable.ts`,
  applied once where the encoder takes the text in,
  same length,
  so every offset stands),
  and the checks that decide which runs are written so read the run folded in the same way.
  A Windows line ending stays as it is,
  and a run shipped as recorded keeps every ending it has.
  The corpus reader folds CRLF to LF at the read (`slice-reading/src/line-endings.ts`)
  and leaves a lone carriage return alone,
  so the fold changes no page that the corpus brings today.
  A page that shows the original of a run differently from the source ships reported and is never refused
  (the owner,
  2026-10-25,
  "Add the comparison",
  after a search found texts the encoder wrote so that a reader sees other text).
  `readingDifferenceOf` (`page-check/src/corpus-run/source-reading-difference.ts`) reduces the page,
  read by the strict grammar,
  and the source,
  read as plain Markdown with GFM,
  math and raw HTML,
  to the text of their leaves,
  and the composition runs it over every run it writes as literal text (`source-ships-differences.ts`).
  A run that ships as recorded is not compared,
  since what the site compiles is what the original is upstream.
  The answer is `same` when the two accounts agree,
  and `differs` when they do not
  and no footnote range the two readings disagree on starts inside a block quote or a list item.
  Where such a range exists the answer is decided for each top-level block (`source-reading-blocks.ts`).
  The two-readings ruling takes the page's reading of a footnote in a block quote or a list item,
  which a reader of plain Markdown cannot give,
  so a block that a footnote range the two readings disagree on reaches is `undecided`,
  and every other block is compared on its own.
  The whole texts differ whenever this is asked,
  so the answer is `differs` or `undecided` and never `same`.
  Three defects carry the answers,
  each naming slices and no text of the page:
  `source-reads-differently` for a block that shows other text than the source,
  `source-reading-undecided` for a block the comparison cannot decide,
  and `source-reading-not-compared` for a run the comparison could not be made for,
  a page that parses and cannot be read against its source.
  A link target is decoded as far as it reads,
  a percent escape that is no text in UTF-8 included,
  so no known text raises the comparison;
  the defect is kept,
  and the composition takes the comparison as an argument so that a failing one is tested through it.
  The composition reports them,
  not a publish check that throws:
  a run always ships,
  and every reading difference is reported,
  so a throw of the comparison is a defect of the run and never stops the entry.
  A page shipped without runs reads as agreed with its artifact:
  the page-agreement check runs the same composition over what the artifact records
  and weighs the page against the artifact without the originals the composition leaves off,
  and `verify-published` prints one line naming the entry,
  `page-source-unparseable` and the slices.
  A page that never carried originals still disagrees wherever the composition keeps them,
  so a pass publishes it again.
  The artifact keeps its record of the slice (`lanes.translate.delivery`),
  and the would-ship reading names the role,
  `source-ships` (`page-check/src/corpus-run/source-ships.ts`),
  so the page-agreement check,
  `verify-published` (`source-shipped=`)
  and the `TALLY` line (`pageSourceShipped`,
  beside `pageSilent`) read the page as the artifact says it ships.
  The slice stays uncached,
  so the next run asks again (ledger B827).
