import type { CreateOnceRule, } from '@oxlint/plugins';

import { banDisableRule, } from './_ban-disable-factory.ts';

/**
 Bans inline suppression of `no-non-null-assertion`
 (including `typescript/no-non-null-assertion` and
 `typescript-eslint/no-non-null-assertion` variants), built via
 {@link banDisableRule}.
 
 Use `nonNullishOrThrow()` from `\@monochromatic-dev/or-throw-core`
 or an explicit null check instead of the `!` postfix operator.
 */
export const noDisableNoNonNullAssertion: CreateOnceRule = banDisableRule({
  ruleId: 'no-non-null-assertion',
  description:
    'Disallow disabling no-non-null-assertion. Use nonNullishOrThrow() instead.',
  message:
    'Disabling no-non-null-assertion is not allowed. Use nonNullishOrThrow() or an explicit null check.',
},);
