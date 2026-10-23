//region Direct key variables
// THE TWO ENVIRONMENT VARIABLES THE DIRECT PROVIDERS ARE KEYED BY, named once for the run, the budget sample and
// the tests. An absent one leaves its provider unconfigured and never borrows another provider's key.

/**
 Environment variable carrying DeepSeek's key, named without the package prefix because the owner's grant
 of 2026-10-22 names it so.
 */
export const DEEPSEEK_KEY_VARIABLE = 'DEEPSEEK_API_KEY';

/**
 Environment variable carrying Xiaomi's key.
 */
export const XIAOMI_KEY_VARIABLE = 'TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY';

/**
 Environment variable carrying the owner's Claude key (Anthropic's own API), granted on 2026-10-22 for Sonnet 5.5 and
 Haiku 5.5 only, at about $200 a month.
 */
export const CLAUDE_KEY_VARIABLE = 'TRANSLATION_REPAIR_CLAUDE_API_KEY';

//endregion Direct key variables
