import {
  type ArchiveOriginalSpan,
  documentLineEnding,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { archiveNotesMissingFrom, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  rewriteEverySlice,
  type TextRewrite,
} from './page-slice-rewrite.ts';

//region Archive note carry
// THE PAGE KEEPS THE NOTES THE ARCHIVE'S TRANSLATORS LEFT EACH OTHER, whatever
// wording wins the slice. XingZ60's archive ends a sentence with
// `<!-- 前面这句中文懂什么意思，先这么翻译，大家看一下 -->`, a comment in Han
// asking the other translators about the Chinese before it. The repair lane
// keeps a comment as markup and the translate lane does not, so until the
// judging sheets stopped showing such notes (`translators-note.ts`) the page
// carried it exactly when the lane that kept it won, and a lane contest was
// decided on it. With the judges blind to the note, that rule would make its
// survival chance: the page would lose the note whenever the translate
// wording won.
//
// WHY IT IS KEPT AT ALL when a reader never sees it. The translators who open
// the page next do, and a note can steer the pass that reads the page after
// this one: `hakureico`'s note that everything below it is the author's own
// English seals that span (`archive-original-note.ts`), and the entry-notes
// lines (`entry-notes.ts`) hand every archive comment to the sheets as
// vocabulary and voice evidence. A page that drops them unseals the span and
// forgets the vocabulary on the next pass.
//
// WHICH NOTES. The closed comments whose words hold Han in the archive's text
// for the slice (`translatorsNotesIn`); the original's own comments are the
// lanes' to carry and are not touched. A note the shipped text already carries,
// even re-wrapped, is left where it stands.
//
// WHERE. On a line of its own after the slice's last block, in the archive's
// order, because the wording the note stood beside is gone or reworded and no
// position in it is the note's own any more. A slice the page removes outright
// (empty text) carries nothing.

/**
 Quotes a note for the finding that names it, so a note over several lines
 stays on one line of the log.

 @param text - the note as the archive writes it

 @returns The note as a quoted string

 @example
 ```ts
 const quoted = quotedNote({ text: '<!-- 注 -->', },); // '"<!-- 注 -->"'
 ```
 */
export function quotedNote({ text, }: { readonly text: string; },): string {
  return JSON.stringify(text,);
}

/**
 A text with notes added on lines of their own after its last block, in the
 order given.

 THE ONE PLACEMENT RULE of every note put back: the wording the note stood
 beside is gone or reworded, so no position in it is the note's own, and
 the whitespace that ends the text stays at its end. A text that shows a
 reader nothing is replaced by the notes alone. The breaks written are the
 text's own line ending, so a CRLF text gains no bare line feed.

 @param text - wording the notes follow

 @param notes - each note as the archive writes it, in archive order

 @returns The wording followed by each note

 @example
 ```ts
 const carried = appendedNotes({ text: 'The cat napped.', notes: [ '<!-- 大家看一下 -->', ], },);
 // 'The cat napped.\n\n<!-- 大家看一下 -->'
 ```
 */
export function appendedNotes(
  {
    text,
    notes,
  }: {
    readonly text: string;
    readonly notes: readonly string[];
  },
): string {
  /**
   The line ending the text uses, LF when it shows none.
   */
  const eol = documentLineEnding({ targetText: text, },);
  /**
   The break between two paragraphs in that ending.
   */
  const paragraphBreak = `${eol}${eol}`;
  /**
   The notes, one paragraph each.
   */
  const written = notes.join(paragraphBreak,);
  if (rendersAsNothing({ text, },))
    return written;
  /**
   The text without the whitespace that ends it, which stays at the end.
   */
  const body = text.trimEnd();
  return `${body}${paragraphBreak}${written}${text.slice(body.length,)}`;
}

/**
 One slice's text with the archive's notes it lacks added after its last block.

 @param text - what the page carries for the slice

 @param archive - the archive's text for the slice

 @returns The text and each note added; the text unchanged when it lacks none

 @example
 ```ts
 const carried = withArchiveNotes({ text: 'The cat napped.', archive: 'The cat slept. <!-- 大家看一下 -->', },);
 ```
 */
function withArchiveNotes(
  {
    text,
    archive,
  }: {
    readonly text: string;
    readonly archive: string;
  },
): TextRewrite {
  if (rendersAsNothing({ text, },))
    return {
      text,
      changed: [],
    };
  /**
   The archive's notes the text lacks.
   */
  const missing = archiveNotesMissingFrom({
    archive,
    page: text,
  },);
  if (missing.length === 0)
    return {
      text,
      changed: [],
    };
  return {
    text: appendedNotes({
      text,
      notes: missing.map(function noteText(note,): string {
        return note.text;
      },),
    },),
    changed: missing.map(function quoted(note,): string {
      return quotedNote({ text: note.text, },);
    },),
  };
}

/**
 Adds to every slice the notes between translators the archive holds for it
 and the page's wording lacks.

 @param slices - prepared pairs, whose archive text stands where no row replaces it

 @param replacements - what the page would write per slice

 @param archiveOriginalSpans - spans sealed as the English original, which
 ship as the archive has them

 @returns Replacements with the notes added, the rewritten rows alone, and one
 finding per slice changed

 @example
 ```ts
 const page = carryArchiveNotes({ slices, replacements, archiveOriginalSpans: [], },);
 ```
 */
export function carryArchiveNotes(
  {
    slices,
    replacements,
    archiveOriginalSpans,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
    readonly archiveOriginalSpans: readonly ArchiveOriginalSpan[];
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
  readonly findings: readonly string[];
} {
  return rewriteEverySlice({
    slices,
    replacements,
    archiveOriginalSpans,
    rewrite: withArchiveNotes,
    findingName: 'archive-note-carried',
  },);
}

//endregion Archive note carry
