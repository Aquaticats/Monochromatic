/**
 Checked-in NeuralWatt listing snapshot data, one segment. @module
 */

import type { NeuralWattListingEntry, } from './listing.ts';

//region Public API

/**
 Listing entries 1 to 7 of the checked-in snapshot, in endpoint
 order. Split into segments so each module stays within the repository
 file size limit; `neuralWattListingSnapshot` joins them.
 */
export const neuralWattListingSnapshotChunk1:
  readonly NeuralWattListingEntry[] = [
    {
      id: 'qwen-3.8-27b',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Qwen 3.8 27B',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'xhigh',
              'medium',
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
            max: 'xhigh',
            high: 'xhigh',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 262_128,
          max_output_tokens: 131_072,
        },
      },
    },
    {
      id: 'qwen-3.8-27b-flex',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Qwen 3.8 27B (flex)',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'xhigh',
              'medium',
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
            max: 'xhigh',
            high: 'xhigh',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 262_128,
          max_output_tokens: 131_072,
        },
      },
    },
    {
      id: 'mimo-v2.6-pro',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'MiMo-V2.6-Pro',
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
            minimal: 'none',
          },
        },
        limits: {
          max_context_length: 1_048_560,
          max_output_tokens: 393_216,
        },
      },
    },
    {
      id: 'mimo-v2.6-pro-flex',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'MiMo-V2.6-Pro (flex)',
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
            minimal: 'none',
          },
        },
        limits: {
          max_context_length: 1_048_560,
          max_output_tokens: 393_216,
        },
      },
    },
    {
      id: 'nw-large',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'Neuralwatt Large',
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
      id: 'nw-large-flex',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'Neuralwatt Large (flex)',
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
      id: 'nw-flash',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'Neuralwatt Flash',
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
  ];

//endregion Public API
