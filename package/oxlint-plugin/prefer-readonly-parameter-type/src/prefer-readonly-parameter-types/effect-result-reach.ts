/**
 Every expression a value can have come from, for finding the call underneath it.
 
 `expressionResultSites` normalises access layers and identity-keeping wrappers and then asks
 whether a call underlies what is left. That answers for `firstRow(config,).row` and for an
 alias hop, and it stops at the first shape that is neither, which left a family of writes
 attributing nothing. All of these were falsified, with every offer applied, type-checking
 clean beside a control whose direct write was rejected, and a driver observing the change:
 
 ```ts
 (pick ? firstRow(config,) : firstRow(config,)).label = 'written';
 
 const { row, } = { row: firstRow(config,), };
 row.label = 'written';
 
 return [firstRow(config,),][0] as Row;
 ```
 
 A conditional, a property of a literal, and an element of one.
 Each holds what a call handed back, and the normalisation walk cannot see through any of them,
 because each is a place a value came from rather than a layer over it.
 
 Two relations, kept apart on purpose. Value sources are what `effect-possible-values.ts`
 already answers, and reusing it here rather than restating it is the point. Aggregate members
 are added on top, and only here: that walk answers what a value **is** for deciding callable
 identity, where descending a literal would be wrong, while this one asks where a value can
 have **come from**, where descending it is the whole question.
 
 Widening this walk can only add call sites, and adding one can only add an effect, so every
 shape it reaches is a hole closed rather than a risk taken.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import {
  isArrayLiteralExpression,
  isObjectLiteralExpression,
  isPropertyAssignment,
  isSpreadAssignment,
  isSpreadElement,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { possibleValueNodes, } from './effect-possible-values.ts';

/**
 Collects every expression one value can have come from, through sources and aggregates.
 
 @param project - TypeScript project resolving identifiers to their declarations.
 
 @param node - Expression whose origins are being searched.
 
 @returns expressions the value can have come from, including the expression itself.
 
 @example
 ```ts
 reachableValueSources({ project, node });
 ```
 */
export function reachableValueSources({
  project,
  node,
}: {
  readonly project: Project;
  readonly node: Node;
},): readonly Node[] {
  /**
   Sources found so far, keyed by span so a cyclic alias terminates.
   */
  const seen = new Map<string, Node>();
  /**
   Expressions still to expand.
   */
  const pending: Node[] = [node,];
  while (pending.length > 0) {
    /**
     Next expression being expanded.
     */
    const current = pending.pop();
    if (current === undefined)
      continue;
    /**
     Span identifying this expression across one analysis.
     */
    const key = `${current.getSourceFile()
      .fileName}:${String(current.pos,)}:${String(current.end,)}`;
    if (seen.has(key,))
      continue;
    seen.set(
      key,
      current,
    );
    possibleValueNodes({
      project,
      node: current,
    },)
      .forEach(function enqueueSource(source,): void {
        pending.push(source,);
      },);
    aggregateMembers({ node: current, },)
      .forEach(function enqueueMember(member,): void {
        pending.push(member,);
      },);
  }
  return [...seen.values(),];
}

/**
 Names the values an authored aggregate holds.
 
 Only an authored literal answers. A value read out of something else is reached by the access
 layers the normalisation walk already strips, and guessing at the contents of anything else
 would claim a member the source never wrote.
 
 Every member counts, without asking which one a later read selects. A property key or an
 element index is not tracked here, so narrowing to one member would need a claim this walk
 cannot support, and taking all of them can only add a call site.
 
 @param node - Expression that may be an authored aggregate.
 
 @returns values the aggregate holds.
 
 @example
 ```ts
 aggregateMembers({ node });
 ```
 */
function aggregateMembers({ node, }: { readonly node: Node; },): readonly Node[] {
  if (isArrayLiteralExpression(node,))
    return node.elements
      .map(function elementValue(element,): Node {
        return isSpreadElement(element,) ? element.expression : element;
      },);
  if (!isObjectLiteralExpression(node,))
    return [];
  return node.properties
    .flatMap(function propertyValue(property,): readonly Node[] {
      if (isPropertyAssignment(property,))
        return [property.initializer,];
      if (isSpreadAssignment(property,))
        return [property.expression,];
      return [];
    },);
}
