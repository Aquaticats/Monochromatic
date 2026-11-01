import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  applyFootnoteRelabel,
  footnoteProtectedRanges,
  footnoteRenameTouchesOriginal,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  relabelsFootnote,
  requireFootnoteRewriteRefusal,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

import {
  definitionLabelOrder,
  reorderFootnoteDefinitions,
} from '@monochromatic-dev/translation-repair-archive/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { DefinitionLabelPair, } from '@monochromatic-dev/translation-repair-stage/ts';
import { readClosedRelabel, } from './pass-footnote-relabel-read.ts';

//region Pass footnote relabel
// How the pass makes the archive's footnotes the original's
// (`archive-footnote-relabel.ts`, `archive-footnote-order.ts`): read the
// label map off the definitions the roster paired by content, off the
// paired slices when it paired none, or off both where the definitions alone
// do not close (class ninety-five, `pass-footnote-relabel-read.ts`), rewrite the archive's labels, move its
// definitions into the original's order, and let the caller prepare again
// over the rewritten text. Split out of `pass-prepare.ts` so that file keeps
// to its shape and its line budget.

/**
 Archive text after the relabel and the reorder, with whether anything
 changed and what the artifact's findings should say about it.

 @example
 ```ts
 const relabel: RelabelledArchive = { archiveText, changed: false, findings: [], };
 ```
 */
export type RelabelledArchive = {
  /**
   Archive text under the original's labels and definition order, or as it
   came when nothing changed.
   */
  readonly archiveText: string;

  /**
   Whether the text differs from what came in, so the caller knows to
   prepare again.
   */
  readonly changed: boolean;

  /**
   Preparation findings naming what moved or why nothing did.
   */
  readonly findings: readonly string[];

  /**
   A refused operation is distinct from a verified no-op.
   */
  readonly withheld?: 'correspondence' | 'protected-original' | 'rewrite-validation';
};

/**
 Rewrites the archive's footnote labels to the original's and moves its
 definitions into the original's order, logging what moved or why the
 archive stands.

 @param entryId - entry being prepared, for the log

 @param slices - first preparation's slices

 @param definitionPairs - definitions the roster paired, by label

 @param sourceText - original page, whose definition order the archive takes

 @param archiveText - archive text the first preparation was over

 @param l - entry logger

 @returns Archive text to prepare again over when changed

 @example
 ```ts
 const relabel = attemptArchiveFootnoteRelabel({ entryId, slices, definitionPairs, sourceText, archiveText, l, },);
 ```
 */
function attemptArchiveFootnoteRelabel(
  {
    entryId,
    slices,
    definitionPairs,
    sourceText,
    archiveText,
    l,
  }: {
    readonly entryId: string;
    readonly slices: readonly ChunkPair[];
    readonly definitionPairs: readonly DefinitionLabelPair[];
    readonly sourceText: string;
    readonly archiveText: string;
    readonly l: Logger;
  },
): RelabelledArchive {
  /**
   What the evidence says about the labels, which evidence, and the map
   closed over the archive's labels.
   */
  const read = readClosedRelabel({
    entryId,
    definitionPairs,
    slices,
    sourceText,
    archiveText,
    l,
  },);
  if (read.kind === 'ambiguous') {
    l.warn(
      `FOOTNOTES entry=${entryId} archive labels stand, since ${read.basis} disagree: ${read.detail}`,
    );
    return {
      archiveText,
      changed: false,
      findings: [ `footnotes: archive labels stand, since ${read.basis} disagree: ${read.detail}`, ],
      withheld: 'correspondence',
    };
  }
  /**
   The reading, its evidence and its closure.
   */
  const {
    reading,
    basis,
    closure,
  } = read;
  /**
   Findings for the slices the reading left out, each logged as it is.
   */
  const findings = reading.skipped
    .map(function toFinding(detail,): string {
      l.warn(`FOOTNOTES entry=${entryId} left out of the relabel reading: ${detail}`,);
      return `footnotes: left out of the relabel reading: ${detail}`;
    },);
  if (closure.kind === 'open') {
    l.warn(`FOOTNOTES entry=${entryId} archive labels stand, since the map read off ${basis} does not close: ${closure.detail}`,);
    return {
      archiveText,
      changed: false,
      findings: [
        ...findings,
        `footnotes: archive labels stand, since the map read off ${basis} does not close: ${closure.detail}`,
      ],
      withheld: 'correspondence',
    };
  }
  /**
   The closed map, empty where the labels already agree.
   */
  const {
    map: closed,
    correspondences,
    eliminated,
    retained,
  } = closure;
  /**
   Exact sealed text and its declaration in the original coordinate space.
   */
  const protectedRanges = footnoteProtectedRanges({ text: archiveText, },);
  if (footnoteRenameTouchesOriginal({
    text: archiveText,
    map: closed,
    protectedRanges,
  },)) {
    l.warn(`FOOTNOTES entry=${entryId} whole operation withheld to preserve protected English-original bytes`,);
    return {
      archiveText,
      changed: false,
      withheld: 'protected-original',
      findings: [
        ...findings,
        'footnotes: whole operation withheld because a rename would change protected English-original bytes',
      ],
    };
  }
  /**
   The archive under the original's labels.
   */
  const relabelled = (closed.length > 0)
    ? applyFootnoteRelabel({
      text: archiveText,
      map: closed,
    },)
    : archiveText;
  /**
   Original definition order, read from active syntax rather than raw literals.
   */
  const order = definitionLabelOrder({ text: sourceText, },);
  /**
   Protection is reparsed after allowed renames, so changed label widths cannot stale its offsets.
   */
  const reordered = reorderFootnoteDefinitions({
    text: relabelled,
    order,
    protectedRanges: footnoteProtectedRanges({ text: relabelled, },),
  },);
  if (reordered.blockedByProtection === true) {
    l.warn(`FOOTNOTES entry=${entryId} whole operation withheld because definition movement would touch protected English-original bytes`,);
    return {
      archiveText,
      changed: false,
      withheld: 'protected-original',
      findings: [
        ...findings,
        'footnotes: whole operation withheld because definition movement would touch protected English-original bytes',
      ],
    };
  }
  /**
   Supplied correspondence rewrites, distinct from identity evidence and operational displacement.
   */
  const correspondenceRewrites = correspondences.filter(function changes(relation,): boolean {
    return relabelsFootnote({ relation, },);
  },);
  if (correspondenceRewrites.length > 0) {
    /**
     Only supplied source correspondences receive the original evidence attribution.
     */
    const spelled = correspondenceRewrites.map(function spell(relabel,): string {
      return `[^${relabel.from}]->[^${relabel.to}]`;
    },)
      .join(', ',);
    l.info(`FOOTNOTES entry=${entryId} relabelled ${spelled} off ${basis}: the archive's footnote labels follow the original's`,);
    findings.push(`footnotes: archive relabelled ${spelled} off ${basis} to follow the original's labels`,);
  }
  if (eliminated.length > 0) {
    /**
     Existing one-pair elimination is not attributed to an unrecorded model vote.
     */
    const spelled = eliminated.map(function spell(relabel,): string {
      return `[^${relabel.from}]->[^${relabel.to}]`;
    },)
      .join(', ',);
    l.info(`FOOTNOTES entry=${entryId} completed ${spelled} by forced elimination`,);
    findings.push(`footnotes: archive relabelled ${spelled} by forced elimination`,);
  }
  if (retained.length > 0) {
    /**
     Fresh labels preserve unmatched archive notes without claiming source correspondence.
     */
    const spelled = retained.map(function spell(move,): string {
      return `[^${move.from}]->[^${move.retainedAs}]`;
    },)
      .join(', ',);
    l.info(`FOOTNOTES entry=${entryId} retained unmatched archive labels ${spelled}; these moves are not source correspondences`,);
    findings.push(`footnotes: unmatched archive labels retained as ${spelled}, without source correspondence`,);
  }
  if (reordered.changed) {
    /**
     The order, spelled.
     */
    const spelledOrder = order
      .map(function spell(label,): string {
        return `[^${label}]`;
      },)
      .join(', ',);
    l.info(`FOOTNOTES entry=${entryId} definitions moved into the original's order: ${spelledOrder}`,);
    findings.push(`footnotes: archive definitions moved into the original's order: ${spelledOrder}`,);
  }
  if (reordered.note !== undefined) {
    l.warn(`FOOTNOTES entry=${entryId} definitions stand: ${reordered.note}`,);
    findings.push(`footnotes: archive definitions stand: ${reordered.note}`,);
  }
  if ((!reordered.changed) && (closed.length === 0))
    l.debug(`${relabelArchiveFootnotes.name}: entry ${entryId} archive footnotes agree with the original's`,);
  return {
    archiveText: reordered.text,
    changed: reordered.text !== archiveText,
    findings,
  };
}

/**
 Applies the complete archive-footnote operation or retains the original bytes with a structured refusal.
 Unexpected failures propagate; only named rewrite-validation failures become withheld operations.

 @param input - canonical pages, initial pairing evidence and entry logger

 @returns Complete candidate or unchanged archive, with actual applied provenance only

 @example
 ```ts
 const result = relabelArchiveFootnotes({ entryId, slices, definitionPairs, sourceText, archiveText, l });
 ```
 */
export function relabelArchiveFootnotes(input: Parameters<typeof attemptArchiveFootnoteRelabel>[0],): RelabelledArchive {
  /**
   Operation-owned logger, retaining the caller's tags.
   */
  const l = tagged({
    l: input.l,
    tag: relabelArchiveFootnotes.name,
  },);
  try {
    return attemptArchiveFootnoteRelabel({
      ...input,
      l,
    },);
  }
  catch (error) {
    /**
     The rewrite's refusal; anything else keeps propagating.
     */
    const refusal = requireFootnoteRewriteRefusal({ error, },);
    l.warn(refusal.message,);
    return {
      archiveText: input.archiveText,
      changed: false,
      withheld: 'rewrite-validation',
      findings: [`footnotes: archive kept unchanged: ${refusal.message}`,],
    };
  }
}

//endregion Pass footnote relabel
