//region Footnote graph model
// Two conventions coexist in the corpora: GFM `[^1]` syntax (memorial texts; upstream
// renders it quasi-literally, so emitted repairs preserve the exact textual form) and
// plain-text `〔1〕` markers (archive texts; not markdown syntax at all). The graph
// records references, definitions, and integrity findings; findings are inputs to
// human checkpoints, never thrown errors, because defective documents are ordinary
// inputs to a repair pipeline.

/**
 Syntax family a footnote reference or definition was expressed in.

 @example
 ```ts
 const convention: FootnoteConvention = 'gfm';
 ```
 */
export type FootnoteConvention = 'gfm' | 'fullwidth-bracket';

/**
 One in-text reference to a footnote.

 @example
 ```ts
 const ref: FootnoteReferenceHit = {
   convention: 'gfm',
   identifier: '1',
   nodeId: 'block/4',
   offset: 128,
 };
 ```
 */
export type FootnoteReferenceHit = {
  /**
   Syntax family carrying this reference.
   */
  readonly convention: FootnoteConvention;

  /**
   Normalized identifier: GFM identifier verbatim,
   or bracket number rendered as ASCII digits.
   */
  readonly identifier: string;

  /**
   Block node containing this reference.
   */
  readonly nodeId: string;

  /**
   Absolute character offset of reference start within full document source.
   */
  readonly offset: number;
};

/**
 One footnote definition.

 @example
 ```ts
 const def: FootnoteDefinitionHit = {
   convention: 'gfm',
   identifier: '1',
   nodeId: 'block/9',
 };
 ```
 */
export type FootnoteDefinitionHit = {
  /**
   Syntax family carrying this definition.
   */
  readonly convention: FootnoteConvention;

  /**
   Normalized identifier matching referencing hits.
   */
  readonly identifier: string;

  /**
   Block node holding this definition.
   */
  readonly nodeId: string;
};

/**
 Integrity defect discovered while validating reference-to-definition mapping.
 Findings feed human checkpoints;
 marker-count parity alone proves nothing, so validation works on the graph.

 @example
 ```ts
 const finding: FootnoteGraphFinding = {
   kind: 'unresolved-reference',
   convention: 'gfm',
   identifier: '2',
   nodeId: 'block/4',
 };
 ```
 */
export type FootnoteGraphFinding = {
  /**
   Defect class:
   reference without definition,
   definition never referenced,
   or identifier defined more than once.
   */
  readonly kind: 'unresolved-reference' | 'orphan-definition' | 'duplicate-definition';

  /**
   Syntax family the defective identifier belongs to.
   */
  readonly convention: FootnoteConvention;

  /**
   Identifier at fault.
   */
  readonly identifier: string;

  /**
   Block node where defect manifests.
   */
  readonly nodeId: string;
};

/**
 Complete footnote graph of one document.

 @example
 ```ts
 const graph: FootnoteGraph = { references: [], definitions: [], findings: [], };
 ```
 */
export type FootnoteGraph = {
  /**
   Every in-text reference in source order.
   */
  readonly references: readonly FootnoteReferenceHit[];

  /**
   Every definition in source order.
   */
  readonly definitions: readonly FootnoteDefinitionHit[];

  /**
   Integrity defects; empty means every reference resolves uniquely
   and no definition is orphaned.
   */
  readonly findings: readonly FootnoteGraphFinding[];
};

//endregion Footnote graph model
