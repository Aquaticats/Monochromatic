import {
  quoteIsIn,
  type VerifiedAttestation,
} from './reference-attest-match.ts';
import type { AttestationItemWire, } from './reference-attest-wire.ts';
import {
  type NumberedReferenceLine,
  numberedReferenceLines,
  referenceLinePageText,
} from './reference-line-head.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Reference attestation verdicts
// What the check made of each item one voice answered: kept as answered, kept
// under the reference that states it, or dropped with the side not found
// (ledger B25). The reference quote is looked for in what one reference line
// says its page states, so the sheet line credits the page that states the
// detail, a quote cannot run from one page's line into the next, and an address
// or a failure note verifies nothing (ledger B104). An item naming a reference
// that does not state its quote takes the number of the first that does
// rather than losing its vote: the quote was found word for word, and a lost
// vote can cost the detail its quorum, which is the Mio20 loss the cited
// reference rule was written against. The confirmation round still asks the
// bench about every candidate.

/**
 What the check made of one item.

 @example
 ```ts
 const verdict: AttestationVerdict = { kind: 'verified', attestation: { modelId, item, }, };
 ```
 */
export type AttestationVerdict =
  | {
    /**
     Both quotes found, the reference quote in the reference the item names.
     */
    readonly kind: 'verified';
    /**
     Item as answered, with its voice.
     */
    readonly attestation: VerifiedAttestation;
  }
    | {
    /**
     Both quotes found, the reference quote only under another reference.
     */
      readonly kind: 'relabelled';
    /**
     Item under the first reference stating it, with its voice.
     */
      readonly attestation: VerifiedAttestation;
    /**
     Reference the item named.
     */
      readonly answered: number;
    }
    | {
    /**
     A quote not found.
     */
      readonly kind: 'dropped';
    /**
     Voice that answered.
     */
      readonly modelId: RosterModelId;
    /**
     Item as answered.
     */
      readonly item: AttestationItemWire;
    /**
     Whether the archive quote was found in the archive.
     */
      readonly archiveFound: boolean;
    /**
     Whether the reference quote was found under any reference.
     */
      readonly referenceFound: boolean;
    };

/**
 Verdict on one item.

 @param modelId - voice that answered

 @param item - item as answered

 @param archiveText - archive rendering the archive quote is read against

 @param lines - reference lines with their numbers

 @returns Verdict

 @example
 ```ts
 const verdict = verdictOf({ modelId, item, archiveText, lines, },);
 ```
 */
function verdictOf(
  {
    modelId,
    item,
    archiveText,
    lines,
  }: {
    readonly modelId: RosterModelId;
    readonly item: AttestationItemWire;
    readonly archiveText: string;
    readonly lines: readonly NumberedReferenceLine[];
  },
): AttestationVerdict {
  /**
   Whether the archive quote is in the archive.
   */
  const archiveFound = quoteIsIn({
    quote: item.archiveQuote,
    text: archiveText,
  },);
  /**
   Reference lines whose page states the reference quote, in block order.
   Read in what the page states, never the line's head or the lookup's note
   on a page it could not fetch or read (ledger B104).
   */
  const stating = lines.filter(function states(line,): boolean {
    return quoteIsIn({
      quote: item.referenceQuote,
      text: referenceLinePageText({ line, },),
    },);
  },);
  /**
   The named reference, when it states the quote.
   */
  const named = stating.find(function isNamed(line,): boolean {
    return line.reference === item.reference;
  },);
  /**
   First reference stating it, when any does.
   */
  const [first,] = stating;
  if (archiveFound && (named !== undefined)) {
    return {
      kind: 'verified',
      attestation: {
        modelId,
        item,
      },
    };
  }
  if (archiveFound && (first !== undefined)) {
    return {
      kind: 'relabelled',
      attestation: {
        modelId,
        item: {
          ...item,
          reference: first.reference,
        },
      },
      answered: item.reference,
    };
  }
  return {
    kind: 'dropped',
    modelId,
    item,
    archiveFound,
    referenceFound: first !== undefined,
  };
}

/**
 Verdicts on every item of one reply, in answer order.

 @param modelId - voice that answered

 @param items - items as answered

 @param archiveText - archive rendering the archive quotes are read against

 @param referenceContext - reference lines as `citedReferenceBlock` joins them

 @returns One verdict per item

 @throws ReferenceLineHeadError when a reference line has no numbered head

 @example
 ```ts
 const verdicts = attestationVerdicts({ modelId, items, archiveText, referenceContext, },);
 ```
 */
export function attestationVerdicts(
  {
    modelId,
    items,
    archiveText,
    referenceContext,
  }: {
    readonly modelId: RosterModelId;
    readonly items: readonly AttestationItemWire[];
    readonly archiveText: string;
    readonly referenceContext: string;
  },
): readonly AttestationVerdict[] {
  /**
   Reference lines with their numbers, read once for every item.
   */
  const lines = numberedReferenceLines({ referenceContext, },);
  return items.map(function verdictFor(item,): AttestationVerdict {
    return verdictOf({
      modelId,
      item,
      archiveText,
      lines,
    },);
  },);
}

/**
 Items the verdicts keep, each under the reference that states it.

 @param verdicts - verdicts in answer order

 @returns Kept items with their voices, in the same order

 @example
 ```ts
 const verified = keptAttestations({ verdicts, },);
 ```
 */
export function keptAttestations(
  { verdicts, }: { readonly verdicts: readonly AttestationVerdict[]; },
): readonly VerifiedAttestation[] {
  return verdicts.flatMap(function keptOf(verdict,): readonly VerifiedAttestation[] {
    return (verdict.kind === 'dropped') ? [] : [verdict.attestation,];
  },);
}

/**
 Log line for one verdict: the kept item as the sheets will credit it, or
 which quote was not found, so a run log shows why a vote was lost.

 @param verdict - verdict on one item

 @returns One line

 @example
 ```ts
 al.info(attestationVerdictLine({ verdict, },),);
 ```
 */
export function attestationVerdictLine(
  { verdict, }: { readonly verdict: AttestationVerdict; },
): string {
  if (verdict.kind === 'dropped') {
    /**
     Item as answered.
     */
    const { item, } = verdict;
    return `ATTESTED dropped ${verdict.modelId}: archive quote ${verdict.archiveFound ? 'found' : 'not found'}, `
      + `reference quote ${verdict.referenceFound ? 'found' : 'not found under any reference'}: "${item.archiveQuote}" `
      + `against reference ${String(item.reference,)} ("${item.referenceQuote}")`;
  }
  /**
   Item as kept.
   */
  const { item, } = verdict.attestation;
  /**
   Note naming the reference answered, when the item was relabelled.
   */
  const relabel = (verdict.kind === 'relabelled') ? ` (answered reference ${String(verdict.answered,)})` : '';
  /**
   Voice that answered.
   */
  const { modelId, } = verdict.attestation;
  return `ATTESTED item ${modelId}${relabel}: "${item.archiveQuote}" is stated by reference ${
    String(item.reference,)
  } ("${item.referenceQuote}")`;
}

//endregion Reference attestation verdicts
