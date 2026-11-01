import type { FootnoteRelabel, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type {
  FootnoteLabelRewrite,
  RetainedArchiveFootnoteLabel,
} from './footnote-label-rewrite.ts';

//region Closure provenance
// Correspondence evidence, forced elimination and operational displacement remain distinct.

/**
 Why an archive must remain unchanged instead of merging or guessing note identities.

 @example
 ```ts
 const open: OpenFootnoteClosure = { kind: 'open', detail: 'source correspondence remains incomplete' };
 ```
 */
export type OpenFootnoteClosure = {
  /**
   No safe composed rewrite is established.
   */
  readonly kind: 'open';
  /**
   Identifier-level explanation, never note prose.
   */
  readonly detail: string;
};

/**
 Closed simultaneous rewrite and the separate authority for each kind of move.

 @example
 ```ts
 if (closure.kind === 'closed') applyFootnoteRelabel({ text, map: closure.map });
 ```
 */
export type RelabelClosure = OpenFootnoteClosure | {
  /**
   Every occupied destination is retained uniquely or moved away.
   */
  readonly kind: 'closed';
  /**
   Actual rewrites, excluding normalization-equivalent identity relations.
   */
  readonly map: readonly FootnoteLabelRewrite[];
  /**
   Supplied positive relations, including identity correspondences.
   */
  readonly correspondences: readonly FootnoteRelabel[];
  /**
   At most one relation supplied by the existing forced-elimination rule.
   */
  readonly eliminated: readonly FootnoteRelabel[];
  /**
   Fresh labels preserving colliding unmatched archive notes without pairing them.
   */
  readonly retained: readonly RetainedArchiveFootnoteLabel[];
};

/**
 Checked normalized namespaces and independent label relations.

 @example
 ```ts
 if (input.kind === 'ready') inspect(input.correspondences);
 ```
 */
export type FootnoteClosureInput = OpenFootnoteClosure | {
  /**
   Membership and injectivity have been checked.
   */
  readonly kind: 'ready';
  /**
   Normalized archive identifiers with first encountered source spelling.
   */
  readonly archive: ReadonlyMap<string, string>;
  /**
   Normalized original identifiers with first encountered source spelling.
   */
  readonly original: ReadonlyMap<string, string>;
  /**
   Supplied relations, duplicate identical claims folded without adding authority.
   */
  readonly correspondences: readonly FootnoteRelabel[];
};

//endregion Closure provenance
