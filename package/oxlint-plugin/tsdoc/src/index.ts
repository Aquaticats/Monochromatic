import {
  eslintCompatPlugin,
  type Plugin,
} from '@oxlint/plugins';

import { checkMutates, } from './rule/mutates.ts';
import {
  checkParamNames,
  requireParam,
  requireParamDescription,
  requireParamName,
} from './rule/params.ts';
import { requireExample, } from './rule/require-example.ts';
import { requireTsdoc, } from './rule/require-tsdoc.ts';
import {
  requireReturns,
  requireReturnsCheck,
  requireReturnsDescription,
} from './rule/returns.ts';
import {
  checkAlignment,
  emptyTags,
  escapeInlineTags,
  multilineBlocks,
  noMultiAsterisks,
  tagLines,
} from './rule/structural.ts';
import {
  checkAccess,
  checkTagNames,
  noTypes,
  validTypes,
} from './rule/tag-validation.ts';
import {
  requireYields,
  requireYieldsCheck,
} from './rule/yields.ts';

/**
 Oxlint JS plugin providing TSDoc validation rules.
 
 Ports the eslint-plugin-jsdoc recommended-typescript ruleset to the oxlint
 jsPlugin API, adapting all rules to conform to TSDoc (not JSDoc) specs.
 Uses an in-house TSDoc comment scanner for parsing, with no external
 dependency on `\@microsoft/tsdoc`.
 
 @example
 ```typescript
 // oxlint.config.ts
 import { defineConfig } from 'oxlint';
 export default defineConfig({
   jsPlugins: ['\@monochromatic-dev/oxlint-plugin-tsdoc'],
 });
 ```
 */
const plugin: Plugin = eslintCompatPlugin({
  meta: {
    name: 'tsdoc',
  },
  rules: {
    //region Presence
    'require-tsdoc': requireTsdoc,
    'require-example': requireExample,
    //endregion Presence

    //region Structural formatting
    'check-alignment': checkAlignment,
    'multiline-blocks': multilineBlocks,
    'no-multi-asterisks': noMultiAsterisks,
    'tag-lines': tagLines,
    'empty-tags': emptyTags,
    'escape-inline-tags': escapeInlineTags,
    //endregion Structural formatting

    //region Tag validation
    'check-tag-names': checkTagNames,
    'check-access': checkAccess,
    'valid-types': validTypes,
    'no-types': noTypes,
    //endregion Tag validation

    //region Parameter documentation
    'check-param-names': checkParamNames,
    'require-param': requireParam,
    'require-param-name': requireParamName,
    'require-param-description': requireParamDescription,
    //endregion Parameter documentation

    //region Mutation contracts
    'check-mutates': checkMutates,
    //endregion Mutation contracts

    //region Return documentation
    'require-returns': requireReturns,
    'require-returns-check': requireReturnsCheck,
    'require-returns-description': requireReturnsDescription,
    //endregion Return documentation

    //region Yield documentation
    'require-yields': requireYields,
    'require-yields-check': requireYieldsCheck,
    //endregion Yield documentation
  },
},);

export default plugin;

//region Built-artifact exports
// The scanners and per-rule helpers this package's own unit tests exercise. A
// test reaches this package through the artifact it ships, which
// `test-import/require-eventual-artifact` enforces, so every symbol such a test
// needs must be reachable from the package entry. None of them carries a
// compatibility promise.

/**
 Structural messages one comment yields, exposed for built-artifact verification.

 @internal
 */
export { collectStructuralMessages, } from './tsdoc-structural-messages.ts';

/**
 TSDoc comment parsing, exposed for built-artifact verification.

 @internal
 */
export { parseTsdocComment, } from './tsdoc-comments.ts';

/**
 Comment block splitting, exposed for built-artifact verification.

 @internal
 */
export { splitDocComment, } from './tsdoc-blocks.ts';

/**
 File-extension filtering, exposed for built-artifact verification.

 @internal
 */
export {
  IGNORED_EXTENSIONS,
  shouldIgnoreFile,
} from './tsdoc-utils.ts';

/**
 Tag and inline-code reading, exposed for built-artifact verification.

 @internal
 */
export {
  collectTags,
  stripInlineCodeSpans,
} from './comment-text.ts';

/**
 Type-annotation reading, exposed for built-artifact verification.

 @internal
 */
export { findTypeAnnotations, } from './rule/type-annotations.ts';

/**
 Leading-tag extraction, exposed for built-artifact verification.

 @internal
 */
export {
  extractLeadingTag,
  NO_LEADING_TAG,
} from './rule/structural-tags.ts';

/**
 Access-tag detection, exposed for built-artifact verification.

 @internal
 */
export { containsBoundedAccessTag, } from './rule/tag-validation.ts';

/**
 Comment-close escaping, exposed for built-artifact verification.

 @internal
 */
export { hasUnescapedCommentClose, } from './rule/tag-escaping.ts';

/**
 Tagged-line parsing, exposed for built-artifact verification.

 @internal
 */
export {
  parseTaggedLine,
  UNTAGGED_LINE,
} from './rule/empty-tags.ts';

//endregion Built-artifact exports
