/**
 Pi-ai builtin `qwen-token-plan-individual` catalog rows, copied verbatim as
 the golden fixture the shared listing core's thinking-level expansion pins
 against. @module

 Provenance: pi-ai 1.5.2
 `package/pi-plugin/neuralwatt/node_modules/@earendil-works/pi-ai/dist/providers/data/qwen-token-plan-individual.json`,
 object key `openai-completions`, row keys `chat:<id>`, extracted 2026-11-11.
 Every field the source rows carry travels here (`inputLimits`, `cost`,
 `type`, `api`, `baseUrl`, `provider`, ...) so the pi plugin can reuse
 this fixture for full-row equality against its own projection
 (`@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/test-fixtures`).
 */

//region Row shapes

/**
 Image resize limits one builtin row carries for its vision input.
 */
export type PiAiQwenTokenPlanIndividualCatalogInputLimits = {
  readonly images: {
    readonly resize: {
      readonly maxWidth: number;
      readonly maxHeight: number;
      readonly maxBytes: number;
      readonly jpegQuality: number;
    };
  };
};

/* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors pi-ai's `thinkingLevelMap` value shape (`@earendil-works/pi-ai` `providers/model-schema.ts`), where `null` is not absence but "level unsupported", so the golden comparison sees both spellings */
/**
 One builtin catalog row exactly as pi-ai 1.5.2 ships it: every field the
 source row carries, in the source field order, so a projected row can compare
 equal to it value for value.
 */
export type PiAiQwenTokenPlanIndividualCatalogRow = {
  readonly id: string;
  readonly name: string;
  readonly api: string;
  readonly provider: string;
  readonly baseUrl: string;
  readonly reasoning: boolean;
  readonly input: readonly string[];
  readonly cost: {
    readonly input: number;
    readonly output: number;
    readonly cacheRead: number;
    readonly cacheWrite: number;
  };
  readonly contextWindow: number;
  readonly maxTokens: number;
  readonly compat: {
    readonly supportsStrictMode: boolean;
    readonly thinkingFormat: string;
    readonly supportsDeveloperRole: boolean;
    readonly supportsStore: boolean;
    readonly supportsReasoningEffort: boolean;
  };
  readonly thinkingLevelMap?: {
    readonly off?: string | null;
    readonly minimal?: string | null;
    readonly low?: string | null;
    readonly medium?: string | null;
    readonly high?: string | null;
    readonly xhigh?: string | null;
    readonly max?: string | null;
  };
  readonly inputLimits?: PiAiQwenTokenPlanIndividualCatalogInputLimits;
  readonly type: string;
};
/* oxlint-enable no-restricted-syntax/no-nullish-union */

//endregion Row shapes

//region Public API

/**
 The 10 builtin `qwen-token-plan-individual` rows by their catalog key
 (`chat:<id>`), verbatim from the pi-ai 1.5.2 catalog JSON.
 */
export const piAiQwenTokenPlanIndividualCatalog: Readonly<Record<string, PiAiQwenTokenPlanIndividualCatalogRow>> = {
  'chat:deepseek-v4-flash-0731': {
    id: 'deepseek-v4-flash-0731',
    name: 'DeepSeek V4 Flash 0731',
    api: 'openai-completions',
    provider: 'qwen-token-plan-individual',
    baseUrl: 'https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1',
    reasoning: true,
    input: [
      'text',
    ],
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: 1_000_000,
    maxTokens: 384_000,
    compat: {
      supportsStrictMode: true,
      thinkingFormat: 'qwen',
      supportsDeveloperRole: false,
      supportsStore: false,
      supportsReasoningEffort: true,
    },
    thinkingLevelMap: {
      off: null,
      minimal: null,
      low: null,
      medium: null,
      high: 'high',
      xhigh: null,
      max: 'max',
    },
    type: 'chat',
  },
  'chat:deepseek-v4-pro': {
    id: 'deepseek-v4-pro',
    name: 'DeepSeek V4 Pro',
    api: 'openai-completions',
    provider: 'qwen-token-plan-individual',
    baseUrl: 'https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1',
    reasoning: true,
    input: [
      'text',
    ],
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: 1_000_000,
    maxTokens: 384_000,
    compat: {
      supportsStrictMode: true,
      thinkingFormat: 'qwen',
      supportsDeveloperRole: false,
      supportsStore: false,
      supportsReasoningEffort: true,
    },
    thinkingLevelMap: {
      off: null,
      minimal: null,
      low: null,
      medium: null,
      high: 'high',
      xhigh: null,
      max: 'max',
    },
    type: 'chat',
  },
  'chat:deepseek-v4-pro-0813': {
    id: 'deepseek-v4-pro-0813',
    name: 'DeepSeek V4 Pro 0813',
    api: 'openai-completions',
    provider: 'qwen-token-plan-individual',
    baseUrl: 'https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1',
    reasoning: true,
    input: [
      'text',
    ],
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: 1_000_000,
    maxTokens: 384_000,
    compat: {
      supportsStrictMode: true,
      thinkingFormat: 'qwen',
      supportsDeveloperRole: false,
      supportsStore: false,
      supportsReasoningEffort: true,
    },
    thinkingLevelMap: {
      off: null,
      minimal: null,
      low: null,
      medium: null,
      high: 'high',
      xhigh: null,
      max: 'max',
    },
    type: 'chat',
  },
  'chat:glm-5.2': {
    id: 'glm-5.2',
    name: 'GLM-5.2',
    api: 'openai-completions',
    provider: 'qwen-token-plan-individual',
    baseUrl: 'https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1',
    reasoning: true,
    input: [
      'text',
    ],
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: 1_000_000,
    maxTokens: 131_072,
    compat: {
      supportsStrictMode: true,
      thinkingFormat: 'qwen',
      supportsDeveloperRole: false,
      supportsStore: false,
      supportsReasoningEffort: true,
    },
    thinkingLevelMap: {
      off: null,
      minimal: null,
      low: null,
      medium: null,
      high: 'high',
      xhigh: null,
      max: 'max',
    },
    type: 'chat',
  },
  'chat:glm-5.3': {
    id: 'glm-5.3',
    name: 'GLM-5.3',
    api: 'openai-completions',
    provider: 'qwen-token-plan-individual',
    baseUrl: 'https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1',
    reasoning: true,
    input: [
      'text',
    ],
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: 1_000_000,
    maxTokens: 131_072,
    compat: {
      supportsStrictMode: true,
      thinkingFormat: 'qwen',
      supportsDeveloperRole: false,
      supportsStore: false,
      supportsReasoningEffort: true,
    },
    thinkingLevelMap: {
      off: null,
      minimal: null,
      low: 'low',
      medium: null,
      high: 'high',
      xhigh: null,
      max: 'max',
    },
    type: 'chat',
  },
  'chat:qwen3.6-flash': {
    id: 'qwen3.6-flash',
    name: 'Qwen3.6 Flash',
    api: 'openai-completions',
    provider: 'qwen-token-plan-individual',
    baseUrl: 'https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1',
    reasoning: true,
    input: [
      'text',
      'image',
    ],
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: 1_000_000,
    maxTokens: 65_536,
    compat: {
      supportsStrictMode: true,
      thinkingFormat: 'qwen',
      supportsDeveloperRole: false,
      supportsStore: false,
      supportsReasoningEffort: false,
    },
    inputLimits: {
      images: {
        resize: {
          maxWidth: 2_000,
          maxHeight: 2_000,
          maxBytes: 4_718_592,
          jpegQuality: 80,
        },
      },
    },
    type: 'chat',
  },
  'chat:qwen3.7-max': {
    id: 'qwen3.7-max',
    name: 'Qwen3.7 Max',
    api: 'openai-completions',
    provider: 'qwen-token-plan-individual',
    baseUrl: 'https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1',
    reasoning: true,
    input: [
      'text',
    ],
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: 1_000_000,
    maxTokens: 131_072,
    compat: {
      supportsStrictMode: true,
      thinkingFormat: 'qwen',
      supportsDeveloperRole: false,
      supportsStore: false,
      supportsReasoningEffort: false,
    },
    type: 'chat',
  },
  'chat:qwen3.7-plus': {
    id: 'qwen3.7-plus',
    name: 'Qwen3.7 Plus',
    api: 'openai-completions',
    provider: 'qwen-token-plan-individual',
    baseUrl: 'https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1',
    reasoning: true,
    input: [
      'text',
      'image',
    ],
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: 1_000_000,
    maxTokens: 65_536,
    compat: {
      supportsStrictMode: true,
      thinkingFormat: 'qwen',
      supportsDeveloperRole: false,
      supportsStore: false,
      supportsReasoningEffort: false,
    },
    inputLimits: {
      images: {
        resize: {
          maxWidth: 2_000,
          maxHeight: 2_000,
          maxBytes: 4_718_592,
          jpegQuality: 80,
        },
      },
    },
    type: 'chat',
  },
  'chat:qwen3.8-flash': {
    id: 'qwen3.8-flash',
    name: 'Qwen3.8 Flash',
    api: 'openai-completions',
    provider: 'qwen-token-plan-individual',
    baseUrl: 'https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1',
    reasoning: true,
    input: [
      'text',
      'image',
    ],
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: 1_000_000,
    maxTokens: 131_072,
    compat: {
      supportsStrictMode: true,
      thinkingFormat: 'qwen',
      supportsDeveloperRole: false,
      supportsStore: false,
      supportsReasoningEffort: true,
    },
    thinkingLevelMap: {
      off: null,
      minimal: null,
      low: 'low',
      medium: 'medium',
      high: null,
      xhigh: 'xhigh',
      max: null,
    },
    inputLimits: {
      images: {
        resize: {
          maxWidth: 2_000,
          maxHeight: 2_000,
          maxBytes: 4_718_592,
          jpegQuality: 80,
        },
      },
    },
    type: 'chat',
  },
  'chat:qwen3.8-max': {
    id: 'qwen3.8-max',
    name: 'Qwen3.8 Max',
    api: 'openai-completions',
    provider: 'qwen-token-plan-individual',
    baseUrl: 'https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1',
    reasoning: true,
    input: [
      'text',
      'image',
    ],
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: 1_000_000,
    maxTokens: 131_072,
    compat: {
      supportsStrictMode: true,
      thinkingFormat: 'qwen',
      supportsDeveloperRole: false,
      supportsStore: false,
      supportsReasoningEffort: true,
    },
    thinkingLevelMap: {
      off: null,
      minimal: null,
      low: 'low',
      medium: 'medium',
      high: null,
      xhigh: 'xhigh',
      max: null,
    },
    inputLimits: {
      images: {
        resize: {
          maxWidth: 2_000,
          maxHeight: 2_000,
          maxBytes: 4_718_592,
          jpegQuality: 80,
        },
      },
    },
    type: 'chat',
  },
};

//endregion Public API
