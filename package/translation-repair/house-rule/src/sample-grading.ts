import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Sample grading model
// The milestone-three headline gate is precision of accepted issues on a
// human-graded sample, because a judge ensemble drawn from the same seven
// models re-affirming its own panel's acceptances is circular. The draw is
// stratified by page-size band: the accumulation pool skews small, so a plain
// uniform draw would under-represent the heavily-sliced large tail, exactly
// where the lockstep slicing fix most affects precision. This module owns the
// corpus-free model -- size-band cuts, the grading candidate, and flattening
// one accepted issue into a candidate. The seeded draw lives in
// `sample-draw.ts` and the sheet rendering in `grading-sheet.ts`; the
// corpus-reading wiring lives in the corpus-run script, since sheet content
// quotes UNLICENSED corpus text and must never be committed.

/**
 Exclusive upper byte bound of the small band: a page whose zh source is
 under this many UTF-8 bytes is small. The value is the corpus source-byte
 lower tertile the accumulation loop already sorts by, so sample bands match
 accumulation bands exactly.
 */
export const SMALL_BAND_MAX_BYTES = 1_843;

/**
 Exclusive upper byte bound of the medium band: a page at or above
 {@link SMALL_BAND_MAX_BYTES} yet under this many UTF-8 bytes is medium, and
 at or above it the page is large. The value is the corpus source-byte upper
 tertile.
 */
export const MEDIUM_BAND_MAX_BYTES = 3_686;

/**
 Default headline-sample size; the user accepted 50 when scoping milestone
 three.
 */
export const DEFAULT_SAMPLE_SIZE = 50;

/**
 Default precision bar the graded sample must clear; the user accepted 0.9
 when scoping milestone three.
 */
export const DEFAULT_PRECISION_BAR = 0.9;

/**
 Default draw seed. Fixing it makes the draw deterministic: the same
 candidate pool and seed always produce the same sample, so a draw can be
 reproduced and audited.

 The seed MUST change whenever a graded sample is used to change the
 pipeline. Re-drawing with the seed the grades came from partially
 re-selects the very items a human already judged, so the next measurement
 would be scored partly on its own calibration set and would read better
 than the pipeline is. The suffix records which measurement round a seed
 belongs to; the first round's seed was `milestone-three-precision`, whose
 fifty grades produced the identity, alignment, and policy fixes, and
 `-round-two` produced the roster, editor-ensemble, and house-policy changes.

 A new seed does NOT guarantee that no already-graded issue is drawn again: a
 different shuffle can reselect one. It is the population that mostly changes
 between rounds, since round three draws from artifacts produced by a fresh
 pass, and issue ids are content-derived rather than stable across runs.
 */
export const DEFAULT_SAMPLE_SEED = 'milestone-three-precision-round-three';

/**
 Bands in presentation order, smallest first.

 THE TYPE IS DERIVED FROM THIS LIST, so a band cannot be added to the type
 without being listed here, and the draw and the sheet that walk this list
 cannot skip it.
 */
export const SIZE_BANDS = [
  'small',
  'medium',
  'large',
] as const;

/**
 One of the page-size bands the corpus stratifies into.

 @example
 ```ts
 const band: SizeBand = 'medium';
 ```
 */
export type SizeBand = typeof SIZE_BANDS[number];

/**
 A per-band count: candidates available, or slots allocated.
 */
export type BandQuota = Readonly<Record<SizeBand, number>>;

/**
 One accepted issue flattened into everything a human grader needs to judge
 whether it is a real defect, and everything the draw needs to stratify.

 @example
 ```ts
 const candidate: GradingCandidate = {
   entryId: 'MushroomGuuuu',
   band: 'medium',
   issueId: 'adjudicated/abcd',
   category: 'accuracy/omission',
   severity: 'minor',
   summary: 'The closing greeting is dropped.',
   sourceQuotes: ['再见'],
   targetQuotes: [],
 };
 ```
 */
export type GradingCandidate = {
  /**
   Corpus entry the issue was found in.
   */
  readonly entryId: string;

  /**
   Size band of the entry, for stratification.
   */
  readonly band: SizeBand;

  /**
   Deterministic adjudicated-issue identity, also the draw's shuffle key.
   */
  readonly issueId: string;

  /**
   Category of the issue's primary claim, as a plain string: this is a
   display field on a human sheet, so it is never narrowed to the closed
   taxonomy. Keeping it a string means the parser never has to skip an
   off-taxonomy accepted issue, which would silently bias the precision
   denominator.
   */
  readonly category: string;

  /**
   Adjudicated severity of the issue, as a plain display string for the same
   reason as {@link GradingCandidate.category}.
   */
  readonly severity: string;

  /**
   One-sentence statement of the primary claimed defect.
   */
  readonly summary: string;

  /**
   Distinct zh source quotes the issue anchors, in first-seen order.
   */
  readonly sourceQuotes: readonly string[];

  /**
   Distinct en target quotes the issue anchors, in first-seen order.
   */
  readonly targetQuotes: readonly string[];

  /**
   Why the source side carries no quote, when it carries none. An addition
   claim legitimately anchors to an EMPTY insertion point, so a bare
   `(none)` conflated two different situations: a claim anchored at a real
   place that has no text there, and a claim with no source anchor at all.
   The second asserts something is absent from the original while pointing
   at nothing in it, which a grader cannot check.
   */
  readonly sourceAnchor: SourceAnchorKind;

  /**
   What the accuracy stage wrote for this issue and what became of it, when
   the run recorded that at all.

   ABSENT is not the same as a repair with no regions, and no sheet may merge
   them. Absent is a run predating repair recording, where repair quality is
   unknowable and the item cannot enter any repair denominator; a recorded
   repair reporting `no-region` is a real measurement that belongs in the
   coverage denominator.
   */
  readonly repair?: GradableRepair;
};

/**
 One replaced region as a grading sheet reads it.

 @example
 ```ts
 const region: GradableRepairRegion = {
   issueIds: ['adjudicated/one',],
   before: 'The cat is doing the sleeping.',
   editorAfter: 'The cat is asleep.',
 };
 ```
 */
export type GradableRepairRegion = {
  /**
   Every accepted issue the replaced region was cut for, so a shared edit is
   disclosed as shared rather than presented as this issue's own repair.
   */
  readonly issueIds: readonly string[];

  /**
   Region content before the replacement; empty at an insertion point.
   */
  readonly before: string;

  /**
   Replacement the selected accuracy patch carried.
   */
  readonly editorAfter: string;
};

/**
 One issue's repair as a grading sheet reads it.

 @example
 ```ts
 const repair: GradableRepair = {
   disposition: 'shipped',
   regions: [],
   refined: false,
 };
 ```
 */
export type GradableRepair = {
  /**
   What became of the repair in the returned document, kept a plain display
   string for the same reason as {@link GradingCandidate.category}: a value
   the sheet does not recognize must still reach the grader rather than drop
   the item out of the population.
   */
  readonly disposition: string;

  /**
   Replaced regions serving this issue, in document order.
   */
  readonly regions: readonly GradableRepairRegion[];

  /**
   Whether the naturalness lane rewrote this issue's slice afterwards, making
   every {@link GradableRepairRegion.editorAfter} pre-refinement wording.
   */
  readonly refined: boolean;

  /**
   Final text of the issue's slice, carried only when
   {@link GradableRepair.refined} is set, which is exactly where the recorded
   replacement stopped being the returned wording.
   */
  readonly finalSliceText?: string;
};

/**
 How an issue's source side is anchored.

 @example
 ```ts
 const anchor: SourceAnchorKind = 'insertion-point';
 ```
 */
export type SourceAnchorKind =
  /**
   The issue quotes source text, which the sheet shows.
   */
  | 'quoted'
  /**
   The issue anchors an empty span: a real place in the original with no
   text at it, which is how an insertion is correctly anchored.
   */
    | 'insertion-point'
  /**
   The issue anchors nothing in the original at all.
   */
    | 'unanchored';

/**
 One anchored span the grading extraction reads: which document side it
 points into and the exact text it quotes.
 */
export type GradableSpan = {
  /**
   Document side the span points into.
   */
  readonly side: 'source' | 'target';

  /**
   Exact text the span quotes; empty for an insertion anchor.
   */
  readonly quotedText: string;
};

/**
 One member claim the grading extraction reads.
 */
export type GradableClaim = {
  /**
   Category slug of the claim, kept a plain string (see
   {@link GradingCandidate.category}).
   */
  readonly category: string;

  /**
   One-sentence statement of the claimed defect.
   */
  readonly summary: string;

  /**
   Anchored spans the claim cites.
   */
  readonly spans: readonly GradableSpan[];
};

/**
 The minimal accepted-issue shape the grading extraction reads: identity,
 severity, and member claims. A full pipeline `AdjudicatedIssue` satisfies it
 structurally, and so does an issue parsed back from a run artifact, so the
 extractor is decoupled from the pipeline's richer internal shape.
 */
export type GradableIssue = {
  /**
   Adjudicated-issue identity, also the draw's shuffle key.
   */
  readonly issueId: string;

  /**
   Adjudicated severity, kept a plain string (see
   {@link GradingCandidate.severity}).
   */
  readonly severity: string;

  /**
   Member claims, primary first.
   */
  readonly claims: readonly {
    /**
     The member claim.
     */
    readonly claim: GradableClaim;
  }[];
};

/**
 Raised when a sample cannot support a repair measurement because some of its
 issues predate repair recording.
 A STATED REFUSAL, since the message counts samples and says what to do next:
 a command reports it as declined, with no frames, rather than as a fault of
 the command.

 @example
 ```ts
 throw new UnmeasurableRepairError({ unrecorded: 50, sampled: 50, },);
 ```
 */
export class UnmeasurableRepairError extends StatedRefusalError {
  /**
   Declared here as well as inherited, so the source scan that keeps the
   marked-class inventory sees it: the message counts samples and quotes
   nothing.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Builds the refusal from the counts that make the sample unmeasurable.

   @param unrecorded - sampled issues carrying no recorded repair

   @param sampled - size of the drawn sample
   */
  constructor(
    {
      unrecorded,
      sampled,
    }: {
      readonly unrecorded: number;
      readonly sampled: number;
    },
  ) {
    super({
      says: `refusing a final draw: ${String(unrecorded,)} of ${
        String(sampled,)
      } sampled ${
        wordForCount({
          count: sampled,
          one: 'issue carries',
          many: 'issues carry',
        },)
      } no recorded repair, so repair quality cannot be `
        + `measured over this sample. Those artifacts predate repair recording; `
        + `move them aside and rerun the pass into a fresh artifacts directory.`,
    },);
    this.name = 'UnmeasurableRepairError';
  }
}

/**
 Counts sampled issues whose run never recorded what was written.

 @param sample - drawn candidates

 @returns How many carry no repair provenance at all

 @example
 ```ts
 const unrecorded = countUnrecordedRepairs({ sample, },);
 ```
 */
export function countUnrecordedRepairs(
  { sample, }: { readonly sample: readonly GradingCandidate[]; },
): number {
  return sample.filter(function lacksRepair(candidate,) {
    return candidate.repair === undefined;
  },)
    .length;
}

/**
 Refuses a gate sample that cannot state what the pipeline wrote.

 The failure this prevents is silent rather than loud: the sheets render, every
 ungradable item reads as such, and the round still reports a repair number
 over whatever fraction happened to be recorded. It is reachable by simply
 drawing against a directory that still holds an earlier round's artifacts,
 since the corpus pass never overwrites one.

 @param sample - drawn candidates

 @throws {@link UnmeasurableRepairError} when any sampled issue carries no
 recorded repair

 @example
 ```ts
 assertRepairMeasurable({ sample, },);
 ```
 */
export function assertRepairMeasurable(
  { sample, }: { readonly sample: readonly GradingCandidate[]; },
): void {
  /**
   Sampled issues whose run never recorded a repair.
   */
  const unrecorded = countUnrecordedRepairs({ sample, },);
  if (unrecorded === 0)
    return;
  throw new UnmeasurableRepairError({
    unrecorded,
    sampled: sample.length,
  },);
}

/**
 UTF-8 byte length of an entry's zh source, branded so a CHARACTER count
 cannot be passed where a byte count belongs.

 Branded rather than left a bare `number` because the two quantities differ by
 roughly a factor of two on this corpus and the wrong one silently classifies
 large pages as small. Mint it with {@link sourceBytesOf}; a raw number will
 not type-check.

 @example
 ```ts
 const bytes: SourceBytes = sourceBytesOf({ text: source, },);
 ```
 */
export type SourceBytes = number & { readonly __brand: 'SourceBytes'; };

/**
 Narrows a count already known to be a UTF-8 byte length.

 An assertion function rather than a cast, because `no-unsafe-type-assertion`
 refuses a narrowing `as` and this repository's guidance names assertion
 functions as the mechanism for runtime narrowing. The check is real: a byte
 length is a non-negative safe integer, so a fractional or negative count is a
 caller bug rather than a small measurement error.

 Calling this is an EXPLICIT claim that the number was measured in bytes. It
 cannot verify that, which is the point of routing ordinary use through
 {@link sourceBytesOf} instead; reach for this only where the byte count is
 the thing under test, as at a band boundary.

 @param count - byte length to narrow

 @returns Nothing; it narrows `count` in the caller on success

 @throws {@link RangeError} when the count is not a non-negative safe integer

 @example
 ```ts
 assertSourceBytes(SMALL_BAND_MAX_BYTES - 1,);
 ```
 */
export function assertSourceBytes(
  count: number,
): asserts count is SourceBytes {
  if ((!Number.isSafeInteger(count,)) || (count < 0))
    throw new RangeError(
      `A UTF-8 byte length is a non-negative safe integer; received ${
        String(count,)
      }. A fractional or negative value means the caller measured something `
        + 'other than bytes.',
    );
}

/**
 Measures source text in UTF-8 bytes.

 The ordinary way to obtain a {@link SourceBytes}: it takes the TEXT rather
 than a count, so there is no opportunity to hand it the wrong unit. A number
 has already lost the evidence of which unit it is.

 @param text - source text to measure

 @returns Its UTF-8 byte length

 @example
 ```ts
 const bytes = sourceBytesOf({ text: '雨', },); // 3
 ```
 */
export function sourceBytesOf(
  { text, }: { readonly text: string; },
): SourceBytes {
  /**
   Encoded length of the text, unbranded until asserted.
   */
  const { length, } = new TextEncoder()
    .encode(text,);
  assertSourceBytes(length,);
  return length;
}

/**
 Classifies a page into its size band by zh source byte length, using the
 same tertile cuts the accumulation loop sorts by.

 BYTES, NOT CHARACTERS, and the gap is wide enough to move an entry two bands.
 The corpus is mostly Chinese, so UTF-8 spends about three bytes per han
 character; measured whole-file across settled pages the ratio lands between
 1.65 and 2.61 once Latin names, punctuation and markup are mixed in. A
 CHARACTER count passed here silently classifies large pages as small.

 This has happened. The settled artifact records `sourceChars`, a CHARACTER
 count, and it is the nearest-looking field to hand when writing an ad-hoc
 census over the artifacts directory. Passing it straight in reported a
 20-entry pool as 16 small, 4 medium and 0 large when the true spread was 7,
 7 and 6, and produced a recommendation to keep accumulating until large
 entries appeared, of which six had already settled.

 Take the bytes from the SOURCE TEXT, never from the artifact:

 ```ts
 const source = await readCorpusFile({ pin, relPath: `people/${id}/page.md`, },);
 const band = classifyBand({
   sourceBytes: new TextEncoder()
     .encode(source,)
     .length,
 },);
 ```

 `corpus-run/draw-entry-load.ts` is the one production caller and does exactly
 that. Copy it rather than reaching for the artifact field.

 @param sourceBytes - UTF-8 BYTE length of the entry's zh source, never a
 character count

 @returns The entry's size band

 @example
 ```ts
 const band = classifyBand({ sourceBytes: 1_934, },); // 'medium'
 ```
 */
export function classifyBand(
  { sourceBytes, }: { readonly sourceBytes: SourceBytes; },
): SizeBand {
  if (sourceBytes < SMALL_BAND_MAX_BYTES)
    return 'small';
  if (sourceBytes < MEDIUM_BAND_MAX_BYTES)
    return 'medium';
  return 'large';
}

/**
 Classifies how an issue anchors its source side, distinguishing a correctly
 anchored insertion from a claim that points at nothing in the original.

 @param issue - the adjudicated issue whose spans are inspected

 @returns Which anchor kind the source side has

 @example
 ```ts
 const anchor = classifySourceAnchor({ issue, },);
 ```
 */
export function classifySourceAnchor(
  { issue, }: { readonly issue: GradableIssue; },
): SourceAnchorKind {
  /**
   Every span pointing into the original.
   */
  const sourceSpans = issue.claims
    .flatMap(function claimSpans(member,) {
      return member.claim
        .spans
        .filter(function onSource(span,) {
          return span.side === 'source';
        },);
    },);
  if (sourceSpans.length === 0)
    return 'unanchored';
  return sourceSpans.some(function hasText(span,) {
    return span.quotedText
      .length
      > 0;
  },)
    ? 'quoted'
    : 'insertion-point';
}

/**
 Distinct non-empty span quotes on one side of an adjudicated issue, in
 first-seen order. A `Set` collapses the repeats a multi-claim issue anchors
 onto the same text while preserving insertion order.

 @param issue - the adjudicated issue whose spans are gathered

 @param side - which document side to keep quotes from

 @returns Distinct quotes for that side
 */
function sideQuotes(
  {
    issue,
    side,
  }: {
    readonly issue: GradableIssue;
    readonly side: 'source' | 'target';
  },
): readonly string[] {
  return [
    ...new Set(
      issue.claims
        .flatMap(function claimSpans(member,) {
          return member.claim
            .spans
            .filter(function onSide(span,) {
              return span.side === side;
            },)
            .map(function toQuote(span,) {
              return span.quotedText;
            },);
        },)
        .filter(function nonEmpty(quote,) {
          return quote.length > 0;
        },),
    ),
  ];
}

/**
 Flattens one adjudicated issue into a grading candidate. Primary category,
 severity and summary come from the issue's first member claim in document
 order; source and target quotes gather across every member claim.

 @param issue - the accepted adjudicated issue

 @param entryId - corpus entry the issue was found in

 @param band - size band of that entry

 @param repair - what became of this issue's repair, omitted for artifacts
 written before repair recording existed

 @returns The flattened grading candidate

 @example
 ```ts
 const candidate = extractGradingCandidate({
   issue,
   entryId: 'MushroomGuuuu',
   band: 'medium',
 },);
 ```
 */
export function extractGradingCandidate(
  {
    issue,
    entryId,
    band,
    repair,
  }: {
    readonly issue: GradableIssue;
    readonly entryId: string;
    readonly band: SizeBand;
    readonly repair?: GradableRepair;
  },
): GradingCandidate {
  /**
   First member claim in document order, carrying the headline category and
   summary a grader reads first; `at` keeps the empty-claims narrowing the
   fallbacks in this function rely on.
   */
  const primary = issue.claims
    .at(0,);

  return {
    entryId,
    band,
    issueId: issue.issueId,
    category: primary?.claim
      .category
      ?? '(uncategorized)',
    severity: issue.severity,
    summary: primary?.claim
      .summary
      ?? '(no claim summary)',
    sourceAnchor: classifySourceAnchor({ issue, },),
    sourceQuotes: sideQuotes({
      issue,
      side: 'source',
    },),
    targetQuotes: sideQuotes({
      issue,
      side: 'target',
    },),
    ...(repair === undefined ? {} : { repair, }),
  };
}

//endregion Sample grading model
