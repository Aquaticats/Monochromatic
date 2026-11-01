import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  type ArtifactFileName,
  entryIdOfArtifact,
} from './artifact-file-name.ts';

//region Artifact provenance
// What a reader must prove about the BYTES IT ACTUALLY READ before a number
// drawn from them may name a pipeline.
//
// The generation filter answers "which entries may be pooled" from a census.
// That census is a SEPARATE directory read from the one each reader performs
// afterwards, and the accumulation writes new artifacts continuously, so the
// metadata the census returned is not automatically metadata for the bytes a
// reader later loaded. A file can arrive between the two reads, and a file can
// change between being classified and being loaded.
//
// Nothing here re-reads or re-derives. It compares what the census said against
// what the reader observed, and refuses when they disagree, which is the only
// way that gap surfaces at all: every other outcome of it looks like ordinary
// output.

/**
 Fewest characters of a commit ever shown, so short reports stay scannable.

 Nine rather than seven because this repository already has commits colliding
 at seven. It is a FLOOR, not the answer: see {@link abbreviate}.
 */
const MIN_ABBREVIATION = 9;

/**
 Shortens hex ids just enough that the ones being shown stay distinguishable.

 Takes ids rather than commits because a report names two kinds of them now:
 the digest identifying a built pipeline, and the commit the pass that ran it
 started under. Both are lowercase hex nobody wants printed in full, and
 sizing them together is the point, since what has to stay distinguishable is
 whatever appears side by side.

 A fixed width is a bet that no two ids in a report share a prefix, and the
 bet has already been lost once here at seven characters. Widening to nine
 only moved the bet. Two generations rendering as the same string is worse
 than a long string: a report whose two lines read alike is a report nobody
 can act on, and this one is read precisely when a pool is suspected of
 spanning versions.

 Grows from {@link MIN_ABBREVIATION} until every input is unique, so an
 ordinary report is short and only an actual collision pays for length.

 @param ids - every hex id that will appear in this report

 @returns Function shortening one id to the agreed width

 @example
 ```ts
 const short = abbreviate({ ids: census.groups.map(toDigest), },);
 console.log(short({ id, },),);
 ```
 */
export function abbreviate(
  { ids, }: { readonly ids: readonly string[]; },
): (input: { readonly id: string; }) => string {
  /**
   Longest id shown, the point past which growing cannot help.
   */
  const longest = ids.reduce(
    function toLongest(
      soFar,
      id,
    ): number {
      return Math.max(
        soFar,
        id.length,
      );
    },
    MIN_ABBREVIATION,
  );

  /**
   Every width worth testing, shortest first.
   */
  const candidates = Array.from(
    { length: (longest - MIN_ABBREVIATION) + 1, },
    function toWidth(
      _unused,
      offset,
    ): number {
      return MIN_ABBREVIATION + offset;
    },
  );

  /**
   Shortest width at which no two DISTINCT ids read alike.

   Distinct is what the comparison measures, and deliberately: a report
   naming one pipeline twice should print it the same way both times, so a
   repeated id is separated at the floor rather than growing the whole
   report chasing a difference that does not exist. Full length is reached
   only for ids in a prefix relation, and reaching it always separates them:
   the last candidate is the longest id's length, at which every id is read
   whole, so the search always finds a width. A fallback to that same width
   followed it, and no input could reach it (ledger T8).
   */
  const width = nonNullishOrThrow(candidates.find(function separates(candidate,): boolean {
    return new Set(ids.map(function toPrefix(id,): string {
      return id.slice(
        0,
        candidate,
      );
    },),).size === new Set(ids,).size;
  },),);
  return function short({ id, }: { readonly id: string; },): string {
    return id.slice(
      0,
      width,
    );
  };
}

/**
 How a reader chose which pipeline generations it would pool.

 Recorded rather than inferred from the required commit alone, because the
 three modes are not distinguishable after the fact and they license different
 claims. Only `single-generation` licenses "produced by pipeline X".

 @example
 ```ts
 const selection: GenerationSelection = { kind: 'single-generation', digest, };
 ```
 */
export type GenerationSelection =
  | Readonly<{
    /**
     Entries whose recorded pipeline CONTAINS a named commit.

     This is an ancestry floor, not a generation. Every descendant tip
     qualifies, and descendants may differ from each other arbitrarily, so a
     pool selected this way is a post-baseline cohort rather than one version.
     */
    kind: 'required-commit';

    /**
     Commit an entry's pipeline must contain.
     */
    commit: string;
  }>
    | Readonly<{
    /**
     Entries recording exactly one built pipeline, the only selection that
     licenses describing a rate as belonging to that pipeline.
     */
      kind: 'single-generation';

    /**
     Digest of the built output every pooled entry ran.
     */
      digest: string;
    }>
    | Readonly<{
    /**
     Every generation present, pooled deliberately. Legitimate for a census,
     never for a rate.
     */
      kind: 'all-generations';
    }>;

/**
 Raised when loaded bytes disagree with what the pool said about them.
 */
export class ArtifactProvenanceError extends Error {
  /**
   Names the artifact, the disagreement, and why it is fatal rather than
   skippable.

   @param name - artifact file name as read from disk

   @param field - what disagreed

   @param expected - value the pool recorded

   @param observed - value the loaded bytes carry

   @example
   ```ts
   throw new ArtifactProvenanceError({ name, field: 'tip', expected, observed, },);
   ```
   */
  constructor(
    {
      name,
      field,
      expected,
      observed,
    }: {
      readonly name: string;
      readonly field: string;
      readonly expected: string;
      readonly observed: string;
    },
  ) {
    super(
      [
        `Artifact ${name} disagrees with the pool about its ${field}.`,
        `  pool recorded  ${expected}`,
        `  bytes carry    ${observed}`,
        '',
        'The pool is built from one directory read and each artifact is loaded',
        'by a later one, while the accumulation keeps writing. A disagreement',
        'means the entry admitted to the pool is not the entry that was read,',
        'so every number drawn from it describes something other than what was',
        'filtered. Re-run the reader against a directory nothing is writing to.',
      ].join('\n',),
    );
    this.name = 'ArtifactProvenanceError';
  }
}

/**
 What the pool recorded for one admitted entry.

 @example
 ```ts
 const expected: ExpectedProvenance = { tip, digest, };
 ```
 */
export type ExpectedProvenance = Readonly<{
  /**
   Repo commit the entry's pass started under.
   */
  tip: string;

  /**
   Built pipeline the entry's pass executed.
   */
  digest: string;
}>;

/**
 Refuses a loaded artifact that is not the one the pool admitted.

 Called by READERS rather than by the census, deliberately. The census now
 runs inside `assertArtifactsPlaceable` at pass startup, and a throw there
 would abort an accumulation over a telemetry invariant; a throw here costs
 only the report.

 TAKES THE POOL'S RECORD WHOLE, a tip and a digest together or neither. The
 pool admits only placed entries and records both for every one, so a tip
 without a digest was never a record it could hand over; it stood here as a
 separate optional field with a tip-only check behind it that no reader
 reached (ledger T8).

 @param name - artifact file name, whose stem is the id the pool keyed on

 @param observedId - entry id the loaded bytes record

 @param observedTip - pipeline commit the loaded bytes record

 @param observedDigest - built pipeline the loaded bytes record, empty when
 they record none

 @param expected - commit and built pipeline the pool recorded for this
 entry, absent when the pool did not admit it

 @throws ArtifactProvenanceError when the file name, the recorded id, the
 recorded tip, or the recorded pipeline disagree

 @example
 ```ts
 assertArtifactProvenance({ name, observedId, observedTip, observedDigest, expected, },);
 ```
 */
export function assertArtifactProvenance(
  {
    name,
    observedId,
    observedTip,
    observedDigest,
    expected,
  }: {
    readonly name: ArtifactFileName;
    readonly observedId: string;
    readonly observedTip: string;
    readonly observedDigest: string;
    readonly expected?: ExpectedProvenance;
  },
): void {
  /**
   Entry id the pool keyed on, which is the artifact's own file name.
   */
  const keyedId = entryIdOfArtifact({ name, },);

  if (observedId !== keyedId)
    throw new ArtifactProvenanceError({
      name,
      field: 'entry id',
      expected: keyedId,
      observed: observedId,
    },);

  // Absent rather than empty means the pool never placed this entry, which is
  // how a malformed artifact reaches its reporting reader. That is a state the
  // pool defines, and the check it needs is the OPPOSITE one: the pool said
  // these bytes could not be placed, so bytes that place perfectly well mean
  // the file changed between the two reads. Reading it as "nothing to compare"
  // let exactly the race this module exists to close through, since a
  // half-written artifact is classified malformed and is valid moments later.
  if (expected === undefined) {
    if ((observedTip !== '') && (observedDigest !== ''))
      throw new ArtifactProvenanceError({
        name,
        field: 'admission',
        expected: 'unplaceable, as the pool classified it',
        observed: `tip ${observedTip}, pipeline ${observedDigest}`,
      },);
    return;
  }

  if (observedTip !== expected.tip)
    throw new ArtifactProvenanceError({
      name,
      field: 'tip',
      expected: expected.tip,
      observed: observedTip,
    },);

  // Checked after the tip and separately from it, because the two disagree for
  // different reasons and only this one moves the entry out of the pool's
  // generation. A tip mismatch means the file was rewritten; a digest mismatch
  // means the file was rewritten BY A DIFFERENT PIPELINE, which is the case the
  // whole census exists to catch.
  if (observedDigest !== expected.digest)
    throw new ArtifactProvenanceError({
      name,
      field: 'pipeline digest',
      expected: expected.digest,
      observed: observedDigest,
    },);
}

//endregion Artifact provenance
