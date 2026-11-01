import type { RegionDefectTally, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Repair damage evidence
// WHAT THE LANE CONTEST IS SHOWN of the introduced-defect probe: the
// corroborated claims of added damage against the repair lane's text, as
// evidence lines, so the judges that choose between the lanes see what the
// probe saw. Nothing acts on the claims; the probe stays in shadow mode
// (`introduced-defect-probe.ts`), and the judges weigh them as they weigh the
// passages.
//
// BOTH EDITS THE LANE MAKES. The accuracy repair is probed against the archive
// text (`introducedDefects`) and the naturalness rewrite against the repaired
// text (`refinementDefects`, present only where the rewrite shipped,
// `refine-slice-settle.ts`). The candidate the judges see is the text after
// both, so damage either edit added is damage in it; the accuracy repair's
// only where its patch won the slice, since a lost patch is not in the text
// (ledger L7). keyword233 on 2026-09-03 (the section "The Hyper-slow judge is
// seated per provider" of
// `doc/planning/translation-repair-roster-calibration-2026-09-01.md`): the rewrite moved a paragraph
// into the present tense, three probers corroborated it, and the contest chose
// against the repair 5 of 7 on that tense without being shown the claims,
// because only the accuracy probe was read then.
//
// MEASURED ON 2026-09-03 over the four landings that carried the probe: ten
// regions with a corroborated claim, six true (three of them the tense rule,
// which the contest that chose the damaged lane 7 of 7 never saw), three false
// (two misparse 我方才知道 as "our side", one calls 十几 "fifteen") and one
// borderline. That precision is why the claims are shown and not obeyed.
// Record: `doc/planning/translation-repair-roster-calibration-2026-09-01.md`.

/**
 Screened regions of one probe report, the only part the evidence reads.

 @example
 ```ts
 const report: ProbedRegions = { regions: [], };
 ```
 */
export type ProbedRegions = {
  readonly regions: readonly RegionDefectTally[];
};

/**
 What one probed chunk contributes: its slice and the screened regions of
 each edit's probe.

 Structural rather than the lane's whole outcome type, so a test can feed
 this from a fixture and the pass from the lane result alike.

 @example
 ```ts
 const chunk: ProbedChunk = { sliceIndex: 3, introducedDefects: { regions: [], }, };
 ```
 */
export type ProbedChunk = {
  /**
   Slice this chunk's text ships at.
   */
  readonly sliceIndex: number;

  /**
   Whether the accuracy repair's patched text won its selection. Where it
   lost, the repair lane ships the archive or a rewrite of it, never the
   patched text its probe quoted, so that probe's claims describe text the
   judges' candidate does not carry (ledger L7).
   */
  readonly accuracyPatchSelected: boolean;

  /**
   Accuracy repair's probe report, absent where the chunk changed nothing and
   nothing was probed.
   */
  readonly introducedDefects?: ProbedRegions;

  /**
   Naturalness rewrite's probe report, absent where no rewrite shipped.
   */
  readonly refinementDefects?: ProbedRegions;
};

/**
 Edit a claim audits, named the way the judges' block names it.

 @example
 ```ts
 const stage: ProbedStage = 'accuracy repair';
 ```
 */
export type ProbedStage = 'accuracy repair' | 'naturalness rewrite';

/**
 Renders one corroborated claim as a line a judge can check against the
 ORIGINAL.

 @param modelId - prober that made the claim

 @param stage - edit the prober audited, since the two start from different
 texts and a judge checking the quote needs to know which

 @param category - defect class in the prober's words, may be empty

 @param evidence - wording quoted from the repair text

 @param reason - why the prober says the text before the edit lacked it

 @returns One line

 @example
 ```ts
 const line = claimLine({ modelId: 'minimax-m3', stage: 'accuracy repair', category: 'tense', evidence: 'is', reason: 'the page holds past tense', },);
 ```
 */
function claimLine(
  {
    modelId,
    stage,
    category,
    evidence,
    reason,
  }: {
    readonly modelId: string;
    readonly stage: ProbedStage;
    readonly category: string;
    readonly evidence: string;
    readonly reason: string;
  },
): string {
  /**
   Category as written, or a placeholder when the prober gave none: a
   category showing a reader nothing is none (ledger B40), where an
   empty-string comparison printed it to the judges as `[ ]`.
   */
  const kind = rendersAsNothing({ text: category, },) ? 'unspecified' : category;
  return `- ${modelId} [${kind}] on the ${stage} quotes "${evidence}": ${reason}`;
}

/**
 Lines one probe report's corroborated claims make, none where nothing was
 probed or nothing was corroborated.

 @param regions - one edit's screened regions, none where that edit was not
 probed

 @param stage - edit the report audited

 @returns Lines in region and claim order

 @example
 ```ts
 const lines = corroboratedLinesOf({ regions: chunk.introducedDefects?.regions ?? [], stage: 'accuracy repair', },);
 ```
 */
function corroboratedLinesOf(
  {
    regions,
    stage,
  }: {
    readonly regions: readonly RegionDefectTally[];
    readonly stage: ProbedStage;
  },
): readonly string[] {
  return regions
    .flatMap(function linesOfRegion(region,): readonly string[] {
      return region.claims
        .filter(function isCorroborated(claim,): boolean {
          return claim.admissibility === 'corroborated';
        },)
        .map(function toLine(claim,): string {
          return claimLine({
            modelId: claim.modelId,
            stage,
            category: claim.category,
            evidence: claim.evidence,
            reason: claim.reason,
          },);
        },);
    },);
}

/**
 Collects, per slice, the corroborated added-damage claims against the repair
 lane's text.

 ONLY CORROBORATED CLAIMS: a claim whose quote the differential confirmed as
 added by the edit. Contradicted, unanchored, pre-existing and dropped-content
 claims stay in the record and out of the judges' sight, since the first two
 failed a deterministic check and the last two are not about added damage.

 FROM BOTH EDITS: the accuracy repair's claims first, then the naturalness
 rewrite's, each line naming its edit. The accuracy repair's only where its
 patch won: a lost patch never reaches the candidate the judges read, and
 XingZ6014 slice 3 showed them damage quoting "she came out as trans", which
 its repair candidate did not carry (ledger L7). The rewrite's claims stand
 either way, since the rewrite ships over whatever the accuracy stage kept.

 @param lane - repair lane result, read for its probed chunks

 @returns Lines keyed by slice, absent for slices with no corroborated claim

 @example
 ```ts
 const bySlice = damageClaimLinesBySlice({ lane: lanes.repair, },);
 ```
 */
export function damageClaimLinesBySlice(
  { lane, }: { readonly lane: { readonly chunks: readonly ProbedChunk[]; }; },
): ReadonlyMap<number, readonly string[]> {
  return lane.chunks
    .reduce(
    function collect(
      bySlice: Map<number, readonly string[]>,
      chunk: ProbedChunk,
    ): Map<number, readonly string[]> {
        /**
         Regions the accuracy repair's probe reported on, none where the
         chunk changed nothing and nothing was probed.
         */
      const accuracyRegions = chunk.introducedDefects
        ?.regions
        ?? [];
        /**
         Lines this chunk's corroborated claims make, both edits' probes.
         */
      const lines = [
        ...corroboratedLinesOf({
          regions: chunk.accuracyPatchSelected ? accuracyRegions : [],
          stage: 'accuracy repair',
        },),
        ...corroboratedLinesOf({
          regions: chunk.refinementDefects
            ?.regions
            ?? [],
          stage: 'naturalness rewrite',
        },),
      ];
      if (lines.length === 0)
        return bySlice;
      bySlice.set(
        chunk.sliceIndex,
        [
          ...(bySlice.get(chunk.sliceIndex,) ?? []),
          ...lines,
        ],
      );
      return bySlice;
    },
    new Map<number, readonly string[]>(),
    );
}

//endregion Repair damage evidence
