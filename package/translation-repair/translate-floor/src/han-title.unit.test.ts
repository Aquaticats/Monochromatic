/**
 Guards class ninety-eight (XingZ629, 2026-09-23): a candidate that keeps a
 work's title the original brackets in 《》 in Han, where the title carries
 no Latin letter and the page the candidate would replace never wrote it,
 is refused before any judge reads it, since a work the original names is
 called by its English title on the page. A title translated, a title kept
 beside its English in parentheses, a title the page itself keeps, and a
 title standing inside an HTML comment are all left to the judges. A title
 that already carries Latin letters is not this floor's: the Han residue
 floor reads its Han (ledger F-3), refusing it bare and passing it glossed.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  bracketedTitles,
  titleText,
} from '@monochromatic-dev/translation-repair-markup';
import {
  type SliceValidation,
  validateTranslatedSlice,
} from '../dist/final/node/index.mjs';
import {
  hanResidueFinding,
  hanTitleFinding,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/han-findings.test-fixture.ts';

/**
 Original naming the cat's favourite song.
 */
const NAMED = '她最爱的歌是《猫猫摇篮曲》。';

/**
 Rendering that left the title in Han.
 */
const LEFT = 'Her favourite song was 《猫猫摇篮曲》.';

/**
 Rendering that translated the title.
 */
const TRANSLATED = 'Her favourite song was “Kitten Lullaby”.';

/**
 Rendering that translated the title and kept the Han beside it.
 */
const GLOSSED = 'Her favourite song was “Kitten Lullaby” (猫猫摇篮曲).';

/**
 Original naming an album whose title carries Latin letters.
 */
const LATIN_NAMED = '她最爱的专辑是《Nyan物语》。';

/**
 Rendering that kept the Latin-bearing title as written.
 */
const LATIN_KEPT = 'Her favourite album was 《Nyan物语》.';

/**
 Rendering that kept the Latin-bearing title with its English beside it.
 */
const LATIN_GLOSSED = 'Her favourite album was 《Nyan物语》 (Nyan Story).';

/**
 Original naming the song through a link inside the brackets.
 */
const LINKED = '她唱了《[猫猫摇篮曲](https://example.test/song)》。';

/**
 Rendering that kept the link text in Han.
 */
const LINK_LEFT = 'She sang 《[猫猫摇篮曲](https://example.test/song)》.';

/**
 Rendering that translated the link text.
 */
const LINK_TRANSLATED = 'She sang *[Kitten Lullaby](https://example.test/song)*.';

/**
 Verdict on a candidate no floor refuses, read against a page.
 */
const VALID_ON_PAGE: SliceValidation = {
  kind: 'valid',
  pageGrammar: 'strict',
};

/**
 Verdict on a candidate no floor refuses, with no page behind it.
 */
const VALID_WITHOUT_PAGE: SliceValidation = {
  kind: 'valid',
  pageGrammar: 'absent',
};

/**
 Verdict refusing the song's title left in Han.
 */
const LULLABY_REFUSED: SliceValidation = {
  kind: 'invalid',
  findings: [hanTitleFinding({ title: '猫猫摇篮曲', },),],
};

await describe({
  name: 'a work title the original brackets in 《》 (class ninety-eight)',
  children: [
    it({
      name: 'REFUSES a candidate that leaves a linked title\'s text in Han and ACCEPTS it translated',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: LINKED,
          candidateText: LINK_LEFT,
        },),).toEqual(LULLABY_REFUSED,);
        expect(validateTranslatedSlice({
          sourceText: LINKED,
          candidateText: LINK_TRANSLATED,
        },),).toEqual(VALID_WITHOUT_PAGE,);
      },
    },),
    it({
      name: 'REFUSES a candidate that leaves the title in Han where the page never wrote it',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: NAMED,
          candidateText: LEFT,
        },),).toEqual(LULLABY_REFUSED,);
      },
    },),
    it({
      name: 'REFUSES A TITLE WHOSE PARENTHESIS IS NO ENGLISH GLOSS: one broken by a newline, one holding another '
        + 'parenthesis, and one never closed',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: NAMED,
          candidateText: 'Her favourite song was 《猫猫摇篮曲》 (Kitten\nLullaby).',
        },),).toEqual(LULLABY_REFUSED,);
        expect(validateTranslatedSlice({
          sourceText: NAMED,
          candidateText: 'Her favourite song was 《猫猫摇篮曲》 (see (Kitten Lullaby)).',
        },),).toEqual(LULLABY_REFUSED,);
        expect(validateTranslatedSlice({
          sourceText: NAMED,
          candidateText: 'Her favourite song was 《猫猫摇篮曲》 (Kitten Lullaby',
        },),).toEqual(LULLABY_REFUSED,);
      },
    },),
    it({
      name: 'REFUSES A TITLE LEFT BARE at the end of the text, on a later line, after a closed parenthesis, and '
        + 'inside a parenthesis that never closes',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '她最爱的歌是《猫猫摇篮曲》',
          candidateText: 'Her favourite song: 《猫猫摇篮曲》',
        },),).toEqual(LULLABY_REFUSED,);
        expect(validateTranslatedSlice({
          sourceText: '她最爱的歌是\n《猫猫摇篮曲》。',
          candidateText: 'Her favourite song was\n《猫猫摇篮曲》.',
        },),).toEqual(LULLABY_REFUSED,);
        expect(validateTranslatedSlice({
          sourceText: '她（猫）最爱的歌是《猫猫摇篮曲》。',
          candidateText: 'She (the cat) loved 《猫猫摇篮曲》.',
        },),).toEqual(LULLABY_REFUSED,);
        expect(validateTranslatedSlice({
          sourceText: NAMED,
          candidateText: 'Her favourite song was Kitten Lullaby (《猫猫摇篮曲》',
        },),).toEqual(LULLABY_REFUSED,);
      },
    },),
    it({
      name: 'REFUSES A LATIN-BEARING TITLE LEFT BARE through the Han residue floor, not this one, where it once '
        + 'fell between this floor and the Latin title floor (ledger F-3)',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: LATIN_NAMED,
          candidateText: LATIN_KEPT,
        },),).toEqual({
          kind: 'invalid',
          findings: [hanResidueFinding({ runs: ['物语',], },),],
        },);
      },
    },),
    it({
      name: 'READS AN OPENING MARK THAT NEVER CLOSED AS NO TITLE (ledger B38), so the title after a stray 《 is the '
        + 'only one named',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '她的歌：《喵，还有《猫猫摇篮曲》。',
          candidateText: 'Her songs: 《喵，还有《猫猫摇篮曲》.',
        },),).toEqual(LULLABY_REFUSED,);
      },
    },),
    it({
      name: 'ACCEPTS the title translated, glossed, Latin-bearing and glossed, kept by the page, or standing in a comment',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: NAMED,
          candidateText: TRANSLATED,
        },),).toEqual(VALID_WITHOUT_PAGE,);
        expect(validateTranslatedSlice({
          sourceText: NAMED,
          candidateText: GLOSSED,
        },),).toEqual(VALID_WITHOUT_PAGE,);
        expect(validateTranslatedSlice({
          sourceText: LATIN_NAMED,
          candidateText: LATIN_GLOSSED,
        },),).toEqual(VALID_WITHOUT_PAGE,);
        expect(validateTranslatedSlice({
          sourceText: NAMED,
          candidateText: LEFT,
          pageText: LEFT,
        },),).toEqual(VALID_ON_PAGE,);
        expect(validateTranslatedSlice({
          sourceText: '<!-- 《猫猫摇篮曲》 -->她睡了。',
          candidateText: 'She slept.',
        },),).toEqual(VALID_WITHOUT_PAGE,);
      },
    },),

    it({
      name: 'READS A LINKED TITLE down to its text and READS A LINKLESS BRACKET as its whole self, and '
        + 'SKIPS a title with a line end or a repeat',
      fn: async () => {
        expect(titleText({ bracketed: '[猫猫摇篮曲](cat-song)', },),).toBe('猫猫摇篮曲',);
        expect(titleText({ bracketed: '[猫猫摇篮曲)', },),).toBe('[猫猫摇篮曲)',);
        expect(bracketedTitles({
          text: '《猫\n猫》,《猫猫》,《猫猫》',
          form: 'carrying-han',
        },),).toEqual(['猫猫',],);
      },
    },),
  ],
},);
