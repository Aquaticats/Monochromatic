import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import { withoutTranslatorsNotes, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { selectFence, } from './prompt-fence.ts';

//region Section pairing wire
// WHICH SECTION OF THE ORIGINAL DOES EACH SECTION OF THE TRANSLATION RENDER,
// asked of a model because the deterministic aligner has no signal to answer it
// with.
//
// MEASURED, not assumed. `headingAffinity` is token overlap, and Chinese
// headings share no tokens with English ones. Over the two corpus entries the
// forced aligner actually runs on: all 72 of `XIEPT2`'s affinity cells read
// 0.00, and 192 of `XingZ60`'s 195 read 0.00. The three that do not read 1.00,
// and they are exactly the English headings carrying a romanised name that also
// appears in the Chinese heading. With an all-zero grid every pairing scores
// alike, so every pairing is optimal, so the aligner refuses everything and
// `XIEPT2` reaches no slice at all. Reading two languages and saying which
// section renders which is comprehension, which is what
// `doc/decision/llm-assisted-block-pairing.md` already decided for blocks.
//
// SECTIONS GO WHOLE. The smallest context window in `synthetic-catalog.ts` is
// 131072 tokens and the largest sheet this corpus produces is 49921 characters,
// so a length cap would buy nothing and could only withhold the body text the
// question is about. Only two of 92 entries are ever asked, once each, cached.
//
// STRICTLY ONE-TO-ONE, unlike the block pairing. A `ChunkPair` carries one
// section on each side, so a merge or a split has nowhere to live downstream,
// and a reply describing one is refused rather than flattened into a pairing
// that would put mismatched sections in front of every later stage.

/**
 Signals a section pairing a model returned that cannot be used as one.

 @example
 ```ts
 throw new SectionPairingError({ message: 'pairing moves backwards on the original side at position 2', },);
 ```
 */
export class SectionPairingError extends Error {
  /**
   Names the class for callers matching on it.
   */
  public override readonly name = 'SectionPairingError';

  /**
   @param message - what about the returned pairing cannot be used
   */
  public constructor({ message, }: { readonly message: string; },) {
    super(message,);
  }
}

/**
 The refusal a catch around `readSectionPairing` holds, for a catch that
 treats an unusable reply as a lost voice.

 ONE NARROWING PER CLASS, as `requireBlockPairingRefusal`
 (`pair-blocks-wire.ts`) is for block pairings: the reader raises every
 refusal as a {@link SectionPairingError}, so a rethrow written in the catch
 is a statement only a broken reader reaches. It stands here once, where a
 case reaches it.

 @param error - what the catch caught

 @returns Refusal

 @throws The caught value unchanged when it is anything but a pairing
 refusal, an unexpected state that must keep propagating

 @example
 ```ts
 const refusal = requireSectionPairingRefusal({ error, },);
 ```
 */
export function requireSectionPairingRefusal({ error, }: { readonly error: unknown; },): SectionPairingError {
  if (error instanceof SectionPairingError)
    return error;
  throw error;
}

/**
 One heading-bounded section on one side, as the sheet numbers it.

 SEPARATE FROM `NumberedBlock` despite the identical shape, because the two
 are numbered against different documents and handing one to the other's
 reader would validate indices against the wrong counts.

 @example
 ```ts
 const section: NumberedSection = { index: 0, text: '## Paws\n\nThe tabby dozed.', };
 ```
 */
export type NumberedSection = {
  /**
   Position in document order, zero-based, as the sheet shows it.
   */
  readonly index: number;

  /**
   Section's own text, heading included, because the heading is most of what
   identifies a section and on a barely translated page it is all that is
   there.
   */
  readonly text: string;
};

/**
 One committed correspondence between the two sides' sections.

 @example
 ```ts
 const pair: SectionPair = { source: 2, target: 3, };
 ```
 */
export type SectionPair = {
  /**
   Original-side section index.
   */
  readonly source: number;

  /**
   Translation-side section index.
   */
  readonly target: number;
};

/**
 What a model returns for one document's sections.

 Unpaired sections are ABSENT rather than listed against a sentinel, for the
 reason the block wire gives: a sentinel invites a model to pair everything and
 mark the doubtful ones, which is the behaviour this exists to prevent.

 @example
 ```ts
 const wire: SectionPairingWire = { pairs: [{ source: 0, target: 0, },], };
 ```
 */
export type SectionPairingWire = {
  /**
   Correspondences the model committed to, in document order.
   */
  readonly pairs: readonly SectionPair[];
};

/**
 Renders one side's sections as a numbered, fenced list.

 @param sections - sections in document order

 @param fence - fence no section text can reproduce

 @returns Sheet section listing every section against its index

 @example
 ```ts
 const rendered = renderSections({ sections, fence: '=====', },);
 ```
 */
function renderSections(
  {
    sections,
    fence,
  }: {
    readonly sections: readonly NumberedSection[];
    readonly fence: string;
  },
): string {
  return sections
    .map(function toEntry(section,): string {
      return `[${String(section.index,)}]\n${fence}\n${section.text}\n${fence}`;
    },)
    .join('\n\n',);
}

/**
 Builds the sheet asking one model to pair two documents' sections.

 @param sourceSections - original sections in document order

 @param targetSections - translation sections in document order

 @returns Messages for one pairing call

 @example
 ```ts
 const messages = buildSectionPairingMessages({ sourceSections, targetSections, },);
 ```
 */
export function buildSectionPairingMessages(
  {
    sourceSections,
    targetSections,
  }: {
    readonly sourceSections: readonly NumberedSection[];
    readonly targetSections: readonly NumberedSection[];
  },
): readonly ChatMessage[] {
  // THE MODEL IS SHOWN THE TRANSLATION'S SECTIONS WITHOUT THE NOTES THE
  // ARCHIVE'S TRANSLATORS LEFT EACH OTHER (`withoutTranslatorsNotes`): no reader
  // sees one, so a note may not make a section look as if it had a body.
  // Every index and the length of the list stay as the caller numbered them, and
  // the pairs the model returns name those indices, so a section that held
  // nothing else is listed with its heading alone or empty. The original's own
  // comments stay in view.
  /**
   Texts whose comments are the original's own.
   */
  const originals = sourceSections
    .map(function toText(section,): string {
      return section.text;
    },)
    .join('\n',);
  /**
   Translation sections as the model reads them, each at its index.
   */
  const shownTargetSections = targetSections.map(function shown(section,): NumberedSection {
    return {
      index: section.index,
      text: withoutTranslatorsNotes({
        text: section.text,
        original: originals,
      },),
    };
  },);

  /**
   Fence chosen against every section this sheet carries.

   Both sides are arbitrary prose and either may contain a run of equals
   signs (a setext heading underline is one), so a fixed fence would let a
   section close its own listing and have the rest read as sheet structure.
   `selectFence` fenced with equals signs from `b111fc376`, before this sheet
   was written, yet this comment said backticks until ledger D16.
   */
  const fence = selectFence({
    texts: [
      ...sourceSections.map(function toText(section,): string {
        return section.text;
      },),
      ...targetSections.map(function toText(section,): string {
        return section.text;
      },),
      ...shownTargetSections.map(function toText(section,): string {
        return section.text;
      },),
    ],
  },);

  return [
    {
      role: 'system',
      content: 'You pair the SECTIONS of an ORIGINAL document with the SECTIONS of a '
        + 'TRANSLATION of it. A section is a heading and everything under it. Return only '
        + 'which original section each translation section renders.\n\n'
        + 'A SECTION WHOSE BODY WAS NEVER TRANSLATED STILL CORRESPONDS. Pages are often '
        + 'left half finished, with the headings rendered into the translation and the '
        + 'text under them still missing or still in the original language. Where a '
        + 'translation section is an empty heading that clearly stands for an original '
        + 'section, PAIR THEM: saying so is what lets the missing text be written in the '
        + 'right place. Judge correspondence by what the section is about and where it '
        + 'sits, not by how much of it was translated.\n\n'
        + 'PAIR ONLY WHAT CORRESPONDS. An original may carry sections the translation '
        + 'never had, and a translation may carry sections of its own. Where a section '
        + 'has no counterpart, LEAVE IT OUT: an omitted section is a correct answer and a '
        + 'wrong pairing is worse than none, because later stages will report differences '
        + 'between two passages that were never about the same thing.\n\n'
        + 'ONE TO ONE. Each original section may be named at most once and each '
        + 'translation section may be named at most once. If you believe one section was '
        + 'split into two or two were merged into one, pair the closest correspondence '
        + 'and leave the rest out.\n\n'
        + 'ORDER IS PRESERVED. Both documents say things in the same order, so your pairs '
        + 'must never move backwards on either side.\n\n'
        + 'Return JSON: {"pairs":[{"source":0,"target":0}]} with indices exactly as '
        + 'numbered below.',
    },
    {
      role: 'user',
      content: `ORIGINAL SECTIONS\n\n${
        renderSections({
          sections: sourceSections,
          fence,
        },)
      }\n\nTRANSLATION SECTIONS\n\n${
        renderSections({
          sections: shownTargetSections,
          fence,
        },)
      }`,
    },
  ];
}

//endregion Section pairing wire
