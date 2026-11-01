import {
  eslintCompatPlugin,
  type Plugin,
} from '@oxlint/plugins';

import { noArrowFunction, } from './rule/no-arrow-function.ts';
import {
  noArrayCallbackReference,
} from './rule/no-array-callback-reference.ts';
import { noClass, } from './rule/no-class.ts';
import { noDisableMaxLines, } from './rule/no-disable-max-lines.ts';
import { noDisableNoArrowFunction, } from './rule/no-disable-no-arrow-function.ts';
import { noDisableNoEnum, } from './rule/no-disable-no-enum.ts';
import { noDisableNoForIn, } from './rule/no-disable-no-for-in.ts';
import { noDisableNoHasownproperty, } from './rule/no-disable-no-hasownproperty.ts';
import { noDisableNoMisusedPromises, } from './rule/no-disable-no-misused-promises.ts';
import {
  noDisableNoNonNullAssertion,
} from './rule/no-disable-no-non-null-assertion.ts';
import { noDisableNoPromiseCatch, } from './rule/no-disable-no-promise-catch.ts';
import { noDisableNoPromiseFinally, } from './rule/no-disable-no-promise-finally.ts';
import { noDisableNoRestParams, } from './rule/no-disable-no-rest-params.ts';
import { noDisableNoSwitch, } from './rule/no-disable-no-switch.ts';
import { noDisableNoTrimLeftRight, } from './rule/no-disable-no-trim-left-right.ts';
import { noDisableNoTryFinally, } from './rule/no-disable-no-try-finally.ts';
import { noDisableNoUselessReturn, } from './rule/no-disable-no-useless-return.ts';
import {
  noDisableNoVariableFunctionExpression,
} from './rule/no-disable-no-variable-function-expression.ts';
import { noDisablePreferReadonlyParameterTypes, } from './rule/no-disable-prefer-readonly-parameter-types.ts';
import { noDisablePreferRegexpExec, } from './rule/no-disable-prefer-regexp-exec.ts';
import {
  noDisableRequireDestructuredParams,
} from './rule/no-disable-require-destructured-params.ts';
import { noDisableRequireReturns, } from './rule/no-disable-require-returns.ts';
import { noDisableRequireTsdoc, } from './rule/no-disable-require-tsdoc.ts';
import { noEnum, } from './rule/no-enum.ts';
import { noForIn, } from './rule/no-for-in.ts';
import { noFunctionRootLet, } from './rule/no-function-root-let.ts';
import { noHasownproperty, } from './rule/no-hasownproperty.ts';
import { noImmediateMutation, } from './rule/no-immediate-mutation.ts';
import {
  noLowInformationSymbolDescription,
} from './rule/no-low-information-symbol-description/index.ts';
import { noModuleRootLet, } from './rule/no-module-root-let.ts';
import { noNullishUnion, } from './rule/no-nullish-union.ts';
import { catchBinding, } from './rule/catch-binding.ts';
import { noOptionalEscape, } from './rule/no-optional-escape.ts';
import { noPromiseCatch, } from './rule/no-promise-catch.ts';
import { noPromiseFinally, } from './rule/no-promise-finally.ts';
import { noRegex, } from './rule/no-regex.ts';
import { noRestParams, } from './rule/no-rest-params.ts';
import { noSwitch, } from './rule/no-switch.ts';
import { noSync, } from './rule/no-sync.ts';
import { noSystemTmpdir, } from './rule/no-system-tmpdir.ts';
import { noTrimLeftRight, } from './rule/no-trim-left-right.ts';
import { noTryFinally, } from './rule/no-try-finally.ts';
import { noUselessSpread, } from './rule/no-useless-spread/index.ts';
import {
  noVariableFunctionExpression,
} from './rule/no-variable-function-expression.ts';
import {
  preferDescribeFunctionRefName,
} from './rule/prefer-describe-function-ref-name.ts';
import { preferCaughtValueText, } from './rule/prefer-caught-value-text.ts';
import { preferErrorIsError, } from './rule/prefer-error-is-error.ts';
import { preferSpread, } from './rule/prefer-spread/index.ts';
import { requireDestructuredParams, } from './rule/require-destructured-params.ts';
import { requireQueryselectorGeneric, } from './rule/require-queryselector-generic.ts';
import { startSpreadEvidenceBridge, } from './rule/spread-evidence/bridge-start.ts';

/* The spread rules need the TypeScript bridge, whose native child cannot be spawned once Oxlint has reserved
 * address space for its workers on a host with many cores. Plugins load before the workers start, so the child
 * starts here and a failure is kept for the first spread rule that needs a type to report. */
startSpreadEvidenceBridge();

/**
 Oxlint JS plugin implementing `no-restricted-syntax` rules
 that oxlint does not support natively.
 
 Oxlint lacks ESLint's `no-restricted-syntax` rule because it requires
 a full AST selector engine. This plugin provides individual rules
 for each banned syntax pattern instead.
 
 Also includes `no-disable-*` rules that prevent inline `oxlint-disable`
 comments from suppressing specific rules. These enforce that certain
 conventions cannot be sidestepped with disable comments.
 
 @example
 ```typescript
 // oxlint.config.ts
 import { defineConfig } from 'oxlint';
 export default defineConfig({
   jsPlugins: ['\@monochromatic-dev/oxlint-plugin-no-restricted-syntax'],
 });
 ```
 */
const plugin: Plugin = eslintCompatPlugin({
  meta: {
    name: 'no-restricted-syntax',
  },
  rules: {
    //region Syntax rules
    'no-arrow-function': noArrowFunction,
    'no-array-callback-reference': noArrayCallbackReference,
    'no-class': noClass,
    'no-enum': noEnum,
    'no-for-in': noForIn,
    'no-function-root-let': noFunctionRootLet,
    'no-hasownproperty': noHasownproperty,
    'no-immediate-mutation': noImmediateMutation,
    'no-low-information-symbol-description': noLowInformationSymbolDescription,
    'no-module-root-let': noModuleRootLet,
    'no-nullish-union': noNullishUnion,
    'catch-binding': catchBinding,
    'no-optional-escape': noOptionalEscape,
    'no-promise-catch': noPromiseCatch,
    'no-promise-finally': noPromiseFinally,
    'no-regex': noRegex,
    'no-rest-params': noRestParams,
    'no-switch': noSwitch,
    'no-sync': noSync,
    'no-system-tmpdir': noSystemTmpdir,
    'no-trim-left-right': noTrimLeftRight,
    'no-try-finally': noTryFinally,
    'no-useless-spread': noUselessSpread,
    'no-variable-function-expression': noVariableFunctionExpression,
    'prefer-describe-function-ref-name': preferDescribeFunctionRefName,
    'prefer-caught-value-text': preferCaughtValueText,
    'prefer-error-is-error': preferErrorIsError,
    'prefer-spread': preferSpread,
    'require-destructured-params': requireDestructuredParams,
    'require-queryselector-generic': requireQueryselectorGeneric,
    //endregion Syntax rules

    //region Ban-disable rules -- prevent inline oxlint-disable for specific rules
    'no-disable-max-lines': noDisableMaxLines,
    'no-disable-no-arrow-function': noDisableNoArrowFunction,
    'no-disable-no-enum': noDisableNoEnum,
    'no-disable-no-for-in': noDisableNoForIn,
    'no-disable-no-hasownproperty': noDisableNoHasownproperty,
    'no-disable-no-misused-promises': noDisableNoMisusedPromises,
    'no-disable-no-promise-catch': noDisableNoPromiseCatch,
    'no-disable-no-promise-finally': noDisableNoPromiseFinally,
    'no-disable-no-rest-params': noDisableNoRestParams,
    'no-disable-no-switch': noDisableNoSwitch,
    'no-disable-no-trim-left-right': noDisableNoTrimLeftRight,
    'no-disable-no-try-finally': noDisableNoTryFinally,
    'no-disable-no-useless-return': noDisableNoUselessReturn,
    'no-disable-no-variable-function-expression': noDisableNoVariableFunctionExpression,
    'no-disable-no-non-null-assertion': noDisableNoNonNullAssertion,
    'no-disable-prefer-readonly-parameter-types': noDisablePreferReadonlyParameterTypes,
    'no-disable-prefer-regexp-exec': noDisablePreferRegexpExec,
    'no-disable-require-destructured-params': noDisableRequireDestructuredParams,
    'no-disable-require-returns': noDisableRequireReturns,
    'no-disable-require-tsdoc': noDisableRequireTsdoc,
    //endregion Ban-disable rules
  },
},);

export default plugin;

/**
 Exposed so built-artifact tests reach the export-surface classifier behind
 `prefer-describe-function-ref-name`.
 
 @internal
 */
export {
  classifyNameInSourceText,
  type ExportedNameKind,
  isAllCapsSnakeName,
} from './rule/prefer-describe-function-ref-name.export-scan.ts';
