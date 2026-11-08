/**
 Checked-in NeuralWatt listing snapshot data, one segment. @module
 */

import type { NeuralWattListingEntry, } from './listing.ts';

//region Public API

/**
 Listing entries 25 to 27 of the checked-in snapshot, in endpoint
 order. Split into segments so each module stays within the repository
 file size limit; `neuralWattListingSnapshot` joins them.
 */
export const neuralWattListingSnapshotChunk7:
  readonly NeuralWattListingEntry[] = [
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
        pricing: {
          input_per_million: 3,
          output_per_million: 15,
          cached_input_per_million: 0.3,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'standard',
          flex_discount_multiplier: null,
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
        pricing: {
          input_per_million: 3,
          output_per_million: 15,
          cached_input_per_million: 0.3,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'standard',
          flex_discount_multiplier: null,
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
        pricing: {
          input_per_million: 1.95,
          output_per_million: 9.75,
          cached_input_per_million: 0.195,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'flex',
          flex_discount_multiplier: 0.65,
        },
      },
    },
  ];

//endregion Public API
