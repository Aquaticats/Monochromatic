import type { ModelCaller, } from './chat-contract.ts';

//region Provider callers
// THE TEXT CALL OF EACH CONFIGURED PROVIDER, in its own file since the fifth
// provider put `provider-router.ts` over the line budget.

/**
 Each configured provider's text call. A provider with no key has none: its
 meter reads dry (`UNCONFIGURED_METER`), so no route names it.

 @example
 ```ts
 const callers: ProviderCallers = { synthetic, hyper, };
 ```
 */
export type ProviderCallers = {
  /**
   Fifth provider's text call, when its key is set; first in spending order.
   */
  readonly neuralwatt?: Pick<ModelCaller, 'chatText'>;

  /**
   First provider's text call, when its key is set.
   */
  readonly synthetic?: Pick<ModelCaller, 'chatText'>;

  /**
   Second provider's text call, when its key is set.
   */
  readonly hyper?: Pick<ModelCaller, 'chatText'>;

  /**
   Fourth provider's text call, when its key is set.
   */
  readonly bedrock?: Pick<ModelCaller, 'chatText'>;

  /**
   DeepSeek's own API's text call, when its key is set; it serves one model.
   */
  readonly deepseek?: Pick<ModelCaller, 'chatText'>;

  /**
   Xiaomi's own API's text call, when its key is set; it serves one model.
   */
  readonly xiaomi?: Pick<ModelCaller, 'chatText'>;

  /**
   Anthropic's own API's text call, when the Claude key is set; it serves Sonnet 5.5 and Haiku 5.5.
   */
  readonly anthropic?: Pick<ModelCaller, 'chatText'>;

  /**
   Third provider's text call, when its key is set.
   */
  readonly openrouter?: Pick<ModelCaller, 'chatText'>;
};

//endregion Provider callers
