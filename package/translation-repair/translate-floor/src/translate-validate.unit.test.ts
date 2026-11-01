/**
 Tests for the structural comparison between a translated slice and its
 original.

 What this can and cannot claim is the whole design, so the cases are chosen
 to pin both edges. It compares what survives a translation, meaning block
 structure, footnote markers, link and image destinations, and inline code. It
 says nothing about wording, and the numbers-and-names atoms that
 `inspect-paragraph.ts` protects are deliberately absent here, because 三只猫
 becomes "three cats" and no digit survives on either side.

 Findings are written for the MODEL that produced the candidate rather than
 for a log, since an invalid candidate is handed back to its author rather
 than dropped, so the cases assert on what those sentences say.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

/**
 Original carrying a heading, a paragraph with a footnote marker, and that
 footnote's definition.
 */
const SOURCE_TEXT = `## 猫猫的一天

早上它在窗台上打盹[^1]。

[^1]: 窗台朝东。`;

/**
 What the untranslated floor tells a candidate that copied the original.
 */
const UNTRANSLATED_FINDING = 'Your translation repeats the ORIGINAL untranslated, in all but whitespace. Write the '
  + 'passage in English; keep only the names, handles, links and code the original carries.';

/**
 A page neither grammar reads: plain markdown accepts any text, and refuses
 only when reading exhausts the parser, and 16,000 nested quotation markers
 overflow its stack where 8,000 already did when measured on 2026-09-30.
 */
const UNREADABLE_PAGE = `${'>'.repeat(16_000,)} cat`;

await describe({
  name: validateTranslatedSlice.name,
  children: [
    it({
      name: 'accepts a translation that carries the same blocks and the same '
        + 'markers, which is the case that must stay quiet: a validator that '
        + 'fired on ordinary work would send every candidate back to its '
        + 'author and double the lane',
      fn: async () => {
        expect(
          validateTranslatedSlice({
            sourceText: SOURCE_TEXT,
            candidateText: `## A Day in the Cat's Life

In the morning it dozes on the windowsill[^1].

[^1]: The windowsill faces east.`,
          },).kind,
        ).toBe('valid',);
      },
    },),

    it({
      name: 'REPORTS the neutral pronoun left untranslated, naming the spelling and its count, and stays '
        + 'quiet on the same letters inside a word or a handle (one entry, slice 5, 2026-09-04: an '
        + 'untranslated Ta shipped on a page that says they everywhere else)',
      fn: async () => {
        const verdict = validateTranslatedSlice({
          // AN ORIGINAL THAT WRITES Ta, as the slice this case stands for did:
          // the floor asks only where the original writes the pronoun (F-9).
          sourceText: SOURCE_TEXT.replace('它', 'Ta ',),
          candidateText: `## A Day in the Cat's Life

In the morning Ta dozes on the windowsill[^1], and Ta's DATA stays with @ta_cat.

[^1]: The windowsill faces east.`,
        },);

        expect(verdict.kind,).toBe('invalid',);
        if (verdict.kind !== 'invalid')
          throw new Error('expected invalid',);
        expect(verdict.findings.length,).toBe(1,);
        expect(verdict.findings[0],).toContain('untranslated as "Ta" (2 times)',);
        expect(verdict.findings[0],).toContain('singular they',);
      },
    },),

    it({
      name: 'REPORTS a backslash before a quotation mark as a JSON escape leaked into the text, and stays '
        + 'quiet where the original or the page carries the sequence itself: the Carena0442 page of '
        + '2026-09-02 shipped `so-called \\"common sense.\\"` through every guard',
      fn: async () => {
        /**
         Verdict over a candidate carrying the leaked escape.
         */
        const leaked = validateTranslatedSlice({
          sourceText: '早上它在所谓的“窗台”上打盹。',
          pageText: 'In the morning it dozes on the so-called “windowsill.”',
          candidateText: String.raw`In the morning it dozes on the so-called \"windowsill.\"`,
        },);
        expect(leaked.kind,).toBe('invalid',);
        expect(
          (leaked.kind === 'invalid') ? leaked.findings.join('\n',) : '',
        ).toContain('backslash before a quotation mark',);

        /**
         Verdict over the same candidate where the original carries the
         sequence, so it is a rendering rather than a leak.
         */
        const carried = validateTranslatedSlice({
          sourceText: String.raw`早上它在所谓的 \"窗台\" 上打盹。`,
          pageText: 'In the morning it dozes on the so-called “windowsill.”',
          candidateText: String.raw`In the morning it dozes on the so-called \"windowsill.\"`,
        },);
        expect(carried.kind,).toBe('valid',);
      },
    },),

    it({
      name: 'REPORTS a translation that merged the original heading into its '
        + 'prose, naming both shapes. The translator sheet asks for one block '
        + 'per original block, so a merge is the model not doing what it was '
        + 'asked rather than a rendering choice',
      fn: async () => {
        /**
         Verdict over a candidate that dropped the heading level.
         */
        const validation = validateTranslatedSlice({
          sourceText: SOURCE_TEXT,
          candidateText: `A Day in the Cat's Life. In the morning it dozes on the windowsill[^1].

[^1]: The windowsill faces east.`,
        },);
        expect(validation.kind,).toBe('invalid',);
        expect(
          (validation.kind === 'invalid') ? validation.findings.join('\n',) : '',
        ).toContain('heading (level 2)',);
      },
    },),

    it({
      name: 'REPORTS a footnote marker the translation dropped, which reads '
        + 'perfectly well to a judge and breaks the document: the definition '
        + 'is left pointing at nothing once the slice is spliced back in',
      fn: async () => {
        /**
         Verdict over a candidate that dropped the reference but kept the
         definition.
         */
        const validation = validateTranslatedSlice({
          sourceText: SOURCE_TEXT,
          candidateText: `## A Day in the Cat's Life

In the morning it dozes on the windowsill.

[^1]: The windowsill faces east.`,
        },);
        expect(validation.kind,).toBe('invalid',);
        expect(
          (validation.kind === 'invalid') ? validation.findings.join('\n',) : '',
        ).toContain('footnote 1',);
      },
    },),

    it({
      name: 'REPORTS a link the translation invented, since a destination is '
        + 'not something a rendering can supply: it came from the model rather '
        + 'than from the passage',
      fn: async () => {
        /**
         Verdict over a candidate carrying a link the original never had.
         */
        const validation = validateTranslatedSlice({
          sourceText: '猫猫在窗台上打盹。',
          candidateText: 'The [cat](https://cats.example/naps) dozes on the windowsill.',
        },);
        expect(validation.kind,).toBe('invalid',);
        expect(
          (validation.kind === 'invalid') ? validation.findings.join('\n',) : '',
        ).toContain('https://cats.example/naps',);
      },
    },),

    it({
      name: 'ignores where a link MOVED to, because a translation reorders '
        + 'clauses legitimately and a destination shifting within a sentence '
        + 'is not damage. Comparing atoms in order rather than as a multiset '
        + 'would refuse ordinary work',
      fn: async () => {
        expect(
          validateTranslatedSlice({
            sourceText: '见[窗台](https://a.example)和[暖气片](https://b.example)。',
            candidateText: 'See the [radiator](https://b.example) and the [windowsill](https://a.example).',
          },).kind,
        ).toBe('valid',);
      },
    },),

    it({
      name: 'ACCEPTS EITHER DESTINATION where the page rewrote the original\'s link, '
        + 'and REFUSES neither or both: one archive links x.com where the '
        + 'original links twitter.com, and owing a candidate both left nothing that could ship',
      fn: async () => {
        /**
         Original linking one destination.
         */
        const sourceText = '她的头像由[画师](https://twitter.com/cat)绘制。';
        /**
         Page that rewrote the destination.
         */
        const pageText = 'Her avatar was drawn by [the artist](https://x.com/cat).';
        expect(validateTranslatedSlice({
          sourceText,
          pageText,
          candidateText: 'Her avatar was drawn by [the artist](https://x.com/cat).',
        },).kind,).toBe('valid',);
        expect(validateTranslatedSlice({
          sourceText,
          pageText,
          candidateText: 'Her avatar was drawn by [the artist](https://twitter.com/cat).',
        },).kind,).toBe('valid',);
        /**
         Verdict over a candidate carrying no link at all.
         */
        const neither = validateTranslatedSlice({
          sourceText,
          pageText,
          candidateText: 'Her avatar was drawn by the artist.',
        },);
        expect(neither.kind,).toBe('invalid',);
        expect(
          (neither.kind === 'invalid') ? neither.findings.join('\n',) : '',
        ).toContain('must carry exactly 1 of these, taken from either side; it carries 0',);
        /**
         Verdict over a candidate carrying both renderings.
         */
        const both = validateTranslatedSlice({
          sourceText,
          pageText,
          candidateText: 'Her avatar was drawn by [the artist](https://x.com/cat) ([Twitter](https://twitter.com/cat)).',
        },);
        expect(both.kind,).toBe('invalid',);
        expect(
          (both.kind === 'invalid') ? both.findings.join('\n',) : '',
        ).toContain('it carries 2',);
      },
    },),

    it({
      name: 'answers UNKNOWN rather than invalid when the ORIGINAL cannot be '
        + 'parsed, since that says nothing about the candidate and charging it '
        + 'to the model would send a good translation back for a fault in the '
        + 'passage it was given',
      fn: async () => {
        /**
         Verdict where the original itself refuses the strict grammar.
         */
        const validation = validateTranslatedSlice({
          sourceText: '猫猫 <未闭合 的标签 在这里。',
          candidateText: 'The cat dozes on the windowsill.',
        },);
        expect(validation,).toEqual({
          kind: 'unknown',
          detail: 'original could not be read: MdxParseError: MDX body refused to parse at 1:16 '
            + '(micromark-extension-mdx-jsx/unexpected-character); corpus documents compile as MDX upstream, so '
            + 'failure signals corruption or an unsupported construct.',
        },);
      },
    },),

    it({
      name: 'STILL RUNS THE FLOORS THAT NEED NO GRAMMAR where the ORIGINAL cannot be parsed: a candidate left '
        + 'untranslated or keeping the neutral pronoun is refused, since neither needs a block to compare '
        + '(ledger F-5)',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '猫猫 <未闭合 的标签 在这里。',
          candidateText: '猫猫 <未闭合 的标签 在这里。',
        },),).toEqual({
          kind: 'invalid',
          findings: [UNTRANSLATED_FINDING,],
        },);
        expect(validateTranslatedSlice({
          sourceText: 'TA 在睡觉 <未闭合 的标签 在这里。',
          candidateText: 'TA is sleeping here.',
        },),).toEqual({
          kind: 'invalid',
          findings: [
            'Your translation carries the pronoun untranslated as "TA" (1 time): the ORIGINAL writes its neutral '
              + 'pronoun as TA, Ta or ta, and English renders it as singular they (they, them, their), with TA 们 as '
              + 'plural they; a Ta left standing in the English is an untranslated word, not a preserved choice.',
          ],
        },);
      },
    },),

    it({
      name: 'COMPARES an original that carries an HTML comment rather than answering unknown, whether '
        + 'the translation carries the comment rendered or leaves it out, since the document reader '
        + 'masks comments before its strict parse and the slice reader must read the same grammar '
        + '(one entry, 2026-09-06: every commented slice was inadmissible on both gates)',
      fn: async () => {
        /**
         Original with a translator note between its blocks.
         */
        const sourceText = '## 猫猫\n\n<!-- （这一段用猫的口吻来写） -->\n\n猫猫在窗台上打盹。';

        expect(
          validateTranslatedSlice({
            sourceText,
            candidateText: '## The cat\n\n<!-- (This paragraph is written in the cat\'s voice.) -->\n\nThe cat dozes on the windowsill.',
          },).kind,
        ).toBe('valid',);
        expect(
          validateTranslatedSlice({
            sourceText,
            candidateText: '## The cat\n\nThe cat dozes on the windowsill.',
          },).kind,
        ).toBe('valid',);
        // The comparison still runs: a dropped heading is still reported.
        expect(
          validateTranslatedSlice({
            sourceText,
            candidateText: '<!-- (This paragraph is written in the cat\'s voice.) -->\n\nThe cat dozes on the windowsill.',
          },).kind,
        ).toBe('invalid',);
      },
    },),

    it({
      name: 'REFUSES contributor respelling even when ORIGINAL grammar is unknown',
      fn: async () => {
        const validation = validateTranslatedSlice({
          sourceText: '猫猫 <未闭合 的标签 在这里。',
          pageText: 'Contributors for this entry: [Snow](https://example.test/snow)',
          candidateText: 'Contributors for this entry: Snowflake',
        },);
        expect(validation.kind,).toBe('invalid',);
      },
    },),

    it({
      name: 'REPORTS a candidate the parser refuses, and hands the parser\'s '
        + 'own account back, so the model is told what to fix rather than that '
        + 'something was wrong',
      fn: async () => {
        /**
         Verdict where the candidate refuses the strict grammar.
         */
        const validation = validateTranslatedSlice({
          sourceText: '猫猫在窗台上打盹。',
          candidateText: 'The cat dozes {unclosed on the windowsill.',
        },);
        expect(validation.kind,).toBe('invalid',);
        expect(
          (validation.kind === 'invalid') ? validation.findings.join('\n',) : '',
        ).toContain('could not be parsed',);
      },
    },),

    it({
      name: 'ACCEPTS a rendering shaped like the PAGE where the page merges '
        + 'what the original splits, which is the case that made the source '
        + 'anchor send six good renderings back to their authors',
      fn: async () => {
        expect(
          validateTranslatedSlice({
            sourceText: '猫猫在窗台上打盹。\n\n（邻居留）',
            pageText: '> The cat dozes on the windowsill.\n\n—left by a neighbour',
            candidateText: '> The cat naps on the windowsill.\n\n—left by a neighbour',
          },).kind,
        ).toBe('valid',);
      },
    },),

    it({
      name: 'REFUSES a rendering shaped like the ORIGINAL when the page it replaces is shaped '
        + 'otherwise in KIND, since the block quote is what says somebody left this passage rather '
        + 'than wrote it, and a kind the original lacks is not a split',
      fn: async () => {
        /**
         Verdict where the candidate keeps the original's shape and loses the
         page's quote.
         */
        const validation = validateTranslatedSlice({
          sourceText: '猫猫在窗台上打盹。\n\n（邻居留）',
          pageText: '> The cat dozes on the windowsill.\n\n—left by a neighbour',
          candidateText: 'The cat naps on the windowsill.\n\nLeft by a neighbour.',
        },);
        expect(validation.kind,).toBe('invalid',);
        expect(
          (validation.kind === 'invalid') ? validation.findings.join('\n',) : '',
        ).toContain('PAGE AS IT STANDS',);
      },
    },),

    it({
      name: 'ACCEPTS a rendering shaped like the ORIGINAL where the page only splits the original\'s '
        + 'blocks into more of the same kind, the owner\'s either-rendering decision of 2026-09-07',
      fn: async () => {
        expect(
          validateTranslatedSlice({
            sourceText: '猫猫在窗台上打盹。它每天都来。',
            pageText: 'The cat dozes on the windowsill.\n\nIt comes every day.',
            candidateText: 'The cat naps on the windowsill; it comes every day.',
          },).kind,
        ).toBe('valid',);
      },
    },),

    it({
      name: 'REFUSES a rendering shaped like NEITHER reference, which is what a dropped passage '
        + 'looks like and what the sixth consolidation bed shipped',
      fn: async () => {
        /**
         Verdict where the candidate carries one paragraph against two on
         each side.
         */
        const validation = validateTranslatedSlice({
          sourceText: '猫猫在窗台上打盹。\n\n（邻居留）',
          pageText: '> The cat dozes on the windowsill.\n\n—left by a neighbour',
          candidateText: 'The cat naps on the windowsill, left by a neighbour.',
        },);
        expect(validation.kind,).toBe('invalid',);
        expect(
          (validation.kind === 'invalid') ? validation.findings.join('\n',) : '',
        ).toContain('PAGE AS IT STANDS',);
      },
    },),

    it({
      name: 'ACCEPTS the original\'s two hard-broken stanzas where the page splits the poem into '
        + 'five paragraphs, and still REFUSES one stanza (one entry, 2026-09-07)',
      fn: async () => {
        /**
         Poem as the source writes it, lines ending in `<br/>`.
         */
        const source = '春风又来，<br/>\n却唤不回你。\n\n此生太短，<br/>\n此别太长。';

        /**
         Poem as the archive writes it, one couplet per paragraph and more.
         */
        const page = 'The spring wind returns,\n\nbut cannot call you back.\n\nThis life was too short,'
          + '\n\nthis parting too long.\n\nWritten in tears.';

        expect(validateTranslatedSlice({
          sourceText: source,
          pageText: page,
          candidateText: 'The spring wind returns,<br/>\nbut cannot call you back.\n\n'
            + 'This life was too short,<br/>\nthis parting too long.',
        },).kind,).toBe('valid',);
        expect(validateTranslatedSlice({
          sourceText: source,
          pageText: page,
          candidateText: 'The spring wind returns but cannot call you back; this life was too short, '
            + 'this parting too long.',
        },).kind,).toBe('invalid',);
      },
    },),

    it({
      name: 'ACCEPTS a rendering that RESTORES a block the page left out, '
        + 'which the page-as-ceiling rule deleted again on the next round',
      fn: async () => {
        expect(
          validateTranslatedSlice({
            sourceText: '猫猫在窗台上打盹。\n\n> 邻居说它每天都来。',
            pageText: 'The cat dozes on the windowsill.',
            candidateText: 'The cat dozes on the windowsill.\n\n> The neighbour '
              + 'says it comes every day.',
          },).kind,
        ).toBe('valid',);
      },
    },),

    it({
      name: 'ACCEPTS the page\'s own paragraph followed by the original\'s block quote where the page '
        + 'stands a paragraph of another kind in the quote\'s place (class thirty-two, 2026-09-16)',
      fn: async () => {
        expect(
          validateTranslatedSlice({
            sourceText: '> 猫猫走了。\n> ——邻居留',
            pageText: 'Rest well, little cat.',
            candidateText: 'Rest well, little cat.\n\n> The cat has gone.\n> — left by a neighbour',
          },).kind,
        ).toBe('valid',);
      },
    },),

    it({
      name: 'STILL REFUSES the original\'s block quote alone there, since the paragraph is the '
        + 'archive\'s and dropping it deletes it, and SAYS the quote may follow the page\'s blocks',
      fn: async () => {
        /**
         Verdict where the candidate keeps the original's quote and loses the
         page's farewell.
         */
        const validation = validateTranslatedSlice({
          sourceText: '> 猫猫走了。\n> ——邻居留',
          pageText: 'Rest well, little cat.',
          candidateText: '> The cat has gone.\n> — left by a neighbour',
        },);
        expect(validation.kind,).toBe('invalid',);
        expect(
          (validation.kind === 'invalid') ? validation.findings.join('\n',) : '',
        ).toContain('no block of its kind on the PAGE AS IT STANDS',);
      },
    },),

    it({
      name: 'REFUSES a block neither the page nor the original carries, so '
        + 'restoring stays restoring rather than becoming licence to invent',
      fn: async () => {
        /**
         Verdict where the candidate adds a block from nowhere.
         */
        const validation = validateTranslatedSlice({
          sourceText: '猫猫在窗台上打盹。',
          pageText: 'The cat dozes on the windowsill.',
          candidateText: 'The cat dozes on the windowsill.\n\nIt is a tabby.',
        },);
        expect(validation.kind,).toBe('invalid',);
        expect(
          (validation.kind === 'invalid') ? validation.findings.join('\n',) : '',
        ).toContain('Add a block only to carry',);
      },
    },),

    it({
      name: 'KEEPS a footnote the PAGE carries and the original never had, '
        + 'because accurate detail the archive added is not a candidate\'s to '
        + 'drop',
      fn: async () => {
        /**
         Verdict where the candidate carries the page's own footnote.
         */
        const validation = validateTranslatedSlice({
          sourceText: '猫猫在窗台上打盹。',
          pageText: 'The cat dozes on the windowsill[^1].\n\n[^1]: The sill '
            + 'faces east.',
          candidateText: 'The cat naps on the windowsill[^1].\n\n[^1]: The sill '
            + 'faces east.',
        },);
        expect(validation.kind,).toBe('valid',);
      },
    },),

    it({
      name: 'REPORTS a footnote the page carries and the candidate dropped, '
        + 'naming both references so the model knows which text asked for it',
      fn: async () => {
        /**
         Verdict where the candidate drops the page's footnote.
         */
        const validation = validateTranslatedSlice({
          sourceText: '猫猫在窗台上打盹。',
          pageText: 'The cat dozes on the windowsill[^1].\n\n[^1]: The sill '
            + 'faces east.',
          candidateText: 'The cat naps on the windowsill.',
        },);
        expect(validation.kind,).toBe('invalid',);
        expect(
          (validation.kind === 'invalid') ? validation.findings.join('\n',) : '',
        ).toContain('ORIGINAL or the PAGE AS IT STANDS',);
      },
    },),

    it({
      name: 'NAMES WHICH GRAMMAR READ THE PAGE on all three readings, since a '
        + 'pass carries no findings and the reader of a valid verdict has '
        + 'nothing else to tell an empty page from one that was downgraded. '
        + 'The case beside this one pins only the downgraded reading being '
        + 'accepted, not that a reader can see it happened',
      fn: async () => {
        /**
         Pass at a slice the archive never translated, where there is no page
         to read at all.
         */
        const absent = validateTranslatedSlice({
          sourceText: '猫猫在窗台上打盹。',
          pageText: '',
          candidateText: 'The cat dozes on the windowsill.',
        },);

        /**
         Pass over a page this grammar reads as written.
         */
        const strict = validateTranslatedSlice({
          sourceText: '猫猫在窗台上打盹。',
          pageText: 'The cat dozes on the windowsill.',
          candidateText: 'The cat naps on the windowsill.',
        },);

        /**
         Pass over a page only the relaxed reading gets through.
         */
        const relaxed = validateTranslatedSlice({
          sourceText: '猫猫在窗台上打盹。',
          pageText: 'The cat dozes {unclosed on the windowsill.',
          candidateText: 'The cat naps on the windowsill.',
        },);

        expect([
          absent.kind === 'valid' ? absent.pageGrammar : 'not-valid',
          strict.kind === 'valid' ? strict.pageGrammar : 'not-valid',
          relaxed.kind === 'valid' ? relaxed.pageGrammar : 'not-valid',
        ],).toEqual([
          'absent',
          'strict',
          'relaxed',
        ],);
      },
    },),
    it({
      name: 'REFUSES CANDIDATE that respells target-authoritative contributor identity',
      fn: async () => {
        const validation = validateTranslatedSlice({
          sourceText: '本条目贡献者：雪猫',
          pageText: 'Contributors for this entry: [Snow](https://example.test/snow)',
          candidateText: 'Contributors for this entry: Snowflake',
        },);
        expect(validation.kind,).toBe('invalid',);
        const findings = (validation.kind === 'invalid')
          ? validation.findings.join('\n',)
          : '';
        expect(findings,).toContain('Target-authoritative contributor identity',);
        expect(findings,).not.toContain('Snow',);
      },
    },),

    it({
      name: 'ACCEPTS TARGET CONTRIBUTOR after harmless attribution reflow',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '本条目贡献者：雪猫',
          pageText: 'Contributors for this entry: [Snow](https://example.test/snow)',
          candidateText: 'Contributors for this entry:\n[Snow](https://example.test/snow)',
        },).kind,).toBe('valid',);
      },
    },),

    it({
      name: 'READS THE PAGE UNDER PLAIN MARKDOWN when the strict grammar refuses it, since an archive written '
        + 'before this grammar existed is not the candidate\'s fault, and passes a rendering carrying its blocks',
      fn: async () => {
        expect(
          validateTranslatedSlice({
            sourceText: '猫猫在窗台上打盹。',
            pageText: 'The cat dozes {unclosed on the windowsill.',
            candidateText: 'The cat naps on the windowsill.',
          },),
        ).toEqual({
          kind: 'valid',
          pageGrammar: 'relaxed',
        },);
      },
    },),

    it({
      name: 'LEAVES A CANDIDATE UNVALIDATED where neither grammar reads the page, as where none reads the original, '
        + 'rather than passing it against the original alone, and still runs the floors that read text (ledger '
        + 'T8, sixth batch)',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '猫猫在窗台上打盹。',
          pageText: UNREADABLE_PAGE,
          candidateText: 'The cat naps on the windowsill.',
        },),).toEqual({
          kind: 'unknown',
          detail: 'page could not be read: plain markdown also refused: MarkdownParseError: Plain markdown body '
            + 'refused to parse because it is nested too deeply to read: its container markers pass the bound of '
            + '256 at line 1, column 257.',
        },);
        expect(validateTranslatedSlice({
          sourceText: '猫猫在窗台上打盹。',
          pageText: UNREADABLE_PAGE,
          candidateText: '猫猫在窗台上打盹。',
        },),).toEqual({
          kind: 'invalid',
          findings: [UNTRANSLATED_FINDING,],
        },);
      },
    },),
    it({
      name: 'REFUSES a translation that drops the container closer its original owns, and ACCEPTS one '
        + 'that carries it (one entry, 2026-09-07)',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '它醒了。\n\n</details>',
          candidateText: 'It wakes.',
        },).kind,).toBe('invalid',);
        expect(validateTranslatedSlice({
          sourceText: '它醒了。\n\n</details>',
          candidateText: 'It wakes.\n\n</details>',
        },).kind,).toBe('valid',);
      },
    },),
    it({
      name: 'REFUSES a translation that keeps every top-level block but drops a quote nested inside a container '
        + 'tag, which the top-level block comparison cannot see, and ACCEPTS one that keeps the nested quote '
        + '(ledger B110)',
      fn: async () => {
        /**
         Verdict where the candidate keeps the container tag the page and
         the original both carry, same kind and same position, and turns
         the quote nested inside it into prose. `compareBlocks` sees one
         `mdxJsxFlowElement` on every side and returns no finding for it.
         */
        const dropped = validateTranslatedSlice({
          sourceText: '<details>\n\n> 中午喂我。\n\n</details>',
          pageText: '<details>\n\n> Feed me at noon.\n\n</details>',
          candidateText: '<details>\n\nShe asked to be fed at noon.\n\n</details>',
        },);
        expect(dropped.kind,).toBe('invalid',);
        expect(
          (dropped.kind === 'invalid') ? dropped.findings.join('\n',) : '',
        ).toContain('quoted passage',);

        // Control: the same container tag, the nested quote kept rather than
        // dropped, which must still pass.
        expect(validateTranslatedSlice({
          sourceText: '<details>\n\n> 中午喂我。\n\n</details>',
          pageText: '<details>\n\n> Feed me at noon.\n\n</details>',
          candidateText: '<details>\n\n> She asked to be fed at noon.\n\n</details>',
        },).kind,).toBe('valid',);
      },
    },),
  ],
},);
