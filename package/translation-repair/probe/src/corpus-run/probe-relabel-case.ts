import {
  type AdjudicatedIssue,
  readRunJson,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  type CorpusPin,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  prepareDocumentPair,
  type RepairRegion,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import { parseSampleManifest, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import { readArtifactRecords, } from './probe-relabel-artifact.ts';

//region Probe relabel cases
// Rebuilds the exact prober inputs for regions a HUMAN read as damaged, so the
// probe can be asked about them again under a changed prompt.
//
// The cat fixtures answered what the probe can do on clean, quotable damage:
// three voices of three. They cannot answer why it says nothing on real edits,
// because their damage is not the damage production makes. These cases are the
// opposite trade: real corpus text, and ground truth that came from reading
// rather than from the pipeline.
//
// Nothing here is written to the repository. Corpus text is read through git at
// the pinned commit and stays in memory.

/**
 Sample positions whose repair a human read as damaged, with what was seen.

 From the round-three repair sheet, which was deliberately left ungraded
 because the repairs were too broken to score. Written as records rather than
 bare numbers so each position carries the observation that put it here; a
 list of integers would say nothing about why these and not others.

 @example
 ```ts
 const first = DAMAGED_CASES[0]?.position;
 ```
 */
export const DAMAGED_CASES = [
  {
    position: 2,
    damage: 'deleted a source-supported clause while fixing an addition claim',
  },
  {
    position: 7,
    damage: 'same edit, drawn again under a second accepted issue',
  },
  {
    position: 11,
    damage: 'deleted source-supported content beyond the quoted defect',
  },
  {
    position: 15,
    damage: 'deleted source-supported content beyond the quoted defect',
  },
  {
    position: 20,
    damage: 'reordered sentences the defect did not concern',
  },
  {
    position: 21,
    damage: 'deleted a contributor credit from an edit asked only to change a colon',
  },
  {
    position: 37,
    damage: 'replaced the sense of a verb, reminiscing became pleading',
  },
  {
    position: 43,
    damage: 'invented wording that appears in neither source nor translation',
  },
] as const;

/**
 Positions {@link DAMAGED_CASES} names, for membership tests.
 */
const DAMAGED_POSITIONS: ReadonlySet<number> = new Set(
  DAMAGED_CASES.map(function toPosition(entry,) {
    return entry.position;
  },),
);

/**
 Everything one prober call needs, rebuilt for a single damaged region.

 @example
 ```ts
 const [first,] = await gatherRelabelCases({ manifestPath, pin, },);
 ```
 */
export type RelabelCase = {
  /**
   Corpus entry the region belongs to.
   */
  readonly entryId: string;

  /**
   Sample positions that drew this region; more than one means the sheet
   showed the same edit under several accepted issues.
   */
  readonly positions: readonly number[];

  /**
   Region as the pipeline recorded it.
   */
  readonly region: RepairRegion;

  /**
   Accepted issues the region served, exactly as production renders them.
   */
  readonly issues: readonly AdjudicatedIssue[];

  /**
   Source text of the slice this region sits in.
   */
  readonly sourceText: string;

  /**
   Translation of that slice before any replacement.
   */
  readonly baselineText: string;

  /**
   What the probe said about this region during the run, for comparison.
   */
  readonly recorded: string;
};

/**
 A case while the sample positions drawing its region are still being
 gathered, so a later position joins the list in place (ledger B74).
 */
type GatheringCase = Omit<RelabelCase, 'positions'> & {
  /**
   Sample positions that drew this region so far.
   */
  readonly positions: number[];
};

/**
 Finds the slices whose translation contains a region's replaced text, which
 can be none or one and never more.

 EVERY SLICE IS GATHERED, never the first taken: a short replaced text can
 stand in more than one slice, and a prompt rebuilt from whichever came first
 asks the prober about a passage production may not have sent.

 @param slices - slices of the pair, from the preparation

 @param before - replaced text to locate

 @returns An empty list when no slice carries the text, otherwise the one that does

 @throws {@link ArtifactParseError} when more than one slice carries the text,
 naming the text's length and never the text

 @example
 ```ts
 const [holder,] = holdingSlices({ slices, before, },);
 ```
 */
export function holdingSlices<SliceT extends { readonly target: { readonly text: string; }; }>(
  {
    slices,
    before,
  }: {
    readonly slices: readonly SliceT[];
    readonly before: string;
  },
): readonly SliceT[] {
  /**
   Slices whose translation carries the replaced text.
   */
  const holders = slices
    .filter(function holdsBefore(slice,): boolean {
      return slice.target
        .text
        .includes(before,);
    },);
  if (holders.length > 1) {
    // NAMES THE LOOKUP, NEVER THE TEXT, for the reason `locateSlice` gives.
    throw new ArtifactParseError({
      path: `slice holding the replaced text of ${String(before.length,)} ${
        wordForCount({
          count: before.length,
          one: 'character',
          many: 'characters',
        },)
      }`,
      reason:
        'present in one slice; more than one slice carries it, so which of them production sent cannot be read',
    },);
  }
  return holders;
}

/**
 Finds the slice whose translation contains a region's replaced text.

 Located by CONTENT rather than by the recorded chunk index, because an index
 carries a convention and a convention is the kind of thing that silently
 shifts between a run and a later reading. Text either contains the region or
 it does not.

 @param sourceText - whole original document

 @param targetText - whole translation

 @param before - replaced text to locate

 @returns Slice texts surrounding the region

 @throws {@link ArtifactParseError} when no slice carries the replaced text,
 which means slicing no longer reproduces the run and every later comparison
 would use a different prompt than production sent, or when more than one
 does, which leaves the slice production sent unreadable

 @example
 ```ts
 const slice = locateSlice({ sourceText, targetText, before, },);
 ```
 */
export function locateSlice(
  {
    sourceText,
    targetText,
    before,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly before: string;
  },
): {
  readonly sourceText: string;
  readonly baselineText: string;
} {
  /**
   Slices of the pair, from the deterministic preparation itself.

   THE PREPARATION, NOT A COPY OF IT (audit area six, 2026-09-28). This probe
   and its control re-carved by aligning sections and subdividing them, which
   matched the preparation on all 92 pinned pairs except for the front-matter
   slice the preparation leads with, so the copy's slice numbers ran one
   behind the run's. Nothing here reads a number, since the slice is found by
   its text, but a copy that claims to be the pipeline and is not is how a
   later measurement goes wrong.
   */
  const { slices, } = prepareDocumentPair({
    sourceText,
    targetText,
  },);

  /**
   The one slice whose translation carries the replaced text, if any does.
   */
  const [holder,] = holdingSlices({
    slices,
    before,
  },);
  if (holder === undefined) {
    // NAMES THE LOOKUP, NEVER THE TEXT. `ArtifactParseError` carries
    // `messageNamesOnly`, which `reportingRefusals` reads as permission to
    // print the whole message, and the marker's own justification is that the
    // class "names the artifact path and the shape the value failed to satisfy,
    // and quotes neither the value nor the file". Until 2026-08-25 this site
    // put 80 characters of the replaced TRANSLATION in the path, so a probe
    // that could not find its slice printed a memorial page's wording to a
    // terminal and into whatever log the run was writing. It was the only one
    // of 47 interpolating paths in the package that quoted text rather than a
    // structural position. The length is the whole diagnosis anyway: it says
    // which lookup failed and how big the missing text was, and the artifact
    // holds the text for anyone who needs to read it.
    throw new ArtifactParseError({
      path: `slice holding the replaced text of ${String(before.length,)} ${
        wordForCount({
          count: before.length,
          one: 'character',
          many: 'characters',
        },)
      }`,
      reason:
        'present in one slice; absence means slicing no longer reproduces the run, so any comparison would use a different prompt than production sent',
    },);
  }

  return {
    sourceText: holder.source
      .text,
    baselineText: holder.target
      .text,
  };
}

/**
 Rebuilds every damaged-region case named by {@link DAMAGED_CASES}.

 @param manifestPath - sample manifest the positions index into

 @param pin - corpus commit to read the pages at: the pin its entry reads in a run, a
 throwaway clone in a test. REQUIRED: a default read the clone for any caller
 that left it out (ledger M43, X24)

 @param runsDir - runs directory the entries' settled artifacts are read from,
 the one the procedure was handed

 @returns One case per distinct region, in sample order

 @throws {@link ArtifactParseError} when a manifest, artifact, or slice lookup
 does not reproduce the run

 @example
 ```ts
 const cases = await gatherRelabelCases({ manifestPath, pin, runsDir, },);
 ```
 */
export async function gatherRelabelCases(
  {
    manifestPath,
    pin,
    runsDir,
  }: {
    readonly manifestPath: string;
    readonly pin: CorpusPin;
    readonly runsDir: string;
  },
): Promise<readonly RelabelCase[]> {
  /**
   Drawn items, validated and digest-checked against their own contents.
   */
  const manifest = parseSampleManifest({
    value: await readRunJson({ path: manifestPath, },),
  },);

  /**
   Drawn items this rebuild probes.
   */
  const wanted = manifest.items
    .filter(function isDamaged(item,) {
      return DAMAGED_POSITIONS.has(item.position,);
    },);

  /**
   Cases keyed by entry and envelope, so one edit drawn twice is probed once.
   */
  const byRegion = new Map<string, GatheringCase>();
  /* oxlint-disable no-await-in-loop -- sequential on purpose: each iteration reads one artifact and two git blobs, and running them together would multiply peak memory by the entry count for no wall-clock gain on a diagnostic */
  for (const item of wanted) {
    /**
     Settled records of the entry this item was drawn from.
     */
    const records = await readArtifactRecords({
      runsDir,
      entryId: item.entryId,
    },);

    /**
     Record carrying the drawn issue.
     */
    const drawn = records
      .find(function isDrawn(candidate,) {
        /**
         Settled id of the candidate record.
         */
        const candidateId = candidate.issue
          .issueId;

        return candidateId === item.issueId;
      },);
    if (drawn === undefined)
      continue;

    /**
     Original document at the pinned commit.
     */
    const sourceText = await readCorpusFile({
      pin,
      relPath: `people/${item.entryId}/page.md`,
    },);

    /**
     Translation at the same commit.
     */
    const targetText = await readCorpusFile({
      pin,
      relPath: `people/${item.entryId}/page.en.md`,
    },);

    for (const region of drawn.repairRegions) {
      /**
       Identity of this edit within the corpus.
       */
      const key = `${item.entryId} ${region.envelopeId}`;

      /**
       Case an earlier position already built for this edit.
       */
      const seen = byRegion.get(key,);
      if (seen !== undefined) {
        seen
          .positions
          .push(item.position,);
        continue;
      }

      byRegion.set(
        key,
        {
          entryId: item.entryId,
          positions: [item.position,],
          region,
          issues: records
            .filter(function isServed(candidate,) {
              return region.issueIds
                .includes(candidate.issue
                  .issueId,);
            },)
            .map(function toIssue(candidate,) {
              return candidate.issue;
            },),
          ...locateSlice({
            sourceText,
            targetText,
            before: region.before,
          },),
          recorded: drawn.recorded[region.envelopeId] ?? 'not probed',
        },
      );
    }
  }
  /* oxlint-enable no-await-in-loop */

  return [...byRegion.values(),];
}

//endregion Probe relabel cases
