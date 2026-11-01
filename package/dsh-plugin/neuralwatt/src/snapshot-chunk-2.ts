/**
 Checked-in NeuralWatt listing snapshot data, one segment. @module
 */

import type { NeuralWattListingEntry, } from './listing.ts';

//region Public API

/**
 Listing entries 8 to 14 of the checked-in snapshot, in endpoint
 order. Split into segments so each module stays within the repository
 file size limit; `neuralWattListingSnapshot` joins them.
 */
export const neuralWattListingSnapshotChunk2:
  readonly NeuralWattListingEntry[] = [
    {
      id: 'nw-flash-flex',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'Neuralwatt Flash (flex)',
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
      id: 'glm-5.3',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'GLM 5.3',
        capabilities: {
          reasoning: true,
          vision: false,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
              'high',
              'low',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
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
      id: 'glm-5.3-flex',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'GLM 5.3 (flex)',
        capabilities: {
          reasoning: true,
          vision: false,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
              'high',
              'low',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
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
      id: 'glm-5.3-flash',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'GLM-5.3 Flash',
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
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
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
      id: 'glm-5.3-flash-flex',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'GLM-5.3 Flash (flex)',
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
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
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
      id: 'nw-small',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Neuralwatt Small',
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
      id: 'nw-small-flex',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Neuralwatt Small (flex)',
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
  ];

//endregion Public API
