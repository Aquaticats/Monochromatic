import {
  AssemblyContractError,
  checkedChangeSets,
  type OrderedChangeSets,
  orderedChangeSets,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import {
  ArtifactParseError,
  requireArray,
  requireCount,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  readArtifactSchemaVersion,
  ARTIFACT_SCHEMA_VERSION_V1,
} from './artifact-schema-version.ts';
import { CHUNK_SPELLED_KEYS, } from './artifact-key-vocabulary.ts';

//region Artifact change sets
// Which slices a settled run's document carried a change for, read back out of
// the artifact with absence kept absent.
//
// The one rule this file exists for: an artifact that never recorded these sets
// must not read as a run that changed nothing. Both are an empty array once a
// reader defaults them, and the difference between "we did not write this down"
// and "this run shipped no repair" is the difference between an unknown and a
// measurement.
//
// EVERY KEY HERE IS SPELLED THE WAY GENERATION 1 SPELLED IT, taken from
// `CHUNK_SPELLED_KEYS` rather than written out, and it stays that way. This
// generation is closed: production stopped writing it, and re-spelling the keys
// of a generation nobody writes any more would stop the files that generation
// left behind from being read at all, in exchange for nothing.

/**
 What a settled artifact says about the slices its document changed.

 THREE KINDS BECAUSE THERE ARE THREE ANSWERS, and folding any two of them
 together is exactly the defect this replaces. The sets can be absent, present
 with the count they are out of, or present without it: an artifact written on
 2026-08-15 carries both index arrays and no `sliceCount`, so nothing can
 range-check them, while everything else about them is still checkable.

 @example
 ```ts
 const sets: ArtifactChangeSets = { kind: 'unrecorded', };
 ```
 */
export type ArtifactChangeSets = {
  /**
   Run predates index recording, so which slices it changed is unknown. NOT
   the same as a run that changed nothing, and readers must not treat it so.
   */
  readonly kind: 'unrecorded';
} | {
  /**
   Both sets are recorded, and no `sliceCount` bounds them.
   */
  readonly kind: 'uncounted';

  /**
   Slices the document carries a change for, ascending.
   */
  readonly shipped: readonly number[];

  /**
   Slices whose change the assembly guard took back, ascending.
   */
  readonly withdrawn: readonly number[];
} | {
  /**
   Both sets are recorded, bounded by the slices the preparation produced.
   */
  readonly kind: 'counted';

  /**
   Slices the preparation produced, which both sets are out of.
   */
  readonly sliceCount: number;

  /**
   Slices the document carries a change for, ascending.
   */
  readonly shipped: readonly number[];

  /**
   Slices whose change the assembly guard took back, ascending.
   */
  readonly withdrawn: readonly number[];
};

/**
 Reads one index array, requiring every entry to be a slice index.

 @param value - array as parsed from JSON

 @param path - dotted path for error messages

 @returns Every index, in the order the artifact recorded them

 @throws {@link ArtifactParseError} when the value is not an array of
 non-negative whole numbers

 @example
 ```ts
 const shipped = readIndexArray({ value: artifact[shippedKey], path: `${id}.${shippedKey}`, },);
 ```
 */
function readIndexArray(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): readonly number[] {
  return requireArray({
    value,
    path,
  },)
    .map(function readIndexAt(
      entry,
      entryIndex,
    ): number {
      return requireCount({
        value: entry,
        path: `${path}[${String(entryIndex,)}]`,
      },);
    },);
}

/**
 Reads both index arrays and applies the lane's own index rules to them.

 REPORTED AS AN ARTIFACT DEFECT rather than passed through as the assembly
 contract error it arrives as. A repeat, an overlap or an out-of-range index
 is a broken contract wherever it is found, but a reader holding a file cannot
 know WHO broke it: the run that wrote it, an edit, a truncation, or a merge
 all look identical from here. So it reports what the artifact contains and
 where, and says nothing about how it got that way. The contract error's own
 message is carried through, since it states the violation better than a
 rewording would, and the entry it belongs to is added, which the assembly
 message has no way to know.

 @param artifact - artifact record, freshly parsed

 @param path - dotted path for error messages, usually the entry id

 @param sliceCount - slices the preparation produced, when the artifact
 recorded it; omitted for a generation that did not, which drops the range
 rule and keeps every other one

 @returns Both sets ascending

 @throws {@link ArtifactParseError} when either array is malformed or the two
 break a rule the lanes hold them to

 @throws Error when the set checks raise anything but their own refusal, a
 fault of this build rather than of the file

 @example
 ```ts
 const sets = readCheckedSets({ artifact, path, sliceCount, },);
 ```
 */
function readCheckedSets(
  {
    artifact,
    path,
    sliceCount,
  }: {
    readonly artifact: Readonly<Record<string, unknown>>;
    readonly path: string;
    readonly sliceCount?: number;
  },
): OrderedChangeSets {
  /**
   Slices the artifact says its document carries a change for.
   */
  const shipped = readIndexArray({
    value: artifact[CHUNK_SPELLED_KEYS.changedSliceIndices],
    path: `${path}.${CHUNK_SPELLED_KEYS.changedSliceIndices}`,
  },);

  /**
   Slices the artifact says the guard took back.
   */
  const withdrawn = readIndexArray({
    value: artifact[CHUNK_SPELLED_KEYS.withdrawnSliceIndices],
    path: `${path}.${CHUNK_SPELLED_KEYS.withdrawnSliceIndices}`,
  },);
  try {
    if (sliceCount === undefined) {
      return checkedChangeSets({
        shipped,
        withdrawn,
      },);
    }
    return orderedChangeSets({
      sliceCount,
      shipped,
      withdrawn,
    },);
  }
  catch (error) {
    // The index arrays reach here as safe non-negative integers
    // (`readIndexArray` through `requireCount`), over which `checkedChangeSets`
    // and `orderedChangeSets` raise only `AssemblyContractError`: every other
    // statement in both iterates, builds a `Set`, filters or compares (ledger
    // T8).
    if (!(error instanceof AssemblyContractError)) {
      throw new Error(
        'unreachable: checkedChangeSets or orderedChangeSets raised something other than AssemblyContractError',
        { cause: error, },
      );
    }
    throw new ArtifactParseError({
      path: `${path} index sets`,
      reason: `sets one document could carry (${error.message})`,
    },);
  }
}

/**
 Reads both index sets out of a settled artifact.

 DISPATCHES ON THE VERSION, then on presence, and refuses the shapes no writer
 has ever produced. Exactly one of the two arrays is the important refusal:
 every generation wrote both or neither, so one alone means an artifact was
 edited, merged, or truncated, and reading its lone array would report a
 shipped set with no withdrawals or the reverse as though a run had said so.

 A VERSIONED ARTIFACT MUST CARRY BOTH, plus the `sliceCount` they are out of.
 That is what the version buys: presence stops being a question, so a missing
 field is a defect rather than a generation.

 AND THE COUNT WITHOUT A VERSION IS REFUSED, which reads like pedantry until
 you ask what produces it. No writer ever did. What does is a CURRENT artifact
 that lost its version field to an edit or a merge, and accepting it as an
 older generation would discard a denominator the run actually recorded.

 @param artifact - artifact record, freshly parsed

 @param path - dotted path for error messages, usually the entry id

 @returns Both sets with their generation named

 @throws {@link ArtifactParseError} when one index array is present without
 the other, when a versioned artifact omits either of them or `sliceCount`,
 when an unversioned one carries `sliceCount`, when an index is not a
 non-negative whole number, or when the two sets break a rule the writing
 lanes hold them to

 @example
 ```ts
 const sets = readArtifactChangeSets({ artifact, path: id, },);
 ```
 */
export function readArtifactChangeSets(
  {
    artifact,
    path,
  }: {
    readonly artifact: Readonly<Record<string, unknown>>;
    readonly path: string;
  },
): ArtifactChangeSets {
  /**
   Generation the artifact names, or a named absence.
   */
  const reading = readArtifactSchemaVersion({
    artifact,
    path,
  },);

  /**
   Whether the shipped set was written at all, by key rather than by value, so
   an explicit JSON `null` counts as present and is refused by the index-set read rather than
   passing for an artifact that predates the field.
   */
  const hasShipped = Object.hasOwn(
    artifact,
    CHUNK_SPELLED_KEYS.changedSliceIndices,
  );

  /**
   Whether the withdrawn set was written at all.
   */
  const hasWithdrawn = Object.hasOwn(
    artifact,
    CHUNK_SPELLED_KEYS.withdrawnSliceIndices,
  );
  if (hasShipped !== hasWithdrawn)
    throw new ArtifactParseError({
      path: `${path}.${
        hasShipped ? CHUNK_SPELLED_KEYS.withdrawnSliceIndices : CHUNK_SPELLED_KEYS.changedSliceIndices
      }`,
      reason: 'both index sets or neither, since every generation wrote them together',
    },);

  if (reading.kind === 'unversioned') {
    // THE COUNT ARRIVED WITH THE VERSION, so an artifact carrying one without
    // the other is a shape no writer produced. The case that matters is not a
    // hand-written file: it is a CURRENT artifact whose version field was lost
    // to an edit or a merge, which would otherwise read as a generation that
    // predates the count and throw away a denominator the run recorded.
    if (
      Object.hasOwn(
        artifact,
        'sliceCount',
      )
    ) {
      throw new ArtifactParseError({
        path: `${path}.artifactSchemaVersion`,
        reason: 'a schema version, since this artifact records the slice count that arrived with one',
      },);
    }
    if (!hasShipped)
      return { kind: 'unrecorded', };
    return {
      kind: 'uncounted',
      ...readCheckedSets({
        artifact,
        path,
      },),
    };
  }

  // VERSION 2 IS REFUSED HERE rather than read, and refused by this reader
  // rather than by the version dispatcher: the dispatcher's job is to say which
  // generation an artifact belongs to, and this function's job is to answer
  // with ONE change set. A two-lane artifact has two ledgers and no singular
  // anything, so there is no answer to give and the honest move is to say so.
  // Reading its absent top-level sets as `unrecorded` would report a run that
  // changed nothing.
  if (reading.version !== ARTIFACT_SCHEMA_VERSION_V1) {
    throw new ArtifactParseError({
      path: `${path}.artifactSchemaVersion`,
      reason: `schema version ${
        String(ARTIFACT_SCHEMA_VERSION_V1,)
      } or an unversioned artifact, since this reading answers with one change set `
        + 'and a later generation records one per lane',
    },);
  }

  if (!hasShipped)
    throw new ArtifactParseError({
      path: `${path}.${CHUNK_SPELLED_KEYS.changedSliceIndices}`,
      reason: `index sets, which schema version ${String(reading.version,)} records for every run`,
    },);

  /**
   Slices the preparation produced, which bounds both sets.
   */
  const sliceCount = requireCount({
    value: artifact.sliceCount,
    path: `${path}.sliceCount`,
  },);

  return {
    kind: 'counted',
    sliceCount,
    ...readCheckedSets({
      artifact,
      path,
      sliceCount,
    },),
  };
}

//endregion Artifact change sets
