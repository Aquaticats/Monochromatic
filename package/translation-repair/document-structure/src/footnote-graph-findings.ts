import type {
  FootnoteConvention,
  FootnoteDefinitionHit,
  FootnoteGraphFinding,
  FootnoteReferenceHit,
} from './footnote-model.ts';

//region Footnote graph findings
// The integrity findings of a footnote graph, read off the references and
// definitions one document walk collected. Split out of `footnote-graph.ts`
// at its line budget.

/**
 Composite key joining convention and identifier for grouping.

 @param convention - syntax family

 @param identifier - normalized identifier

 @returns Collision-free grouping key

 @example
 ```ts
 graphKey({ convention: 'gfm', identifier: '1', },);
 ```
 */
function graphKey(
  {
    convention,
    identifier,
  }: {
    readonly convention: FootnoteConvention;
    readonly identifier: string;
  },
): string {
  return `${convention}\u0000${identifier}`;
}

/**
 Computes integrity findings from collected references and definitions.

 @param references - every reference in source order

 @param definitions - every definition in source order

 @returns Findings for unresolved references, orphan definitions, and duplicates

 @example
 ```ts
 footnoteGraphFindings({ references, definitions, },);
 ```
 */
export function footnoteGraphFindings(
  {
    references,
    definitions,
  }: {
    readonly references: readonly FootnoteReferenceHit[];
    readonly definitions: readonly FootnoteDefinitionHit[];
  },
): readonly FootnoteGraphFinding[] {
  /**
   Definition count per grouping key, driving duplicate detection.
   */
  const definitionCounts = new Map<string, number>();
  for (const definition of definitions) {
    /**
     Grouping key of this definition.
     */
    const key = graphKey(definition,);
    definitionCounts.set(
      key,
      (definitionCounts.get(key,) ?? 0) + 1,
    );
  }

  /**
   Keys of identifiers referenced at least once, driving orphan detection.
   */
  const referencedKeys = new Set(references.map(function toKey(reference,): string {
    return graphKey(reference,);
  },),);

  /**
   Unresolved references: no definition carries their key.
   */
  const unresolved = references
    .filter(function lacksDefinition(reference,): boolean {
      return !definitionCounts.has(graphKey(reference,),);
    },)
    .map(function toFinding(reference,): FootnoteGraphFinding {
      return {
        kind: 'unresolved-reference',
        convention: reference.convention,
        identifier: reference.identifier,
        nodeId: reference.nodeId,
      };
    },);

  /**
   Orphan definitions: never referenced anywhere.
   */
  const orphans = definitions
    .filter(function neverReferenced(definition,): boolean {
      return !referencedKeys.has(graphKey(definition,),);
    },)
    .map(function toFinding(definition,): FootnoteGraphFinding {
      return {
        kind: 'orphan-definition',
        convention: definition.convention,
        identifier: definition.identifier,
        nodeId: definition.nodeId,
      };
    },);

  /**
   Keys more than one definition carries, read off the counts so no
   definition needs a count looked up again.
   */
  const repeatedKeys = new Set<string>();
  for (const [key, count,] of definitionCounts) {
    if (count > 1)
      repeatedKeys.add(key,);
  }

  /**
   Duplicate definitions: identifier defined more than once.
   */
  const duplicates = definitions
    .filter(function definedTwice(definition,): boolean {
      return repeatedKeys.has(graphKey(definition,),);
    },)
    .map(function toFinding(definition,): FootnoteGraphFinding {
      return {
        kind: 'duplicate-definition',
        convention: definition.convention,
        identifier: definition.identifier,
        nodeId: definition.nodeId,
      };
    },);

  return [
    ...unresolved,
    ...orphans,
    ...duplicates,
  ];
}

//endregion Footnote graph findings
