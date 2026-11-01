//region Slice cost report text
// THE LONG SENTENCES THE SLICE COST REPORT PRINTS, as the cases of its
// printers and of its built command both assert them.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 Sentence closing the spread block.
 */
export const SPREAD_NOTE: string = '  A large ratio with the DEARER slice no larger than the cheaper one refutes'
  + ' size as the driver outright, whatever this report\'s bands show, and points at'
  + ' how much each slice turned out to need instead.';

/**
 Paragraph telling a reader how to read the ms/char column.
 */
export const READ_NOTE: string = '\nREAD THE ms/char COLUMN. Flat across bands means size drives the cost and'
  + ' slicing differently changes the bill. Falling steeply as slices grow means'
  + ' a fixed per-slice overhead divided by more characters, and only asking'
  + ' fewer times changes anything.';

/**
 Sentence printed where the log held no cost line.
 */
export const NOTHING_NOTE: string = 'NOTHING TO READ YET. A pass writes these as it goes, so an empty'
  + ' reading means the run has not finished a slice rather than that slices are free.';

/**
 A log of four computed slices over three lanes, one resumed slice that is
 not priced, one line cut short and a line about something else.
 */
export const COST_LOG: string = [
  '[info] [2026-08-17T01:17:12.580Z] [Mittens] [repairPreparedDocument] critic stage: 6/6 heard',
  '[info] [2026-08-17T01:17:13.000Z] [Mittens] [repair] SLICE-COST lane=repair chunk=0 sourceChars=40 ms=60000 exit=computed',
  '[info] [2026-08-17T01:17:14.000Z] [Mittens] [translate] SLICE-COST lane=translate chunk=1 sourceChars=150 ms=120000 exit=computed',
  '[info] [2026-08-17T01:17:15.000Z] [Mittens] [translate] SLICE-COST lane=translate chunk=2 sourceChars=3000 ms=360000 exit=computed',
  '[info] [2026-08-17T01:17:16.000Z] [Mittens] [consolidate] SLICE-COST lane=consolidation chunk=3 sourceChars=40 ms=30000 exit=computed',
  '[info] [2026-08-17T01:17:17.000Z] [Mittens] [repair] SLICE-COST lane=repair chunk=4 sourceChars=600 ms=60000 exit=resumed',
  '[info] [2026-08-17T01:17:18.000Z] [Mittens] [repair] SLICE-COST lane=repair chunk=5',
  '',
].join('\n',);

//endregion Slice cost report text
