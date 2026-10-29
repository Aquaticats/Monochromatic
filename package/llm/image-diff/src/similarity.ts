/**
 Compute the dot product of two vectors.
 Voyage AI embeddings are unit-normalized, so the dot product equals cosine similarity.
 
 @param a - first embedding vector
 
 @param b - second embedding vector
 
 @returns dot product (cosine similarity for unit vectors)
 
 @throws when vectors have different lengths
 
 @example
 ```ts
 const sim = dotProduct({ a: [1, 0], b: [0, 1] }); // 0
 const same = dotProduct({ a: [1, 0], b: [1, 0] }); // 1
 ```
 */
export function dotProduct({
  a,
  b,
}: {
  readonly a: readonly number[];
  readonly b: readonly number[];
},): number {
  if (a.length
    !== b
      .length) {
    throw new Error(
      `Vector length mismatch: ${String(a.length,)} vs ${String(b.length,)}`,
    );
  }

  /**
   Running dot-product accumulator.
   
   `let` is required because each loop iteration contributes one term; `reduce` would force
   a `b[i]!` non-null assertion at the same place the explicit guard below already handles.
   */
  let sum = 0;
  for (let loopIndex = 0; loopIndex < a
    .length; loopIndex++) {
    /**
     Element from the first vector at index `i`; guarded against jagged-array sparsity.
     */
    const ai = a[loopIndex];
    /**
     Element from the second vector at index `i`; guarded against jagged-array sparsity.
     */
    const bi = b[loopIndex];
    if ((ai === undefined) || (bi === undefined))
      break;
    sum += ai * bi;
  }
  return sum;
}

/**
 Compute cosine similarity between two embedding vectors.
 For Voyage AI embeddings (already unit-normalized), this is equivalent to {@link dotProduct}.
 This function handles arbitrary vectors by normalizing them first.
 
 @param a - first embedding vector
 
 @param b - second embedding vector
 
 @returns cosine similarity between -1 and 1
 
 @throws when vectors have different lengths or either has zero magnitude
 
 @example
 ```ts
 const sim = cosineSimilarity({ a: [3, 4], b: [4, 3] }); // ~0.96
 ```
 */
export function cosineSimilarity({
  a,
  b,
}: {
  readonly a: readonly number[];
  readonly b: readonly number[];
},): number {
  if (a.length
    !== b
      .length) {
    throw new Error(
      `Vector length mismatch: ${String(a.length,)} vs ${String(b.length,)}`,
    );
  }

  /**
   Numerator of the cosine formula; reused below after the magnitudes are computed.
   */
  const dot = dotProduct({
    a,
    b,
  },);
  /**
   Denominator of the cosine formula: product of the two vector lengths via the self-dot-product identity.
   */
  const magnitude = Math.sqrt(dotProduct({
    a,
    b: a,
  },),)
    * Math
      .sqrt(dotProduct({
        a: b,
        b,
      },),);
  if (magnitude === 0) {
    throw new Error(
      'Cannot compute cosine similarity: one or both vectors have zero magnitude',
    );
  }

  return dot / magnitude;
}
