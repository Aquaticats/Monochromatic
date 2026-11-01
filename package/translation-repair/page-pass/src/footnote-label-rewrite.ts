//region Footnote rewrite roles
// Moving an unmatched archive label never establishes a correspondence with an original note.

/**
 Operational rewrite of a footnote label, independent of why it moves.

 @example
 ```ts
 const rewrite: FootnoteLabelRewrite = { from: '1', to: '4' };
 ```
 */
export type FootnoteLabelRewrite = {
  /**
   Label before simultaneous rewriting.
   */
  readonly from: string;
  /**
   Label after rewriting, not necessarily present in the original document.
   */
  readonly to: string;
};

/**
 Collision-avoidance rename preserving an unmatched archive note.
 `retainedAs` deliberately does not name an original-side correspondence.

 @example
 ```ts
 const retained: RetainedArchiveFootnoteLabel = { from: '1', retainedAs: '4' };
 ```
 */
export type RetainedArchiveFootnoteLabel = {
  /**
   Archive label displaced because a supported correspondence needs its identifier.
   */
  readonly from: string;
  /**
   Fresh identifier outside both input namespaces.
   */
  readonly retainedAs: string;
};

//endregion Footnote rewrite roles
