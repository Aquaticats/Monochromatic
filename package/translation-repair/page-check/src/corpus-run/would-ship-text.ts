import type { ArtifactContestSlice, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  type ArtifactConsistency,
  type ArtifactPageAssembly,
  consistencyOverrideAt,
  NO_CONSISTENCY,
  NO_PAGE_ASSEMBLY,
  pageAssemblyOverrideAt,
  type ParsedArchiveText,
  type ParsedTwoLaneArtifact,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import type { ArtifactComparisonRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { restoreTypography, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  carryingSource,
  markRuns,
} from './source-ships.ts';
import {
  consolidatedWordingAt,
  polishedWordingAt,
} from './would-ship-consolidation.ts';
import {
  type InterimCarrier,
  isUnsettledSlice,
} from './would-ship-interim.ts';

//region Would-ship vocabulary
// WHAT A READER WOULD SEE AT ONE SLICE, and no field holds it. Two deciding
// stages sit above the two lanes, each free to replace what the one below it
// left, and the artifact records what each stage DECIDED rather than what
// survived all of them. Deriving that is this file's whole job.
//
// NAMED "WOULD SHIP" RATHER THAN "SHIPPED", deliberately. `publish-fixed.ts`
// writes a page from exactly these readings, but a reading exists whether or
// not a page was written for it, so the name claims no publication: this is
// what a page composed from these decisions carries. The founding defect
// of this family was `repairDisposition: 'shipped'`, a name that went on
// asserting a publication after the stage that performed it stopped being last.
// A name claiming less than it can prove is the correction.

/**
 Which stage settled the wording a slice would contribute.

 ATTRIBUTION IS PART OF THE ANSWER, not a diagnostic beside it. A consumer
 auditing one lane's work needs to know that a lane's wording reached the
 page because that lane won, rather than because no other stage had anything
 to say; those are different facts about the roster and read identically off
 the text alone.

 @example
 ```ts
 const decidedBy: WouldShipDecider = 'consolidation';
 ```
 */
export type WouldShipDecider =
  | 'page-assembly'
    | 'consistency'
    | 'polish'
    | 'consolidation'
    | 'contest'
    | 'lanes-agreed'
    | 'archive';

/**
 Why a slice would contribute no wording at all.

 NEVER REPRESENTED AS AN EMPTY STRING, which is the trap this whole shape
 exists to close. `standingTextFor` returns `''` at a declined contest, and
 `ArtifactConsolidateShipped` documents that a consumer writing a bare per
 slice `text` into a document would delete every declined slice outright. A
 reading that carries no `text` key at all makes that unrepresentable rather
 than warned against.

 @example
 ```ts
 const reason: WouldShipSilence = 'contest-declined-and-archive-silent';
 ```
 */
export type WouldShipSilence =
  | 'page-assembly-withdrew-and-archive-silent'
    | 'no-valid-wording-and-archive-silent'
    | 'contest-declined-and-archive-silent'
    | 'contest-unasked-and-archive-silent'
    | 'interim-unsettled-and-archive-silent'
    | 'lanes-agreed-on-no-wording';

/**
 What one slice would contribute to a document assembled today.

 @example
 ```ts
 const reading: WouldShipReading = { kind: 'nothing-ships', reason: 'lanes-agreed-on-no-wording', };
 ```
 */
export type WouldShipReading =
  | {
    /**
     This slice carries wording, and `decidedBy` names who settled it.
     */
    readonly kind: 'wording';

    /**
     Exactly what a document would carry here, wrapped as the stage left it.
     */
    readonly text: string;

    /**
     Stage whose decision survived every stage after it.
     */
    readonly decidedBy: WouldShipDecider;
  }
    | {
    /**
     This slice carries nothing, and `reason` names which stage left it so.
     */
      readonly kind: 'nothing-ships';

    /**
     Why nothing stands here.
     */
      readonly reason: WouldShipSilence;

    /**
     Whether the archive held any wording at this slice.

     ON THE SILENCE RATHER THAN ON THE SLICE, because silence is where the
     ambiguity lives and this is what resolves it. Every reason in
     {@link WouldShipSilence} covers two opposite events at once, and
     `lanesAgreedOn` says so in its own comment: an agreed empty string
     "covers a gap neither lane wrote into and text both lanes removed".
     Wording the deciders REMOVED is a change a reader must look at; a gap
     they LEFT is the archive standing exactly as it was. Nothing else on a
     silent reading tells them apart.

     READ OFF THE COMPARISON ROW, never off an empty incumbent, per
     `translate-absence.ts`: absence is a mode decided once from the target
     chunk, and testing the text would conflate an anchor with a content span
     whose archive wording genuinely is blank.
     */
      readonly incumbentKind: 'present' | 'absent';
    }
    | {
    /**
     This slice ships the original text of its passage, because the archive holds no
     text here and no wording was backed (the owner, 2026-10-23, "Source text +
     note"); `source-ships.ts` says what the page carries.
     */
      readonly kind: 'source-ships';

    /**
     Which silence the slice would otherwise have been left in.
     */
      readonly reason: WouldShipSilence;

    /**
     The archive holds no text here, which is the only place a slice ships its original.
     */
      readonly incumbentKind: 'absent';

    /**
     Original text of the slice, exactly as the artifact records it.
     */
      readonly text: string;

    /**
     Whether the note that the passage has not been translated yet stands before this
     slice, true for the first slice of a contiguous run and false for the ones after it.
     */
      readonly opensRun: boolean;
    };

/**
 Fields of a parsed artifact a reading is derived from.

 NARROWER THAN THE WHOLE ARTIFACT deliberately, so this file names its own
 inputs rather than taking everything and reading three things. A whole
 `ParsedTwoLaneArtifact` satisfies it unchanged, so consumers pass what they
 already hold, and the types still come from the parsed contract: that is
 what makes a key this file misspells a type error rather than an
 `undefined` that reads as an answer.

 @example
 ```ts
 const source: WouldShipSource = parseSettledTwoLaneArtifact({ value, },);
 ```
 */
export type WouldShipSource = Pick<
  ParsedTwoLaneArtifact,
  'comparison' | 'consolidation' | 'laneSelection'
  > & ArchiveTextCarrier & PageAssemblyCarrier & ConsistencyCarrier & SourceTextCarrier & InterimCarrier;

/**
 The original of every slice, as either artifact shape records it in the translate lane's
 delivery rows: a source built from the three reading fields alone carries none, which
 reads as it always did, a slice nothing fills being silent.

 @example
 ```ts
 const carrier: SourceTextCarrier = { lanes: { translate: { delivery: [{ sliceIndex: 0, sourceText: '猫', },], }, }, };
 ```
 */
type SourceTextCarrier = {
  /**
   The two lanes, of which only the translate lane's rows are read.
   */
  readonly lanes?: {
    /**
     The lane that renders every slice afresh from the original.
     */
    readonly translate: {
      /**
       One row per slice, each carrying the original it was cut from.
       */
      readonly delivery: readonly {
        /**
         Slice the row is for.
         */
        readonly sliceIndex: number;

        /**
         Original text of the slice.
         */
        readonly sourceText: string;
      }[];
    };
  };
};

/**
 What the page-level guard did, as either artifact shape carries it: the
 settled artifact and the parsed one both hold the section, and a source
 built from the three reading fields alone carries none, which reads as a
 guard that touched nothing.

 @example
 ```ts
 const carrier: PageAssemblyCarrier = { pageAssembly: NO_PAGE_ASSEMBLY, };
 ```
 */
type PageAssemblyCarrier = {
  /**
   What the page-level guard trimmed, withdrew and found.
   */
  readonly pageAssembly?: ArtifactPageAssembly;
};

/**
 What the page-level consistency pass did, as either artifact shape carries
 it: a source built from the three reading fields alone carries none, which
 reads as a pass that changed nothing.

 @example
 ```ts
 const carrier: ConsistencyCarrier = { consistency: NO_CONSISTENCY, };
 ```
 */
type ConsistencyCarrier = {
  /**
   What the page-level consistency pass changed.
   */
  readonly consistency?: ArtifactConsistency;
};

/**
 Archive English as either artifact shape carries it: the settled artifact a
 pass writes holds the string, and the parsed artifact a reader opens wraps it
 with whether it was recorded. Optional, so a source built from the three
 reading fields alone still reads.

 @example
 ```ts
 const carrier: ArchiveTextCarrier = { preparation: { archiveText: 'The cat’s asleep.', }, };
 ```
 */
type ArchiveTextCarrier = {
  /**
   The slicing's record of the archive, when the source carries one.
   */
  readonly preparation?: {
    /**
     Whole archive English, or the parsed statement of whether it was kept.
     */
    readonly archiveText: string | ParsedArchiveText;
  };
};

/**
 Whole archive English a source carries, which is the page's quote
 convention, or empty when the source carries none.

 @param artifact - source being read

 @returns Archive English verbatim, or empty

 @example
 ```ts
 const convention = archiveConventionOf({ artifact, },);
 ```
 */
function archiveConventionOf(
  { artifact, }: { readonly artifact: WouldShipSource; },
): string {
  /**
   The slicing's record, absent on a source built from the reading fields alone.
   */
  const { preparation, } = artifact;
  if (preparation === undefined)
    return '';

  /**
   Archive English in whichever shape this source carries it.
   */
  const { archiveText, } = preparation;
  if ((typeof archiveText) === 'string')
    return archiveText;
  if (archiveText.kind === 'stored')
    return archiveText.text;
  return '';
}

/**
 One slice's reading, beside the index both lanes name it by.

 @example
 ```ts
 const slice: WouldShipSlice = { sliceIndex: 0, reading, };
 ```
 */
export type WouldShipSlice = {
  /**
   Slice this answers, matching its comparison row.
   */
  readonly sliceIndex: number;

  /**
   What that slice would contribute.

   A SILENT READING CARRIES `incumbentKind` and this does not, which is not
   an oversight. The distinction only decides anything where nothing ships,
   and two fields that must always agree with nothing enforcing it is worse
   than one field where it is needed.
   */
  readonly reading: WouldShipReading;
};

//endregion Would-ship vocabulary

//region Would-ship reader

/**
 Raised when the lanes differ at a slice the contest record never answered.

 A CONTRADICTION IN THE ARTIFACT, not a state a run can reach. `contestEligibleIndexes`
 makes every slice whose lane texts differ eligible, and the parser returns a
 lane selection only once it agrees with the comparison it recomputed. Reading
 such a slice as though the lanes agreed would pick one lane's wording with
 nothing behind it, so it is refused instead.

 MARKED, AND BUILT FROM THE SLICE'S INDEX, so a `TALLY` line and a command's
 refusal line say which slice, as they say the sentence of every sibling
 refusal that stops an entry. Until 2026-10-10 it forwarded a finished
 message and was unmarked, so `refusalText` printed its class alone.

 @example
 ```ts
 throw new UnansweredContestSliceError({ sliceIndex: 3, },);
 ```
 */
export class UnansweredContestSliceError extends Error {
  /**
   Message contains a slice index only.
   */
  readonly messageNamesOnly: true = true;

  /**
   Slice whose lanes differ and which the contest record skipped.
   */
  readonly sliceIndex: number;

  /**
   Builds the failure naming the slice the contest record skipped.

   @param sliceIndex - slice whose lanes differ

   @example
   ```ts
   throw new UnansweredContestSliceError({ sliceIndex: 3, },);
   ```
   */
  public constructor({ sliceIndex, }: { readonly sliceIndex: number; },) {
    super(`slice ${String(sliceIndex,)} differs across lanes and the contest names it nowhere`,);
    this.name = 'UnansweredContestSliceError';
    this.sliceIndex = sliceIndex;
  }
}

/**
 Names what the archive holds here, or that it holds nothing.

 THE ARCHIVE IS THE INCUMBENT, which is what makes this the floor rather than
 an empty string. This pipeline repairs an English translation that already
 exists, so a slice no later stage displaced still carries whatever the
 archive published. That is a different question from the consolidation's
 `no-standing-text`, which asks what a slate must beat and correctly answers
 "nothing" on a decline.

 @param row - comparison row carrying the archive's own English

 @param silence - why nothing displaced the archive, used when it is empty too

 @returns Archive wording, or a named absence

 @example
 ```ts
 const reading = archiveStandsOr({ row, silence: 'contest-declined-and-archive-silent', },);
 ```
 */
function archiveStandsOr(
  {
    row,
    silence,
  }: {
    readonly row: ArtifactComparisonRow;
    readonly silence: WouldShipSilence;
  },
): WouldShipReading {
  if ((row.incumbentKind === 'present') && (row.incumbentText !== ''))
    return {
      kind: 'wording',
      text: row.incumbentText,
      decidedBy: 'archive',
    };

  return {
    kind: 'nothing-ships',
    reason: silence,

    // BOTH SILENCES THIS FUNCTION PRODUCES NAME AN ARCHIVE THAT SAYS NOTHING,
    // and neither says whether there is a span here at all. A content slice
    // whose archive wording is blank reaches this line as readily as an anchor,
    // and the two are not the same passage.
    incumbentKind: row.incumbentKind,
  };
}

/**
 Names what an uncontested slice contributes, where both lanes offer one wording.

 AGREEMENT NEEDS NO DECIDER, which is why this case has its own name rather
 than falling through to the archive. `contestEligibleIndexes` makes a slice
 eligible exactly where the lane texts differ, so a slice the contest never
 saw is one where they match, and that matching wording is what stands.

 @param row - comparison row whose lane texts agree

 @returns Agreed wording, or a named absence when both lanes offer none

 @example
 ```ts
 const reading = lanesAgreedOn({ row, },);
 ```
 */
function lanesAgreedOn(
  { row, }: { readonly row: ArtifactComparisonRow; },
): WouldShipReading {
  if (row.repairText !== row.translateText)
    throw new UnansweredContestSliceError({ sliceIndex: row.sliceIndex, },);

  // AN AGREED EMPTY STRING IS A DECISION, not an absence to be filled from the
  // archive. It covers a gap neither lane wrote into and text both lanes
  // removed, and reviving the archive under either would republish wording the
  // lanes agreed to drop.
  if (row.repairText === '')
    return {
      kind: 'nothing-ships',
      reason: 'lanes-agreed-on-no-wording',
      incumbentKind: row.incumbentKind,
    };

  return {
    kind: 'wording',
    text: row.repairText,
    decidedBy: 'lanes-agreed',
  };
}

/**
 Reads what one slice would contribute to a document assembled today.

 WALKS THE DECIDERS IN REVERSE ORDER OF WHEN THEY RAN, taking the first that
 has something to say, because each stage was free to replace what the one
 below it left. Reading any single stage's record answers a narrower question:
 the repair lane's ledger claimed `replacement-shipped` at 6 rows of which 0
 reached the page, since the contest had already chosen another lane at 5 and
 the consolidation then overrode 4.

 @param artifact - parsed artifact, so a wrong key is a type error rather than
 an `undefined` that reads as an answer

 @param row - comparison row to answer for

 @returns Wording this slice would carry, or a named reason it would carry none

 @throws {@link UnansweredContestSliceError} when the lanes differ at a slice
 the contest record never answered

 @example
 ```ts
 const reading = wouldShipTextFor({ artifact, row, },);
 ```
 */
export function wouldShipTextFor(
  {
    artifact,
    row,
  }: {
    readonly artifact: WouldShipSource;
    readonly row: ArtifactComparisonRow;
  },
): WouldShipReading {
  return carryingSource({
    artifact,
    sliceIndex: row.sliceIndex,
    reading: stagesReading({
      artifact,
      row,
    },),
  },);
}

/**
 Reads what the stages left at one slice, before an anchor nobody filled is
 given the original it stands for.

 @param artifact - parsed artifact, so a wrong key is a type error

 @param row - comparison row to answer for

 @returns Wording the stages left, or the named silence they left

 @throws {@link UnansweredContestSliceError} when the lanes differ at a slice
 the contest record never answered

 @example
 ```ts
 const reading = stagesReading({ artifact, row, },);
 ```
 */
function stagesReading(
  {
    artifact,
    row,
  }: {
    readonly artifact: WouldShipSource;
    readonly row: ArtifactComparisonRow;
  },
): WouldShipReading {
  // A SLICE OF AN INTERIM PAGE NOTHING SETTLED reads as the archive's text, and as the original where the archive has
  // none (`carryingSource`), whatever else the artifact says of it.
  if (isUnsettledSlice({
    artifact,
    sliceIndex: row.sliceIndex,
  },))
    return archiveStandsOr({
      row,
      silence: 'interim-unsettled-and-archive-silent',
    },);

  /**
   What the page-level guard did here, read FIRST: it read the page `lanes`,
   `laneSelection` and `consolidation` compose, and its record says what that page carries.
   */
  const override = pageAssemblyOverrideAt({
    pageAssembly: artifact.pageAssembly ?? NO_PAGE_ASSEMBLY,
    sliceIndex: row.sliceIndex,
  },);
  if (override.kind === 'trimmed')
    return {
      kind: 'wording',
      text: override.text,
      decidedBy: 'page-assembly',
    };
  if (override.kind === 'withdrawn')
    return archiveStandsOr({
      row,
      silence: 'page-assembly-withdrew-and-archive-silent',
    },);

  /**
   What the consistency pass wrote here, read BELOW the page assembly, which
   describes the page this text is composed into, and ABOVE the polish, the
   consolidation and the contest, which settled the text the pass changed.
   */
  const consistent = consistencyOverrideAt({
    consistency: artifact.consistency ?? NO_CONSISTENCY,
    sliceIndex: row.sliceIndex,
  },);
  if (consistent.kind === 'replaced')
    return {
      kind: 'wording',
      text: consistent.text,
      decidedBy: 'consistency',
    };

  /**
   Final naturalness polish, absent before generation six or when base stood.
   */
  const polished = polishedWordingAt({
    artifact,
    sliceIndex: row.sliceIndex,
  },);
  if (polished.kind === 'wording') {
    return {
      kind: 'wording',
      text: polished.text,
      decidedBy: 'polish',
    };
  }

  /**
   Third rendering's wording, absent wherever it replaced nothing.
   */
  const consolidated = consolidatedWordingAt({
    artifact,
    sliceIndex: row.sliceIndex,
  },);
  if (consolidated.kind === 'wording')
    return {
      kind: 'wording',
      text: consolidated.text,
      decidedBy: 'consolidation',
    };
  // THE ARCHIVE KEEPS A SLICE NO WORDING PASSED, read from this row so a
  // silent archive stays a named silence rather than an empty string.
  if (consolidated.kind === 'archive-kept')
    return archiveStandsOr({
      row,
      silence: 'no-valid-wording-and-archive-silent',
    },);

  /**
   Which lane ships, or that nobody has been asked over this entry yet.
   */
  const { laneSelection, } = artifact;
  if (laneSelection.kind !== 'contested')
    return archiveStandsOr({
      row,
      silence: 'contest-unasked-and-archive-silent',
    },);

  /**
   What the roster settled here, absent where this slice was never eligible.
   */
  const contested = laneSelection
    .slices
    .find(function namesIt(candidate: ArtifactContestSlice,): boolean {
      return candidate.sliceIndex === row.sliceIndex;
    },);
  if (contested === undefined)
    return lanesAgreedOn({ row, },);

  /**
   Whether a lane carried the voices, or was the only one with a passage here
   (ledger L6), and which one.
   */
  const { verdict, } = contested;
  if ((verdict.kind === 'lane-won') || (verdict.kind === 'sole-lane'))
    return {
      kind: 'wording',
      text: (verdict.lane === 'repair') ? row.repairText : row.translateText,
      decidedBy: 'contest',
    };

  // NEITHER LANE CARRIED THE ROOM, whether because the voices split or because
  // too few arrived. No lane displaced the archive, so the archive stands.
  return archiveStandsOr({
    row,
    silence: 'contest-declined-and-archive-silent',
  },);
}

/**
 Reads every slice of one artifact in comparison-row order.

 @param artifact - parsed artifact to read whole

 @returns One reading per comparison row, in the order the rows carry

 @throws {@link UnansweredContestSliceError} when any slice's lanes differ and
 the contest record never answered it

 @example
 ```ts
 const slices = wouldShipTextPerSlice({ artifact, },);
 ```
 */
export function wouldShipTextPerSlice(
  { artifact, }: { readonly artifact: WouldShipSource; },
): readonly WouldShipSlice[] {
  /**
   Whole archive English, which is the page's quote convention; empty where
   the source carries none, which leaves each row's own incumbent as the only
   witness.
   */
  const convention = archiveConventionOf({ artifact, },);

  return markRuns({
    slices: artifact
      .comparison
      .map(function readIt(row: ArtifactComparisonRow,): WouldShipSlice {
        return {
          sliceIndex: row.sliceIndex,
          reading: inArchiveTypography({
            reading: wouldShipTextFor({
              artifact,
              row,
            },),
            row,
            convention,
          },),
        };
      },),
  },);
}

/**
 Puts a stage's wording into the archive's quote style before it ships.

 THE REPAIR LANE ALREADY DID THIS and nothing else did. `restore-typography.ts`
 runs on every editor and refiner replacement, so a repair-lane wording reaches
 the page in the page's own convention; a translate-lane wording, a
 consolidation proposal and a polish rewrite never passed through it, and on
 2026-09-06 the yulianNyanner page carried five straight apostrophes among
 thirty curly ones, two days after the Uekawakuyuurei page shipped the same
 mix unread. Done here, on the reading every publisher and checker derives
 the page from, rather than in each producing stage, so the artifact keeps
 what the stages wrote and the page and its checks agree on what ships. An
 archive wording is the convention itself and is left as it stands.

 @param reading - what the slice would contribute

 @param row - comparison row it came from, whose incumbent is the archive
 wording at this slice

 @param convention - whole archive English, or empty when unrecorded

 @returns Same reading, its wording in the archive's quote style

 @example
 ```ts
 const shipped = inArchiveTypography({ reading, row, convention, },);
 ```
 */
function inArchiveTypography(
  {
    reading,
    row,
    convention,
  }: {
    readonly reading: WouldShipReading;
    readonly row: ArtifactComparisonRow;
    readonly convention: string;
  },
): WouldShipReading {
  if ((reading.kind !== 'wording') || (reading.decidedBy === 'archive'))
    return reading;
  return {
    ...reading,
    text: restoreTypography({
      replacement: reading.text,
      replaced: row.incumbentText,
      convention,
    },),
  };
}

//endregion Would-ship reader
