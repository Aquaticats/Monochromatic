//region Publish may write
// THE QUESTION A CALLER MAY PUT TO THE PUBLISHER BEFORE ANYTHING IS WRITTEN (B1242, B1300). `publishFixedPage` composes
// the page and runs its checks first; a caller that must not replace a better page at the same path (an interim page,
// `interim-page.ts`) is asked with what the composition did to the slices, and answers whether to write.

/**
 Answers whether the composed page may be written.

 The argument names the slices the page carries as the archive has them whatever the artifact says: those the
 page-grammar fallback put back to the archive, and those whose original the composition left off the page because no
 writing of it made the page parse.

 @param composed - what the composition and the grammar did to the slices of the page

 @returns False to leave the page standing at its path untouched

 @example
 ```ts
 const mayWrite: MayWrite = ({ keptArchiveAt, sourceDroppedAt, }) =>
   (keptArchiveAt.length + sourceDroppedAt.length) === 0;
 ```
 */
export type MayWrite = (
  composed: {
    /**
     Slices the page-grammar fallback put back to the archive.
     */
    readonly keptArchiveAt: readonly number[];

    /**
     Slices whose original the composition left off the page.
     */
    readonly sourceDroppedAt: readonly number[];
  },
) => boolean;

//endregion Publish may write
