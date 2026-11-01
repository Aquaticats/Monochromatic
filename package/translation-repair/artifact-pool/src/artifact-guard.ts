import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Artifact guards
// The shape checks every artifact reader shares. They live apart from
// `artifact-read.ts` because the repair reader needs the same primitives, and
// having it reach back into the issue reader for them would tie two independent
// parsers together through their least interesting parts.
//
// Every one of these THROWS rather than returning a fallback. That is the whole
// doctrine of this layer: these parsers feed a precision measurement, and a
// silently skipped accepted issue shrinks the denominator without leaving a
// trace anyone would notice.

/**
 Thrown when an artifact, or a value within it, is structurally malformed.
 Aborting loudly is deliberate: a skipped accepted issue would bias the
 precision denominator without a trace.
 */
export class ArtifactParseError extends Error {
  /**
   Declares this message safe to forward: it names the artifact path and the shape the value failed to satisfy, and quotes neither the value nor the file.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the malformed path.

   @param path - dotted path to malformed value

   @param reason - what value was expected to be

   @example
   ```ts
   throw new ArtifactParseError({ path: 'Kitten issues[3].issue.status', reason: 'a string', },);
   ```
   */
  public constructor(
    {
      path,
      reason,
    }: {
      readonly path: string;
      readonly reason: string;
    },
  ) {
    super(`artifact parse failed at ${path}: expected ${reason}.`,);
    this.name = 'ArtifactParseError';
  }
}

/**
 The parse refusal a catch around an artifact reader holds, for a catch that
 acts on the refusal alone.

 SHARED RATHER THAN INLINE (ledger T8, ninth batch), as `requireMdxRefusal`
 is for the grammar's refusal: the schema census's catch around the version
 reader tested the class and rethrew anything else, and no test reached the
 rethrow, since the reader raises every refusal as this class. The narrowing
 stands here, where a case reaches the rethrow.

 @param error - what the catch caught

 @returns The refusal

 @throws The caught value unchanged when it is anything but an artifact
 parse refusal, an unexpected state that must keep propagating

 @example
 ```ts
 const refusal = requireArtifactParseRefusal({ error, },);
 ```
 */
export function requireArtifactParseRefusal({ error, }: { readonly error: unknown; },): ArtifactParseError {
  if (error instanceof ArtifactParseError)
    return error;
  throw error;
}

/**
 Reads a required string, throwing when the value is any other shape.

 @param value - value to check

 @param path - dotted path for error message

 @returns Value as a string

 @throws {@link ArtifactParseError} when the value is not a string

 @example
 ```ts
 const id = requireString({ value: artifact.id, path: 'artifact.id', },);
 ```
 */
export function requireString(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): string {
  if ((typeof value) !== 'string')
    throw new ArtifactParseError({
      path,
      reason: 'a string',
    },);
  return value;
}

/**
 Reads a required boolean, throwing when the value is any other shape.

 @param value - value to check

 @param path - dotted path for error message

 @returns Value as a boolean

 @throws {@link ArtifactParseError} when the value is not a boolean

 @example
 ```ts
 const refined = requireBoolean({ value: record.refined, path: 'refined', },);
 ```
 */
export function requireBoolean(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): boolean {
  if ((typeof value) !== 'boolean')
    throw new ArtifactParseError({
      path,
      reason: 'a boolean',
    },);
  return value;
}

/**
 Reads a required record, throwing when the value is any other shape.

 Arrays are refused, as `isJsonRecord` refuses them. Every call site here
 reads a JSON object: a manifest, an item, a region, a claim, an artifact, an
 issue, a tally. Letting an array through would return something whose named
 properties are all `undefined`, so the failure would surface later as a
 confusing complaint about a missing field rather than here as the shape error
 it is. This module refused arrays on its own until `isJsonRecord` did
 (ledger B92).

 @param value - value to check

 @param path - dotted path for error message

 @returns Value as a record

 @throws {@link ArtifactParseError} when the value is not an object, or is an
 array

 @example
 ```ts
 const issue = requireRecord({ value: record.issue, path: 'issue', },);
 ```
 */
export function requireRecord(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): Record<string, unknown> {
  if (!isJsonRecord(value,))
    throw new ArtifactParseError({
      path,
      reason: 'an object',
    },);
  return value;
}

/**
 Reads a required array, throwing when the value is any other shape.

 @param value - value to check

 @param path - dotted path for error message

 @returns Value as an array

 @throws {@link ArtifactParseError} when the value is not an array

 @example
 ```ts
 const spans = requireArray({ value: claim.spans, path: 'claim.spans', },);
 ```
 */
export function requireArray(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): readonly unknown[] {
  if (!isJsonArray(value,))
    throw new ArtifactParseError({
      path,
      reason: 'an array',
    },);
  return value;
}

/**
 Reads a required list of strings, throwing at the first entry that is not
 one and naming its position.

 The polish, polish gate, contest ballot and consolidation-part readers each
 kept their own copy of this (audit area six, 2026-09-28).

 @param value - value to check

 @param path - dotted path for error message; an entry's path is this with
 its position in brackets

 @returns Value as a list of strings

 @throws {@link ArtifactParseError} when the value is not an array, or an
 entry is not a string

 @example
 ```ts
 const findings = requireStringList({ value: record.findings, path: 'polish.findings', },);
 ```
 */
export function requireStringList(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): readonly string[] {
  return requireArray({
    value,
    path,
  },)
    .map(function readOne(
      entry,
      at,
    ): string {
      return requireString({
        value: entry,
        path: `${path}[${String(at,)}]`,
      },);
    },);
}

/**
 Reads a required count, throwing on any other shape.

 Rejects fractions and negatives as well as non-numbers, because every count
 this layer reads is a tally of votes or regions. A fractional one means the
 writer and the reader disagree about what the field holds, and coercing it
 here would carry that disagreement into a measurement unnoticed.

 SAFE integers rather than whole ones, which is a narrower rule than it looks:
 `Number.isInteger` answers true above 2 to the 53rd, where JSON round trips
 and arithmetic stop being exact, so a value that large is not a count anybody
 wrote. No tally or slice index this layer reads can legitimately reach it.

 @param value - value to check

 @param path - dotted path for error message

 @returns Value as a non-negative integer

 @throws {@link ArtifactParseError} when the value is not one

 @example
 ```ts
 const heard = requireCount({ value: probe.heardProbers, path: 'probe.heardProbers', },);
 ```
 */
export function requireCount(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): number {
  if ((typeof value) !== 'number')
    throw new ArtifactParseError({
      path,
      reason: 'a number',
    },);
  if ((!Number.isSafeInteger(value,)) || (value < 0))
    throw new ArtifactParseError({
      path,
      reason: 'a non-negative integer no larger than JSON carries exactly',
    },);
  return value;
}

/**
 Reads a value that must be a finite number, of any sign or scale.

 SEPARATE FROM `requireCount`, which exists for indices and tallies and
 rejects everything below zero and everything fractional. A ballot's weight
 is one half when a judge names its own writing, and a ballot naming no
 candidate records a sentinel index, so neither is a count.

 @param value - value to check

 @param path - dotted path for error message

 @returns Value as a finite number

 @throws {@link ArtifactParseError} when the value is not one

 @example
 ```ts
 const weight = requireFinite({ value: ballot.weight, path: 'ballot.weight', },);
 ```
 */
export function requireFinite(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): number {
  if ((typeof value) !== 'number')
    throw new ArtifactParseError({
      path,
      reason: 'a number',
    },);
  if (!Number.isFinite(value,))
    throw new ArtifactParseError({
      path,
      reason: 'a finite number',
    },);
  return value;
}

//endregion Artifact guards
