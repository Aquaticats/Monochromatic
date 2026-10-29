import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  Fix,
  Fixer,
  SourceCode,
  Span,
  VisitorWithHooks,
} from '@oxlint/plugins';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  type ChainNode,
  effectiveTop,
  hasReflowableComment,
  isChainRoot,
} from '../utility/chain.ts';
import { chainBreaks, } from '../utility/chain-flatten.ts';
import { renderCanonical, } from '../utility/chain-render.ts';
import { baseIndentAt, } from '../utility/indent.ts';

/**
 Per-file cache of full source text, keyed by the file's `SourceCode` identity.
 */
const sourceTextCache = new WeakMap<SourceCode, string>();

/**
 Returns the file's full source text, computing it once per file.
 
 `createOnce` reuses one visitor across files, and the visitor fires on every
 member, call, binary, and logical node, so recomputing `getText()` per
 invocation is wasteful. Keying the cache on `SourceCode` identity refreshes
 it exactly when oxlint moves to the next file. The parameter is the rule
 {@link Context} rather than its `sourceCode` directly because oxlint's `SourceCode`
 is an anonymous type the readonly-params allow-list cannot name-match.
 
 @param context - rule context whose `sourceCode` describes the current file
 
 @returns full source text of the file `context.sourceCode` describes
 */
function sourceTextOf(context: ForeignBorrowed<Context>,): string {
  /**
   Source code accessor identifying the current file.
   */
  const { sourceCode, } = context;
  /**
   Cached text for this file, if already computed.
   */
  const cached = sourceTextCache.get(sourceCode,);
  if (cached !== undefined)
    return cached;
  /**
   Full source text, computed once and memoised against this `SourceCode`.
   */
  const text = sourceCode.getText();
  sourceTextCache.set(
    sourceCode,
    text,
  );
  return text;
}

/**
 Enforces one chain segment per source line for binary, logical, member, and
 call chains, laying out the operator and member axes independently.
 
 Firing once on the outermost chain root, the rule computes break offsets on
 decoupled axes: a member or call chain breaks on its own member-step count,
 and an operator chain breaks on its own operator count, so neither axis
 inflates the other. A member chain keeps the leaf and the first member step on the head line and
 breaks every later step; an operator chain keeps the source-first operator on
 the head line and breaks the rest. Each continuation line indents two columns
 past the line carrying its chain's head, so a member operand that broke onto
 an operator's line nests one level deeper. A single operator
 whose operand is a member access (`a.b === c`) therefore stays on one line,
 while a multi-step member operand breaks and carries its operator onto a line
 of its own (`a.b.c > 0` becomes `a.b` then `.c` then `> 0`). Computed access
 (`[expr]`) and call steps (`(args)`) stay attached, so `arr[0][1]` and
 `obj.method()` keep to one line. It reports when the region's source differs
 from this canonical layout and replaces the whole region in one fix, except
 when a comment sits in the collapsible head (before the first break), where it
 reports without a fix; a comment at or after the first break (such as one in a
 trailing call's arguments) rides verbatim on its continuation slice, so the
 fix still applies. The fix only inserts newlines at break offsets and slices
 everything else verbatim, so it never collapses whitespace at non-break
 points: a legacy split with no break offsets is left as-is rather than
 rejoined.
 `no-mixed-operators` runs alongside and remains the authority on precedence
 parentheses; on a shared region their fixes need two `oxlint --fix` passes (an
 upstream single-pass limitation).
 
 @example
 ```ts
 // Bad
 const r1 = a + b + c;
 const r2 = items.map(toName).filter(isReady).sort();
 
 // Good
 const r1 = a + b
   + c;
 const r2 = items.map(toName)
   .filter(isReady)
   .sort();
 ```
 */
export const chainPerLine: CreateOnceRule = {
  meta: {
    type: 'layout',
    fixable: 'whitespace',
    docs: {
      description:
        'Require one chain segment per source line for binary, logical, member, and call chains.',
      recommended: true,
    },
    messages: {
      chain:
        'Put each operator, member, or method step in this chain on its own line.',
    },
  },
  /**
   Handles effectful plugin callback.
   
   @param context - Foreign callback value carrying diagnostic capability.
   
   @mutates context - Emits Oxlint diagnostics through foreign rule context.
   
   @example
   ```ts
   createOnce(context);
   ```
   */
  createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
    /**
     Visitor entry for every chain-capable node. Bails unless the node is the
     outermost root, then reports when its layout is not canonical.
     
     @param node - candidate `MemberExpression`, `CallExpression`,
       `BinaryExpression`, or `LogicalExpression`
     */
    function check(node: ForeignBorrowed<Span>,): void {
      /* oxlint-disable typescript/no-unsafe-type-assertion -- visitor nodes always carry the type/object/callee/left/right/operator/parent fields ChainNode reads; oxlint types them only as bare Span */
      /**
       Node narrowed to the structural view the chain walk reads.
       */
      const root = node as ChainNode;
      /* oxlint-enable typescript/no-unsafe-type-assertion */
      if (!isChainRoot({
        context,
        node: root,
      },)) {
        return;
      }
      /**
       Breaks that begin a continuation line; empty when the chain fits on one line.
       */
      const breaks = chainBreaks({
        context,
        node: root,
      },);
      if (breaks.length
        === 0)
        return;
      /**
       Full source text, cached per file.
       */
      const sourceText = sourceTextOf(context,);
      /**
       Outermost region node, past any trailing `!`/`as`/`satisfies` wrapper.
       */
      const top = effectiveTop(root,);
      /**
       Byte offset where the chain region begins.
       */
      const regionStart = root.start;
      /**
       Byte offset where the chain region ends, past trailing wrapper text.
       */
      const regionEnd = top.end;
      /**
       Head indent: the head line's own indentation, which every continuation
       nests past according to its anchor's line.
       */
      const headIndent = baseIndentAt({
        sourceText,
        offset: regionStart,
      },);
      /**
       Canonical multi-line layout of the region.
       */
      const canonical = renderCanonical({
        sourceText,
        regionStart,
        regionEnd,
        breaks,
        headIndent,
      },);
      if (sourceText.slice(
        regionStart,
        regionEnd,
      )
        === canonical) {
        return;
      }
      /**
       First break offset; defined because an empty `breaks` returned above.
       */
      const firstBreak = nonNullishOrThrow(breaks[0],)
        .breakOffset;
      /**
       Whether the region is free of comments the render would reflow.
       */
      const fixable = !hasReflowableComment({
        context,
        node: top,
        firstBreak,
      },);
      context.report({
        node,
        messageId: 'chain',
        ...fixable
          ? {
            fix(fixer: ForeignBorrowed<Fixer>,): Fix {
              return fixer.replaceTextRange(
                [
                  regionStart,
                  regionEnd,
                ],
                canonical,
              );
            },
          }
          : {},
      },);
    }

    return {
      BinaryExpression: check,
      LogicalExpression: check,
      MemberExpression: check,
      CallExpression: check,
    };
  },
};
