/**
 Tests explicit front matter slicing and structural translation validation.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { validateFrontMatterTranslation, } from '@monochromatic-dev/translation-repair-translate-floor';
import { frontMatterSlice, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  FrontMatterParseError,
  splitFrontMatter,
} from '@monochromatic-dev/translation-repair-document-structure';
import { archiveDisputesOf, } from '@monochromatic-dev/translation-repair-archive';
import { frontMatterRepairOutcome, } from '../dist/final/node/index.mjs';

/**
 Source metadata fixture.
 */
const SOURCE = {
  raw: '---\nname: 猫猫\ninfo:\n  alias: 猫\n---\n',
  data: {
    name: '猫猫',
    info: { alias: '猫', },
  },
  startOffset: 0,
};

/**
 Target metadata fixture.
 */
const TARGET = {
  raw: '---\nname: Maomao\ninfo:\n  alias: Cat\n---\n',
  data: {
    name: 'Maomao',
    info: { alias: 'Cat', },
  },
  startOffset: 0,
};

/**
 A translation of the source fixture carrying the target's shape.
 */
const TRANSLATED = '---\nname: Mao\ninfo:\n  alias: Kitty\n---\n';

/**
 Fenced metadata whose YAML the parser refuses.
 */
const BROKEN_YAML = '---\nname: [broken\n---\n';

/**
 Source metadata where visible name and alias identify same person form.
 */
const SAME_IDENTITY_SOURCE = '---\nname: 猫猫\ninfo:\n  alias: 猫猫\n---\n';

/**
 Archive metadata whose visible name is entry id rather than declared alias.
 */
const DIRECTORY_ID_TARGET = '---\nname: CatEntry\ninfo:\n  alias: Maomao\n---\n';

/**
 Source metadata carrying contributor attribution in location comment.
 */
const COMMENT_SOURCE = '---\nname: 猫猫\ninfo:\n  alias: 猫猫\n  location: 广东 #清远, by 魔骨\n---\n';

/**
 Archive metadata establishing Latin contributor attribution at same path.
 */
const COMMENT_TARGET = '---\nname: Maomao\ninfo:\n  alias: Maomao\n  location: Guangdong #Qingyuan, by MoguHandle\n---\n';

/**
 Archive establishing one contributor spelling at info.location, paired with
 `COMMENT_AUTHORITY_CANDIDATE`, which establishes a different spelling:
 the two differ so a source that wrongly resolved a comment relation would
 route through to a refusal, not coincidentally stay quiet.
 */
const COMMENT_AUTHORITY_PAGE = '---\nname: 猫猫\ninfo:\n  alias: 猫猫\n  location: Guangdong #Qingyuan, by MoguHandle\n---\n\nBody.\n';

/**
 Candidate establishing its own contributor spelling, deliberately different
 from COMMENT_AUTHORITY_PAGE's.
 */
const COMMENT_AUTHORITY_CANDIDATE = '---\nname: Maomao\ninfo:\n  alias: Maomao\n  location: Guangdong #Qingyuan, by 魔骨\n---\n';

/**
 Source whose front matter declares no info mapping at all.
 */
const INFO_MISSING_SOURCE = '---\nname: 猫猫\n---\n\nBody.\n';

/**
 Source whose info.location is a bare scalar with no inline comment.
 */
const LOCATION_NO_COMMENT_SOURCE = '---\nname: 猫猫\ninfo:\n  alias: 咪咪\n  location: Garden Shed\n---\n\nBody.\n';

/**
 Source whose info.location comment names a place but no contributor.
 */
const PLACE_ONLY_COMMENT_SOURCE = '---\nname: 猫猫\ninfo:\n  alias: 咪咪\n  location: Garden Shed #a sunny corner\n---\n\nBody.\n';

/**
 Source whose info.location comment ends at the contributor marker with
 nothing, not even whitespace, trimmed out of it.
 */
const DANGLING_MARKER_SOURCE = '---\nname: 猫猫\ninfo:\n  alias: 咪咪\n  location: Garden Shed #Garden Shed, by \n---\n\nBody.\n';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: frontMatterSlice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CREATES EXPLICIT SLICE ZERO over exact source and target metadata bytes',
          fn: async () => {
            const result = frontMatterSlice({ source: SOURCE, target: TARGET, });
            expect(result.kind,).toBe('paired',);
            if (result.kind !== 'paired')
              throw new Error('expected paired front matter fixture',);
            expect(result.slice,).toEqual({
              syntax: 'front-matter',
              source: {
                kind: 'content',
                sliceIndex: 0,
                nodes: [],
                startOffset: 0,
                endOffset: SOURCE.raw.length,
                text: SOURCE.raw,
              },
              target: {
                kind: 'content',
                sliceIndex: 0,
                nodes: [],
                startOffset: 0,
                endOffset: TARGET.raw.length,
                text: TARGET.raw,
              },
            },);
          },
        },),

        it({
          name: 'RETURNS EXPLICIT NONE when neither document declares metadata',
          fn: async () => {
            expect(frontMatterSlice({},),).toEqual({ kind: 'none', },);
          },
        },),

        it({
          name: 'CREATES INSERTION SLICE when source metadata has no target rendering',
          fn: async () => {
            const result = frontMatterSlice({ source: SOURCE, },);
            expect(result.kind,).toBe('paired',);
            if (result.kind !== 'paired')
              throw new Error('source-only metadata did not create insertion slice',);
            expect(result.slice.target,).toEqual({
              kind: 'insertion',
              sliceIndex: 0,
              nodes: [],
              startOffset: 0,
              endOffset: 0,
              text: '',
            },);
          },
        },),

        it({
          name: 'PRESERVES TARGET-ONLY METADATA outside localized slice',
          fn: async () => {
            expect(frontMatterSlice({ target: TARGET, },),).toEqual({ kind: 'none', },);
          },
        },),

        it({
          name: 'PLACES BOTH SPANS AFTER A LEADING MARK each page opens with, the mark outside the slice text so no '
            + 'rewrite of the metadata reaches it',
          fn: async () => {
            /**
             Source metadata of a page opening with the mark.
             */
            const marked = { ...SOURCE, startOffset: 1, };

            /**
             Target metadata of a page opening with the mark.
             */
            const markedTarget = { ...TARGET, startOffset: 1, };

            expect(frontMatterSlice({ source: marked, target: markedTarget, },),).toEqual({
              kind: 'paired',
              slice: {
                syntax: 'front-matter',
                source: {
                  kind: 'content',
                  sliceIndex: 0,
                  nodes: [],
                  startOffset: 1,
                  endOffset: 1 + SOURCE.raw.length,
                  text: SOURCE.raw,
                },
                target: {
                  kind: 'content',
                  sliceIndex: 0,
                  nodes: [],
                  startOffset: 1,
                  endOffset: 1 + TARGET.raw.length,
                  text: TARGET.raw,
                },
              },
            },);
          },
        },),

        it({
          name: 'INSERTS AFTER THE TARGET\'S LEADING MARK where the target has no metadata, so the mark stays '
            + 'the first character of the page',
          fn: async () => {
            /**
             Result for a source with metadata and a target opening with the mark and no metadata.
             */
            const result = frontMatterSlice({ source: SOURCE, targetInsertionOffset: 1, },);
            expect(result.kind,).toBe('paired',);
            if (result.kind !== 'paired')
              throw new Error('source-only metadata did not create insertion slice',);
            expect(result.slice.target,).toEqual({
              kind: 'insertion',
              sliceIndex: 0,
              nodes: [],
              startOffset: 1,
              endOffset: 1,
              text: '',
            },);
          },
        },),
      ],
    },),

    describe({
      name: frontMatterRepairOutcome.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'EMITS EXPLICIT UNCHANGED REPAIR ROW so prose editors never touch YAML',
          fn: async () => {
            const outcome = frontMatterRepairOutcome({
              sliceIndex: 0,
              targetText: TARGET.raw,
            },);
            expect(outcome.changed,).toBe(false,);
            expect(outcome.repairedText,).toBe(TARGET.raw,);
            expect(outcome.findings,).toContain(
              'repair-front-matter-not-applicable (translate ensemble owns YAML metadata)',
            );
          },
        },),
        // LEDGER B29: the front-matter contest checks its lanes without the
        // slice's disputed wordings (`lane-contest-eligibility.ts`), which is
        // sound only while no dispute can exist at a front-matter slice.
        it({
          name: 'RAISES NO ISSUE, so no dispute and no disputed wording exists at a front-matter slice',
          fn: async () => {
            /**
             Repair row for the metadata slice.
             */
            const outcome = frontMatterRepairOutcome({
              sliceIndex: 0,
              targetText: TARGET.raw,
            },);
            expect(outcome.issues,).toEqual([],);
            expect(archiveDisputesOf({ chunks: [outcome,], },).size,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: validateFrontMatterTranslation.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS TRANSLATED SCALARS under exact archive key and container shape',
          fn: async () => {
            expect(validateFrontMatterTranslation({
              sourceText: SOURCE.raw,
              pageText: TARGET.raw,
              candidateText: '---\nname: Mao\ninfo:\n  alias: Kitty\n---\n',
            },).kind,).toBe('valid',);
          },
        },),

        it({
          name: 'ACCEPTS SOURCE-SHAPED METADATA when target page has none',
          fn: async () => {
            expect(validateFrontMatterTranslation({
              sourceText: SOURCE.raw,
              pageText: '',
              candidateText: '---\nname: Mao\ninfo:\n  alias: Kitty\n---\n',
            },).kind,).toBe('valid',);
          },
        },),

        it({
          name: 'JUDGES A CANDIDATE BY THE ORIGINAL\'S SHAPE where the page\'s front matter holds nothing, '
            + 'since an empty block has no established keys to keep',
          fn: async () => {
            /**
             Page whose fence pair holds no metadata at all.
             */
            const emptyPage = '---\n---\n\nThe cat naps.\n';
            expect(validateFrontMatterTranslation({
              sourceText: SOURCE.raw,
              pageText: emptyPage,
              candidateText: TRANSLATED,
            },).kind,).toBe('valid',);
            expect(validateFrontMatterTranslation({
              sourceText: SOURCE.raw,
              pageText: emptyPage,
              candidateText: '---\n---\n',
            },),).toEqual({
              kind: 'invalid',
              findings: ['Your translation changed YAML field names, nesting, container lengths, or scalar kinds.',],
            },);
          },
        },),

        it({
          name: 'REFUSES A LIST-SHAPED info.alias THAT DROPS THE NAME, as the comma-separated form is refused '
            + '(ledger B97)',
          fn: async () => {
            /**
             Original whose alias list holds only its name, so name and alias are one identity.
             */
            const listSource = '---\nname: 猫猫\ninfo:\n  alias:\n    - 猫猫\n---\n';
            /**
             Archive in the same list shape, which a candidate keeps.
             */
            const listPage = '---\nname: Maomao\ninfo:\n  alias:\n    - Maomao\n---\n';
            expect(validateFrontMatterTranslation({
              sourceText: listSource,
              pageText: listPage,
              candidateText: '---\nname: Maomao\ninfo:\n  alias:\n    - Kitty\n---\n',
            },),).toEqual({
              kind: 'invalid',
              findings: [
                'Your translation must carry the name among the comma-separated renderings in info.alias because ORIGINAL declares name and info.alias as the same identity.',
              ],
            },);
            expect(validateFrontMatterTranslation({
              sourceText: listSource,
              pageText: listPage,
              candidateText: listPage,
            },).kind,).toBe('valid',);
          },
        },),

        it({
          name: 'READS AN info.alias LIST HOLDING ANYTHING BUT TEXT AS ANOTHER SCHEMA, holding no identity rule over '
            + 'it, where the same list holding the name as text refuses a candidate dropping the name',
          fn: async () => {
            expect(validateFrontMatterTranslation({
              sourceText: '---\nname: 猫猫\ninfo:\n  alias:\n    - text: 猫猫\n---\n',
              pageText: '---\nname: Maomao\ninfo:\n  alias:\n    - text: Maomao\n---\n',
              candidateText: '---\nname: Maomao\ninfo:\n  alias:\n    - text: Kitty\n---\n',
            },).kind,).toBe('valid',);
            // THE CONTROL: the one item written as text makes name and alias
            // one identity, and the candidate dropping the name is refused.
            expect(validateFrontMatterTranslation({
              sourceText: '---\nname: 猫猫\ninfo:\n  alias:\n    - 猫猫\n---\n',
              pageText: '---\nname: Maomao\ninfo:\n  alias:\n    - Maomao\n---\n',
              candidateText: '---\nname: Maomao\ninfo:\n  alias:\n    - Kitty\n---\n',
            },).kind,).toBe('invalid',);
          },
        },),

        it({
          name: 'REFUSES FIELD LOSS, BODY PROSE, AND MALFORMED YAML',
          fn: async () => {
            expect(validateFrontMatterTranslation({
              sourceText: SOURCE.raw,
              pageText: TARGET.raw,
              candidateText: '---\nname: Mao\n---\n',
            },).kind,).toBe('invalid',);
            expect(validateFrontMatterTranslation({
              sourceText: SOURCE.raw,
              pageText: TARGET.raw,
              candidateText: `${TARGET.raw}explanation`,
            },).kind,).toBe('invalid',);
            expect(validateFrontMatterTranslation({
              sourceText: SOURCE.raw,
              pageText: TARGET.raw,
              candidateText: '---\nname: [broken\n---\n',
            },).kind,).toBe('invalid',);
          },
        },),

        it({
          name: 'LEAVES A CANDIDATE UNVALIDATED where the original or the page carries no fenced front matter, '
            + 'and still charges the candidate its own faults first',
          fn: async () => {
            expect(validateFrontMatterTranslation({
              sourceText: 'name: 猫猫\n',
              pageText: TARGET.raw,
              candidateText: TRANSLATED,
            },),).toEqual({
              kind: 'unknown',
              detail: 'source front matter could not be read',
            },);
            expect(validateFrontMatterTranslation({
              sourceText: SOURCE.raw,
              pageText: 'name: Maomao\n',
              candidateText: TRANSLATED,
            },),).toEqual({
              kind: 'unknown',
              detail: 'page front matter could not be read',
            },);
            expect(validateFrontMatterTranslation({
              sourceText: 'name: 猫猫\n',
              pageText: TARGET.raw,
              candidateText: `${TRANSLATED}explanation`,
            },),).toEqual({
              kind: 'invalid',
              findings: ['Your translation added text outside YAML front matter block.',],
            },);
          },
        },),

        it({
          name: 'LEAVES A CANDIDATE UNVALIDATED where the original\'s or the page\'s YAML is refused, rather than '
            + 'charging that refusal to the candidate as its own (ledger B44)',
          fn: async () => {
            /**
             What the splitter says of the refused YAML, which each side's detail
             carries.
             */
            const refusal = caught(function splitBroken(): void {
              splitFrontMatter({ text: BROKEN_YAML, },);
            },);
            if (!(refusal instanceof FrontMatterParseError))
              throw new Error('the broken fixture parsed, so it no longer exercises a refusal',);

            expect(validateFrontMatterTranslation({
              sourceText: BROKEN_YAML,
              pageText: TARGET.raw,
              candidateText: TRANSLATED,
            },),).toEqual({
              kind: 'unknown',
              detail: `source front matter could not be read: ${refusal.message}`,
            },);
            expect(validateFrontMatterTranslation({
              sourceText: SOURCE.raw,
              pageText: BROKEN_YAML,
              candidateText: TRANSLATED,
            },),).toEqual({
              kind: 'unknown',
              detail: `page front matter could not be read: ${refusal.message}`,
            },);
          },
        },),

        it({
          name: 'REFUSES SOURCE-SCRIPT COMMENT ATTRIBUTION replacing established target form',
          fn: async () => {
            expect(validateFrontMatterTranslation({
              sourceText: COMMENT_SOURCE,
              pageText: COMMENT_TARGET,
              candidateText: '---\nname: Maomao\ninfo:\n  alias: Maomao\n  location: Guangdong #Qingyuan, by 魔骨\n---\n',
            },).kind,).toBe('invalid',);
            expect(validateFrontMatterTranslation({
              sourceText: COMMENT_SOURCE,
              pageText: COMMENT_TARGET,
              candidateText: COMMENT_TARGET,
            },).kind,).toBe('valid',);
          },
        },),

        it({
          name: 'REFUSES DIRECTORY ID when source visible name and alias are same identity',
          fn: async () => {
            expect(validateFrontMatterTranslation({
              sourceText: SAME_IDENTITY_SOURCE,
              pageText: DIRECTORY_ID_TARGET,
              candidateText: DIRECTORY_ID_TARGET,
            },).kind,).toBe('invalid',);
            expect(validateFrontMatterTranslation({
              sourceText: SAME_IDENTITY_SOURCE,
              pageText: DIRECTORY_ID_TARGET,
              candidateText: '---\nname: Maomao\ninfo:\n  alias: Maomao\n---\n',
            },).kind,).toBe('valid',);
          },
        },),

        it({
          name: 'FINDS NO COMMENT RELATION WHEN SOURCE HAS NO info MAPPING, so a missing block establishes '
            + 'nothing to protect',
          fn: async () => {
            expect(validateFrontMatterTranslation({
              sourceText: INFO_MISSING_SOURCE,
              pageText: COMMENT_AUTHORITY_PAGE,
              candidateText: COMMENT_AUTHORITY_CANDIDATE,
            },).kind,).toBe('valid',);
          },
        },),

        it({
          name: 'FINDS NO COMMENT RELATION WHEN SOURCE\'S LOCATION CARRIES NO INLINE COMMENT, so a bare '
            + 'scalar establishes nothing to protect',
          fn: async () => {
            expect(validateFrontMatterTranslation({
              sourceText: LOCATION_NO_COMMENT_SOURCE,
              pageText: COMMENT_AUTHORITY_PAGE,
              candidateText: COMMENT_AUTHORITY_CANDIDATE,
            },).kind,).toBe('valid',);
          },
        },),

        it({
          name: 'FINDS NO COMMENT RELATION WHEN SOURCE\'S COMMENT NAMES NO CONTRIBUTOR, so a place-only note '
            + 'establishes nothing to protect',
          fn: async () => {
            expect(validateFrontMatterTranslation({
              sourceText: PLACE_ONLY_COMMENT_SOURCE,
              pageText: COMMENT_AUTHORITY_PAGE,
              candidateText: COMMENT_AUTHORITY_CANDIDATE,
            },).kind,).toBe('valid',);
          },
        },),

        it({
          name: 'FINDS NO COMMENT RELATION WHEN SOURCE\'S CONTRIBUTOR MARKER IS FOLLOWED BY NOTHING, so a '
            + 'dangling `, by ` establishes nothing to protect',
          fn: async () => {
            expect(validateFrontMatterTranslation({
              sourceText: DANGLING_MARKER_SOURCE,
              pageText: COMMENT_AUTHORITY_PAGE,
              candidateText: COMMENT_AUTHORITY_CANDIDATE,
            },).kind,).toBe('valid',);
          },
        },),

        it({
          name: 'READS other-schema FROM A NULL FRONT MATTER ROOT, so the identity rule stays quiet instead '
            + 'of guessing a name',
          fn: async () => {
            expect(validateFrontMatterTranslation({
              sourceText: '---\n---\n',
              pageText: '',
              candidateText: '---\n---\n',
            },).kind,).toBe('valid',);
          },
        },),

        it({
          name: 'READS other-schema FROM FRONT MATTER WITH NO info BLOCK, so the identity rule has nothing to '
            + 'carry the name into',
          fn: async () => {
            expect(validateFrontMatterTranslation({
              sourceText: '---\nname: Tuxedo\n---\n\nBody.\n',
              pageText: '---\nname: Biscuit\n---\n\nBody.\n',
              candidateText: '---\nname: Whiskers\n---\n',
            },).kind,).toBe('valid',);
          },
        },),

        it({
          name: 'REFUSES A CANDIDATE WHOSE VISIBLE name IS NOT A STRING, so a numeric placeholder never '
            + 'counts as carrying the name into info.alias',
          fn: async () => {
            expect(validateFrontMatterTranslation({
              sourceText: '---\nname: Mittens\ninfo:\n  alias: Mittens\n---\n\nBody.\n',
              pageText: '---\nname: 42\ninfo:\n  alias: Mittens\n---\n\nBody.\n',
              candidateText: '---\nname: 42\ninfo:\n  alias: Mittens\n---\n',
            },),).toEqual({
              kind: 'invalid',
              findings: [
                'Your translation must carry the name among the comma-separated renderings in info.alias because ORIGINAL declares name and info.alias as the same identity.',
              ],
            },);
          },
        },),

        it({
          name: 'ACCEPTS A NULL-VALUED FIELD MATCHING THE ARCHIVE, so two front matters whose info.location '
            + 'is left empty compare as the same shape, AND REFUSES THE SAME FIELD WRITTEN AS TEXT',
          fn: async () => {
            /**
             Source establishing a distinct name and alias, so the identity rule
             stays quiet whatever shape outcome this case asserts.
             */
            const sourceText = '---\nname: Clementine\ninfo:\n  alias: Clem\n---\n\nBody.\n';
            /**
             Archive whose info.location key is declared with nothing after it.
             */
            const pageText = '---\nname: Clementine\ninfo:\n  alias: Clem\n  location:\n---\n\nBody.\n';

            expect(validateFrontMatterTranslation({
              sourceText,
              pageText,
              candidateText: '---\nname: Clementine\ninfo:\n  alias: Clem\n  location:\n---\n',
            },).kind,).toBe('valid',);
            expect(validateFrontMatterTranslation({
              sourceText,
              pageText,
              candidateText: '---\nname: Clementine\ninfo:\n  alias: Clem\n  location: Garden Shed\n---\n',
            },),).toEqual({
              kind: 'invalid',
              findings: ['Your translation changed YAML field names, nesting, container lengths, or scalar kinds.',],
            },);
          },
        },),

        it({
          name: 'ACCEPTS A TRANSLATED TAG LIST MATCHING SHAPE, AND REFUSES THE SAME KEYS WITH AN ITEM '
            + 'DROPPED OR WRITTEN AS A MAPPING, since a YAML list signs by position and length, never by '
            + 'key count alone',
          fn: async () => {
            /**
             Source establishing a distinct name and alias, so the identity rule
             stays quiet whatever shape outcome this case asserts.
             */
            const sourceText = '---\nname: Clementine\ninfo:\n  alias: Clem\n---\n\nThe cat naps.\n';
            /**
             Archive declaring a two-item tag list.
             */
            const pageText =
              '---\nname: Clementine\ninfo:\n  alias: Clem\n  tags:\n    - orange\n    - fluffy\n---\n\nThe cat naps.\n';
            /**
             Shape-mismatch finding shared by every assertion here except the first.
             */
            const changedShapeFinding = {
              kind: 'invalid',
              findings: ['Your translation changed YAML field names, nesting, container lengths, or scalar kinds.',],
            };

            expect(validateFrontMatterTranslation({
              sourceText,
              pageText,
              candidateText: '---\nname: Clementine\ninfo:\n  alias: Clem\n  tags:\n    - 橘色\n    - 蓬松\n---\n',
            },).kind,).toBe('valid',);
            expect(validateFrontMatterTranslation({
              sourceText,
              pageText,
              candidateText: '---\nname: Clementine\ninfo:\n  alias: Clem\n  tags:\n    - 橘色\n---\n',
            },),).toEqual(changedShapeFinding,);
            expect(validateFrontMatterTranslation({
              sourceText,
              pageText,
              candidateText: '---\nname: Clementine\ninfo:\n  alias: Clem\n  tags:\n    \'0\': 橘色\n    \'1\': 蓬松\n---\n',
            },),).toEqual(changedShapeFinding,);
          },
        },),
      ],
    },),
  ],
},);
