/**
 Mixin engine: collects `\@mixin` definitions, expands nested `\@apply`
 references between definitions, and splices bodies into documents.
 
 Internal to the package; consumers use `buildCss` or `expandCssMixins`.
 One visitor ({@link applyVisitor}) drives both nested-definition expansion
 and document expansion, and immutable css-edit nodes make body splices
 reference-shared instead of cloned.
 */
import {
  type CssAtRule,
  type CssNode,
  type CssStylesheet,
  type CssVisitor,
  isCssAtRule,
  rawTextOfTokens,
  transformNodes,
  transformStylesheet,
} from '@monochromatic-dev/css-core-edit/ts';
import {
  CircularCssMixinError,
  UnknownCssMixinError,
} from './errors.ts';

//region Registry type

/**
 Mixin definitions by name. Bodies are immutable css-edit nodes, so registry
 entries splice into documents by reference without cloning.
 */
export type CssMixinRegistry = ReadonlyMap<string, readonly CssNode[]>;

//endregion Registry type

//region Helpers

/**
 Trimmed prelude text of an at-rule, used as the mixin identifier for both
 `\@mixin` definitions and `\@apply` references.
 
 @param node - At-rule carrying the identifier.
 
 @returns Trimmed prelude text; empty when the at-rule has no prelude.
 */
function atRuleParam(node: CssAtRule,): string {
  return rawTextOfTokens({ tokens: node.preludeTokens, },)
    .trim();
}

/**
 Whether a node list has any non-trivia content; a mixin body of pure
 whitespace counts as missing.
 
 @param nodes - Candidate body nodes.
 
 @returns Whether structural content exists.
 */
function hasStructuralContent(nodes: readonly CssNode[],): boolean {
  return nodes.some(function isStructural(node,) {
    return node.kind !== 'trivia';
  },);
}

/**
 Builds the shared `\@apply`-splicing visitor: every `\@apply` at-rule is
 replaced by whatever the resolver returns for its name; all other nodes
 pass through. Used for nested expansion inside definitions and for
 document expansion, so reference resolution differs but splicing never does.
 
 @param resolve - Maps a referenced mixin name to its replacement body.
 
 @returns Visitor for css-edit transforms.
 */
function applyVisitor({
  resolve,
}: {
  readonly resolve: (name: string,) => readonly CssNode[];
},): CssVisitor {
  return function expandApplyNode(node,) {
    if ((!isCssAtRule(node,)) || (node.name !== 'apply'))
      return node;

    /**
     Referenced mixin name from the apply prelude.
     */
    const mixinName = atRuleParam(node,);
    if (!mixinName)
      throw new Error('Mixin name is required: @apply --name;',);

    return resolve(mixinName,);
  };
}

//endregion Helpers

//region Collection

/**
 Collects `\@mixin` definitions out of a stylesheet: definitions land in the
 returned registry and disappear from the returned tree (leading blank space
 pruned, comments kept).
 
 @param root - Parsed stylesheet to strip.
 
 @returns Stylesheet without definitions plus the collected registry.
 
 @throws When a `\@mixin` has no name or no body (definitions require content).
 
 @example
 ```ts
 const { root, mixins, } = collectMixins({ root: parsed.root, },);
 mixins.has('--card',); // => true
 ```
 */
export function collectMixins({
  root,
}: {
  readonly root: CssStylesheet;
},): {
  readonly root: CssStylesheet;
  readonly mixins: CssMixinRegistry;
} {
  /**
   Definitions found during the strip pass.
   */
  const collected = new Map<string, readonly CssNode[]>();

  /**
   Stylesheet with every definition removed.
   */
  const stripped = transformStylesheet({
    root,
    visit: function collectMixinNode(node,) {
      if ((!isCssAtRule(node,)) || (node.name !== 'mixin'))
        return node;

      /**
       Mixin identifier from the definition prelude.
       */
      const mixinName = atRuleParam(node,);
      if (!mixinName)
        throw new Error('@mixin requires a name: @mixin --name { ... }',);

      if ((node.block === undefined)
        || (!hasStructuralContent(node.block
          .children,)))
        throw new Error('mixin definition must include body',);

      collected.set(
        mixinName,
        node.block
          .children,
      );
      return [];
    },
    pruneTriviaBeforeRemoved: true,
  },);

  return {
    root: stripped,
    mixins: collected,
  };
}

//endregion Collection

//region Expansion

/**
 Expands nested `\@apply` references inside every mixin body, returning a
 registry whose entries contain no `\@apply` at-rules.
 
 Definition-chain recursion with an explicit trail: each mixin expands
 exactly once (memoized), and a cycle surfaces as the exact reference chain
 instead of a pass-count heuristic.
 
 @param mixins - Raw registry from {@link collectMixins}.
 
 @returns Registry with fully expanded bodies.
 
 @throws UnknownCssMixinError when a nested `\@apply` references an
 unregistered mixin.
 
 @throws CircularCssMixinError when definitions reference each other in a
 cycle.
 
 @example
 ```ts
 const expanded = expandMixinRegistry({ mixins, },);
 ```
 */
export function expandMixinRegistry({
  mixins,
}: {
  readonly mixins: CssMixinRegistry;
},): CssMixinRegistry {
  /**
   Fully expanded bodies, filled on demand.
   */
  const expanded = new Map<string, readonly CssNode[]>();

  /**
   Expands one mixin body along a definition trail.
   
   @param name - Mixin under expansion.
   
   @param trail - Definition names already on the recursion path.
   
   @returns Fully expanded body nodes.
   
   @throws UnknownCssMixinError for unregistered references.
   
   @throws CircularCssMixinError when name already sits on the trail.
   */
  function expandName({
    name,
    trail,
  }: {
    readonly name: string;
    readonly trail: readonly string[];
  },): readonly CssNode[] {
    /**
     Previously expanded body for this name.
     */
    const memoized = expanded.get(name,);
    if (memoized !== undefined)
      return memoized;

    if (trail.includes(name,))
      throw new CircularCssMixinError({
        trail: [
          ...trail,
          name,
        ],
      },);

    /**
     Raw body as collected from the definition.
     */
    const body = mixins.get(name,);
    if (body === undefined)
      throw new UnknownCssMixinError({
        message: `Unknown mixin referenced in nested @apply: ${name}`,
        mixinName: name,
      },);

    /**
     Body with nested references spliced in.
     */
    const result = transformNodes({
      nodes: body,
      visit: applyVisitor({
        resolve: function resolveNested(referenced,) {
          return expandName({
            name: referenced,
            trail: [
              ...trail,
              name,
            ],
          },);
        },
      },),
      pruneTriviaBeforeRemoved: true,
    },);

    expanded.set(
      name,
      result,
    );
    return result;
  }

  for (const name of mixins.keys()) {
    expandName({
      name,
      trail: [],
    },);
  }

  return expanded;
}

/**
 Replaces every `\@apply` in a document with the referenced mixin body.
 
 @param root - Stylesheet containing `\@apply` references.
 
 @param mixins - Registry, normally pre-expanded by {@link expandMixinRegistry}.
 
 @returns Stylesheet with references replaced.
 
 @throws When an `\@apply` carries no mixin name.
 
 @throws UnknownCssMixinError when a reference is not in the registry.
 
 @example
 ```ts
 const finalRoot = expandApplyRules({ root, mixins: expanded, },);
 ```
 */
export function expandApplyRules({
  root,
  mixins,
}: {
  readonly root: CssStylesheet;
  readonly mixins: CssMixinRegistry;
},): CssStylesheet {
  return transformStylesheet({
    root,
    visit: applyVisitor({
      resolve: function resolveFromRegistry(mixinName,) {
        /**
         Stored body nodes for the referenced mixin.
         */
        const body = mixins.get(mixinName,);
        if (body === undefined)
          throw new UnknownCssMixinError({
            message: `Unknown mixin: ${mixinName}`,
            mixinName,
          },);
        return body;
      },
    },),
    pruneTriviaBeforeRemoved: true,
  },);
}

//endregion Expansion
