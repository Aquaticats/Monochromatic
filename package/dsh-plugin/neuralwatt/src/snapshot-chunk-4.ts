/**
 Checked-in NeuralWatt listing snapshot data, one segment. @module
 */

import type { NeuralWattListingEntry, } from './listing.ts';

//region Public API

/**
 Listing entries 22 to 27 of the checked-in snapshot, in endpoint
 order. Split into segments so each module stays within the repository
 file size limit; `neuralWattListingSnapshot` joins them.
 */
export const neuralWattListingSnapshotChunk4:
  readonly NeuralWattListingEntry[] = [
    {
      id: 'deepseek-v4-flash-flex',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'DeepSeek V4 Flash (flex)',
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
    {
      id: 'deepseek-v4-flash-speed',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'DeepSeek V4 Flash (Speed)',
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
    {
      id: 'gemma-4-31b',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Gemma 4 31B',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
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
            high: 'max',
            medium: 'max',
            low: 'max',
            minimal: 'max',
          },
        },
        limits: {
          max_context_length: 262_128,
          max_output_tokens: 16_384,
        },
      },
    },
    {
      id: 'kimi-k3',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'Kimi K3',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
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
            xhigh: 'max',
            medium: 'high',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 1_048_560,
        },
      },
    },
    {
      id: 'kimi-k3-fast',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'Kimi K3 Fast',
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
          max_context_length: 1_048_560,
        },
      },
    },
    {
      id: 'kimi-k3-flex',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'Kimi K3 (flex)',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
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
            xhigh: 'max',
            medium: 'high',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 1_048_560,
        },
      },
    },
  ];

//endregion Public API
