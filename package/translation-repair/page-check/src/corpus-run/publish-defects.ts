import type { NamingError, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  lifecycleStamp,
  wallClockNow,
} from '@monochromatic-dev/translation-repair-markup/ts';
import type { WouldShipSource, } from './would-ship-text.ts';

//region Publish defects
// A SETTLED PAGE SHIPS WITH ITS DEFECTS REPORTED (the owner, 2026-09-27,
// `package/translation-repair/doc/design-commitments-shipping-and-the-archive.md`).
// The publish-time content checks used to refuse the whole entry, discarding hours of settled work so the archive's
// page shipped instead: seven real entries lost about 10.3 hours that way to a
// single dropped link destination each (ledger E1). Each check still runs and
// still names what it found; its own refusal becomes a reported defect, and
// the page ships.
//
// ONLY THE CHECK'S OWN REFUSAL IS A DEFECT. Anything else a check throws is a
// fault in the code and still stops the entry, so a bug is never shipped as a
// content report. A page that does not parse is not collected here at all:
// it would break the site build, and still refuses.
//
// THE PAGE-AGREEMENT CHECK JOINED THEM by a second answer the same day ("Ship
// with defect reported"): a page that does not carry what its artifact says
// ships is a defect in the assembly code rather than in the text, and it
// ships reported rather than leaving the archive's page in its place.
//
// THE COMPOSITION OF THE ORIGINALS REPORTS FOUR OF THESE ITSELF, and not through a check that throws: `page-source-unparseable`
// (the page ships without the original of a run), and, from the comparison of the page with its source on 2026-10-25
// ("Add the comparison"), `source-reads-differently`, `source-reading-undecided` and `source-reading-not-compared`
// (`source-ships-differences.ts`). A run always ships and every reading difference is reported, so a throw of that
// comparison is a defect of the run, named by the last of them, and does not stop the entry.

/**
 A check at publish whose failure ships as a reported defect.

 @example
 ```ts
 const check: PublishCheck = 'destinations';
 ```
 */
export type PublishCheck =
  | 'archive-original'
    | 'contributor-names'
    | 'destinations'
    | 'front-matter'
    | 'headings'
    | 'no-valid-wording'
    | 'page-agreement'
    | 'page-footnotes'
    | 'page-source-unparseable'
    | 'source-reading-not-compared'
    | 'source-reading-undecided'
    | 'source-reads-differently';

/**
 One failed content check the page shipped with.

 @example
 ```ts
 const defect: PublishDefect = { check: 'headings', message: 'entry Cat collapses 2 source headings into 1', };
 ```
 */
export type PublishDefect = Readonly<{
  /**
   Check that failed.
   */
  check: PublishCheck;

  /**
   What its refusal said, which names ids, counts and slice indices only.
   */
  message: string;
}>;

/**
 One check to run, and the refusal class that marks its failure.

 @example
 ```ts
 const step: PublishCheckStep = { check: 'headings', refusal: CollapsedHeadingError, run: function headings() { assertHeadingsStayDistinct({ entryId, sourceText, pageText, },); }, };
 ```
 */
export type PublishCheckStep = Readonly<{
  /**
   Which check this is.
   */
  check: PublishCheck;

  /**
   Class the check throws when the page fails it, typed as one that declares
   its message safe to repeat, since a defect carries that message into the
   report.
   */
  refusal: abstract new (...parameters: never[]) => NamingError;

  /**
   Runs the check, throwing its refusal on a failure.
   */
  run: () => void;
}>;

/**
 Runs every content check and collects the failures as defects.

 @param steps - checks to run, in order

 @returns Defects the page ships with, empty for a clean page

 @throws Whatever a check raises other than its own refusal

 @example
 ```ts
 const defects = publishDefects({ steps, },);
 ```
 */
export function publishDefects(
  { steps, }: { readonly steps: readonly PublishCheckStep[]; },
): readonly PublishDefect[] {
  return steps.flatMap(function defectOf({
    check,
    refusal,
    run,
  },): readonly PublishDefect[] {
    try {
      run();
      return [];
    }
    catch (error) {
      if (!(error instanceof refusal))
        throw error;
      return [{
        check,
        message: error.message,
      },];
    }
  },);
}

/**
 Reports the slices where no wording passed the deterministic rule, so the
 archive kept them (owner, 2026-09-27, "Keep archive, ship").

 READ OFF THE ARTIFACT, not caught from a check: the consolidation already
 settled these slices, and its record is the one place that names them.

 @param entryId - entry the page belongs to

 @param consolidation - what the third rendering settled over the entry

 @returns One defect naming every such slice, or none

 @example
 ```ts
 const defects = archiveKeptDefects({ entryId: 'BookshopCat', consolidation: artifact.consolidation, },);
 ```
 */
export function archiveKeptDefects(
  {
    entryId,
    consolidation,
  }: {
    readonly entryId: string;
    readonly consolidation: WouldShipSource['consolidation'];
  },
): readonly PublishDefect[] {
  if (consolidation.kind !== 'settled')
    return [];
  /**
   What each reason the archive keeps a slice means, in the order the message names them. THREE THINGS HAPPENED and a
   sentence true of one was false of the others (XingZ6022: sixteen slices said no wording passed where a translate
   wording two judges chose had passed); a record written before the artifact said why records none.
   */
  const groups = [
    {
      reason: 'no-wording-passed',
      meaning: 'no wording passed the deterministic publication rule',
    },
    {
      reason: 'final-round-unseated',
      meaning: 'the final round could not seat two judges and no wording two judges chose passed the rule',
    },
    {
      reason: 'slate-declined',
      meaning: 'the slate judges declined every wording that passed the rule and none drew two ballots',
    },
    {
      reason: 'unrecorded',
      meaning: 'the artifact records no reason',
    },
  ] as const;
  /**
   Sentences saying where the archive kept slices and why, one per reason that occurred.
   */
  const sentences = groups.flatMap(function sentenceOf(
    {
      reason,
      meaning,
    },
  ): readonly string[] {
    /**
     Slices the archive kept for this reason, named as the message prints them.
     */
    const kept = consolidation.slices
      .filter(function keptForReason({ shipped, },): boolean {
        return (shipped.kind === 'archive') && ((shipped.reason ?? 'unrecorded') === reason);
      },)
      .map(function named({ sliceIndex, },): string {
        return `slice ${String(sliceIndex,)}`;
      },);
    return (kept.length === 0) ? [] : [`${kept.join(', ',)}, where ${meaning}`,];
  },);
  if (sentences.length === 0)
    return [];
  return [{
    check: 'no-valid-wording',
    message: `entry ${entryId} kept the archive at ${sentences.join('; at ',)}`,
  },];
}

/**
 The line a pass prints beside its tally for a page that shipped with
 defects, so a grep over a pass totals them.

 @param entryId - entry the page belongs to

 @param defects - failed checks it shipped with

 @param attempt - number of the entry's attempt the page was published in, as the pass's attempts file holds it

 @param wallClock - clock the line's time is read on; the wall clock unless a test fixes it

 @returns `DEFECTS <id> at=<time> attempt=<n> checks=<a>,<b>`, or empty for a clean page

 @throws `LifecycleAttemptError` (markup package) when the page has defects and the attempt is not a whole number from 1

 @example
 ```ts
 const line = defectsLine({ entryId: 'BookshopCat', defects, attempt: 2, },);
 ```
 */
export function defectsLine(
  {
    entryId,
    defects,
    attempt,
    wallClock = wallClockNow,
  }: {
    readonly entryId: string;
    readonly defects: readonly PublishDefect[];
    readonly attempt: number;
    readonly wallClock?: () => Date;
  },
): string {
  if (defects.length === 0)
    return '';
  /**
   Failed checks, in the order they ran.
   */
  const checks = defects.map(function checkOf({ check, },): string {
    return check;
  },);
  return `DEFECTS ${entryId} ${
    lifecycleStamp({
      attempt,
      wallClock,
    },)
  } checks=${checks.join(',',)}`;
}

//endregion Publish defects
