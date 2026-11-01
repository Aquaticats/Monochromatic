import type { FootnoteRelabel, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { FootnoteClosureInput, } from './footnote-closure-model.ts';
import { normalizeFootnoteIdentifier, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Namespace and correspondence validation
// Logical label identity follows the parser, while raw spellings remain available for exact rewriting.

/**
 Keeps the first spelling of each normalized identifier in encounter order.

 @param labels - labels from one document's marker inventory

 @returns Normalized keys and their original spellings

 @example
 ```ts
 const labels = footnoteLabelNamespace({ labels: ['Note', 'note'] });
 ```
 */
export function footnoteLabelNamespace({ labels, }: { readonly labels: readonly string[]; },): ReadonlyMap<string, string> {
  /**
   Owned namespace, never a mutation of the caller's inventory.
   */
  const namespace = new Map<string, string>();
  for (const label of labels) {
    /**
     Parser-equivalent identity of this spelling.
     */
    const key = normalizeFootnoteIdentifier({ identifier: label, },);
    if (!namespace.has(key,))
      namespace.set(
        key,
        label,
      );
  }
  return namespace;
}

/**
 Checks that supplied correspondences belong to both documents and remain injective after normalization.
 Repeated identical claims carry no additional authority and collapse to their first spelling.

 @param map - positive label correspondences, including identity relations

 @param archiveLabels - archive identifier universe

 @param originalLabels - original identifier universe

 @returns Checked input or an explicit refusal to infer a rewrite

 @example
 ```ts
 const input = readFootnoteClosureInput({ map, archiveLabels, originalLabels });
 ```
 */
export function readFootnoteClosureInput(
  {
    map,
    archiveLabels,
    originalLabels,
  }: {
    readonly map: readonly FootnoteRelabel[];
    readonly archiveLabels: readonly string[];
    readonly originalLabels: readonly string[];
  },
): FootnoteClosureInput {
  /**
   Archive spellings grouped by the parser's identity rule.
   */
  const archive = footnoteLabelNamespace({ labels: archiveLabels, },);
  /**
   Original spellings grouped under the same rule.
   */
  const original = footnoteLabelNamespace({ labels: originalLabels, },);
  if (archive.has('',) || original.has('',))
    return {
      kind: 'open',
      detail: 'a footnote namespace contains an empty normalized identifier',
    };
  /**
   Existing destinations per archive identity.
   */
  const forward = new Map<string, string>();
  /**
   Existing owners per original identity.
   */
  const backward = new Map<string, string>();
  /**
   Distinct supplied relations in their initial order.
   */
  const correspondences: FootnoteRelabel[] = [];
  for (const relation of map) {
    /**
     Logical archive identity.
     */
    const from = normalizeFootnoteIdentifier({ identifier: relation.from, },);
    /**
     Logical original identity.
     */
    const to = normalizeFootnoteIdentifier({ identifier: relation.to, },);
    if (!archive.has(from,))
      return {
        kind: 'open',
        detail: `correspondence names unavailable archive label [^${relation.from}]`,
      };
    if (!original.has(to,))
      return {
        kind: 'open',
        detail: `correspondence names unavailable original label [^${relation.to}]`,
      };
    /**
     Earlier destination of the same archive identity, if recorded.
     */
    const earlierTo = forward.get(from,);
    /**
     Earlier archive owner of the same original identity, if recorded.
     */
    const earlierFrom = backward.get(to,);
    if (((earlierTo !== undefined) && (earlierTo !== to)) || ((earlierFrom !== undefined) && (earlierFrom !== from)))
      return {
        kind: 'open',
        detail: `correspondence [^${relation.from}]->[^${relation.to}] conflicts after identifier normalization`,
      };
    if (earlierTo === undefined)
      correspondences.push({
        from: relation.from,
        to: relation.to,
      },);
    forward.set(
      from,
      to,
    );
    backward.set(
      to,
      from,
    );
  }
  return {
    kind: 'ready',
    archive,
    original,
    correspondences,
  };
}

//endregion Namespace and correspondence validation
