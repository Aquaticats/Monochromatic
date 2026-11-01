import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  ArtifactParseError,
  requireArray,
  requireCount,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  assertPipelineDigest,
  isDigestShaped,
  PIPELINE_DIGEST_SHAPE,
  type PipelineDigest,
} from '@monochromatic-dev/translation-repair-glossary/ts';

//region Artifact field readers
// Single-field readers the two-lane artifact parser leans on, kept beside it
// so the parser itself stays within the file budget. Each narrows one
// recorded value by the same check a fresh one passes, and refuses with the
// dotted path of the field rather than its content.

/**
 Reads the digest naming the built output that ran.

 @param value - recorded digest

 @param path - dotted path for error message

 @returns Digest, narrowed by the same check a fresh one passes

 @throws {@link ArtifactParseError} when the value is not a string, or not
 shaped like a digest

 @example
 ```ts
 const digest = requireDigest({ value: artifact.pipelineDigest, path, },);
 ```
 */
export function requireDigest(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): PipelineDigest {
  /**
   Recorded string, before it is known to be a digest.
   */
  const held = requireString({
    value,
    path,
  },);

  // REFUSED BY SHAPE, in words this file writes, before the assertion narrows.
  // Forwarding the assertion's message instead quoted the recorded value into
  // the refusal, which the region's rule forbids (ledger B34).
  if (!isDigestShaped({ value: held, },)) {
    throw new ArtifactParseError({
      path,
      reason: `a pipeline digest, ${PIPELINE_DIGEST_SHAPE}`,
    },);
  }
  assertPipelineDigest(held,);
  return held;
}

/**
 Reads the recorded front matter authority, which can only be the archive's.

 @param value - recorded authority

 @param path - dotted path for error messages

 @returns The one authority a record carries

 @throws {@link ArtifactParseError} when the record carries anything else

 @example
 ```ts
 const authority = requireArchiveAuthority({ value: record.frontMatterAuthority, path, },);
 ```
 */
export function requireArchiveAuthority(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): 'archive' {
  if (value === 'archive')
    return value;
  throw new ArtifactParseError({
    path,
    reason: "'archive', the one authority a preparation records",
  },);
}

/**
 Reads the recorded archive-original spans: a non-empty list of offset pairs,
 each with the note that sealed it.

 @param value - recorded list

 @param path - dotted path for error messages

 @returns Spans as recorded

 @throws {@link ArtifactParseError} when the list is empty, or a span lacks
 its offsets or note, or ends before it starts

 @example
 ```ts
 const spans = requireArchiveOriginalSpans({ value: record.archiveOriginalSpans, path, },);
 ```
 */
export function requireArchiveOriginalSpans(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): readonly ArchiveOriginalSpan[] {
  /**
   Recorded entries, before each is known to be a span.
   */
  const entries = requireArray({
    value,
    path,
  },);
  if (entries.length === 0)
    throw new ArtifactParseError({
      path,
      reason: 'a non-empty list, since nothing sealed is recorded as absence',
    },);
  return entries.map(function toSpan(
    entry,
    at,
  ): ArchiveOriginalSpan {
    /**
     Dotted path of this span.
     */
    const spanPath = `${path}[${String(at,)}]`;
    /**
     Span as a record.
     */
    const record = requireRecord({
      value: entry,
      path: spanPath,
    },);
    /**
     Where the seal starts.
     */
    const startOffset = requireCount({
      value: record.startOffset,
      path: `${spanPath}.startOffset`,
    },);
    /**
     Where the seal ends, exclusive.
     */
    const endOffset = requireCount({
      value: record.endOffset,
      path: `${spanPath}.endOffset`,
    },);
    if (endOffset < startOffset)
      throw new ArtifactParseError({
        path: `${spanPath}.endOffset`,
        reason: 'an end at or after its start',
      },);
    return {
      startOffset,
      endOffset,
      note: requireString({
        value: record.note,
        path: `${spanPath}.note`,
      },),
    };
  },);
}

//endregion Artifact field readers
