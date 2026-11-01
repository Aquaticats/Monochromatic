/**
 Reports that no PATH-exposed candidate can act as real Git.
 
 @example
 ```ts
 throw new RealGitNotFoundError({
   candidateCount: 4,
   skippedSelfShimCount: 1,
 });
 ```
 */
export class RealGitNotFoundError extends Error {
  /**
   Stable error classification independent of minification.
   */
  override readonly name = 'RealGitNotFoundError';

  /**
   Creates resolution failure with attempted-candidate evidence.
   
   @param candidateCount - PATH-derived candidates examined before failure.
   
   @param skippedSelfShimCount - Self-referential wrappers rejected during examination.
   
   @example
   ```ts
   const error = new RealGitNotFoundError({
     candidateCount: 4,
     skippedSelfShimCount: 1,
   });
   ```
   */
  constructor({
    candidateCount,
    skippedSelfShimCount,
  }: {
    readonly candidateCount: number;
    readonly skippedSelfShimCount: number;
  },) {
    super(
      `Could not find a real Git executable after examining ${String(candidateCount,)} PATH candidates `
        + `and skipping ${String(skippedSelfShimCount,)} self-referential wrappers. `
        + 'Ensure Git is installed and PATH/PATHEXT expose its executable.',
    );
  }
}
