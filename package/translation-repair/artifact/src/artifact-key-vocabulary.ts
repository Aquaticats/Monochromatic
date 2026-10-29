import {
  ARTIFACT_SCHEMA_VERSION_V2,
  ARTIFACT_SCHEMA_VERSION_V3,
  ARTIFACT_SCHEMA_VERSION_V4,
  ARTIFACT_SCHEMA_VERSION_V5,
  ARTIFACT_SCHEMA_VERSION_V6,
  ARTIFACT_SCHEMA_VERSION_V7,
  ARTIFACT_SCHEMA_VERSION_V8,
  ARTIFACT_SCHEMA_VERSION_V9,
  ARTIFACT_SCHEMA_VERSION_V10,
  ARTIFACT_SCHEMA_VERSION_V11,
  ARTIFACT_SCHEMA_VERSION_V12,
  ARTIFACT_SCHEMA_VERSION_V13,
  ARTIFACT_SCHEMA_VERSION_V14,
  ARTIFACT_SCHEMA_VERSION_V15,
  ARTIFACT_SCHEMA_VERSION_V16,
  ARTIFACT_SCHEMA_VERSION_V17,
  ARTIFACT_SCHEMA_VERSION_V18,
  ARTIFACT_SCHEMA_VERSION_V19,
  ARTIFACT_SCHEMA_VERSION_V20,
  ARTIFACT_SCHEMA_VERSION_V21,
  ARTIFACT_SCHEMA_VERSION_V22,
} from './corpus-run/artifact-two-lane-contract.ts';
import { ARTIFACT_SCHEMA_VERSION_V1, } from './artifact-schema-version.ts';

//region Artifact key vocabulary
// Which spelling of four keys a settled artifact uses, chosen by the
// generation the file records rather than guessed from what it contains.
//
// Generations 1 and 2 spelled these keys with `chunk`, in the same records that
// already spelled `sliceCount` and `withdrawnSliceCount` with `slice`, about
// the same things. Generation 4 onward spells all of them `slice`. Nothing else about
// the shape moved across any of it, which is why one table of four names covers
// the whole difference and there is no second reader family.
//
// GENERATION 3 IS A MIXTURE and needs its own row rather than a style name: it
// spells the three arrays the way generation 4 does and the per-slice index the
// way generation 2 did. The two moved separately because `sliceIndex` was
// already taken by a different concept, the POSITION in `prepared.slices`, and
// had to be freed before the stamped index could take the name.
//
// NO ARTIFACT IS EVER READ UNDER TWO SPELLINGS. The recorded version selects one
// table, and a file whose keys disagree with the table its own version selected
// fails to parse. That is the right answer rather than a harshness to soften: a
// version 3 file carrying `shippedChunkIndices` is not a version 3 file.
//
// THE TABLE IS TOTAL OVER GENERATIONS THIS PACKAGE KNOWS, and refuses anything
// else by name. `KNOWN_ARTIFACT_SCHEMA_VERSIONS` and this table can drift apart
// in exactly one direction, a version added there and forgotten here, and the
// refusal is what turns that into a stopped read instead of a key silently read
// under the wrong generation's spelling.

/**
 Keys whose spelling moved between generations, under the names the rest of
 this package uses for them.

 SPELLED OUT AS FOUR FIELDS rather than carried as a map from new name to old
 name, so a reader that names a key this table does not cover fails to compile
 instead of reading `undefined` off a lookup and asking the artifact for a key
 called `undefined`.

 THE FIELDS ARE PLAIN STRINGS rather than the literals themselves, because
 every use here is a lookup or a path fragment. A WRITER cannot take its keys
 from this table for that reason: `isolatedDeclarations` requires the
 annotation, the annotation widens the literals, and a computed key of type
 `string` builds an object with an index signature instead of named fields.
 The keys are written out where the results are made: `repair-result.ts` declares `changedSliceIndices`,
 `withdrawnSliceIndices` and `sliceCritics`, `slice-delivery.ts` declares `sliceIndex` and passes `changedSliceIndices`, and
 `translate-assemble.ts` and `repair-assemble.ts` write all of them between them.

 @example
 ```ts
 const keys: ArtifactKeyVocabulary = SLICE_SPELLED_KEYS;
 ```
 */
export type ArtifactKeyVocabulary = {
  /**
   Key naming slices whose returned text differs from the archive.
   */
  readonly changedSliceIndices: string;

  /**
   Key naming slices whose change the assembly guard took back.
   */
  readonly withdrawnSliceIndices: string;

  /**
   Key naming which critics were heard at each slice.
   */
  readonly sliceCritics: string;

  /**
   Key naming which slice a per-slice record belongs to.
   */
  readonly sliceIndex: string;
};

/**
 Spelling generations 1 and 2 wrote.

 @example
 ```ts
 const value = record[CHUNK_SPELLED_KEYS.changedSliceIndices];
 ```
 */
export const CHUNK_SPELLED_KEYS: ArtifactKeyVocabulary = {
  changedSliceIndices: 'shippedChunkIndices',
  withdrawnSliceIndices: 'withdrawnChunkIndices',
  sliceCritics: 'chunkCritics',
  sliceIndex: 'chunkIndex',
};

/**
 Spelling generation 4 onward writes, which is the one the code uses throughout.

 @example
 ```ts
 const value = record[SLICE_SPELLED_KEYS.changedSliceIndices];
 ```
 */
export const SLICE_SPELLED_KEYS: ArtifactKeyVocabulary = {
  changedSliceIndices: 'changedSliceIndices',
  withdrawnSliceIndices: 'withdrawnSliceIndices',
  sliceCritics: 'sliceCritics',
  sliceIndex: 'sliceIndex',
};

/**
 Spelling generation 3 wrote, which is neither table whole.

 WRITTEN AS THE DIFFERENCE rather than spelled out again, so it cannot drift
 from the two tables it sits between: it is generation 4 except for the index,
 which had not moved yet.
 */
const GENERATION_3_KEYS: ArtifactKeyVocabulary = {
  ...SLICE_SPELLED_KEYS,
  sliceIndex: CHUNK_SPELLED_KEYS.sliceIndex,
};

/**
 Which spelling each generation wrote.
 */
const KEYS_BY_GENERATION: Readonly<Record<number, ArtifactKeyVocabulary>> = {
  [ARTIFACT_SCHEMA_VERSION_V1]: CHUNK_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V2]: CHUNK_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V3]: GENERATION_3_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V4]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V5]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V6]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V7]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V8]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V9]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V10]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V11]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V12]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V13]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V14]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V15]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V16]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V17]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V18]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V19]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V20]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V21]: SLICE_SPELLED_KEYS,
  [ARTIFACT_SCHEMA_VERSION_V22]: SLICE_SPELLED_KEYS,
};

/**
 Generation named by an artifact that this table does not cover.

 @example
 ```ts
 throw new UnknownArtifactGenerationError({ version: 9, },);
 ```
 */
export class UnknownArtifactGenerationError extends Error {
  /**
   Declares this message safe to forward: it names a schema version.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names the generation and says what its absence means for the read.

   @param version - generation an artifact recorded
   */
  public constructor({ version, }: { readonly version: number; },) {
    super(
      `no key spelling is recorded for artifact generation ${
        String(version,)
      }, so which name each of its renamed keys carries is unknown and reading `
        + 'one under another generation\'s spelling would report a key as absent rather than as unread',
    );
    this.name = 'UnknownArtifactGenerationError';
  }
}

/**
 Selects the spelling a generation wrote.

 @param version - generation an artifact records

 @returns Spelling that generation used for all four keys

 @throws {@link UnknownArtifactGenerationError} when no spelling is recorded
 for that generation

 @example
 ```ts
 const keys = keyVocabularyOf({ version: reading.version, },);
 ```
 */
export function keyVocabularyOf(
  { version, }: { readonly version: number; },
): ArtifactKeyVocabulary {
  /**
   Spelling recorded for this generation, absent where none is.
   */
  const keys = KEYS_BY_GENERATION[version];
  if (keys === undefined)
    throw new UnknownArtifactGenerationError({ version, },);

  return keys;
}

//endregion Artifact key vocabulary
