/**
 Checked-in NeuralWatt listing snapshot: the model facts served while the
 mount fetch cannot reach the endpoint. @module
 */

import type { NeuralWattListingEntry, } from './listing.ts';
import { neuralWattListingSnapshotChunk1, } from './snapshot-chunk-1.ts';
import { neuralWattListingSnapshotChunk2, } from './snapshot-chunk-2.ts';
import { neuralWattListingSnapshotChunk3, } from './snapshot-chunk-3.ts';
import { neuralWattListingSnapshotChunk4, } from './snapshot-chunk-4.ts';
import { neuralWattListingSnapshotChunk5, } from './snapshot-chunk-5.ts';
import { neuralWattListingSnapshotChunk6, } from './snapshot-chunk-6.ts';
import { neuralWattListingSnapshotChunk7, } from './snapshot-chunk-7.ts';

//region Public API

/**
 Reduced listing snapshot fetched from GET
 `https://api.neuralwatt.com/v1/models` on 2026-10-22: every entry the
 endpoint returned then, keeping only the fields the mapping reads (id,
 display name, capabilities, reasoning efforts, token limits, deprecation).
 The listing's `metadata.pricing` block was merged from the live endpoint on
 2026-11-07T22:41:21-05:00 so harnesses that price models can; entries the
 endpoint has since dropped carry no pricing block. Descriptive fields stay
 dropped: this provider serves models, it does not estimate cost. The entries
 live in the `snapshot-chunk-N` modules and keep listing order here.
 */
export const neuralWattListingSnapshot: readonly NeuralWattListingEntry[] = [
  ...neuralWattListingSnapshotChunk1,
  ...neuralWattListingSnapshotChunk2,
  ...neuralWattListingSnapshotChunk3,
  ...neuralWattListingSnapshotChunk4,
  ...neuralWattListingSnapshotChunk5,
  ...neuralWattListingSnapshotChunk6,
  ...neuralWattListingSnapshotChunk7,
];

//endregion Public API
