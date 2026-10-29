import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  type CorpusPin,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { buildReviewedFidelityReference, } from './fidelity-reference-build.ts';
import { FidelityReferenceError, } from './fidelity-reference-error.ts';
import { REVIEWED_FIDELITY_REFERENCES, } from './fidelity-reference-manifest.ts';
import type {
  FidelityReferenceSpec,
  ReviewedFidelityReference,
} from './fidelity-reference-model.ts';
import { selectReviewedFidelitySpecs, } from './fidelity-reference-select.ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { mapOverlapped, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Pinned reviewed-reference acquisition
// Calibration owns a fixed reviewed manifest, not an opportunistic first-long-archive selection.

/**
 Logger root for the reference read, which takes no logger of its own.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Names a reviewed reference on the lines the overlapped map writes about a
 failure it leaves unthrown.

 @param item - manifest entry, named by its reference

 @returns Reference by its identity

 @example
 ```ts
 specReferenceName({ item: spec, },); // 'reference tabby-nap'
 ```
 */
function specReferenceName({ item, }: { readonly item: { readonly id: string; }; },): string {
  return `reference ${item.id}`;
}

/**
 Reads reviewed reference inputs from their immutable corpus pin.
 Custom manifests are explicit caller-owned review evidence; this loader proves byte identity,
 not semantic correctness of arbitrary supplied prose.

 @param pin - local clone and immutable review revision

 @param specs - reviewed manifest, defaulting to the checked-in source-reviewed set

 @param onlyEntryIds - optional filter that may name only reviewed entries

 @param signal - caller cancellation

 @returns References in manifest order with all reviewed variants verified

 @throws {@link FidelityReferenceError} for unreviewed filters, pin drift or invalid metadata

 @throws {@link CorpusReadError} when a pinned file is unavailable

 @throws The caller's reason when `signal` aborts, once a reference's two page
 reads have settled and each one's own failure is written on a line naming the
 reference and the page

 @example
 ```ts
 const references = await readReviewedFidelityReferences({ pin });
 ```
 */
export async function readReviewedFidelityReferences({
  pin,
  specs = REVIEWED_FIDELITY_REFERENCES,
  onlyEntryIds = [],
  signal,
}: {
  readonly pin: CorpusPin;
  readonly specs?: readonly FidelityReferenceSpec[];
  readonly onlyEntryIds?: readonly string[];
  readonly signal?: AbortSignal;
},): Promise<readonly ReviewedFidelityReference[]> {
  signal?.throwIfAborted();
  /**
   Own and validate metadata selection before any pinned-file reads.
   */
  const selected = selectReviewedFidelitySpecs({
    specs,
    onlyEntryIds,
  },);
  /**
   Pin fields cannot be redirected while the reads are in flight.
   */
  const fixedPin = { ...pin, };
  return await mapOverlapped({
    items: selected,
    overlap: 1,
    // A caller that passes no signal cannot stop the read, which is what a
    // signal that never aborts says; one reference is read at a time in any case.
    signal: signal ?? new AbortController().signal,
    l,
    nameOf: specReferenceName,
    oneItem: async function readReference({ item: spec, },): Promise<ReviewedFidelityReference> {
      signal?.throwIfAborted();
      if (fixedPin.commitSha !== spec.corpusSha)
        throw new FidelityReferenceError({
          referenceId: spec.id,
          operation: 'pin',
        },);
      if ((spec.entryId === '')
        || (spec.entryId === '.')
        || (spec.entryId === '..')
        || spec.entryId
          .includes('/',)
        || spec.entryId
          .includes('\\',)
        || spec.entryId
          .includes('\0',)) {
        throw new FidelityReferenceError({
          referenceId: spec.id,
          operation: 'request',
        },);
      }
      /**
       Both files use the same intrinsic, no-fetch corpus read boundary.
       */
      const [sourceRead, archiveRead,] = await Promise.allSettled([
        readCorpusFile({
          pin: fixedPin,
          relPath: `people/${spec.entryId}/page.md`,
        },),
        readCorpusFile({
          pin: fixedPin,
          relPath: `people/${spec.entryId}/page.en.md`,
        },),
      ],);
      // A PAGE'S OWN FAILURE IS WRITTEN BEFORE THE STOP IS RAISED, since the
      // stop throws in place of both reads and no catch past it sees either.
      // The reads take no signal, so a failure here is always the read's own;
      // written once both have settled, the source page's first, whichever
      // ended first (`mistake-prevention.md`, "Output decided by arrival
      // order").
      if (signal?.aborted === true) {
        /**
         Logger tagged with this reference read.
         */
        const rl = tagged({
          tag: readReference.name,
          l,
        },);
        for (
          const {
            page,
            read,
          } of [
            {
              page: 'source page',
              read: sourceRead,
            },
            {
              page: 'archive page',
              read: archiveRead,
            },
          ]
        ) {
          if ((read.status === 'rejected') && (read.reason !== signal.reason)) {
            rl.warn(
              `${spec.id}: the ${page} could not be read, and the caller stopped the read: ${
                exchangeFailureLogText({ error: read.reason, },)
              }`,
            );
          }
        }
      }
      signal?.throwIfAborted();
      if (sourceRead.status === 'rejected')
        throw sourceRead.reason;
      if (archiveRead.status === 'rejected')
        throw archiveRead.reason;
      return buildReviewedFidelityReference({
        sourceFile: sourceRead.value,
        archiveFile: archiveRead.value,
        spec,
      },);
    },
  },);
}

//endregion Pinned reviewed-reference acquisition
