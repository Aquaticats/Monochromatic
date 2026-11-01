import {
  eslintCompatPlugin,
  type Plugin,
} from '@oxlint/plugins';

import { argumentPerLine, } from './rule/argument-per-line.ts';
import { arrayElementPerLine, } from './rule/array-element-per-line.ts';
import { blockBodyNewline, } from './rule/block-body-newline.ts';
import { chainPerLine, } from './rule/chain-per-line.ts';
import { commaDangle, } from './rule/comma-dangle.ts';
import { destructurePerLine, } from './rule/destructure-per-line.ts';
import { exportPerLine, } from './rule/export-per-line.ts';
import { importPerLine, } from './rule/import-per-line.ts';
import { indentation, } from './rule/indentation.ts';
import { invocationDepthPerLine, } from './rule/invocation-depth-per-line.ts';
import { maxStatementsPerLine, } from './rule/max-statements-per-line.ts';
import { noMixedOperators, } from './rule/no-mixed-operators.ts';
import { objectPropertyPerLine, } from './rule/object-property-per-line.ts';
import { oneVarDeclarationPerLine, } from './rule/one-var-declaration-per-line.ts';
import { paramPerLine, } from './rule/param-per-line.ts';
import { requireAsteriskPrefix, } from './rule/require-asterisk-prefix.ts';
import { semi, } from './rule/semi.ts';
import { tuplePerLine, } from './rule/tuple-per-line.ts';
import { typePropertyPerLine, } from './rule/type-property-per-line.ts';

export type { ChainNode, } from './utility/chain.ts';
export { chainBreaks, } from './utility/chain-flatten.ts';
export {
  type ChainBreak,
  type ChainSegment,
  renderCanonical,
  selectBreaks,
} from './utility/chain-render.ts';
export {
  computeIndentExpectations,
  type ComputeIndentExpectationsParams,
  type IndentExpectationRoute,
  type LineIndentExpectation,
} from './utility/indent-expectation.ts';
export {
  closeValueOfOpenBracket,
} from './utility/indent-routes.ts';
export {
  baseIndentAt,
  leadingWhitespace,
} from './utility/indent.ts';
export type {
  ExpectationNode,
  ExpectationToken,
} from './utility/indent-tree.ts';

/**
 Oxlint JS plugin for TypeScript stylistic rules: indentation from each
 line's structural anchor, one-item-per-line formatting across
 multi-element constructs, readable brace-delimited bodies, semicolon
 enforcement, trailing comma enforcement, and explicit operator structure
 in nested expressions.
 
 The indentation rule replaces the TypeScript indentation dprint used to
 own: checked lines anchor at their structural base plus two columns,
 template-literal content stays exempt, and the autofix normalizes deviating
 lines in one pass.
 
 The per-line rules fire when 2 or more items share a source line and
 auto-fix by placing every item on its own line with consistent
 indentation. They are this repository's TypeScript layout authority
 because dprint's TypeScript formatter is disabled; dprint still formats
 non-TypeScript files.
 
 Statement-boundary rules enforce explicit semicolons and one statement
 or declarator per line. Body-boundary rules enforce readable newlines
 inside brace-delimited bodies.
 
 The expression-structure rules surface ambiguous operator precedence
 by requiring explicit parentheses at operator boundaries.
 
 @example
 ```typescript
 // oxlint.config.ts
 import { defineConfig } from 'oxlint';
 export default defineConfig({
   jsPlugins: ['\@monochromatic-dev/oxlint-plugin-stylistic'],
 });
 ```
 */
const plugin: Plugin = eslintCompatPlugin({
  meta: {
    name: 'stylistic',
  },
  rules: {
    //region Indentation: enforce two-space indentation from each line's structural anchor
    indentation,
    //endregion Indentation

    //region Per-line rules: enforce one item per line in multi-element constructs
    'param-per-line': paramPerLine,
    'argument-per-line': argumentPerLine,
    'array-element-per-line': arrayElementPerLine,
    'object-property-per-line': objectPropertyPerLine,
    'import-per-line': importPerLine,
    'export-per-line': exportPerLine,
    'type-property-per-line': typePropertyPerLine,
    'tuple-per-line': tuplePerLine,
    'destructure-per-line': destructurePerLine,
    //endregion Per-line rules

    //region Comment layout: enforce configured TSDoc body prefixes
    'require-asterisk-prefix': requireAsteriskPrefix,
    //endregion Comment layout

    //region Body boundaries: enforce readable newlines inside non-empty braces
    'block-body-newline': blockBodyNewline,
    //endregion Body boundaries

    //region Statement boundaries: enforce one-statement-per-line and one-declarator-per-line
    'one-var-declaration-per-line': oneVarDeclarationPerLine,
    'max-statements-per-line': maxStatementsPerLine,
    semi,
    'comma-dangle': commaDangle,
    //endregion Statement boundaries

    //region Expression structure: enforce explicit parens at operator boundaries and break chains across lines
    'no-mixed-operators': noMixedOperators,
    'chain-per-line': chainPerLine,
    'invocation-depth-per-line': invocationDepthPerLine,
    //endregion Expression structure
  },
},);

export default plugin;
