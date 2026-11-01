import { archiveNotesMissingFrom, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  appendedNotes,
  quotedNote,
} from './archive-note-carry.ts';

//region Archive block notes
// THE ARCHIVE BLOCK REVIEW KEEPS THE NOTES THE BLOCK HELD. zheermao101
// (run zheermao15, 2026-10-20) holds, inside the blockquote that translates
// the One Among Us reply email, a line in Han asking the other translators
// how to translate a sentence. The review revised the whole blockquote, and
// the revision had none: a revision is the reviewer's complete rewrite of the
// block, and a note no reader sees is the first thing a rewrite leaves out.
// The prepared archive then no longer held the note, and the page assembly
// (`archive-note-carry.ts`), which carries a note back only from the archive
// text it compares with, carried nothing. The owner's rule is that the page
// keeps such notes (2026-10-20), and a note can steer the next pass
// (`hakureico`'s seals a span).
//
// SAME NOTES, SAME PLACE AS THE PAGE CARRY. A note is a closed comment whose
// words hold Han (`translatorsNotesIn`); the block's notes the replacement
// lacks, even re-wrapped (`archiveNotesMissingFrom`), go on lines of their own
// after the replacement in the block's order (`appendedNotes`). A comment the
// original itself carries is the lanes' to carry and is left alone, as is a
// comment in English. A block the review removes outright keeps its notes
// too: the notes alone stand where the block stood.

/**
 A replacement for an archive block together with the notes the block held.

 @example
 ```ts
 const kept: BlockReplacementWithNotes = { text: 'The cat napped.\n\n<!-- 注 -->', kept: [ '<!-- 注 -->', ], };
 ```
 */
export type BlockReplacementWithNotes = {
  /**
   The replacement with each missing note after it, or as it came when none
   was missing.
   */
  readonly text: string;

  /**
   Each note added, as the block writes it, in the block's order.
   */
  readonly kept: readonly string[];
};

/**
 The finding that names the notes a replacement kept for its block.

 @param identity - the block's identity in the review's findings

 @param kept - notes added, as the block writes them

 @returns One line naming the block and each note

 @example
 ```ts
 const finding = keptNotesFinding({ identity, kept: [ '<!-- 注 -->', ], },);
 ```
 */
export function keptNotesFinding(
  {
    identity,
    kept,
  }: {
    readonly identity: string;
    readonly kept: readonly string[];
  },
): string {
  return `archive block replacement kept the notes between translators the block held: ${identity}: ${
    kept
      .map(function quoted(note,): string {
        return quotedNote({ text: note, },);
      },)
      .join(', ',)
  }`;
}

/**
 A replacement for an archive block with the block's notes between
 translators put back where the replacement lacks them.

 @param blockText - the block as the archive writes it

 @param replacement - what the review adopts for the block, empty for a removal

 @param original - the original section the block was reviewed against, whose
 own comments are the lanes' to carry and are never put back

 @returns The replacement with each missing note after it, and those notes

 @example
 ```ts
 const { text, kept, } = withBlockNotes({ blockText, replacement: 'The cat napped.', original: '猫睡了。', },);
 ```
 */
export function withBlockNotes(
  {
    blockText,
    replacement,
    original,
  }: {
    readonly blockText: string;
    readonly replacement: string;
    readonly original: string;
  },
): BlockReplacementWithNotes {
  /**
   Notes of the block the replacement lacks and the original does not carry.
   */
  const kept = archiveNotesMissingFrom({
    archive: blockText,
    page: replacement,
  },)
    .map(function noteText(note,): string {
      return note.text;
    },)
    .filter(function isNotOriginals(note,): boolean {
      return !original.includes(note,);
    },);
  if (kept.length === 0)
    return {
      text: replacement,
      kept,
    };
  return {
    text: appendedNotes({
      text: replacement,
      notes: kept,
    },),
    kept,
  };
}

//endregion Archive block notes
