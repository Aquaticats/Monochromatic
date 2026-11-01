/**
 Checked-in NeuralWatt listing snapshot data, one segment. @module
 */

import type { NeuralWattListingEntry, } from './listing.ts';

//region Public API

/**
 Listing entries 15 to 21 of the checked-in snapshot, in endpoint
 order. Split into segments so each module stays within the repository
 file size limit; `neuralWattListingSnapshot` joins them.
 */
export const neuralWattListingSnapshotChunk3:
  readonly NeuralWattListingEntry[] = [
    {
      id: 'qwen3.6-35b',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Qwen3.6 35B',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'high',
              'none',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
              'none',
            ],
          effort_aliases: {
            max: 'high',
            xhigh: 'high',
            medium: 'high',
            low: 'high',
            minimal: 'high',
          },
        },
        limits: {
          max_context_length: 262_128,
        },
      },
    },
    {
      id: 'qwen3.6-35b-fast',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Qwen3.6 35B Fast',
        capabilities: {
          reasoning: false,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'none',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
              'none',
            ],
          effort_aliases: {
            max: 'none',
            xhigh: 'none',
            high: 'none',
            medium: 'none',
            low: 'none',
            minimal: 'none',
          },
        },
        limits: {
          max_context_length: 262_128,
        },
      },
    },
    {
      id: 'qwen3.6-35b-flex',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Qwen3.6 35B (flex)',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'high',
              'none',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
              'none',
            ],
          effort_aliases: {
            max: 'high',
            xhigh: 'high',
            medium: 'high',
            low: 'high',
            minimal: 'high',
          },
        },
        limits: {
          max_context_length: 262_128,
        },
      },
    },
    {
      id: 'deepseek-v4.1-flash',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'DeepSeek V4.1 Flash',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
              'xhigh',
              'high',
              'low',
              'none',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
              'none',
            ],
          effort_aliases: {
            medium: 'high',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 1_048_560,
          max_output_tokens: 393_216,
        },
      },
    },
    {
      id: 'deepseek-v4.1-flash-flex',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'DeepSeek V4.1 Flash (flex)',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
              'xhigh',
              'high',
              'low',
              'none',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
              'none',
            ],
          effort_aliases: {
            medium: 'high',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 1_048_560,
          max_output_tokens: 393_216,
        },
      },
    },
    {
      id: 'deepseek-v4.1-flash-speed',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'DeepSeek V4.1 Flash (Speed)',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
              'xhigh',
              'high',
              'low',
              'none',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
              'none',
            ],
          effort_aliases: {
            medium: 'high',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 1_048_560,
          max_output_tokens: 393_216,
        },
      },
    },
    {
      id: 'deepseek-v4-flash',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'DeepSeek V4 Flash',
        capabilities: {
          reasoning: true,
          vision: false,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
              'high',
              'none',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
              'none',
            ],
          effort_aliases: {
            xhigh: 'max',
            medium: 'high',
            low: 'high',
            minimal: 'high',
          },
        },
        limits: {
          max_context_length: 1_048_560,
          max_output_tokens: 393_216,
        },
      },
    },
  ];

//endregion Public API
