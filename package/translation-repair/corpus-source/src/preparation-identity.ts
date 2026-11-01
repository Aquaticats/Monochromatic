import { createHash, } from 'node:crypto';

import { isLowerHexDigit, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { PreparedDocumentPair, } from './prepared-document-pair.ts';

//region Preparation identity
// WHICH SLICING a result describes, as against which entry it names.
//
// Two lanes are compared slice by slice, joined on a global index. Equal slice
// counts, equal indices and equal incumbent text do not prove the two ran over
// one preparation: the same wording can cover different source passages, and
// every insertion anchor's incumbent is the empty string, so a document with
// several untranslated sections offers several rows that look identical. A
// comparison joining two slicings is undetectable afterwards, because every row
// it produces is individually well formed.
//
// So a preparation names itself, once, and everything derived from it records
// that name.
//
// WHAT IT COVERS is the preparation and nothing else: both documents, every
// slice's placement, offsets and exact text, the pairing between the two sides,
// the line-structured flag, and the identity context. What it deliberately
// EXCLUDES is everything about the run: the commit, the pipeline digest, the
// rosters, the call configuration, timings, cache state, and any lane's output.
// A resumed run over the same slicing must produce the same identity, or the
// field answers "was this the same attempt" rather than the question it is for.
//
// The target's PLACEMENT KIND is the field that earns this. Without it a
// content slice that happens to be blank and a place the archive never
// translated hash identically, which is the pair every other guard here exists
// to separate.

/**
 Hash behind the identity.
 */
const DIGEST_ALGORITHM = 'sha256';

/**
 Name of the scheme a recorded identity was produced by.

 Carried in the value rather than assumed, so changing what is hashed or how
 it is framed makes a DIFFERENT string rather than a same-looking one. Without
 it, a later scheme would silently make two incomparable values comparable.
 */
const LEGACY_IDENTITY_FORMAT = 'sha256-preparation-v1';

/**
 Metadata-aware preparation identity scheme.
 */
const IDENTITY_FORMAT = 'sha256-preparation-v2';

/**
 Every identity scheme this reader understands.
 */
const IDENTITY_FORMATS: readonly string[] = [
  LEGACY_IDENTITY_FORMAT,
  IDENTITY_FORMAT,
];

/**
 Character between the scheme name and the hex, chosen because neither side
 can contain it.
 */
const FORMAT_SEPARATOR = ':';

/**
 Identity of one slicing of one document pair.

 Branded so it cannot be assigned where a pipeline digest or a git object id
 belongs. All three are 64 hex characters behind a label and they answer
 different questions: this one names WHAT WAS SLICED, not what ran or when.

 @example
 ```ts
 const identity: PreparationIdentity = preparationIdentity({ prepared, },);
 ```
 */
export type PreparationIdentity = string & { readonly __brand: 'PreparationIdentity'; };

/**
 Characters of a sha256 hex digest.
 */
const DIGEST_LENGTH = 64;

/**
 What a recorded identity looks like, in words, for every message refusing
 one.

 BUILT FROM THE CONSTANTS the check reads, so a new scheme or length changes
 the sentence with it, and a reader refusing an identity can say what it
 expected without repeating what it found (ledger B34).
 */
export const PREPARATION_IDENTITY_SHAPE: string = `${
  IDENTITY_FORMATS
    .map(function prefixOf(format,): string {
      return JSON.stringify(`${format}${FORMAT_SEPARATOR}`,);
    },)
    .join(' or ',)
} followed by ${String(DIGEST_LENGTH,)} lowercase hex characters`;

/**
 Whether a string could be an identity this module produced.

 Scanned rather than matched with a pattern: after the scheme name the rule is
 one predicate per character over a fixed-length string, a linear pass that
 cannot backtrack.

 @param value - string claiming to be an identity

 @returns Whether it is a scheme name this reader understands, the separator,
 and 64 lowercase hex characters

 @example
 ```ts
 const usable = isPreparationIdentityShaped({ value: recorded, },);
 ```
 */
export function isPreparationIdentityShaped(
  { value, }: { readonly value: string; },
): boolean {
  /**
   Scheme this recorded identity declares.
   */
  const format = IDENTITY_FORMATS.find(function matches(candidate,): boolean {
    return value.startsWith(`${candidate}${FORMAT_SEPARATOR}`,);
  },);
  if (format === undefined)
    return false;

  /**
   Hex half, once the scheme name is off.
   */
  const hex = value.slice(`${format}${FORMAT_SEPARATOR}`.length,);
  if (hex.length !== DIGEST_LENGTH)
    return false;
  for (const character of hex) {
    // Upper case is refused because this module only ever emits lower case,
    // so another spelling came from elsewhere and would name a second slicing.
    if (!isLowerHexDigit({ character, },))
      return false;
  }
  return true;
}

/**
 Raised when a recorded identity is not one this scheme could have produced.

 NOT MARKED to forward: its sentence quotes the value it refused, which is a
 recorded identity string rather than document text, and a reader refusing a
 stored identity writes its own reason instead (ledger B34).

 @example
 ```ts
 throw new PreparationIdentityError({ value: 'whiskers', },);
 ```
 */
export class PreparationIdentityError extends Error {
  /**
   Builds the refusal naming the shape an identity has and the value refused.

   @param value - string that claimed to be an identity

   @example
   ```ts
   throw new PreparationIdentityError({ value: 'whiskers', },);
   ```
   */
  constructor({ value, }: { readonly value: string; },) {
    super(`A preparation identity is ${PREPARATION_IDENTITY_SHAPE}; received ${JSON.stringify(value,)}.`,);
    this.name = 'PreparationIdentityError';
  }
}

/**
 Narrows a recorded string to an identity, or refuses it.

 The brand is built THROUGH this rather than asserted at the construction
 site, so a value read back from an artifact passes exactly the check a fresh
 one does, and a hand-written or truncated string cannot become an identity by
 assertion alone.

 @param value - string claiming to be an identity

 @returns Nothing; it narrows `value` in the caller on success

 @throws {@link PreparationIdentityError} when it is not a scheme name this
 reader understands, the separator, and 64 lowercase hex characters

 @example
 ```ts
 assertPreparationIdentity(recorded,);
 ```
 */
export function assertPreparationIdentity(
  value: string,
): asserts value is PreparationIdentity {
  if (!isPreparationIdentityShaped({ value, },))
    throw new PreparationIdentityError({ value, },);
}

/**
 Frames one field so no field's content can forge another's boundary.

 LENGTH PREFIXED rather than separated by a byte assumed absent from the text.
 Slice text is arbitrary document content: it can hold any separator anyone
 might pick, including newlines and null bytes, so a separator scheme would be
 forgeable by a document that contained it. A byte count cannot be forged by
 the bytes it counts.

 @param value - field content

 @returns Byte count, a colon, then the content

 @example
 ```ts
 const framed = framed({ value: 'The cat naps.', },);
 ```
 */
function framed({ value, }: { readonly value: string; },): string {
  /**
   Bytes this field occupies, which is what the hash consumes and therefore
   what the count has to describe.
   */
  const bytes = Buffer.byteLength(
    value,
    'utf8',
  );
  return `${String(bytes,)}:${value}`;
}

/**
 Frames a number, so a count and a string cannot collide.

 @param value - number to frame

 @returns Framed decimal form

 @example
 ```ts
 const framedIndex = framedNumber({ value: 3, },);
 ```
 */
function framedNumber({ value, }: { readonly value: number; },): string {
  return framed({ value: String(value,), },);
}

/**
 Canonical form of one prepared slice.

 BOTH SIDES IN ONE ROW, which is what records the pairing: two preparations
 that produced the same passages and paired them differently have the same
 fields in a different order, and hashing rows rather than two lists is what
 makes that a different identity.

 @param slice - prepared pair

 @param lineStructured - whether this slice is governed line by line, which
 changes what every stage is allowed to do to it

 @returns Framed fields of this slice, in fixed order

 @example
 ```ts
 const row = sliceRow({ slice, lineStructured: false, },);
 ```
 */
function sliceRow(
  {
    slice,
    lineStructured,
  }: {
    readonly slice: ChunkPair;
    readonly lineStructured: boolean;
  },
): string {
  return [
    framedNumber({ value: slice.target
      .sliceIndex, },),
    ...((slice.syntax === undefined)
      ? []
      : [framed({ value: `syntax:${slice.syntax}`, },),]),
    // BOTH INDICES, though they are equal today. The pairing is what this row
    // exists to record, and one-sided slicing touches exactly the
    // assumption that one number names both sides. Adding it now costs nothing;
    // after the first artifact is written it would cost a scheme version.
    framedNumber({ value: slice.source
      .sliceIndex, },),
    // The source side is always existing content; its kind is framed anyway, so
    // a later one-sided slicing cannot change the meaning of a row without
    // changing its bytes.
    framed({ value: 'content', },),
    framedNumber({ value: slice.source
      .startOffset, },),
    framedNumber({ value: slice.source
      .endOffset, },),
    framed({ value: slice.source
      .text, },),
    framed({ value: isInsertionChunk(slice.target,) ? 'insertion' : 'content', },),
    framedNumber({ value: slice.target
      .startOffset, },),
    framedNumber({ value: slice.target
      .endOffset, },),
    framed({ value: slice.target
      .text, },),
    framed({ value: lineStructured ? 'line-structured' : 'free', },),
  ].join('',);
}

/**
 Names the slicing a prepared pair represents.

 @param prepared - preparation to name, read as it stands rather than from any
 record derived from it

 @returns Identity of this slicing, stable across runs and resumptions

 @example
 ```ts
 const identity = preparationIdentity({ prepared, },);
 ```
 */
export function preparationIdentity(
  { prepared, }: { readonly prepared: PreparedDocumentPair; },
): PreparationIdentity {
  /**
   Identity scheme selected by preparation generation.
   */
  const format = prepared.legacyIdentity === true
    ? LEGACY_IDENTITY_FORMAT
    : IDENTITY_FORMAT;
  /**
   Whole preparation payload, framed field by field in a fixed order under
   the selected identity scheme.

   Both document texts are hashed beside the slices rather than trusted to be
   implied by them: a section neither side sliced appears in no row, so two
   preparations differing only outside every slice would otherwise agree.
   */
  const payload = [
    framed({ value: format, },),
    framed({ value: prepared.sourceText, },),
    framed({ value: prepared.targetText, },),
    framedNumber({ value: prepared.slices
      .length, },),
    ...prepared.slices
      .map(function toRow(slice,): string {
        return sliceRow({
          slice,
          lineStructured: prepared.lineStructuredSliceIndices
            .has(slice.target
              .sliceIndex,),
        },);
      },),
    // STATED EITHER WAY, because absent context and empty context are different
    // preparations: one asks the models about a document with no declared
    // names, the other about one whose declared names are nothing.
    framed({ value: (prepared.identityContext === undefined) ? 'no-identity-context' : 'identity-context', },),
    framed({ value: prepared.identityContext ?? '', },),
  ].join('',);
  /**
   Recorded value, naming the scheme that produced it.
   */
  const identity = `${format}${FORMAT_SEPARATOR}${
    createHash(DIGEST_ALGORITHM,)
      .update(
        payload,
        'utf8',
      )
      .digest('hex',)
  }`;
  assertPreparationIdentity(identity,);
  return identity;
}

//endregion Preparation identity
