/**
 Tests for the one reading of whether the deterministic source floor can
 compare anything on a slice, which the floor reads its `unknown` verdict
 from and the translate stage asks before it buys a round (ledger B43).

 The cases pin each way a side goes unread, which side it is, that the
 detail is the one the floor's `unknown` verdict carries for the same slice,
 since two readers that disagreed there would let a slice be bought and
 shipped unchecked, and the narrowing a caller that asked first reads the
 floor's verdict through.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readSliceSkeleton, } from '@monochromatic-dev/translation-repair-glossary';
import {
  FrontMatterParseError,
  splitFrontMatter,
} from '@monochromatic-dev/translation-repair-document-structure';
import {
  FloorGroundDisagreementError,
  floorReach,
  readFrontMatterGround,
  readMarkdownGround,
  requireComparedVerdict,
  validateTranslatedSlice,
} from '../dist/final/node/index.mjs';

/**
 Original the strict grammar reads.
 */
const SOURCE_TEXT = '猫猫在窗台上打盹。';

/**
 Original carrying an expression the strict grammar never closes.
 */
const UNCLOSED_SOURCE = '猫猫在{窗台上打盹。';

/**
 A page neither grammar reads: plain markdown refuses only when reading
 exhausts the parser, and 16,000 nested quotation markers overflow its stack.
 */
const UNREADABLE_PAGE = `${'>'.repeat(16_000,)} cat`;

/**
 What the floor says of that page, as its own case pins it
 (`translate-validate.unit.test.ts`).
 */
const UNREADABLE_PAGE_DETAIL = 'page could not be read: plain markdown also refused: MarkdownParseError: Plain '
  + 'markdown body refused to parse because it is nested too deeply to read: its container markers pass the '
  + 'bound of 256 at line 1, column 257.';

/**
 A rendering the floors that read text leave alone.
 */
const CANDIDATE_TEXT = 'The cat naps on the windowsill.';

/**
 Fenced metadata the reader reads.
 */
const SOURCE_METADATA = '---\nname: 猫猫\n---\n';

/**
 Metadata as the page carries it.
 */
const PAGE_METADATA = '---\nname: Maomao\n---\n';

/**
 A rendering of that metadata.
 */
const TRANSLATED_METADATA = '---\nname: Mao\n---\n';

/**
 Metadata written without its fences.
 */
const UNFENCED_METADATA = 'name: 猫猫\n';

/**
 Fenced metadata whose YAML the parser refuses.
 */
const BROKEN_METADATA = '---\nname: [broken\n---\n';

/**
 What the strict grammar says of the unclosed original, read rather than
 written out, since the detail is the grammar's own.

 @returns Grammar's refusal detail

 @throws {@link Error} where the fixture reads, which would mean it no longer
 exercises an unread original

 @example
 ```ts
 const detail = unclosedSourceDetail();
 ```
 */
function unclosedSourceDetail(): string {
  /**
   Strict reading of the fixture.
   */
  const read = readSliceSkeleton({ text: UNCLOSED_SOURCE, },);
  if (read.kind !== 'unparseable')
    throw new Error('the unclosed fixture reads, so it no longer exercises an unread original',);
  return read.detail;
}

/**
 What the splitter says of the broken metadata, which a detail naming it
 carries.

 @returns Refusal's message

 @throws {@link Error} where the fixture parses, which would mean it no
 longer exercises a refusal

 @example
 ```ts
 const message = brokenMetadataMessage();
 ```
 */
function brokenMetadataMessage(): string {
  /**
   What splitting the fixture raised.
   */
  const refusal = caught(function splitBroken(): void {
    splitFrontMatter({ text: BROKEN_METADATA, },);
  },);
  if (!(refusal instanceof FrontMatterParseError))
    throw new Error('the broken fixture raised something other than a YAML refusal',);
  return refusal.message;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readMarkdownGround.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS BOTH SIDES, the page under plain markdown where the strict grammar refuses it and as absent '
            + 'where there is none',
          fn: async () => {
            /**
             Grammar each page was read by, or the ground's kind where none was.
             */
            const grammars = [
              'The cat naps.',
              'The cat dozes {unclosed on the windowsill.',
              '',
            ].map(function grammarOf(pageText,): string {
              /**
               Both sides of one slice.
               */
              const ground = readMarkdownGround({
                sourceText: SOURCE_TEXT,
                pageText,
              },);
              return (ground.kind === 'read') ? ground.pageGrammar : ground.kind;
            },);

            expect(grammars,).toEqual(['strict', 'relaxed', 'absent',],);
          },
        },),
        it({
          name: 'NAMES THE ORIGINAL where the strict grammar cannot read it, whatever the page is',
          fn: async () => {
            expect(readMarkdownGround({
              sourceText: UNCLOSED_SOURCE,
              pageText: UNREADABLE_PAGE,
            },),).toEqual({
              kind: 'blind',
              side: 'original',
              detail: `original could not be read: ${unclosedSourceDetail()}`,
            },);
          },
        },),
        it({
          name: 'NAMES THE PAGE where neither grammar reads it',
          fn: async () => {
            expect(readMarkdownGround({
              sourceText: SOURCE_TEXT,
              pageText: UNREADABLE_PAGE,
            },),).toEqual({
              kind: 'blind',
              side: 'page',
              detail: UNREADABLE_PAGE_DETAIL,
            },);
          },
        },),
      ],
    },),

    describe({
      name: readFrontMatterGround.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS BOTH SIDES, and reads no page as none rather than as unread',
          fn: async () => {
            expect(readFrontMatterGround({
              sourceText: SOURCE_METADATA,
              pageText: PAGE_METADATA,
            },),).toEqual({
              kind: 'read',
              source: splitFrontMatter({ text: SOURCE_METADATA, },).frontMatter,
              page: splitFrontMatter({ text: PAGE_METADATA, },).frontMatter,
            },);
            expect(readFrontMatterGround({
              sourceText: SOURCE_METADATA,
              pageText: '',
            },),).toEqual({
              kind: 'read',
              source: splitFrontMatter({ text: SOURCE_METADATA, },).frontMatter,
            },);
          },
        },),
        it({
          name: 'NAMES THE SIDE carrying no fenced block, the original before the page',
          fn: async () => {
            expect(readFrontMatterGround({
              sourceText: UNFENCED_METADATA,
              pageText: UNFENCED_METADATA,
            },),).toEqual({
              kind: 'blind',
              side: 'original',
              detail: 'source front matter could not be read',
            },);
            expect(readFrontMatterGround({
              sourceText: SOURCE_METADATA,
              pageText: UNFENCED_METADATA,
            },),).toEqual({
              kind: 'blind',
              side: 'page',
              detail: 'page front matter could not be read',
            },);
          },
        },),
        it({
          name: 'NAMES THE SIDE whose YAML is refused, with the refusal, rather than raising it (ledger B44)',
          fn: async () => {
            expect(readFrontMatterGround({
              sourceText: BROKEN_METADATA,
              pageText: PAGE_METADATA,
            },),).toEqual({
              kind: 'blind',
              side: 'original',
              detail: `source front matter could not be read: ${brokenMetadataMessage()}`,
            },);
            expect(readFrontMatterGround({
              sourceText: SOURCE_METADATA,
              pageText: BROKEN_METADATA,
            },),).toEqual({
              kind: 'blind',
              side: 'page',
              detail: `page front matter could not be read: ${brokenMetadataMessage()}`,
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'the floor\'s unknown verdict',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES THE GROUND\'S OWN DETAIL for each way a side goes unread, the one definition the floor reads',
          fn: async () => {
            /**
             What each blind ground said beside the floor's verdict on the same
             slice, with a candidate no text floor refuses.
             */
            const answers = [
              {
                ground: readMarkdownGround({
                  sourceText: UNCLOSED_SOURCE,
                  pageText: '',
                },),
                floor: validateTranslatedSlice({
                  sourceText: UNCLOSED_SOURCE,
                  pageText: '',
                  candidateText: CANDIDATE_TEXT,
                },),
              },
              {
                ground: readMarkdownGround({
                  sourceText: SOURCE_TEXT,
                  pageText: UNREADABLE_PAGE,
                },),
                floor: validateTranslatedSlice({
                  sourceText: SOURCE_TEXT,
                  pageText: UNREADABLE_PAGE,
                  candidateText: CANDIDATE_TEXT,
                },),
              },
              {
                ground: readFrontMatterGround({
                  sourceText: BROKEN_METADATA,
                  pageText: PAGE_METADATA,
                },),
                floor: validateTranslatedSlice({
                  sourceText: BROKEN_METADATA,
                  pageText: PAGE_METADATA,
                  candidateText: TRANSLATED_METADATA,
                  syntax: 'front-matter',
                },),
              },
              {
                ground: readFrontMatterGround({
                  sourceText: SOURCE_METADATA,
                  pageText: UNFENCED_METADATA,
                },),
                floor: validateTranslatedSlice({
                  sourceText: SOURCE_METADATA,
                  pageText: UNFENCED_METADATA,
                  candidateText: TRANSLATED_METADATA,
                  syntax: 'front-matter',
                },),
              },
            ].map(function detailsOf({
              ground,
              floor,
            },) {
              return {
                ground: (ground.kind === 'blind') ? ground.detail : ground.kind,
                floor: (floor.kind === 'unknown') ? floor.detail : floor.kind,
              };
            },);

            expect(answers,).toEqual([
              `original could not be read: ${unclosedSourceDetail()}`,
              UNREADABLE_PAGE_DETAIL,
              `source front matter could not be read: ${brokenMetadataMessage()}`,
              'page front matter could not be read',
            ].map(function bothSay(detail,) {
              return {
                ground: detail,
                floor: detail,
              };
            },),);
          },
        },),
      ],
    },),

    describe({
      name: floorReach.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS BY THE SLICE\'S SYNTAX: metadata without fences is blind as front matter and comparable as '
            + 'prose',
          fn: async () => {
            expect(floorReach({
              sourceText: UNFENCED_METADATA,
              pageText: '',
              syntax: 'front-matter',
            },),).toEqual({
              kind: 'blind',
              side: 'original',
              detail: 'source front matter could not be read',
            },);
            expect(floorReach({
              sourceText: UNFENCED_METADATA,
              pageText: '',
            },),).toEqual({ kind: 'comparable', },);
          },
        },),
        it({
          name: 'ANSWERS WITH THE GROUND ITSELF where a side goes unread, the side and the floor\'s own detail',
          fn: async () => {
            expect(floorReach({
              sourceText: SOURCE_TEXT,
              pageText: UNREADABLE_PAGE,
            },),).toEqual(readMarkdownGround({
              sourceText: SOURCE_TEXT,
              pageText: UNREADABLE_PAGE,
            },),);
            expect(floorReach({
              sourceText: SOURCE_METADATA,
              pageText: BROKEN_METADATA,
              syntax: 'front-matter',
            },),).toEqual(readFrontMatterGround({
              sourceText: SOURCE_METADATA,
              pageText: BROKEN_METADATA,
            },),);
          },
        },),
      ],
    },),

    describe({
      name: requireComparedVerdict.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RETURNS a pass or a refusal as the floor gave it, the same object',
          fn: async () => {
            /**
             The floor's pass on a readable slice.
             */
            const pass = validateTranslatedSlice({
              sourceText: SOURCE_TEXT,
              candidateText: CANDIDATE_TEXT,
            },);
            /**
             The floor's refusal of a copied original.
             */
            const refusal = validateTranslatedSlice({
              sourceText: SOURCE_TEXT,
              candidateText: SOURCE_TEXT,
            },);

            expect([pass.kind, refusal.kind,],).toEqual(['valid', 'invalid',],);
            expect(requireComparedVerdict({ verdict: pass, },),).toBe(pass,);
            expect(requireComparedVerdict({ verdict: refusal, },),).toBe(refusal,);
          },
        },),
        it({
          name: 'REFUSES a verdict that compared nothing, carrying the floor\'s detail beside a fixed sentence',
          fn: async () => {
            /**
             The floor's verdict on an original the strict grammar cannot read.
             */
            const blind = validateTranslatedSlice({
              sourceText: UNCLOSED_SOURCE,
              candidateText: CANDIDATE_TEXT,
            },);
            /**
             What the narrowing raised over it.
             */
            const raised = caught(function narrowBlind(): void {
              requireComparedVerdict({ verdict: blind, },);
            },);
            if (!(raised instanceof FloorGroundDisagreementError))
              throw new Error('the narrowing raised something other than the disagreement it names',);

            expect(raised.detail,).toBe(`original could not be read: ${unclosedSourceDetail()}`,);
            expect(raised.message.includes(raised.detail,),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
