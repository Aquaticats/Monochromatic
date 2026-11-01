//region Footnote rewrite refusals
// These messages name operations, never archive prose or supplied labels.

/**
 Closed reasons for withholding a footnote rewrite.

 @example
 ```ts
 const kind: FootnoteRewriteFailure = 'collision';
 ```
 */
export type FootnoteRewriteFailure = 'syntax' | 'position' | 'label' | 'mapping' | 'collision' | 'graph'
  | 'missing-source' | 'slice-scope';

/**
 Actionable diagnostics for each invariant boundary.
 */
const MESSAGES: Readonly<Record<FootnoteRewriteFailure, string>> = {
  'missing-source': 'footnote rewrite: a changing map identifier is absent from the current active document; rebuild correspondence before retrying',
  'slice-scope': 'footnote rewrite: prepared slice ranges or text do not match the current source or archive; prepare the documents again before reading correspondence',
  syntax: 'footnote rewrite: document syntax could not be verified; retain the archive or repair its syntax before retrying',
  position: 'footnote rewrite: parsed marker positions do not match raw syntax; retain the archive and investigate the parser boundary',
  label: 'footnote rewrite: a supplied label does not encode exactly one GFM identifier; supply a valid label before retrying',
  mapping: 'footnote rewrite: normalized labels have conflicting destinations; resolve the correspondence before retrying',
  collision: 'footnote rewrite: distinct archive identifiers would merge; close the map before retrying',
  graph: 'footnote rewrite: the transformed marker graph differs from the intended rename; retain the archive and investigate the transformation',
};

/**
 An operation cannot prove a syntax-preserving and injective rename.

 @example
 ```ts
 throw new FootnoteRewriteError({ kind: 'collision' });
 ```
 */
export class FootnoteRewriteError extends Error {
  /**
   Stable diagnostic operation.
   */
  public override readonly name = 'FootnoteRewriteError';
  /**
   Diagnostic text is restricted to the closed operation messages.
   */
  readonly messageNamesOnly: true = true;
  /**
   Structured refusal for callers that retain unchanged archive bytes.
   */
  readonly kind: FootnoteRewriteFailure;

  /**
   Retains the failing operation without interpolating supplied text.

   @param kind - invariant that could not be established

   @param cause - original parser failure when available

   @example
   ```ts
   const error = new FootnoteRewriteError({ kind: 'syntax', cause });
   ```
   */
  public constructor({
    kind,
    cause,
  }: {
    readonly kind: FootnoteRewriteFailure;
    readonly cause?: unknown
  },) {
    super(
      MESSAGES[kind],
      { cause, },
    );
    this.kind = kind;
  }
}

/**
 The rewrite refusal a catch around a footnote rewrite holds, for a catch
 that acts on the refusal alone.

 SHARED RATHER THAN INLINE (ledger T8, ninth batch), as `requireMdxRefusal`
 is for the grammar's refusal: the pass's relabel catch tested the class and
 rethrew anything else, and no test reached the rethrow, since the rewrite
 raises its refusals as this class and throws otherwise only where an
 invariant breaks. The narrowing stands here, where a case reaches the
 rethrow.

 @param error - what the catch caught

 @returns The refusal

 @throws The caught value unchanged when it is anything but a rewrite
 refusal, an unexpected state that must keep propagating

 @example
 ```ts
 const refusal = requireFootnoteRewriteRefusal({ error, },);
 ```
 */
export function requireFootnoteRewriteRefusal({ error, }: { readonly error: unknown; },): FootnoteRewriteError {
  if (error instanceof FootnoteRewriteError)
    return error;
  throw error;
}

//endregion Footnote rewrite refusals
