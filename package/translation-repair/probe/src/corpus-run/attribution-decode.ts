import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { ArtifactKeyVocabulary, } from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import type {
  SliceCriticView,
  ProposerView,
} from './attribution-report.ts';

//region Attribution decode
// STRICT decoding of the `sliceCritics` subtree, in deliberate contrast to how
// its ABSENCE is treated.
//
// Absence is data: an artifact settled before attribution existed carries no
// such key, and the report counts those entries separately rather than reading
// them as critics that raised nothing. A key that is PRESENT but malformed is
// not that. Tolerating it would move an artifact into the pre-feature
// population on the strength of corruption, and the eligible-versus-ineligible
// split is the one thing every number in the report rests on.
//
// So every decoder in this module throws rather than dropping. A dropped record produces a
// smaller denominator and a plausible-looking rate; a throw names the artifact
// and the path.

/**
 Reads a value that must be a non-negative safe integer.

 `typeof value === 'number'` is not enough: it admits negatives, fractions,
 and `Infinity`, which `JSON.parse` produces from `1e400`. Each of those would
 travel into a count and out again as a rate.

 @param value - parsed value

 @param path - dotted path for the failure message

 @param minimum - smallest acceptable value

 @returns Validated integer

 @throws ArtifactParseError When not an integer at or above minimum

 @example
 ```ts
 const sliceIndex = readCount({ value, path: 'Kitten sliceCritics[0].sliceIndex', minimum: 0, },);
 ```
 */
export function readCount(
  {
    value,
    path,
    minimum,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly minimum: number;
  },
): number {
  if (((typeof value) !== 'number') || (!Number.isSafeInteger(value,))
    || (value < minimum)) {
    throw new ArtifactParseError({
      path,
      reason: `a safe integer of at least ${String(minimum,)}`,
    },);
  }
  return value;
}

/**
 Reads a value that must be an array of distinct strings.

 Distinctness is checked rather than assumed. `heardCriticIds` is a SET
 written as an array, and a repeated member would count one critic twice on
 one chunk, inflating the denominator every rate divides by.

 @param value - parsed value

 @param path - dotted path for the failure message

 @returns Validated strings

 @throws ArtifactParseError When not an array of distinct strings

 @example
 ```ts
 const heard = readDistinctStrings({ value, path: 'Kitten sliceCritics[0].heardCriticIds', },);
 ```
 */
export function readDistinctStrings(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): readonly string[] {
  if (!isJsonArray(value,))
    throw new ArtifactParseError({
      path,
      reason: 'an array',
    },);

  /**
   Members, each of which must be a string.
   */
  const members = value.map(function toMember(
    member,
    index,
  ): string {
    if ((typeof member) !== 'string')
      throw new ArtifactParseError({
        path: `${path}[${String(index,)}]`,
        reason: 'a string',
      },);
    return member;
  },);

  if ((new Set(members,)).size !== members.length)
    throw new ArtifactParseError({
      path,
      reason: 'distinct members, since it is a set',
    },);

  return members;
}

/**
 Decodes the proposers of one attribution.

 @param value - parsed proposers value

 @param path - dotted path for the failure message

 @param heard - critics this slice recorded as heard, so a proposer naming
 anyone else is refused rather than silently credited

 @returns Validated proposers

 @throws ArtifactParseError When malformed, naming one critic twice, or naming
 a proposer this slice did not record as heard

 @example
 ```ts
 const proposers = decodeProposers({
   value,
   path: 'Kitten sliceCritics[0].claimAttributions[0].proposers',
   heard: new Set(['hf:openai/gpt-oss-120b',]),
 },);
 ```
 */
export function decodeProposers(
  {
    value,
    path,
    heard,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly heard: ReadonlySet<string>;
  },
): readonly ProposerView[] {
  if (!isJsonArray(value,))
    throw new ArtifactParseError({
      path,
      reason: 'an array',
    },);

  /**
   One entry per critic that proposed the claim.
   */
  const proposers = value.map(function toProposer(
    entry,
    index,
  ): ProposerView {
    /**
     Path of this proposer, for any failure below it.
     */
    const here = `${path}[${String(index,)}]`;
    if (!isJsonRecord(entry,))
      throw new ArtifactParseError({
        path: here,
        reason: 'a record',
      },);

    /**
     Critic that proposed the claim.
     */
    const { modelId, } = entry;
    if ((typeof modelId) !== 'string')
      throw new ArtifactParseError({
        path: `${here}.modelId`,
        reason: 'a string',
      },);

    // A PROPOSER MUST BE A HEARD CRITIC. `heardCriticIds` is this slice's own
    // roster of who answered at all, and a claim attributed to anyone outside
    // it did not come from a critic this record says was there to raise it;
    // crediting it anyway would manufacture support no critic actually gave.
    if (!heard.has(modelId,))
      throw new ArtifactParseError({
        path: `${here}.modelId`,
        reason: 'a critic named in heardCriticIds, since only heard critics raise claims',
      },);

    return {
      modelId,
      // At least one: a proposer that emitted the claim zero times is not a
      // proposer, and recording one would credit a critic that stayed silent.
      emissionCount: readCount({
        value: entry.emissionCount,
        path: `${here}.emissionCount`,
        minimum: 1,
      },),
    };
  },);

  if ((new Set(proposers.map(function toId(proposer,) {
    return proposer.modelId;
  },),)).size !== proposers.length) {
    throw new ArtifactParseError({
      path,
      reason: 'one entry per critic, since a repeat would double that critic\'s raised count',
    },);
  }

  return proposers;
}

/**
 Decodes one chunk's calibration record.

 `heardCriticIds` is decoded before `claimAttributions`, because each claim's
 proposers are checked against the heard set.

 @param value - parsed record

 @param path - dotted path for the failure message

 @param indexKey - key this artifact's own generation spelled the record's
 index under: generations 1 to 3 wrote `chunkIndex`, generation 4 onward
 `sliceIndex`, and a record carrying the other spelling is refused rather than
 read under a name its writer never used

 @returns Validated chunk view

 @throws ArtifactParseError When malformed, repeating a claim id, or naming a
 proposer this chunk did not record as heard

 @example
 ```ts
 const view = decodeChunkRecord({ value, path: 'Kitten sliceCritics[0]', indexKey: 'sliceIndex', },);
 ```
 */
export function decodeChunkRecord(
  {
    value,
    path,
    indexKey,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly indexKey: string;
  },
): SliceCriticView {
  if (!isJsonRecord(value,))
    throw new ArtifactParseError({
      path,
      reason: 'a record',
    },);

  /**
   Chunk position within the document, under its generation's spelling.
   */
  const sliceIndex = readCount({
    value: value[indexKey],
    path: `${path}.${indexKey}`,
    minimum: 0,
  },);

  /**
   Critics that answered on this chunk, read before `claimAttributions` so
   each proposer there can be checked against it.
   */
  const heardCriticIds = readDistinctStrings({
    value: value.heardCriticIds,
    path: `${path}.heardCriticIds`,
  },);

  /**
   Heard critics as a set, for the membership check every proposer of this
   chunk passes through.
   */
  const heard = new Set(heardCriticIds,);

  /**
   Recorded attributions of this chunk.
   */
  const rawAttributions = value.claimAttributions;
  if (!isJsonArray(rawAttributions,))
    throw new ArtifactParseError({
      path: `${path}.claimAttributions`,
      reason: 'an array',
    },);

  /**
   One entry per claim that survived screening on this chunk.
   */
  const claimAttributions = rawAttributions.map(function toAttribution(
    entry,
    index,
  ): SliceCriticView['claimAttributions'][number] {
    /**
     Path of this attribution.
     */
    const here = `${path}.claimAttributions[${String(index,)}]`;
    if (!isJsonRecord(entry,))
      throw new ArtifactParseError({
        path: here,
        reason: 'a record',
      },);

    /**
     Deterministic identity of the attributed claim.
     */
    const { claimId, } = entry;
    if ((typeof claimId) !== 'string')
      throw new ArtifactParseError({
        path: `${here}.claimId`,
        reason: 'a string',
      },);

    return {
      claimId,
      proposers: decodeProposers({
        value: entry.proposers,
        path: `${here}.proposers`,
        heard,
      },),
    };
  },);

  if ((new Set(claimAttributions.map(function toId(attribution,) {
    return attribution.claimId;
  },),)).size !== claimAttributions.length) {
    throw new ArtifactParseError({
      path: `${path}.claimAttributions`,
      reason: 'one entry per claim, since the writer keys them by claim id',
    },);
  }

  return {
    sliceIndex,
    heardCriticIds,
    claimAttributions,
  };
}

/**
 Decodes an artifact's whole critic-record array.

 @param value - parsed array

 @param entryId - artifact identity, so a failure names the file

 @param keys - spelling this artifact's own generation wrote, for both the
 array and each record's index, so a refusal names something a reader can
 find in the file rather than the name this package happens to use for it

 @returns Validated chunk views

 @throws ArtifactParseError When malformed or repeating a chunk index

 @example
 ```ts
 const sliceCritics = decodeSliceCritics({ value, entryId: 'Kitten', keys, },);
 ```
 */
export function decodeSliceCritics(
  {
    value,
    entryId,
    keys,
  }: {
    readonly value: unknown;
    readonly entryId: string;
    readonly keys: ArtifactKeyVocabulary;
  },
): readonly SliceCriticView[] {
  /**
   Key the array sits under in this generation.
   */
  const criticsKey = keys.sliceCritics;

  if (!isJsonArray(value,)) {
    throw new ArtifactParseError({
      path: `${entryId} ${criticsKey}`,
      reason: 'an array when present at all, since only an ABSENT key means the entry predates attribution',
    },);
  }

  /**
   One record per chunk of the document.
   */
  const records = value.map(function toRecord(
    record,
    index,
  ) {
    return decodeChunkRecord({
      value: record,
      path: `${entryId} ${criticsKey}[${String(index,)}]`,
      indexKey: keys.sliceIndex,
    },);
  },);

  if ((new Set(records.map(function toIndex(record,) {
    return record.sliceIndex;
  },),)).size !== records.length) {
    throw new ArtifactParseError({
      path: `${entryId} ${criticsKey}`,
      reason: 'one record per chunk, since a repeat inflates the chunk count every rate divides by',
    },);
  }

  return records;
}

//endregion Attribution decode
