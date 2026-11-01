import {
  commentBody,
  foldedLine,
} from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import {
  isCommentFinding,
  type RepairDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';


//region Archive-original notes
// WHERE THE ARCHIVE IS THE ORIGINAL, said by the archive's own translators in
// an HTML comment, and what the pipeline does about it. Decided by the owner
// on 2026-09-08 after the twelfth hakureico pass: the archive's note above
// Hanasaka's letter says everything below it was written in English and the
// Chinese is a back-translation, and the page still rewrote the letter in five
// places ("I am never gone" became "I am never really gone"). The rule is the
// front-matter rule's shape applied to a span: where a note says the English
// is the original, the archive's text ships as it stands and no lane writes
// it; where a note says the WHOLE PAGE is the original, the pipeline declines
// the entry rather than repairing it.
//
// THREE WORDINGS IN THE PINNED CORPUS, measured 2026-09-08 and again on
// 2026-09-09: 22 of 93 archive pages carry a translator note, and 3 of them
// say the English is the original, two in Chinese and one in English. `hakureico` carries `这段话以下全部，包括结尾的两句祝愿，原文都是英文，
// 中文是反向翻译的，请仅修可能造成误解或明显的非刻意语法错误，不大修` above the
// letter (a SPAN: everything below the note), and `cheonwoomaeng` carries
// `这篇文章的原文即英文，作者的第一语言为英语，请翻译时不要动本篇。` (the WHOLE PAGE:
// do not touch this page when translating). The readings are keyed to the
// marks those two notes carry. A note that speaks of an original and of
// English in a wording no mark knows reads as an unmarked claim: it seals
// nothing, since a guessed seal could decline a whole page, and the pass logs
// every note with its reading and warns on an unmarked claim
// (`corpus-run/pass-decline.ts`), so a third wording is seen rather than
// silently unsealed. Until ledger E12 (2026-09-28) this comment promised that
// log and no code wrote it.
//
// NOT THE QUOTES NOTE. `hakureico`'s first note (`本文的大部分引用原文都是英文，
// 引用部分请仅修语法和可能造成误解的错误`) says MOST quoted passages were English
// and names no span, so it seals nothing; the lanes read it as the advisory it
// is, and the two short quotes under it shipped verbatim on every read page.
//
// THE ENGLISH WORDING WAS MISSED. `gqt` carries `(Original Language: Engish)`
// at the top of both its pages, the typo the pinned corpus's own, and the
// fourth `gqt` pass of 2026-09-09 repaired Ara's English page from its Chinese
// back-translation before that note was read as anything but advisory. The
// twenty-second class. The English marks are matched on the lowercased note.

/**
 Mark a note carries when the whole page is the author's own English.
 */
const WHOLE_PAGE_MARKS = [
  '原文即英文',
  '不要动本篇',
] as const;

/**
 Marks a note carries, in English, when the whole page is the author's own
 English; the misspelt one is the pinned corpus's spelling on `gqt`.
 */
const ENGLISH_WHOLE_PAGE_MARKS = [
  'original language: english',
  'original language: engish',
] as const;

/**
 Marks a note carries, all of them, when everything below it is the
 English original: "below" (以下), "original" (原文) and "English" (英文).
 */
const SPAN_MARKS = [
  '以下',
  '原文',
  '英文',
] as const;

/**
 Words a note uses for an original, lower case.
 */
const ORIGINAL_TERMS = [
  '原文',
  'original',
] as const;

/**
 Words a note uses for English, lower case, the pinned corpus's misspelling
 on `gqt` among them.
 */
const ENGLISH_TERMS = [
  '英文',
  'english',
  'engish',
] as const;

/**
 Node kind the parser gives a heading, which ends a sealed span.
 */
const HEADING_KIND = 'heading';

/**
 One span of the archive that ships as it stands.

 @example
 ```ts
 const span: ArchiveOriginalSpan = { startOffset: 3966, endOffset: 4561, note: '这段话以下全部...', };
 ```
 */
export type ArchiveOriginalSpan = {
  /**
   First archive-text offset the seal covers: the end of the note.
   */
  readonly startOffset: number;

  /**
   Exclusive end: the next heading's start, or the archive's end.
   */
  readonly endOffset: number;

  /**
   What the translators wrote, folded onto one line.
   */
  readonly note: string;
};

/**
 What one archive's notes say about whose text the page carries.

 @example
 ```ts
 const reading: ArchiveOriginalReading = { kind: 'none', };
 ```
 */
export type ArchiveOriginalReading = {
  /**
   A note says the whole page is the author's English: decline the entry.
   */
  readonly kind: 'whole-page';

  /**
   The note, folded onto one line.
   */
  readonly note: string;
} | {
  /**
   One or more notes each seal everything below them.
   */
  readonly kind: 'spans';

  /**
   Sealed spans in document order.
   */
  readonly spans: readonly ArchiveOriginalSpan[];
} | {
  /**
   No note claims the archive as the original.
   */
  readonly kind: 'none';
};

/**
 How one note reads under the marks: it seals the whole page, it seals a
 span, it speaks of an English original in a wording no mark knows (ledger
 E12: sealing nothing, since a guess could decline a page, but logged by the
 pass so a new wording is seen), or it is advice to the lanes.
 */
export type NoteReading = 'whole-page' | 'span' | 'unmarked-original-claim' | 'advisory';

/**
 One editor comment and how it reads.
 */
export type NoteWithReading = Readonly<{
  /**
   What the translators wrote, folded onto one line.
   */
  note: string;

  /**
   How it reads under the marks.
   */
  reading: NoteReading;
}>;

/**
 One editor comment, folded, beside where its seal would start.
 */
type PlacedNote = {
  /**
   What the translators wrote, on one line.
   */
  readonly note: string;

  /**
   Offset just past the comment's closing delimiter.
   */
  readonly endOffset: number;
};

/**
 Reads one note by the marks it carries.

 @param note - note text, folded

 @returns Whether it seals the page, a span, or nothing

 @example
 ```ts
 readNote({ note: '这篇文章的原文即英文，请翻译时不要动本篇。', },);
 // => 'whole-page'
 ```
 */
export function readNote(
  { note, }: { readonly note: string; },
): NoteReading {
  if (WHOLE_PAGE_MARKS.some(function carried(mark,): boolean {
    return note.includes(mark,);
  },))
    return 'whole-page';
  /**
   The note lowercased, since the English marks are spelled either way.
   */
  const lowered = note.toLowerCase();
  if (ENGLISH_WHOLE_PAGE_MARKS.some(function carriedInEnglish(mark,): boolean {
    return lowered.includes(mark,);
  },))
    return 'whole-page';
  if (SPAN_MARKS.every(function carried(mark,): boolean {
    return note.includes(mark,);
  },))
    return 'span';
  /**
   Whether the note speaks of an original and of English at all.
   */
  const claims = ORIGINAL_TERMS.some(function namesOriginal(term,): boolean {
    return lowered.includes(term,);
  },) && ENGLISH_TERMS.some(function namesEnglish(term,): boolean {
    return lowered.includes(term,);
  },);
  return claims ? 'unmarked-original-claim' : 'advisory';
}

/**
 Every editor comment of a document with where it ends, in document order.

 @param document - parsed archive

 @returns Folded note text beside the offset the seal would start at

 @example
 ```ts
 const notes = documentNotes({ document, },);
 ```
 */
function documentNotes(
  { document, }: { readonly document: RepairDocument; },
): readonly PlacedNote[] {
  return document.parseFindings
    .filter(function isComment(finding,): boolean {
      return isCommentFinding({ finding, },);
    },)
    .map(function toNote(finding,): PlacedNote {
      /**
       Comment as it stands, delimiters included.
       */
      const comment = document.text
        .slice(
        finding.startOffset,
        finding.endOffset,
        );
      return {
        note: foldedLine({ text: commentBody({ comment, },), },),
        endOffset: finding.endOffset,
      };
    },);
}

/**
 Every editor comment of an archive and how it reads, in document order,
 for the pass to log (ledger E12).

 @param document - parsed archive

 @returns Each note beside its reading

 @example
 ```ts
 const notes = archiveNoteReadingsOf({ document, },);
 ```
 */
export function archiveNoteReadingsOf(
  { document, }: { readonly document: RepairDocument; },
): readonly NoteWithReading[] {
  return documentNotes({ document, },)
    .map(function withReading({ note, },): NoteWithReading {
      return {
        note,
        reading: readNote({ note, },),
      };
    },);
}

/**
 Where a span opened at one offset ends: the next heading, or the document.

 @param document - parsed archive

 @param startOffset - where the seal starts

 @returns Exclusive end offset

 @example
 ```ts
 spanEnd({ document, startOffset: 3966, },);
 // => 4561
 ```
 */
function spanEnd(
  {
    document,
    startOffset,
  }: {
    readonly document: RepairDocument;
    readonly startOffset: number;
  },
): number {
  /**
   First heading that starts at or after the seal.
   */
  const heading = document.nodes
    .find(function follows(node,): boolean {
      return (node.kind === HEADING_KIND) && (node.startOffset >= startOffset);
    },);
  if (heading === undefined)
    return document.text
      .length;
  return heading.startOffset;
}

/**
 Reads what an archive's notes say about whose text the page carries.

 A whole-page note outranks every span: the entry is declined and no seal
 matters. Spans that overlap (a second span note inside the first's reach)
 are merged by taking the earlier start, since the seal is the same either
 way.

 @param document - parsed archive, before any lane ran

 @returns The reading

 @example
 ```ts
 const reading = archiveOriginalReadingOf({ document: parseDocument({ text: archiveText, },), },);
 if (reading.kind === 'whole-page') decline();
 ```
 */
export function archiveOriginalReadingOf(
  { document, }: { readonly document: RepairDocument; },
): ArchiveOriginalReading {
  /**
   Every note beside how it reads.
   */
  const notes = documentNotes({ document, },)
    .map(function withReading(entry,): PlacedNote & { readonly reading: NoteReading; } {
      return {
        ...entry,
        reading: readNote({ note: entry.note, },),
      };
    },);
  /**
   The first whole-page note, which decides the entry alone.
   */
  const wholePage = notes.find(function seals(entry,): boolean {
    return entry.reading === 'whole-page';
  },);
  if (wholePage !== undefined)
    return {
      kind: 'whole-page',
      note: wholePage.note,
    };
  /**
   Span notes, each sealing from its end to the next heading.
   */
  const spans = notes
    .filter(function sealsSpan(entry,): boolean {
      return entry.reading === 'span';
    },)
    .map(function toSpan(entry,): ArchiveOriginalSpan {
      return {
        startOffset: entry.endOffset,
        endOffset: spanEnd({
          document,
          startOffset: entry.endOffset,
        },),
        note: entry.note,
      };
    },)
    // A SPAN INSIDE ANOTHER'S REACH adds nothing: the earlier note already
    // seals to the same heading.
    .filter(function firstOfItsReach(
      span,
      at,
      all: readonly ArchiveOriginalSpan[],
    ): boolean {
      return !all.some(function reachesOver(
        other,
        otherAt,
      ): boolean {
        return (otherAt < at)
          && (other.startOffset <= span.startOffset)
          && (other.endOffset >= span.endOffset);
      },);
    },);
  if (spans.length === 0)
    return { kind: 'none', };
  return {
    kind: 'spans',
    spans,
  };
}

//endregion Archive-original notes
