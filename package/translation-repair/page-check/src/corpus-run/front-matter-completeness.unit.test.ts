/**
 Tests fail-closed front matter publication boundary.

 STRUCTURAL CHECKS ONLY, by the owner's decision of 2026-09-02: the metadata
 slice sits where the preparation put it, the page parses, the identity and
 attribution rules hold, and the visible name is not the directory id where
 the source names the person differently. Those apply where the lanes render
 the front matter. Where the archive translated it, the owner's rule of
 2026-09-08 applies instead: the archive's front matter stands, the
 preparation made no metadata slice, and the page carries the archive's
 bytes, which this guard recomputes from the two documents rather than
 trusting the preparation.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { splitFrontMatter, } from '@monochromatic-dev/translation-repair-document-structure';
import { namesDirectoryId, } from '@monochromatic-dev/translation-repair-glossary';
import {
  type ChunkPair,
  frontMatterSlice,
} from '@monochromatic-dev/translation-repair-slice-reading';
import {
  assertFrontMatterComplete,
  FrontMatterCompletenessError,
} from '../../dist/final/node/index.mjs';

/**
 Complete source page fixture.
 */
const SOURCE_TEXT = '---\nname: 猫猫\ninfo:\n  alias: 猫猫\n---\n\nBody.\n';

/**
 Complete target page fixture whose visible name is still the directory id
 beside an English rendering, which makes the id stand (2026-09-07) and so
 the archive's front matter with it (2026-09-08).
 */
const TARGET_TEXT = '---\nname: EntryId\ninfo:\n  alias: Maomao\n---\n\nBody.\n';

/**
 Complete target page fixture whose visible name is the directory id with
 nothing Latin beside it: the directory-id shape, the one the lanes still render.
 */
const FOLDER_TEXT = '---\nname: EntryId\ninfo:\n  alias: 猫咪\n---\n\nBody.\n';

/**
 Complete target page fixture whose metadata is already translated.
 */
const TRANSLATED_TEXT = '---\nname: Maomao\ninfo:\n  alias: Maomao\n---\n\nBody.\n';

/**
 Parsed source metadata fixture.
 */
const sourceFrontMatter = splitFrontMatter({ text: SOURCE_TEXT, }).frontMatter;
/**
 Parsed target metadata fixture.
 */
const targetFrontMatter = splitFrontMatter({ text: TARGET_TEXT, }).frontMatter;
/**
 Parsed translated target metadata fixture.
 */
const translatedFrontMatter = splitFrontMatter({ text: TRANSLATED_TEXT, }).frontMatter;
/**
 Parsed folder-only target metadata fixture.
 */
const folderFrontMatter = splitFrontMatter({ text: FOLDER_TEXT, }).frontMatter;
if ((sourceFrontMatter === undefined) || (targetFrontMatter === undefined) || (translatedFrontMatter === undefined)
  || (folderFrontMatter === undefined))
  throw new Error('front matter fixture did not parse',);
/**
 Explicit metadata slice shared by the rendered cases: the source beside the
 folder-only archive.
 */
const sliceResult = frontMatterSlice({
  source: sourceFrontMatter,
  target: folderFrontMatter,
},);
if (sliceResult.kind !== 'paired')
  throw new Error('front matter fixture did not pair',);
/**
 Explicit metadata slice over the translated archive.
 */
const translatedSliceResult = frontMatterSlice({
  source: sourceFrontMatter,
  target: translatedFrontMatter,
},);
if (translatedSliceResult.kind !== 'paired')
  throw new Error('translated front matter fixture did not pair',);

/**
 Source page whose name and alias differ, so the identity rule in
 `validateFrontMatterTranslation` stays quiet and a directory id as the
 visible name reaches its own check rather than being refused as an invalid
 page first.
 */
const DISTINCT_ALIAS_SOURCE_TEXT = '---\nname: 猫猫\ninfo:\n  alias: 猫咪\n---\n\nBody.\n';

/**
 Parsed distinct-alias source metadata fixture.
 */
const distinctAliasFrontMatter = splitFrontMatter({ text: DISTINCT_ALIAS_SOURCE_TEXT, }).frontMatter;
if (distinctAliasFrontMatter === undefined)
  throw new Error('distinct-alias front matter fixture did not parse',);
/**
 Ordinary body slice preceding metadata in invalid-order fixture.
 */
const BODY_SLICE: ChunkPair = {
  source: {
    kind: 'content',
    sliceIndex: 0,
    nodes: [],
    startOffset: SOURCE_TEXT.length,
    endOffset: SOURCE_TEXT.length,
    text: '',
  },
  target: {
    kind: 'content',
    sliceIndex: 0,
    nodes: [],
    startOffset: FOLDER_TEXT.length,
    endOffset: FOLDER_TEXT.length,
    text: '',
  },
};

/**
 What the guard threw, or `undefined` when it accepted.

 @param run - guarded call

 @returns Thrown value

 @example
 ```ts
 const refusal = thrownBy({ run: () => assertFrontMatterComplete({ ... },), },);
 ```
 */
function thrownBy({ run, }: { readonly run: () => void; },): unknown {
  try {
    run();
    return undefined;
  }
  catch (error) {
    return error;
  }
}

await describe({
  name: assertFrontMatterComplete.name,
  children: [
    it({
      name: 'ACCEPTS PARSEABLE SAME-SHAPE METADATA under explicit reviewed slice where the archive never '
        + 'translated it',
      fn: async () => {
        expect(() => assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: SOURCE_TEXT,
          archiveText: FOLDER_TEXT,
          pageText: TRANSLATED_TEXT,
          slices: [sliceResult.slice,],
        },),).not.toThrow();
      },
    },),

    it({
      name: 'ACCEPTS THE ARCHIVE\'S TRANSLATED METADATA AS IT STANDS with no metadata slice, the owner\'s '
        + 'rule of 2026-09-08, where the 2026-08-28 rule refused the same page as unreviewed and the '
        + '2026-09-02 rule let the lanes rewrite it',
      fn: async () => {
        expect(() => assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: SOURCE_TEXT,
          archiveText: TRANSLATED_TEXT,
          pageText: TRANSLATED_TEXT,
          slices: [],
        },),).not.toThrow();
      },
    },),

    it({
      name: 'REFUSES A RENDERED SLICE OVER A STANDING ARCHIVE, naming archive-front-matter, since a '
        + 'preparation that rendered what the rule leaves alone is refused rather than trusted',
      fn: async () => {
        /**
         What the guard threw for a slice the preparation should not have made.
         */
        const refusal = thrownBy({
          run: () => assertFrontMatterComplete({
            entryId: 'EntryId',
            sourceText: SOURCE_TEXT,
            archiveText: TRANSLATED_TEXT,
            pageText: TRANSLATED_TEXT,
            slices: [translatedSliceResult.slice,],
          },),
        },);
        expect(refusal,).toBeInstanceOf(FrontMatterCompletenessError,);
        expect((refusal as Error).message,).toContain('archive-front-matter',);
      },
    },),

    it({
      name: 'REFUSES A PAGE THAT CHANGES A STANDING ARCHIVE\'S FRONT MATTER, one entry\'s ninth pass: '
        + 'a rendered name over the archive\'s own, and ACCEPTS the archive\'s bytes',
      fn: async () => {
        /**
         The ninth pass's source metadata.
         */
        const mooncatSource = '---\nname: 月见小猫\ninfo:\n    alias: 小月, Tsukimi, Mooncat\n---\n\n正文。\n';
        /**
         The archive's editorial metadata.
         */
        const mooncatArchive = '---\nname: Tsukimi\ninfo:\n    alias: Tsukimi Koneko, Mooncat\n---\n\nBody.\n';
        /**
         What the guard threw for the translate lane's rendering.
         */
        const refusal = thrownBy({
          run: () => assertFrontMatterComplete({
            entryId: 'mooncat',
            sourceText: mooncatSource,
            archiveText: mooncatArchive,
            pageText: '---\nname: Tsukimi Kitten\ninfo:\n    alias: Tsukimi Kitten, Kitten, Tsukimi, Mooncat\n---\n\nBody.\n',
            slices: [],
          },),
        },);
        expect(refusal,).toBeInstanceOf(FrontMatterCompletenessError,);
        expect((refusal as Error).message,).toContain('archive-front-matter',);
        expect(() => assertFrontMatterComplete({
          entryId: 'mooncat',
          sourceText: mooncatSource,
          archiveText: mooncatArchive,
          pageText: mooncatArchive,
          slices: [],
        },),).not.toThrow();
      },
    },),

    it({
      name: 'REFUSES MISSING REVIEW SLICE when both sides declare metadata and the archive never translated it',
      fn: async () => {
        expect(() => assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: SOURCE_TEXT,
          archiveText: FOLDER_TEXT,
          pageText: FOLDER_TEXT,
          slices: [],
        },),).toThrow(FrontMatterCompletenessError,);
      },
    },),

    it({
      name: 'ACCEPTS TRANSLATED SOURCE-ONLY METADATA through insertion slice',
      fn: async () => {
        const sourceOnly = frontMatterSlice({ source: sourceFrontMatter, },);
        if (sourceOnly.kind !== 'paired')
          throw new Error('source-only front matter fixture did not pair',);
        expect(() => assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: SOURCE_TEXT,
          archiveText: 'Body.\n',
          pageText: TRANSLATED_TEXT,
          slices: [sourceOnly.slice,],
        },),).not.toThrow();
      },
    },),

    it({
      name: 'ACCEPTS PAGES OPENING WITH A BYTE ORDER MARK under the metadata slice that starts at each opening '
        + 'fence, and an insertion that stands after the archive\'s mark, and REFUSES that insertion at offset zero, '
        + 'which would write the metadata before the mark',
      fn: async () => {
        /**
         Original and archive metadata of pages opening with the mark.
         */
        const markedSource = splitFrontMatter({ text: `\uFEFF${SOURCE_TEXT}`, },).frontMatter;
        const markedFolder = splitFrontMatter({ text: `\uFEFF${FOLDER_TEXT}`, },).frontMatter;
        if ((markedSource === undefined) || (markedFolder === undefined))
          throw new Error('marked front matter fixture did not parse',);
        /**
         Slice over both marked pages.
         */
        const marked = frontMatterSlice({
          source: markedSource,
          target: markedFolder,
        },);
        /**
         Insertion over a marked archive without metadata, at the offset after its mark.
         */
        const inserted = frontMatterSlice({
          source: markedSource,
          targetInsertionOffset: 1,
        },);
        /**
         Insertion at offset zero over the same archive.
         */
        const misplaced = frontMatterSlice({ source: markedSource, },);
        if ((marked.kind !== 'paired') || (inserted.kind !== 'paired') || (misplaced.kind !== 'paired'))
          throw new Error('marked front matter fixture did not pair',);
        expect(() => assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: `\uFEFF${SOURCE_TEXT}`,
          archiveText: `\uFEFF${FOLDER_TEXT}`,
          pageText: `\uFEFF${TRANSLATED_TEXT}`,
          slices: [marked.slice,],
        },),).not.toThrow();
        expect(() => assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: `\uFEFF${SOURCE_TEXT}`,
          archiveText: '\uFEFFBody.\n',
          pageText: `\uFEFF${TRANSLATED_TEXT}`,
          slices: [inserted.slice,],
        },),).not.toThrow();
        expect(thrownBy({
          run: function assertMisplaced(): void {
            assertFrontMatterComplete({
              entryId: 'EntryId',
              sourceText: `\uFEFF${SOURCE_TEXT}`,
              archiveText: '\uFEFFBody.\n',
              pageText: `\uFEFF${TRANSLATED_TEXT}`,
              slices: [misplaced.slice,],
            },);
          },
        },),).toEqual(new FrontMatterCompletenessError({
          entryId: 'EntryId',
          reason: 'missing-slice',
        },),);
      },
    },),

    it({
      name: 'ACCEPTS UNCHANGED TARGET-ONLY METADATA without localized slice',
      fn: async () => {
        expect(() => assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: 'Body.\n',
          archiveText: TARGET_TEXT,
          pageText: TARGET_TEXT,
          slices: [],
        },),).not.toThrow();
      },
    },),

    it({
      name: 'REFUSES A VISIBLE NAME THAT IS THE DIRECTORY ID with no English rendering beside it, '
        + 'naming directory-id-name, whether the page kept the archive byte for byte or changed '
        + 'another field, since the person would ship under the folder either way',
      fn: async () => {
        /**
         Archive and page whose alias is in the source script only, so no
         rendering stands beside the folder name.
         */
        const folderOnlyText = '---\nname: EntryId\ninfo:\n  alias: 猫咪\n---\n\nBody.\n';

        /**
         Parsed folder-only metadata.
         */
        const folderOnly = splitFrontMatter({ text: folderOnlyText, },).frontMatter;
        if (folderOnly === undefined)
          throw new Error('folder-only fixture did not parse',);

        /**
         Explicit metadata slice over the folder-only pages.
         */
        const folderOnlySlice = frontMatterSlice({
          source: distinctAliasFrontMatter,
          target: folderOnly,
        },);
        if (folderOnlySlice.kind !== 'paired')
          throw new Error('folder-only fixture did not pair',);

        /**
         What the guard threw for the archive kept byte for byte.
         */
        const keptRefusal = thrownBy({
          run: () => assertFrontMatterComplete({
            entryId: 'EntryId',
            sourceText: DISTINCT_ALIAS_SOURCE_TEXT,
            archiveText: folderOnlyText,
            pageText: folderOnlyText,
            slices: [folderOnlySlice.slice,],
          },),
        },);
        expect(keptRefusal,).toBeInstanceOf(FrontMatterCompletenessError,);
        expect((keptRefusal as Error).message,).toContain('directory-id-name',);

        /**
         What the guard threw for a page that changed the alias to another
         source-script rendering and left the directory id as the name.
         */
        const changedRefusal = thrownBy({
          run: () => assertFrontMatterComplete({
            entryId: 'EntryId',
            sourceText: DISTINCT_ALIAS_SOURCE_TEXT,
            archiveText: folderOnlyText,
            pageText: '---\nname: EntryId\ninfo:\n  alias: 小猫\n---\n\nBody.\n',
            slices: [folderOnlySlice.slice,],
          },),
        },);
        expect(changedRefusal,).toBeInstanceOf(FrontMatterCompletenessError,);
        expect((changedRefusal as Error).message,).toContain('directory-id-name',);
      },
    },),

    it({
      name: 'ACCEPTS A VISIBLE NAME THAT IS THE DIRECTORY ID beside an English rendering in the '
        + 'alias, the owner\'s decision of 2026-09-07, since the front matter then carries the '
        + 'name in English; such an archive stands, so it ships with no slice',
      fn: async () => {
        // TARGET_TEXT names the directory and carries `Maomao` as an alias.
        expect(() => assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: DISTINCT_ALIAS_SOURCE_TEXT,
          archiveText: TARGET_TEXT,
          pageText: TARGET_TEXT,
          slices: [],
        },),).not.toThrow();
      },
    },),

    it({
      name: 'ACCEPTS A VISIBLE NAME THAT IS THE DIRECTORY ID when the id is the pinyin of the '
        + 'source name and nothing else in the front matter is Latin (one entry, 2026-09-07)',
      fn: async () => {
        /**
         Source naming the person 猫童 with a distinct alias.
         */
        const pinyinSourceText = '---\nname: 猫童\ninfo:\n  alias: 小童\n---\n\n正文。\n';

        /**
         Page naming the person by the pinyin, which is the id.
         */
        const pinyinPageText = '---\nname: maotong\ninfo:\n  alias: 小童\n---\n\nBody.\n';

        /**
         Parsed source metadata.
         */
        const source = splitFrontMatter({ text: pinyinSourceText, },).frontMatter;

        /**
         Parsed page metadata.
         */
        const target = splitFrontMatter({ text: pinyinPageText, },).frontMatter;
        if ((source === undefined) || (target === undefined))
          throw new Error('pinyin fixture did not parse',);

        expect(namesDirectoryId({
          metadata: target,
          entryId: 'maotong',
        },),).toBe(true,);
        expect(namesDirectoryId({
          metadata: source,
          entryId: 'maotong',
        },),).toBe(false,);
        // The id stands as the pinyin, so the archive stands and ships with no slice.
        expect(() => assertFrontMatterComplete({
          entryId: 'maotong',
          sourceText: pinyinSourceText,
          archiveText: pinyinPageText,
          pageText: pinyinPageText,
          slices: [],
        },),).not.toThrow();
      },
    },),

    it({
      name: 'ACCEPTS A VISIBLE NAME THAT IS THE DIRECTORY ID when the source names the person so '
        + 'too, since the handle is then the name: eight of the pinned corpus\'s 92 entries '
        + 'are named after their directory in both languages',
      fn: async () => {
        /**
         Source page whose name is the directory id, as a handle-named entry's is.
         */
        const handleSourceText = '---\nname: EntryId\ninfo:\n  alias: EntryId\n---\n\n正文。\n';
        /**
         Archive and page carrying the same handle.
         */
        const handlePageText = '---\nname: EntryId\ninfo:\n  alias: EntryId\n---\n\nBody.\n';
        /**
         Parsed handle source metadata.
         */
        const source = splitFrontMatter({ text: handleSourceText, },).frontMatter;
        /**
         Parsed handle page metadata.
         */
        const target = splitFrontMatter({ text: handlePageText, },).frontMatter;
        if ((source === undefined) || (target === undefined))
          throw new Error('handle fixture did not parse',);
        expect(namesDirectoryId({
          metadata: source,
          entryId: 'EntryId',
        },),).toBe(true,);
        expect(namesDirectoryId({
          metadata: target,
          entryId: 'EntryId',
        },),).toBe(true,);
        // The handle is the name, so the archive stands and ships with no slice.
        expect(() => assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: handleSourceText,
          archiveText: handlePageText,
          pageText: handlePageText,
          slices: [],
        },),).not.toThrow();
      },
    },),

    it({
      name: 'REFUSES A CHANGED PAGE whose name and alias diverge where the source\'s agree, as '
        + 'invalid-page, which is the identity rule',
      fn: async () => {
        /**
         What the guard threw for a changed page breaking the identity rule.
         */
        const refusal = thrownBy({
          run: () => assertFrontMatterComplete({
            entryId: 'EntryId',
            sourceText: SOURCE_TEXT,
            archiveText: FOLDER_TEXT,
            pageText: '---\nname: EntryId2\ninfo:\n  alias: Maomao\n---\n\nBody.\n',
            slices: [sliceResult.slice,],
          },),
        },);
        expect(refusal,).toBeInstanceOf(FrontMatterCompletenessError,);
        expect((refusal as Error).message,).toContain('invalid-page',);
      },
    },),

    it({
      name: 'ACCEPTS A PAGE WHOSE ALIAS CARRIES THE NAME AMONG OTHER RENDERINGS where the source\'s '
        + 'name and alias agree, the owner\'s decision of 2026-09-04: seven archives at the pinned corpus '
        + 'publish the original script beside the romanisation, and equality refused one of them after a '
        + 'full run',
      fn: async () => {
        // A throw here fails the test on its own.
        assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: SOURCE_TEXT,
          archiveText: FOLDER_TEXT,
          pageText: '---\nname: Maomao\ninfo:\n  alias: 猫猫, Maomao\n---\n\nBody.\n',
          slices: [sliceResult.slice,],
        },);
        /**
         What the guard threw for an alias that names the person by something
         else entirely.
         */
        const refusal = thrownBy({
          run: () => assertFrontMatterComplete({
            entryId: 'EntryId',
            sourceText: SOURCE_TEXT,
            archiveText: FOLDER_TEXT,
            pageText: '---\nname: Maomao\ninfo:\n  alias: 猫猫, Kitty\n---\n\nBody.\n',
            slices: [sliceResult.slice,],
          },),
        },);
        expect(refusal,).toBeInstanceOf(FrontMatterCompletenessError,);
        expect((refusal as Error).message,).toContain('invalid-page',);
      },
    },),

    it({
      name: 'REFUSES SOURCE-SCRIPT COMMENT ATTRIBUTION replacing established target form',
      fn: async () => {
        /**
         Source page carrying contributor attribution in location comment.
         */
        const sourceText = '---\nname: 猫猫\ninfo:\n  alias: 猫猫\n  location: 广东 #清远, by 魔骨\n---\n\nBody.\n';
        /**
         Archive page establishing target contributor spelling.
         */
        const archiveText = '---\nname: CatEntry\ninfo:\n  alias: 猫猫\n  location: Guangdong #Qingyuan, by MoguHandle\n---\n\nBody.\n';
        /**
         Candidate retaining source-script attribution.
         */
        const pageText = '---\nname: Maomao\ninfo:\n  alias: Maomao\n  location: Guangdong #Qingyuan, by 魔骨\n---\n\nBody.\n';
        /**
         Parsed source metadata.
         */
        const source = splitFrontMatter({ text: sourceText, }).frontMatter;
        /**
         Parsed archive metadata.
         */
        const target = splitFrontMatter({ text: archiveText, }).frontMatter;
        if ((source === undefined) || (target === undefined))
          throw new Error('comment authority fixture did not parse',);
        /**
         Explicit metadata slice at publication boundary.
         */
        const result = frontMatterSlice({ source, target, });
        if (result.kind !== 'paired')
          throw new Error('comment authority fixture did not pair',);
        /**
         What the guard threw for the source-script attribution.
         */
        const refusal = thrownBy({
          run: () => assertFrontMatterComplete({
            entryId: 'CatEntry',
            sourceText,
            archiveText,
            pageText,
            slices: [result.slice,],
          },),
        },);
        expect(refusal,).toBeInstanceOf(FrontMatterCompletenessError,);
        expect((refusal as Error).message,).toContain('invalid-page',);
      },
    },),

    it({
      name: 'REFUSES PAGE THAT DROPS TARGET METADATA FIELD',
      fn: async () => {
        expect(() => assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: SOURCE_TEXT,
          archiveText: FOLDER_TEXT,
          pageText: '---\nname: Maomao\n---\n\nBody.\n',
          slices: [sliceResult.slice,],
        },),).toThrow(FrontMatterCompletenessError,);
      },
    },),

    it({
      name: 'REFUSES METADATA SLICE AT WRONG INDEX OR SPAN',
      fn: async () => {
        expect(() => assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: SOURCE_TEXT,
          archiveText: FOLDER_TEXT,
          pageText: TRANSLATED_TEXT,
          slices: [{
            ...sliceResult.slice,
            source: {
              ...sliceResult.slice.source,
              sliceIndex: 1,
            },
          },],
        },),).toThrow(FrontMatterCompletenessError,);
        expect(() => assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: SOURCE_TEXT,
          archiveText: FOLDER_TEXT,
          pageText: TRANSLATED_TEXT,
          slices: [BODY_SLICE, sliceResult.slice,],
        },),).toThrow(FrontMatterCompletenessError,);
      },
    },),

    it({
      name: 'ACCEPTS TWO PAGES WITHOUT FRONT MATTER and requires no synthetic slice',
      fn: async () => {
        expect(() => assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: 'Body.\n',
          archiveText: 'Body.\n',
          pageText: 'Body.\n',
          slices: [],
        },),).not.toThrow();
        expect(() => assertFrontMatterComplete({
          entryId: 'EntryId',
          sourceText: 'Body.\n',
          archiveText: 'Body.\n',
          pageText: '---\nname: Added\n---\n\nBody.\n',
          slices: [],
        },),).toThrow(FrontMatterCompletenessError,);
      },
    },),

    it({
      name: 'REFUSES A CHANGED TARGET-ONLY METADATA when the source declares none at all, naming '
        + 'invalid-page, since nothing was translated and the page must keep the archive\'s own bytes',
      fn: async () => {
        /**
         What the guard threw for a target-only archive whose bytes the page changed.
         */
        const refusal = thrownBy({
          run: () => assertFrontMatterComplete({
            entryId: 'EntryId',
            sourceText: 'The cat sleeps on the windowsill.\n',
            archiveText: TARGET_TEXT,
            pageText: '---\nname: EntryId\ninfo:\n  alias: Changed\n---\n\nBody.\n',
            slices: [],
          },),
        },);
        expect(refusal,).toBeInstanceOf(FrontMatterCompletenessError,);
        expect((refusal as Error).message,).toContain('invalid-page',);
      },
    },),

    it({
      name: 'REFUSES A PAGE WHOSE ASSEMBLED TEXT DROPPED THE METADATA BLOCK, naming invalid-page, even '
        + 'though the slice bookkeeping still names the right source and archive bytes',
      fn: async () => {
        /**
         What the guard threw for an assembled page with no fence pair at all.
         */
        const refusal = thrownBy({
          run: () => assertFrontMatterComplete({
            entryId: 'EntryId',
            sourceText: SOURCE_TEXT,
            archiveText: FOLDER_TEXT,
            pageText: 'The cat sleeps on the windowsill.\n',
            slices: [sliceResult.slice,],
          },),
        },);
        expect(refusal,).toBeInstanceOf(FrontMatterCompletenessError,);
        expect((refusal as Error).message,).toContain('invalid-page',);
      },
    },),

    it({
      name: 'ACCEPTS A FRESH INSERTION WHOSE SOURCE NAMES ITSELF BY THE DIRECTORY ID, with no prior '
        + 'archive to consult',
      fn: async () => {
        /**
         Entry id carrying a digit, so no pinyin reading of a name can also
         spell it: verified against the installed pinyin-pro package, a digit
         contributes no Latin letters to the reading walk.
         */
        const entryId = 'Tuxedo7';
        /**
         Source page naming itself by the directory id, with a script alias
         distinct from it.
         */
        const sourceText = '---\nname: Tuxedo7\ninfo:\n  alias: 小猫\n---\n\n正文。\n';
        /**
         Parsed source metadata.
         */
        const source = splitFrontMatter({ text: sourceText, },).frontMatter;
        if (source === undefined)
          throw new Error('handle-digit source fixture did not parse',);
        /**
         Insertion slice over the source-only metadata.
         */
        const insertion = frontMatterSlice({ source, },);
        if (insertion.kind !== 'paired')
          throw new Error('handle-digit source fixture did not pair',);

        expect(() => assertFrontMatterComplete({
          entryId,
          sourceText,
          archiveText: 'Body.\n',
          pageText: '---\nname: Tuxedo7\ninfo:\n  alias: 小猫\n---\n\nBody.\n',
          slices: [insertion.slice,],
        },),).not.toThrow();
      },
    },),

    it({
      name: 'ACCEPTS A PAGE THAT ADDS THE LATIN ALIAS A STILL-UNTRANSLATED ARCHIVE NEVER HAD, clearing '
        + 'the directory-id check at publication whether or not an archive exists to consult',
      fn: async () => {
        /**
         Source naming the person in script the id does not spell: verified
         against the installed pinyin-pro package, neither reading of 雪爪
         (xue + zhua or zhao) produces the letters `snowpaw`.
         */
        const sourceText = '---\nname: 雪爪\ninfo:\n  alias: 小雪\n---\n\n正文。\n';
        /**
         Parsed source metadata.
         */
        const source = splitFrontMatter({ text: sourceText, },).frontMatter;
        if (source === undefined)
          throw new Error('snowpaw source fixture did not parse',);
        /**
         Archive still showing the directory id as the name, with a
         script-only alias that carries no Latin rendering of its own.
         */
        const archiveText = '---\nname: Snowpaw\ninfo:\n  alias: 小雪\n---\n\nBody.\n';
        /**
         Parsed archive metadata.
         */
        const archive = splitFrontMatter({ text: archiveText, },).frontMatter;
        if (archive === undefined)
          throw new Error('snowpaw archive fixture did not parse',);
        /**
         Explicit metadata slice the still-untranslated archive requires rendering.
         */
        const renderedSlice = frontMatterSlice({ source, target: archive, },);
        if (renderedSlice.kind !== 'paired')
          throw new Error('snowpaw archive fixture did not pair',);
        /**
         Insertion slice for this case's no-archive variant.
         */
        const insertionSlice = frontMatterSlice({ source, },);
        if (insertionSlice.kind !== 'paired')
          throw new Error('snowpaw insertion fixture did not pair',);
        /**
         Page whose alias now carries a Latin rendering distinct from the id,
         an exemption neither the source nor the archive alone establishes.
         */
        const pageWithLatinAlias = '---\nname: Snowpaw\ninfo:\n  alias: 小雪, Xuezhua\n---\n\nBody.\n';

        // Against an existing, still-untranslated archive.
        expect(() => assertFrontMatterComplete({
          entryId: 'Snowpaw',
          sourceText,
          archiveText,
          pageText: pageWithLatinAlias,
          slices: [renderedSlice.slice,],
        },),).not.toThrow();

        // Against no archive at all, a fresh insertion.
        expect(() => assertFrontMatterComplete({
          entryId: 'Snowpaw',
          sourceText,
          archiveText: 'Body.\n',
          pageText: pageWithLatinAlias,
          slices: [insertionSlice.slice,],
        },),).not.toThrow();

        /**
         What the guard threw for a page keeping the script-only alias, with
         no Latin rendering anywhere to clear the exemption.
         */
        const refusal = thrownBy({
          run: () => assertFrontMatterComplete({
            entryId: 'Snowpaw',
            sourceText,
            archiveText,
            pageText: archiveText,
            slices: [renderedSlice.slice,],
          },),
        },);
        expect(refusal,).toBeInstanceOf(FrontMatterCompletenessError,);
        expect((refusal as Error).message,).toContain('directory-id-name',);
      },
    },),
  ],
},);
