/**
 Pipeline-level benchmark: the css-edit-based `expandCssMixins` against a
 faithful replica of the retired postcss mixin pipeline (clone-per-splice,
 ten-pass fixed-point with full serialization comparison). This is the
 number that matters for the build tool; the raw parse benchmark in
 `parse.bench.ts` measures the parsers alone.
 
 @module
 */

import { expandCssMixins, } from '@monochromatic-dev/css-cli/ts/expand';
import {
  bench,
  do_not_optimize,
  run,
  summary,
} from 'mitata';
import {
  type ChildNode,
  parse as postcssParse,
} from 'postcss';

//region Inputs

/**
 Number of chained mixin definitions.
 */
const MIXIN_COUNT = 30;

/**
 Number of component rules applying mixins.
 */
const APPLY_COUNT = 200;

/**
 Mixin sheet: a chain where each definition applies its predecessor, so the
 old pipeline needs multiple fixed-point passes.
 */
const MIXIN_CSS = Array
  .from(
  { length: MIXIN_COUNT, },
  function mixinAt(
    _unused,
    index,
  ) {
      /**
       Reference to the previous definition, chaining expansion depth.
       */
    const nested = index === 0
      ? ''
      : `@apply --m-${String(index - 1,)}; `;
    return `@mixin --m-${String(index,)} { ${nested}padding-${String(index,)}: ${String(index,)}px; }`;
  },
  )
  .join('\n',);

/**
 Consumer sheet: many rules each applying one chained mixin.
 */
const CONSUMER_CSS = Array
  .from(
  { length: APPLY_COUNT, },
  function ruleAt(
    _unused,
    index,
  ) {
    return `.c-${String(index,)} { @apply --m-${String(index % MIXIN_COUNT,)}; color: red; }`;
  },
  )
  .join('\n',);

//endregion Inputs

//region Old-pipeline replica

/**
 Maximum fixed-point passes, matching the retired pipeline's guard.
 */
const OLD_MAX_PASSES = 10;

/**
 Recursively expands apply at-rules by cloning registry bodies, matching the
 retired `expandApplyInNodes`.
 
 @param nodes - Nodes to expand.
 
 @param registry - Mixin bodies by name.
 
 @returns Expanded clones.
 */
function oldExpandNodes(
  nodes: readonly ChildNode[],
  registry: ReadonlyMap<string, readonly ChildNode[]>,
): ChildNode[] {
  return nodes.reduce<ChildNode[]>(
    /**
     Expands one node into the accumulator.
     
     @param result - Expanded nodes so far.
     
     @param node - Current node.
     
     @returns Updated accumulator.
     */
    function accumulate(
      result,
      node,
    ) {
      if ((node.type === 'atrule') && (node.name === 'apply')) {
        /**
         Referenced body, cloned per splice like the retired pipeline.
         */
        const body = registry.get(node.params
          .trim(),);
        if (body !== undefined) {
          result.push(
            ...oldExpandNodes(
              body.map(function cloneChild(child,) {
                return child.clone();
              },),
              registry,
            ),
          );
        }
        return result;
      }
      if (('nodes' in node) && Array.isArray(node.nodes,)) {
        /**
         Cloned container with expanded children.
         */
        const cloned = node.clone();
        cloned.nodes = oldExpandNodes(
          node.nodes
            .map(function cloneChild(child,) {
              return child.clone();
            },),
          registry,
        );
        result.push(cloned,);
        return result;
      }
      result.push(node.clone(),);
      return result;
    },
    [],
  );
}

/**
 Faithful replica of the retired postcss mixin pipeline: collect with
 clones, fixed-point nested expansion compared by full serialization, then
 clone-per-site document splicing.
 
 @param cssText - Consumer CSS.
 
 @param mixinCssText - Mixin definitions CSS.
 
 @returns Expanded CSS text.
 */
function oldPostcssPipeline({
  cssText,
  mixinCssText,
}: {
  readonly cssText: string;
  readonly mixinCssText: string;
},): string {
  /**
   Mixin registry, bodies cloned at collection like the retired pipeline.
   */
  const registry = new Map<string, readonly ChildNode[]>();
  /**
   Parsed definition sheet.
   */
  const mixinRoot = postcssParse(mixinCssText,);
  mixinRoot.walkAtRules(
    'mixin',
    function collect(node,) {
      registry.set(
        node.params
          .trim(),
        (node.nodes ?? []).map(function cloneChild(child,) {
          return child.clone();
        },),
      );
      node.remove();
    },
  );

  for (let pass = 0; pass < OLD_MAX_PASSES; pass += 1) {
    /**
     Whether any registry entry changed this pass, per serialization
     comparison like the retired pipeline.
     */
    const changed = [...registry,].reduce(
      /**
       Expands and compares one entry.
       
       @param changedSoFar - Whether a prior entry changed.
       
       @param entry - Name and body under expansion.
       
       @returns Whether any entry changed so far.
       */
      function expandEntry(
        changedSoFar: boolean,
        entry: readonly [string, readonly ChildNode[],],
      ) {
        const [name, nodes,] = entry;
        /**
         Freshly expanded body.
         */
        const expanded = oldExpandNodes(
          nodes,
          registry,
        );
        if (nodes.map(String,).join('',) !== expanded.map(String,).join('',)) {
          registry.set(
            name,
            expanded,
          );
          return true;
        }
        return changedSoFar;
      },
      false,
    );
    if (!changed)
      break;
  }

  /**
   Parsed consumer sheet.
   */
  const root = postcssParse(cssText,);
  root.walkAtRules(
    'apply',
    function splice(node,) {
      /**
       Expanded body for this site.
       */
      const body = registry.get(node.params
        .trim(),);
      if (body !== undefined) {
        node.replaceWith(
          ...body.map(function cloneChild(child,) {
            return child.clone();
          },),
        );
      }
    },
  );
  return root.toString();
}

//endregion Old-pipeline replica

//region Benchmarks

console.log(
  `mixins: ${String(MIXIN_COUNT,)} chained, apply sites: ${String(APPLY_COUNT,)}`,
);

summary(function mixinPipelineBenchmarks() {
  bench('css-edit expandCssMixins', function runNewPipeline() {
    do_not_optimize(expandCssMixins({
      css: CONSUMER_CSS,
      mixinCss: MIXIN_CSS,
    },),);
  },);

  bench('retired postcss pipeline', function runOldPipeline() {
    do_not_optimize(oldPostcssPipeline({
      cssText: CONSUMER_CSS,
      mixinCssText: MIXIN_CSS,
    },),);
  },);
},);

await run();

//endregion Benchmarks
