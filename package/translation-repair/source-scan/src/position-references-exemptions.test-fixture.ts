/**
 The references by position the D33 guard leaves, each with its reason:
 sheet text (in `position-references-sheet-exemptions.test-fixture.ts`, spread
 in here), positions in a page's own text, comparisons and orders in time the
 phrase shapes cannot tell apart, Unicode character names, quoted log lines,
 and the guard's own fixtures and examples. Data rather than logic, so the
 guard and a probe listing what it would name read one list.

 @module
 */

import { SHEET_EXEMPTIONS, } from './position-references-sheet-exemptions.test-fixture.ts';
import type { PositionExemption, } from './position-references.test-fixture.ts';

//region Exemptions

/**
 Why a position in a page's text is left.
 */
const PAGE_TEXT = 'a position in a page\'s own text, which the code handles as data';

/**
 Why a Unicode character name is left.
 */
const CHARACTER_NAME = 'a Unicode character name, which names a mark\'s place on its letter';

/**
 References in the package the guard leaves, each with its reason.
 */
export const PACKAGE_EXEMPTIONS: readonly PositionExemption[] = [
  ...SHEET_EXEMPTIONS,
  {
    path: 'corpus-source/archive-original-note.ts',
    holds: 'note above Hanasaka',
    reason: PAGE_TEXT,
  },
  {
    path: 'corpus-source/archive-original-note.ts',
    holds: 'seal everything below them',
    reason: PAGE_TEXT,
  },
  {
    path: 'adjudication/archive-original-note.unit.test.ts',
    holds: 'everything below is the English original',
    reason: PAGE_TEXT,
  },
  {
    path: 'insertion/corpus-run/insertion-carried-shift.ts',
    holds: 'the paragraph after them on its own',
    reason: PAGE_TEXT,
  },
  {
    path: 'page-check/corpus-run/publish-fixed.unit.test.ts',
    holds: 'over the paragraph after it',
    reason: PAGE_TEXT,
  },
  {
    path: 'repair/inspect-paragraph.unit.test.ts',
    holds: 'ends the sentence after it',
    reason: PAGE_TEXT,
  },
  {
    path: 'page-pass/apply-patch.unit.test.ts',
    holds: 'See `The Cat` below',
    reason: 'fixture page text, a position in the page',
  },
  {
    path: 'translate-lane/target-only-run.unit.test.ts',
    holds: 'transcript of the photo above',
    reason: 'fixture page text, a position in the page',
  },
  {
    path: 'glossary/corpus-run/transcription-suspect.unit.test.ts',
    holds: 'Transcription of the note above',
    reason: 'fixture page text, a position in the page',
  },
  {
    path: 'page-check/corpus-run/archive-stub.ts',
    holds: 'Whether the blank above goes',
    reason: 'a blank line of the stub being built, an element of the data',
  },
  {
    path: 'artifact/corpus-run/artifact-two-lane-read-pairing.ts',
    holds: 'Section before this one, absent at the first position',
    reason: 'the section the loop read before this one, an element of the data',
  },
  {
    path: 'house-rule/house-policy.ts',
    holds: 'the God Above',
    reason: 'an example of ordinary English the reader-facing rule quotes',
  },
  {
    path: 'markup/ascii-letters.unit.test.ts',
    holds: 'WITH DOT ABOVE',
    reason: CHARACTER_NAME,
  },
  {
    path: 'markup/latin-letters.unit.test.ts',
    holds: 'ARROWHEAD BELOW',
    reason: CHARACTER_NAME,
  },
  {
    path: 'markup/latin-letters.unit.test.ts',
    holds: 'RING BELOW',
    reason: CHARACTER_NAME,
  },
  {
    path: 'consolidate/consolidate-standing-verdict.unit.test.ts',
    holds: 'as under the rule before it',
    reason: 'the rule in force before it, an order in time',
  },
  {
    path: 'run/corpus-run/cap-override.unit.test.ts',
    holds: 'cut every entry before it',
    reason: 'an entry cut before it bought anything, an order in time',
  },
  {
    path: 'repair/refine-phase-slice.ts',
    holds: 'The former sequential phase',
    reason: '"former" as an adjective for a design since replaced',
  },
  {
    path: 'translate-stage/translate-retry.ts',
    holds: 'nobody named below the rest',
    reason: 'a ranking, a comparison',
  },
  {
    path: 'run/corpus-run/run-config.ts',
    holds: 'read below Flash',
    reason: 'a score below another model\'s, a comparison',
  },
  {
    path: 'doc/roster-changes.md',
    holds: 'read below Flash',
    reason: 'a score below another model\'s, a comparison',
  },
  {
    path: 'adjudication/select-candidate.unit.test.ts',
    holds: 'preservation above ties',
    reason: 'a ranking of two selection criteria, a comparison; "ties" is the noun',
  },
  {
    path: 'archive/reference-precedence.unit.test.ts',
    holds: 'protection above both reference rules',
    reason: 'a ranking of one rule over two others, a comparison; "both" names the rules compared',
  },
  {
    path: 'run/corpus-run/cap-override.ts',
    holds: 'entries above some slice count',
    reason: 'entries past a count of slices, a comparison; "some" is the count',
  },
  {
    path: 'stage/stage-quorum.unit.test.ts',
    holds: 'voice below half an even roster',
    reason: 'a count under a fraction of the roster, a comparison; "half" is the fraction',
  },
  {
    path: 'page-check/consolidation-polish.unit.test.ts',
    holds: 'SHORT BODY PROSE below repair-lane',
    reason: 'prose shorter than a window, a comparison',
  },
  {
    path: 'slice-cache/corpus-run/slice-cost-bands.ts',
    holds: 'anything above is open-ended',
    reason: 'sizes past a band\'s bound, a comparison',
  },
  {
    path: 'doc/mistake-prevention-shell-edits-commits-and-agents.md',
    holds: 'breaking the rule after it took its current wording',
    reason: 'after the rule took its wording, an order in time',
  },
  {
    path: 'run/corpus-run/meter-sample-read.unit.test.ts',
    holds: 'The METERS line above is the record',
    reason: 'a fixture quoting the sampler\'s own log line',
  },
  {
    path: 'run/corpus-run/spend-read.unit.test.ts',
    holds: 'the SPEND line above is the record for this call',
    reason: 'a fixture quoting the spend log\'s own line',
  },
  {
    path: 'doc/audit-ledger-doc-conventions-and-their-guards.md',
    holds: 'the heading after it; the living-docs case',
    reason: 'the heading that closes a kept range, named in the guard that reads it',
  },
  {
    path: 'doc/audit-ledger-process-docs-that-grow-or-go-stale.md',
    holds: 'account paragraphs that pointed at the paragraph before them',
    reason: 'M60\'s heading, naming the mistake it records',
  },
  {
    path: 'source-scan/position-references-exemptions.test-fixture.ts',
    holds: '',
    reason: 'this list, which quotes each phrase it leaves',
  },
  {
    path: 'source-scan/position-references-sheet-exemptions.test-fixture.ts',
    holds: '',
    reason: 'this list, which quotes each phrase it leaves',
  },
  {
    path: 'source-scan/position-references-vocabulary.test-fixture.ts',
    holds: '',
    reason: 'the guard\'s own word lists, which spell the positions',
  },
  {
    path: 'source-scan/position-references.unit.test.ts',
    holds: '',
    reason: 'the guard\'s own fixtures, each a positional phrase by design',
  },
  {
    path: 'source-scan/position-references-text.test-fixture.ts',
    holds: 'the case above',
    reason: 'the guard\'s own examples of the phrase it reads',
  },
  {
    path: 'source-scan/position-references.test-fixture.ts',
    holds: 'phrase: \'case above\'',
    reason: 'the guard\'s own example of a finding',
  },
  {
    path: 'source-scan/position-references.test-fixture.ts',
    holds: 'text: \'// see above\'',
    reason: 'the guard\'s own example of a file it reads',
  },
  {
    path: 'source-scan/position-references-context.test-fixture.ts',
    holds: 'line: \'a `the above` b\'',
    reason: 'the guard\'s own example of a code span',
  },
];

/**
 References in the living repository-level docs the guard leaves.
 */
export const REPOSITORY_EXEMPTIONS: readonly PositionExemption[] = [
  {
    path: 'doc/handover/translation-repair-handover-2026-09-06-letter-and-classes-nineteen-to-twenty-three-2026-09-08-to-09-09.md',
    holds: 'note above Hanasaka',
    reason: PAGE_TEXT,
  },
  {
    path: 'doc/planning/translation-repair-openrouter-2026-09-03-2026-09-08-0038-to-1127-utc.md',
    holds: 'the former lover',
    reason: 'a noun phrase: a lover in the page\'s story',
  },
  {
    path: 'doc/planning/translation-repair-openrouter-2026-09-03-2026-09-23-2210-to-2355-utc.md',
    holds: 'Chinese line above them',
    reason: PAGE_TEXT,
  },
  {
    path: 'doc/planning/translation-repair-openrouter-2026-09-03-2026-09-24-0345-to-0715-utc.md',
    holds: 'Chinese line above them',
    reason: PAGE_TEXT,
  },
];

//endregion Exemptions
