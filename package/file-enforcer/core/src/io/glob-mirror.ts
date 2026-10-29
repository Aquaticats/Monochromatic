/**
 Captured wildcard segment and position of following fixed text.
 */
type CapturedSegment = Readonly<{
  /**
   Captured wildcard text.
   */
  readonly capture: string;

  /**
   Position where next fixed segment begins.
   */
  readonly nextFixedPos: number;
}>;

/**
 Captures one wildcard segment from current remainder.
 
 @param sourcePattern - Glob pattern used to match source.
 
 @param sourcePath - Concrete path being mirrored.
 
 @param sourceParts - Source pattern split by `*`.
 
 @param partIndex - Current fixed-part index.
 
 @param remainder - Remaining unmatched source path.
 
 @returns Captured text and position where next fixed segment begins.
 
 @throws When next fixed segment cannot be found.
 
 @example
 ```ts
 const captured = captureOneSegment({ sourcePattern, sourcePath, sourceParts, partIndex, remainder });
 ```
 */
function captureOneSegment(
  {
    sourcePattern,
    sourcePath,
    sourceParts,
    partIndex,
    remainder,
  }: {
    readonly partIndex: number;
    readonly remainder: string;
    readonly sourceParts: readonly string[];
    readonly sourcePath: string;
    readonly sourcePattern: string;
  },
): CapturedSegment {
  /**
   Fixed segment following current wildcard.
   */
  const nextFixed = sourceParts[partIndex + 1] ?? '';
  /**
   Position where next fixed segment begins.
   */
  const nextFixedPos = (nextFixed === '')
    ? remainder.length
    : remainder.indexOf(nextFixed,);
  if (nextFixedPos === (-1)) {
    throw new Error(
      `Source path "${sourcePath}" does not match pattern "${sourcePattern}"`,
    );
  }
  return {
    capture: remainder.slice(
      0,
      nextFixedPos,
    ),
    nextFixedPos,
  };
}

/**
 Captures wildcard segment values from source path, delegating each
 wildcard to {@link captureOneSegment}.
 
 @param sourcePattern - Glob pattern used to match source.
 
 @param sourcePath - Concrete path being mirrored.
 
 @param sourceParts - Source pattern split by `*`.
 
 @param sourceWildcardCount - Number of wildcard captures expected.
 
 @returns Captured wildcard strings.
 
 @throws When source path does not match source pattern.
 
 @example
 ```ts
 const captures = captureSegments({ sourcePattern, sourcePath, sourceParts, sourceWildcardCount });
 ```
 */
function captureSegments(
  {
    sourcePattern,
    sourcePath,
    sourceParts,
    sourceWildcardCount,
  }: {
    readonly sourcePattern: string;
    readonly sourcePath: string;
    readonly sourceParts: readonly string[];
    readonly sourceWildcardCount: number;
  },
): readonly string[] {
  {
    /**
     Wildcard captures appended in source-pattern order.
     */
    const acc: string[] = [];
    /**
     Unconsumed tail of source path.
     */
    let remainder = sourcePath;
    for (let partIndex = 0; partIndex < sourceParts.length; partIndex++) {
      /**
       Fixed text before or after current wildcard.
       */
      const fixedPart = sourceParts[partIndex];
      if (fixedPart === undefined)
        break;
      if (!remainder.startsWith(fixedPart,)) {
        throw new Error(
          `Source path "${sourcePath}" does not match pattern "${sourcePattern}" at segment "${fixedPart}"`,
        );
      }
      remainder = remainder.slice(fixedPart.length,);

      if (partIndex < sourceWildcardCount) {
        /**
         Captured segment and next fixed-text position.
         */
        const captured = captureOneSegment({
          sourcePattern,
          sourcePath,
          sourceParts,
          partIndex,
          remainder,
        },);
        acc.push(captured.capture,);
        remainder = remainder.slice(captured.nextFixedPos,);
      }
    }
    return acc;
  }
}

/**
 Extracts wildcard segments from a source path using {@link captureSegments},
 then substitutes them into the destination glob pattern.
 
 Each `*` in the source pattern captures one path segment value; those captured
 values are inserted positionally into the `*` slots of the dest pattern.
 
 @param sourcePattern - Glob pattern used to match source.
 
 @param destPattern - Glob pattern for destination.
 
 @param sourcePath - Concrete path that matched sourcePattern.
 
 @returns Concrete destination path with wildcards filled in.
 
 @throws When wildcard counts don't match between source and dest patterns.
 
 @example
 ```ts
 mirrorGlobPath({
   sourcePattern: 'package/*​/src/*.ts',
   destPattern: 'temp/*​/src/*.ts',
   sourcePath: 'package/foo/src/index.ts',
 });
 ```
 */
export function mirrorGlobPath(
  {
    sourcePattern,
    destPattern,
    sourcePath,
  }: {
    readonly sourcePattern: string;
    readonly destPattern: string;
    readonly sourcePath: string;
  },
): string {
  /**
   Segments of source pattern split by `*`.
   */
  const sourceParts = sourcePattern.split('*',);
  /**
   Segments of destination pattern split by `*`.
   */
  const destParts = destPattern.split('*',);

  /**
   Number of wildcards in source pattern.
   */
  const sourceWildcardCount = sourceParts.length - 1;
  /**
   Number of wildcards in destination pattern.
   */
  const destWildcardCount = destParts.length - 1;
  if (sourceWildcardCount !== destWildcardCount) {
    throw new Error(
      `Wildcard count mismatch: source "${sourcePattern}" has ${
        String(sourceWildcardCount,)
      }`
        + ` but dest "${destPattern}" has ${String(destWildcardCount,)}`,
    );
  }

  /**
   Wildcard captures extracted from source path.
   */
  const captured = captureSegments({
    sourcePattern,
    sourcePath,
    sourceParts,
    sourceWildcardCount,
  },);

  /**
   Reconstructed destination path with wildcards replaced by captured values.
   */
  const result = destParts.flatMap(function appendDestSegment(
    part,
    destIndex,
  ): readonly string[] {
    /**
     Raw fixed text for this position.
     */
    const fixed = part ?? '';
    if (destIndex < destWildcardCount) {
      return [
        fixed,
        captured[destIndex] ?? '',
      ];
    }
    return [fixed,];
  },);
  return result.join('',);
}
