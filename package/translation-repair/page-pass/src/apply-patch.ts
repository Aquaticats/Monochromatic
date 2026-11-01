import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  type GatedOperation,
  NO_MARKUP_DELTA,
  settleGatedOperations,
} from './apply-patch-markup.ts';
import {
  markupDelta,
  markupSourceKeys,
  spliceDisjointEdits,
  type SpliceEdit,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { checkPreservation, } from './preservation-check.ts';

import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type EditableEnvelope,
  nestingWithEdit,
  unreadableReason,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { restoreTypography, } from './restore-typography.ts';
import {
  clampQuoteDepth,
  quoteDepthBound,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';

//region Patch application
// The deterministic gate between editor output and the candidate text.
// Every operation must name a known envelope, echo its base hash, and
// actually change the region; anything else is a rejection recorded as
// data. Overlapping envelopes are a construction bug upstream, never editor
// weather, so they throw instead of rejecting.

/**
 One edit an editor proposes: replace one envelope's content wholesale.
 Replacing a zero-width envelope inserts; an empty replacement deletes.

 @example
 ```ts
 const operation: PatchOperation = {
   envelopeId: 'envelope/abc',
   baseHash: envelope.baseHash,
   newText: 'The cat naps on the warm windowsill.',
 };
 ```
 */
export type PatchOperation = {
  /**
   Envelope the edit targets.
   */
  readonly envelopeId: string;

  /**
   Echo of the envelope's base hash, proving the editor wrote against
   the text it replaces.
   */
  readonly baseHash: string;

  /**
   Replacement for the whole envelope content.
   */
  readonly newText: string;
};

/**
 Whether this application enforces the preservation gate, and with what
 licence.

 A DISCRIMINATED CHOICE rather than an optional map, so every caller states
 its intent. The naturalness lane rewrites whole paragraphs by design and has
 no accepted-issue quotes to license that, so an omitted map would silently
 make the gate reject exactly the work that lane exists to do.

 @example
 ```ts
 const preservation: PreservationMode = { mode: 'skip', };
 ```
 */
export type PreservationMode =
  | {
    /**
     Reject an operation that drops content no issue quoted.
     */
    readonly mode: 'enforce';

    /**
     Defect text each envelope's issues quoted, keyed by envelope id. What an
     issue quoted is licensed to disappear.
     */
    readonly licensedQuotes: ReadonlyMap<string, readonly string[]>;

    /**
     Text each envelope's addition claims quoted, keyed by envelope id, the
     only quotes whose markup atoms an edit may remove (ledger L4).
     REQUIRED, like the mode itself, so a caller that forgets it refuses
     every addition removal rather than licensing all of them.
     */
    readonly removableQuotes: ReadonlyMap<string, readonly string[]>;

    /**
     Original the markup is read against: the whole source document where
     the stage has it, else the chunk's. An atom it carries is copied markup
     that must survive; one it does not is markup the translation authored,
     which an edit may re-mark but not drop (ledger L4).
     */
    readonly sourceText: string;
  }
    | {
    /**
     Apply without the gate, for a stage licensed to rewrite wholesale.
     */
      readonly mode: 'skip';
    };

/**
 One rejected operation with its scorecard-stable reason.

 @example
 ```ts
 const rejection: PatchRejection = { operation, reason: 'stale-base-hash', };
 ```
 */
export type PatchRejection = {
  /**
   Operation as proposed.
   */
  readonly operation: PatchOperation;

  /**
   Which gate refused and why, in scorecard-stable wording.
   */
  readonly reason: string;
};

/**
 Everything patch application decided.

 @example
 ```ts
 const outcome: PatchOutcome = applyPatchOperations({ targetText, envelopes, operations, },);
 ```
 */
export type PatchOutcome = {
  /**
   Translation with every accepted operation applied.
   */
  readonly patchedText: string;

  /**
   Operations that passed every gate, in input order.
   */
  readonly applied: readonly PatchOperation[];

  /**
   Operations refused, each with its reason, in input order.
   */
  readonly rejected: readonly PatchRejection[];
};

/**
 Thrown when envelopes overlap: derivation guarantees disjoint envelopes,
 so an overlap is a construction bug the caller must fix, not editor
 output to tolerate.

 @example
 ```ts
 throw new EnvelopeOverlapError({ leftId: 'envelope/a', rightId: 'envelope/b', },);
 ```
 */
export class EnvelopeOverlapError extends Error {
  /**
   Declares this message safe to forward: it names two envelope ids and neither envelope's text.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the overlap report from the two colliding envelope ids.

   @param leftId - envelope earlier in document order

   @param rightId - envelope overlapping it
   */
  constructor(
    {
      leftId,
      rightId,
    }: {
      readonly leftId: string;
      readonly rightId: string;
    },
  ) {
    super(`editable envelopes overlap: ${leftId} and ${rightId}`,);
    this.name = 'EnvelopeOverlapError';
  }
}

/**
 Applies editor operations to the translation through every deterministic
 gate. Gates per operation, in order: the envelope must exist, only one
 operation may claim it, the echoed base hash must match, the document
 region must still equal the envelope base, and the replacement must
 actually change the region. Accepted operations apply in descending
 document order so earlier offsets stay valid.

 @param targetText - full translation the envelopes were derived from

 @param envelopes - non-overlapping envelopes in any order

 @param operations - editor proposals in wire order

 @returns Patched text plus applied and rejected operations as data

 @throws {@link EnvelopeOverlapError} when two envelopes overlap

 @throws {@link FragmentLandingError} when an envelope whose current text matched its base text lies where the targetText cannot hold
 a fragment: an offset that is no whole number, a start before the text, an end past it, or an end before its start. An envelope with
 empty base text and an end before its start passes the drift check (the empty slice of a backward range equals the empty base text)
 and is refused here, which is its refusal; no other refusal of the applier comes before it

 @example
 ```ts
 const { patchedText, rejected, } = applyPatchOperations({ targetText, envelopes, operations, preservation, },);
 ```
 */
export function applyPatchOperations(
  {
    targetText,
    envelopes,
    operations,
    preservation,
  }: {
    readonly targetText: string;
    readonly envelopes: readonly EditableEnvelope[];
    readonly operations: readonly PatchOperation[];
    readonly preservation: PreservationMode;
  },
): PatchOutcome {
  /**
   Envelopes in document order for the overlap check.
   */
  const ordered = [...envelopes,].toSorted(function byStart(
    left,
    right,
  ) {
    return left.startOffset - right.startOffset;
  },);
  for (const [index, envelope,] of ordered.entries()) {
    /**
     Envelope following this one in document order, when present.
     */
    const next = ordered[index + 1];
    if ((next !== undefined) && (next.startOffset < envelope.endOffset)) {
      throw new EnvelopeOverlapError({
        leftId: envelope.envelopeId,
        rightId: next.envelopeId,
      },);
    }
  }

  /**
   Envelopes keyed by id for operation lookup.
   */
  const byId = new Map(envelopes.map(function toEntry(envelope,) {
    return [
      envelope.envelopeId,
      envelope,
    ] as const;
  },),);

  /**
   Operations every per-edit gate passed, in input order, with what each did
   to its envelope's markup.
   */
  const gated: GatedOperation[] = [];

  /**
   The edits of the operations gated so far, as they would be written, which
   the text a further edit leaves is read beside.
   */
  const acceptedEdits: SpliceEdit[] = [];

  /**
   Refusals of the per-edit gates, in input order.
   */
  const rejected: PatchRejection[] = [];

  /**
   Keys of the markup the source carries, read once for every edit.
   */
  const sourceKeys = (preservation.mode === 'enforce')
    ? markupSourceKeys({ sourceText: preservation.sourceText, },)
    : new Set<string>();

  /**
   Envelope ids already claimed by an accepted operation.
   */
  const claimed = new Set<string>();
  for (const operation of operations) {
    /**
     Envelope the operation targets, when known.
     */
    const envelope = byId.get(operation.envelopeId,);
    if (envelope === undefined) {
      rejected.push({
        operation,
        reason: 'unknown-envelope',
      },);
      continue;
    }
    if (claimed.has(operation.envelopeId,)) {
      rejected.push({
        operation,
        reason: 'duplicate-operation',
      },);
      continue;
    }
    if (operation.baseHash !== envelope.baseHash) {
      rejected.push({
        operation,
        reason: 'stale-base-hash',
      },);
      continue;
    }

    /**
     Text currently occupying the envelope region in the document.
     */
    const current = targetText.slice(
      envelope.startOffset,
      envelope.endOffset,
    );
    if ((current !== envelope.baseText)
      || (hashContent({ content: current, },) !== envelope.baseHash))
      {
      rejected.push({
        operation,
        reason: 'envelope-drift',
      },);
      continue;
    }
    /**
     Replacement with the document's quote style restored, which is what
     ships and therefore what the unchanged-region check, `checkPreservation`
     and `markupDelta` read.

     RESTORED BEFORE THE CHECKS rather than after them. Editors flatten curly
     quotes to straight ones often enough that a repaired paragraph ends up
     reading differently from every paragraph around it, so the restoration
     is deterministic and recorded; run last, it altered text the preservation
     gate had already judged, and an edit that only flattened quotes shipped
     as an applied operation. The convention comes from the WHOLE text, not
     the replaced region alone: regions run to a median of 75 characters, so
     most carry no quote to learn from, and a region-scoped rule stays silent
     exactly when an editor writes a fresh contraction into a curly-quoted
     document.
     */
    const restored = clampQuoteDepth({
      replacement: restoreTypography({
        replacement: operation.newText,
        replaced: envelope.baseText,
        convention: targetText,
        // The region is a fragment of the slice, so its prose is the slice's with the fragment in place.
        landing: {
          slice: targetText,
          start: envelope.startOffset,
          end: envelope.endOffset,
        },
      },),
      // No quote nests deeper than its context allows (TianqiChen66621
      // slice 16, `quote-depth-clamp.ts`), clamped here so every gate this
      // call runs reads the depth that ships.
      bound: quoteDepthBound({
        targetText,
        startOffset: envelope.startOffset,
        endOffset: envelope.endOffset,
        sourceText: (preservation.mode === 'enforce') ? preservation.sourceText : '',
      },),
      startsLine: (envelope.startOffset === 0) || (targetText[envelope.startOffset - 1] === '\n'),
    },);
    if (restored === envelope.baseText) {
      rejected.push({
        operation,
        reason: 'unchanged-region',
      },);
      continue;
    }
    claimed.add(operation.envelopeId,);
    if (preservation.mode === 'enforce') {
      /**
       Whether the edit kept everything no issue asked it to change.
       */
      const preserved = checkPreservation({
        before: envelope.baseText,
        after: restored,
        licensedQuotes: preservation.licensedQuotes
          .get(envelope.envelopeId,)
          ?? [],
      },);
      if (!preserved.preserved) {
        // Named in the reason so a scorecard can separate the two shapes: a
        // vanished name or number is a different failure from a span replaced
        // wholesale, and they call for different fixes upstream.
        rejected.push({
          operation,
          reason: (preserved.lostDistinctive
            .length
            > 0)
            ? `preservation-lost-distinctive (${preserved.lostDistinctive
              .join(', ',)})`
            : `preservation-bulk-loss (${preserved.lossFraction
              .toFixed(2,)})`,
        },);
        continue;
      }

    }

    /**
     Where the text the accepted edits and this one leave first passes the
     nesting bound, which no grammar reads past.
     */
    const nesting = nestingWithEdit({
      targetText,
      accepted: acceptedEdits,
      candidate: {
        start: envelope.startOffset,
        end: envelope.endOffset,
        text: restored,
      },
    },);
    if (nesting.kind === 'beyond') {
      rejected.push({
        operation,
        reason: unreadableReason({ excess: nesting, },),
      },);
      continue;
    }

    acceptedEdits.push({
      start: envelope.startOffset,
      end: envelope.endOffset,
      text: restored,
    },);
    gated.push({
      operation,
      restored: {
        ...operation,
        newText: restored,
      },
      // Markup the edit lost that no addition claim quoted, which
      // `checkPreservation` cannot see inside a licensed quote (ledger L4);
      // settled over the whole patch once every edit is gated, since a sibling
      // edit may write it.
      delta: (preservation.mode === 'enforce')
        ? markupDelta({
          before: envelope.baseText,
          after: restored,
          removableQuotes: preservation.removableQuotes
            .get(envelope.envelopeId,)
            ?? [],
          sourceKeys,
        },)
        : NO_MARKUP_DELTA,
    },);
  }

  /**
   What ships and every refusal, once the markup rule has read the patch.
   */
  const settled = settleGatedOperations({
    operations,
    gated,
    rejected,
  },);

  /**
   Operations that passed every gate, in input order.
   */
  const { applied, } = settled;

  // ONE PASS OVER THE TRANSLATION: every accepted operation names its own
  // envelope, and the envelopes were checked apart, so each is written at
  // its offsets into the translation as given (ledger B70). Written one at a
  // time, last first, the whole translation was copied once per operation.
  /**
   Translation with every accepted operation written into its envelope.
   */
  const patchedText = spliceDisjointEdits({
    text: targetText,
    edits: applied.map(function toEdit(operation,) {
      /**
       Envelope of this accepted operation, present by acceptance.
       */
      const envelope = nonNullishOrThrow(byId.get(operation.envelopeId,),);
      return {
        start: envelope.startOffset,
        end: envelope.endOffset,
        text: operation.newText,
      };
    },),
  },);

  return {
    patchedText,
    applied,
    rejected: settled.rejected,
  };
}

//endregion Patch application
