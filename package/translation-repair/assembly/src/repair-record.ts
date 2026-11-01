import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type {
  IssueCheckerReading,
  RegionDefectTally,
  RepairRegion,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { ChunkRepairOutcome, } from './repair-contract.ts';

//region Repair issue record
// The whole-document issue report, flattened out of the per-slice outcomes.
//
// This is also the only layer that can decide whether a repair reached the
// reader. A slice's own selection is not the answer: `repairTranslation` can
// settle several slices and then return the entire original text when
// non-translation dominance blocks the document, which withdraws every slice
// repair at once. Deciding here, after dominance and after the naturalness
// lane, is the difference between a status that is true of the run and one that
// is true of a stage.
//
// One builder serves both driver exits so the two cannot drift; the blocked
// exit already reported `resolved: false` correctly while carrying no repair
// provenance at all, which is exactly the kind of divergence a shared builder
// prevents.

/**
 Every disposition the pipeline writes, as data, each with what it says.

 THE TYPE IS DERIVED FROM THIS LIST, so a disposition cannot be added to the
 type without being listed here, and the reader that checks a recorded value
 against this list cannot fall behind it.

 @example
 ```ts
 const known = REPAIR_DISPOSITIONS.includes('shipped',);
 ```
 */
export const REPAIR_DISPOSITIONS = [
  // A replaced region served this issue and the returned document carries it.
  'shipped',
  // A replaced region served this issue, but the unchanged text won its
  // slice's selection, so that repair reached no reader.
  //
  // Says nothing about the returned TEXT: the naturalness lane runs after the
  // accuracy stage regardless of what that stage's selection decided, so a
  // slice can still have been rewritten. `RepairIssueRecord.refined` is what
  // answers that, and the repair sheet discloses it here too.
  'not-selected',
  // A replaced region served this issue and its slice was repaired, and the
  // document took that repair back: either it was blocked for non-translation
  // and returned its input, which withdraws every slice at once, or the
  // assembly guard withdrew this slice to keep a footnote relation whole.
  //
  // Either way the repair reached no reader, which is what a measurement over
  // these records has to see.
  'withdrawn',
  // No replaced region served this issue at all: either no envelope could be
  // cut from its evidence, or its envelope received no operation that survived
  // the apply gate. The two are merged because both mean the same thing to a
  // measurement, that no targeted repair exists to grade.
  //
  // Also says nothing about the returned text, for the reason given on
  // `not-selected`.
  'no-region',
] as const;

/**
 What became of the repair for one accepted issue in the returned document.

 Each member's meaning is written beside it in {@link REPAIR_DISPOSITIONS}.

 @example
 ```ts
 const disposition: RepairDisposition = 'shipped';
 ```
 */
export type RepairDisposition = typeof REPAIR_DISPOSITIONS[number];

/**
 Disposition of a repair the returned document carries.

 Exported because three separate readers ask this same question of a record,
 and each answering it with its own spelling of the word is how a reader comes
 to disagree with the writer about which records are gradable.

 @example
 ```ts
 const gradable = record.repairDisposition === SHIPPED_DISPOSITION;
 ```
 */
export const SHIPPED_DISPOSITION: RepairDisposition = 'shipped';

/**
 Probe result as one issue's record carries it.

 @example
 ```ts
 const reading: IssueProbeReading = { heardProbers: 3, configuredProbers: 3, regions: [], };
 ```
 */
export type IssueProbeReading = {
  /**
   Probers whose reply arrived and validated for this issue's chunk.
   */
  readonly heardProbers: number;

  /**
   Probers asked, which is the denominator a majority rule needs.
   */
  readonly configuredProbers: number;

  /**
   Screened tallies for the regions serving this issue.
   */
  readonly regions: readonly RegionDefectTally[];
};

/**
 One adjudicated issue in the whole-document report.

 @example
 ```ts
 const record: RepairIssueRecord = {
   sliceIndex: 0,
   issue,
   resolved: false,
   repairRegions: [],
   repairDisposition: 'no-region',
   refined: false,
 };
 ```
 */
export type RepairIssueRecord = {
  /**
   Chunk the issue belongs to.
   */
  readonly sliceIndex: number;

  /**
   Adjudicated issue exactly as the panel decided it.
   */
  readonly issue: AdjudicatedIssue;

  /**
   Whether the checkers confirmed it fixed in the shipped text.
   */
  readonly resolved: boolean;

  /**
   The round behind {@link RepairIssueRecord.resolved}: every ballot, the
   seated roster, and the weights they summed to.

   BECAUSE THE BOOLEAN IS A MAJORITY AND A MAJORITY IS NOT EVIDENCE OF ITSELF.
   A three-to-nil resolution and a two-to-one resolution persist identically
   without this, so a settled artifact could not say whether the checkers
   agreed, could not name a dissenter, and could not be re-read at a roster
   width the run never used. `IssueProbeReading` already keeps its tallies and
   roster size per issue for exactly this reason; the stage that decides what
   ships kept less until this landed.

   ABSENT ON TWO KINDS OF RECORD, and they mean different things: an issue no
   checker round ever ruled on, such as one no repair region served, and a
   record written before this field existed. Neither can be read as agreement.
   */
  readonly checkerReading?: IssueCheckerReading;

  /**
   The naturalness lane's SECOND round about this same issue, where it ran.

   NOT PART OF {@link RepairIssueRecord.resolved} AND NOT MERGED INTO
   {@link RepairIssueRecord.checkerReading}. The refinement recheck asks the
   checkers whether an already-confirmed issue survived a rewrite, and its
   only power is to roll the whole slice back. It never revises the deciding
   round, so a reader that folded the two together would report a verdict
   about text that may have been discarded as the verdict behind what shipped.

   WHERE IT IS ABSENT, in falling order of how often: every slice the lane did
   not rewrite, every slice whose rewrite touched no confirmed issue, and any
   record written before this field existed.

   WHERE IT IS PRESENT AND THE SLICE ROLLED BACK, the refined text it rules on
   is not the text that shipped. That is the case worth reading rather than
   the case to hide: it names which checkers judged the rewrite a regression.
   */
  readonly recheckReading?: IssueCheckerReading;

  /**
   Replaced regions serving this issue, so repair quality can be graded apart
   from whether the issue was real. A region shared with other accepted issues
   names them all, because one edit judged against one issue's claim has to be
   presented as the shared thing it is.
   */
  readonly repairRegions: readonly RepairRegion[];

  /**
   What became of that repair in the returned document.
   */
  readonly repairDisposition: RepairDisposition;

  /**
   Shadow-mode probe result for the regions serving this issue, absent where
   the chunk was never probed.

   Carried per issue rather than per chunk so a graded sheet item and the
   probe's opinion of that same item sit side by side in the artifact, which
   is what a calibration comparing the two has to join on. Nothing reads it to
   decide what ships.

   The roster sizes ride along with the tallies rather than being left on the
   chunk, because without them an artifact cannot answer whether a MAJORITY
   agreed. Heard voices are recoverable from a tally by summing its verdicts,
   but the configured roster is not recoverable from anything, and two of three
   heard is different evidence from two of six configured.
   */
  readonly introducedDefects?: IssueProbeReading;

  /**
   Whether the naturalness lane rewrote this issue's slice afterwards, making
   every {@link RepairRegion.editorAfter} pre-refinement wording.
   */
  readonly refined: boolean;

  /**
   Audit of damage the naturalness REWRITE caused, present only where the lane
   rewrote this issue's slice.

   Every issue of a rewritten slice carries the same report, because the lane
   edits the slice as a whole. Kept apart from
   {@link RepairIssueRecord.introducedDefects}, which audits the accuracy
   stage against a different baseline: one compares the original translation
   with the repaired one, the other the repaired one with the rewrite.

   Carried wherever the lane rewrote the slice, including where nothing then
   reached the reader. Both readings audit a STAGE against the text it was
   handed, and that comparison stands whatever assembly later did with the
   result; suppressing it on a withdrawn slice would hide damage the lane
   demonstrably does.
   */
  readonly refinementDefects?: IssueProbeReading;

  /**
   Final text of this issue's slice, carried ONLY where
   {@link RepairIssueRecord.refined} is set AND the returned document carries
   that rewrite.

   Present exactly where a shipped replacement stopped being the returned
   wording, so a grader judging a shipped repair always judges what shipped.
   Absent otherwise, for three reasons that each make it needless. Under
   {@link RepairDisposition} `shipped` with no refinement, `spliceSlices`
   splices the patched slice verbatim, so the replacement IS the returned
   wording and copying the slice onto every one of its issues would multiply a
   large document's text by its accepted-issue count for no added fact. Under
   `not-selected` or `no-region` with no refinement the slice comes back as the
   archive wrote it, so there is nothing a grader could compare. And where the
   run returned its input, or the assembly guard took the slice back to keep a
   footnote relation whole, the REWRITE reached no reader either: the document
   carries the archive's wording, and naming the rewrite as the final text
   would be exactly the lie this field exists to prevent.
   */
  readonly finalSliceText?: string;
};

/**
 Decides what became of one issue's repair in the returned document.

 @param regions - replaced regions serving this issue

 @param accuracyPatchSelected - whether the slice's patched candidate won

 @param repairReachedReader - whether this slice's replacement is in the
 returned document, which several later steps can each answer no to

 @returns Disposition for this issue

 @example
 ```ts
 const disposition = judgeDisposition({
   regions,
   accuracyPatchSelected: true,
   repairReachedReader: true,
 },);
 ```
 */
function judgeDisposition(
  {
    regions,
    accuracyPatchSelected,
    repairReachedReader,
  }: {
    readonly regions: readonly RepairRegion[];
    readonly accuracyPatchSelected: boolean;
    readonly repairReachedReader: boolean;
  },
): RepairDisposition {
  if (regions.length === 0)
    return 'no-region';
  if (!accuracyPatchSelected)
    return 'not-selected';
  return repairReachedReader
    ? 'shipped'
    : 'withdrawn';
}

/**
 Flattens settled slice outcomes into the whole-document issue report.

 @param outcomes - settled per-slice outcomes in document order

 @param blocked - whether the run returned its input for non-translation, in
 which case no slice repair reached the reader whatever its slice decided

 @param withdrawnSliceIndices - slices whose repair the assembly guard took
 back, whose issues reached no reader for the same reason a blocked document's
 did; absent means none were

 @returns One record per adjudicated issue, in slice then issue order

 @example
 ```ts
 const issues = buildIssueRecords({ outcomes, blocked: false, },);
 ```
 */
export function buildIssueRecords(
  {
    outcomes,
    blocked,
    withdrawnSliceIndices = [],
  }: {
    readonly outcomes: readonly ChunkRepairOutcome[];
    readonly blocked: boolean;
    readonly withdrawnSliceIndices?: readonly number[];
  },
): readonly RepairIssueRecord[] {
  /**
   Slices the assembly guard took back, for membership tests.
   */
  const withdrawn = new Set(withdrawnSliceIndices,);

  return outcomes.flatMap(function toRecords(outcome,) {
    /**
     Whether this slice's replacement is in the returned document.

     Three separate later steps can answer no, and every one of them leaves
     the reader with the archive's wording: dominance blocking the whole
     document, the assembly guard taking this slice back, and the slice's own
     selection producing no replacement to splice. That last term restates
     `repairReplacements`, which emits nothing for an unchanged outcome, so
     this record agrees with assembly by construction rather than by two files
     happening to hold the same invariant.
     */
    const replacementShipped = (!blocked)
      && outcome.changed
      && (!withdrawn.has(outcome.sliceIndex,));
    return outcome.issues
      .map(function toRecord(issue,): RepairIssueRecord {
        /**
         Regions whose envelope was cut for this issue among others.
         */
        const regions = outcome.repairRegions
          .filter(function servesIssue(region,) {
            return region.issueIds
              .includes(issue.issueId,);
          },);

        /**
         The accuracy probe's report, absent where nothing was probed.
         */
        const report = outcome.introducedDefects;

        /**
         Probe tallies for exactly those regions, empty when unprobed.
         */
        const probed = (report === undefined)
          ? []
          : report.regions
            .filter(function coversRegion(tally,) {
              return regions.some(function isSame(region,) {
                return region.envelopeId === tally.envelopeId;
              },);
            },);

        /**
         What the checkers said about this issue, absent where none ruled.
         */
        const reading = outcome.checkerReadings[issue.issueId];

        /**
         What they said about it again after refinement, absent where the
         lane bought no second round.
         */
        const recheck = outcome.recheckReadings[issue.issueId];

        return {
          sliceIndex: outcome.sliceIndex,
          issue,
          resolved: replacementShipped
            && outcome.resolvedIssueIds
              .includes(issue.issueId,),
          ...(reading === undefined
            ? {}
            : { checkerReading: reading, }),
          ...(recheck === undefined
            ? {}
            : { recheckReading: recheck, }),
          repairRegions: regions,
          repairDisposition: judgeDisposition({
            regions,
            accuracyPatchSelected: outcome.accuracyPatchSelected,
            repairReachedReader: replacementShipped,
          },),
          refined: outcome.refined,
          // THE REPORT'S OWN COUNTS, read where it exists: a region it tallied
          // implies it, so no count is ever invented for an unprobed slice.
          ...(((report === undefined) || (probed.length === 0))
            ? {}
            : {
              introducedDefects: {
                heardProbers: report.heardProbers,
                configuredProbers: report.configuredProbers,
                regions: probed,
              },
            }),
          // Gated on reach as well as on refinement. A rewritten slice the
          // document does not carry has a returned wording, but it is the
          // archive's, not this one; writing the rewrite here would state the
          // opposite in the one field a grader reads as what shipped.
          ...((outcome.refined && replacementShipped)
            ? { finalSliceText: outcome.repairedText, }
            : {}),
          // Carried unfiltered, unlike the `introducedDefects` field. That one is
          // narrowed to the regions serving THIS issue, because the accuracy
          // stage replaces one region per envelope and an issue is served by
          // some of them. The lane rewrites the whole slice as one edit, so
          // every issue in the slice shares the same single region and there is
          // nothing to select.
          // Rebuilt field by field rather than spread. The outcome carries an
          // IntroducedDefectReport, which also holds `findings`, and a record
          // is an IssueProbeReading, which does not. Structural typing accepts
          // the wider value silently, so spreading it would write a field into
          // every artifact that nothing declares and nothing reads.
          ...(outcome.refinementDefects === undefined
            ? {}
            : {
              refinementDefects: {
                heardProbers: outcome.refinementDefects
                  .heardProbers,
                configuredProbers: outcome.refinementDefects
                  .configuredProbers,
                regions: outcome.refinementDefects
                  .regions,
              },
            }),
        };
      },);
  },);
}

//endregion Repair issue record
