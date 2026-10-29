/**
 Tests for what survives screening of one auditor's answer.

 THE OBLIGATION RUNS BOTH WAYS, and half of these cases exist for the
 direction the first version left unenforced: a category resting on one side
 must not carry a quote on the other. An `omission` arriving with candidate
 text contradicts itself, and silently erasing that text let a voice file a
 paired claim under a one-sided category and escape the evidence the paired
 one asks for.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  RENDERING_AUDIT_CATEGORIES,
  type RenderingAuditFindingWire,
  quotesRequired,
  screenRenderingAudit,
} from '../dist/final/node/index.mjs';

/**
 Original every case screens against.
 */
const SOURCE_TEXT = '三只猫住在书店的阁楼里。她们不吃罐头，只喝温牛奶。';

/**
 Rendering with the negation dropped.
 */
const CANDIDATE_TEXT = 'Three cats live in the bookshop attic. They eat canned food, and drink warm milk.';

/**
 Characters of a model-supplied word a drop reason may repeat, as the screen
 bounds it.
 */
const DROPPED_WORD_LIMIT = 32;

/**
 Length of a single token that overruns that bound.
 */
const OVERLONG_LENGTH = 40;

/**
 Sides a category rests on when only the original can be quoted for it.
 */
const ORIGINAL_ALONE = {
  needsSource: true,
  needsCandidate: false,
} as const;

/**
 Sides a category rests on when only the candidate can be quoted for it.
 */
const CANDIDATE_ALONE = {
  needsSource: false,
  needsCandidate: true,
} as const;

/**
 Sides a category rests on when both texts state the thing that changed.
 */
const BOTH_SIDES = {
  needsSource: true,
  needsCandidate: true,
} as const;

/**
 Fields a case may replace on the sound finding.

 SPELLED OUT rather than derived from the wire type, because every property
 here is genuinely optional to a case: one that changes only the category
 should not restate four quotes it does not care about.

 @example
 ```ts
 const overrides: FindingOverrides = { category: 'omission', };
 ```
 */
type FindingOverrides = {
  /**
   Category this case names.
   */
  readonly category?: string;

  /**
   Original span identifying the occurrence.
   */
  readonly sourceLocator?: string;

  /**
   Original span carrying the change.
   */
  readonly sourceFocus?: string;

  /**
   Candidate span identifying the occurrence.
   */
  readonly candidateLocator?: string;

  /**
   Candidate span carrying the change.
   */
  readonly candidateFocus?: string;

  /**
   What the voice says the spans amount to.
   */
  readonly reason?: string;
};

/**
 One finding with every field, which each case overrides one part of.

 @param overrides - fields this case changes

 @returns Finding as a voice would send it

 @example
 ```ts
 const finding = claim({ overrides: { category: 'omission', }, },);
 ```
 */
function claim(
  { overrides, }: { readonly overrides: FindingOverrides; },
): RenderingAuditFindingWire {
  return {
    category: 'altered-polarity',
    sourceLocator: '她们不吃罐头',
    sourceFocus: '不吃',
    candidateLocator: 'They eat canned food',
    candidateFocus: 'eat',
    reason: 'the original denies it and the candidate asserts it',
    ...overrides,
  };
}

/**
 Screens one answer carrying one finding.

 @param overrides - fields that finding changes

 @param verdict - what the voice cast

 @returns Screened report

 @example
 ```ts
 const screened = screenOne({ overrides: {}, },);
 ```
 */
function screenOne(
  {
    overrides,
    verdict = 'defects-found',
  }: {
    readonly overrides: FindingOverrides;
    readonly verdict?: string;
  },
): ReturnType<typeof screenRenderingAudit> {
  return screenRenderingAudit({
    report: {
      verdict,
      findings: [claim({ overrides, },),],
    },
    sourceText: SOURCE_TEXT,
    candidateText: CANDIDATE_TEXT,
  },);
}

await describe({
  name: screenRenderingAudit.name,
  children: [
    it({
      name: 'KEEPS a paired finding that anchors on both sides, and reads each side as anchored',
      fn: async () => {
        const screened = screenOne({ overrides: {}, },);
        expect(screened.findings,).toHaveLength(1,);
        expect(screened.dropped,).toEqual([],);
        expect(screened.findings[0]
          ?.source
          .kind,).toBe('anchored',);
        expect(screened.findings[0]
          ?.candidate
          .kind,).toBe('anchored',);
      },
    },),
    it({
      name: 'KEEPS a finding whose candidate locator runs across a note between translators, because the '
        + 'auditor was shown the candidate without the note and the locator is found in the text as shown',
      fn: async () => {
        const screened = screenRenderingAudit({
          report: {
            verdict: 'defects-found',
            findings: [
              claim({
                overrides: {
                  candidateLocator: 'attic.\nThey eat canned food',
                  candidateFocus: 'eat',
                },
              },),
            ],
          },
          sourceText: SOURCE_TEXT,
          candidateText:
            'Three cats live in the bookshop attic.{/* 这句不太对劲 */}\nThey eat canned food, and drink warm milk.',
        },);
        expect(screened.dropped,).toEqual([],);
        expect(screened.findings,).toHaveLength(1,);
        expect(screened.findings[0]
          ?.candidate
          .kind,).toBe('anchored',);
      },
    },),
    it({
      name:
        'KEEPS an omission quoting the ORIGINAL only, since content the candidate never rendered has '
          + 'nothing in the candidate to point at, and marks the other side unused rather than empty',
      fn: async () => {
        const screened = screenOne({
          overrides: {
            category: 'omission',
            sourceLocator: '只喝温牛奶',
            sourceFocus: '温',
            candidateLocator: '',
            candidateFocus: '',
          },
        },);
        expect(screened.findings,).toHaveLength(1,);
        expect(screened.findings[0]
          ?.candidate
          .kind,).toBe('unused',);
      },
    },),
    it({
      name:
        'DROPS an omission carrying candidate text, a claim contradicting itself: it says the candidate '
          + 'rendered nothing here and then quotes what the candidate rendered here',
      fn: async () => {
        const screened = screenOne({
          overrides: {
            category: 'omission',
            sourceLocator: '只喝温牛奶',
            sourceFocus: '温',
            candidateLocator: 'drink warm milk',
            candidateFocus: 'warm',
          },
        },);
        expect(screened.findings,).toEqual([],);
        expect(screened.dropped,).toEqual(['forbidden-side-quote (candidate)',],);
      },
    },),
    it({
      name:
        'KEEPS a one-sided claim whose unused side carries only spaces, a zero-width space or a Hangul filler, '
          + 'screening it exactly as the same claim with that side empty: such a side quotes nothing, so it is '
          + 'no quote on a side the category forbids',
      fn: async () => {
        /**
         Unused-side fillings that show a reader nothing.
         */
        const padding = [
          ' ',
          '\u{200B}',
          '\u{3164}',
          ' \u{200B}\u{3164} ',
        ];
        /**
         An omission, resting on the original alone, with its candidate side as given.

         @param filling - both candidate fields' text

         @returns Screened report

         @example
         ```ts
         const screened = omissionWith('\u{200B}',);
         ```
         */
        function omissionWith(filling: string,): ReturnType<typeof screenRenderingAudit> {
          return screenOne({
            overrides: {
              category: 'omission',
              sourceLocator: '只喝温牛奶',
              sourceFocus: '温',
              candidateLocator: filling,
              candidateFocus: filling,
            },
          },);
        }
        /**
         An unsupported addition, resting on the candidate alone, with its original side as given.

         @param filling - both original fields' text

         @returns Screened report

         @example
         ```ts
         const screened = additionWith('\u{3164}',);
         ```
         */
        function additionWith(filling: string,): ReturnType<typeof screenRenderingAudit> {
          return screenOne({
            overrides: {
              category: 'unsupported-addition',
              sourceLocator: filling,
              sourceFocus: filling,
              candidateLocator: 'They eat canned food',
              candidateFocus: 'canned',
            },
          },);
        }
        expect({
          omissions: padding.map(omissionWith,),
          additions: padding.map(additionWith,),
        },).toEqual({
          omissions: padding.map(function asEmpty() {
            return omissionWith('',);
          },),
          additions: padding.map(function asEmpty() {
            return additionWith('',);
          },),
        },);
        expect(omissionWith('',).dropped,).toEqual([],);
        expect(additionWith('',).dropped,).toEqual([],);
      },
    },),
    it({
      name:
        'DROPS an unsupported-addition carrying original text, the mirror case, so a voice cannot file a '
          + 'paired claim under a one-sided category and skip the evidence a paired category asks for',
      fn: async () => {
        const screened = screenOne({
          overrides: {
            category: 'unsupported-addition',
            sourceLocator: '她们不吃罐头',
            sourceFocus: '不吃',
            candidateLocator: 'They eat canned food',
            candidateFocus: 'canned',
          },
        },);
        expect(screened.findings,).toEqual([],);
        expect(screened.dropped,).toEqual(['forbidden-side-quote (source)',],);
      },
    },),
    it({
      name: 'DROPS a paired finding that anchors on one side only, naming which side failed',
      fn: async () => {
        const screened = screenOne({
          overrides: {
            candidateLocator: 'They devour canned food',
            candidateFocus: 'devour',
          },
        },);
        expect(screened.findings,).toEqual([],);
        expect(screened.dropped,).toEqual(['unanchored-locator (candidate)',],);
      },
    },),
    it({
      name:
        'DROPS a claim whose original quote is another translation`s English rather than the original`s '
          + 'Chinese, which is what keeps a rendering nobody was shown out of this instrument',
      fn: async () => {
        const screened = screenOne({
          overrides: {
            sourceLocator: 'They do not eat canned food',
            sourceFocus: 'not',
          },
        },);
        expect(screened.findings,).toEqual([],);
        expect(screened.dropped,).toEqual(['unanchored-locator (source)',],);
      },
    },),
    it({
      name: 'DROPS a category this version does not name, and says which word it was',
      fn: async () => {
        const screened = screenOne({ overrides: { category: 'altered-whiskers', }, },);
        expect(screened.findings,).toEqual([],);
        expect(screened.dropped,).toEqual(['unknown-category (altered-whiskers)',],);
      },
    },),
    it({
      name: 'BOUNDS THE WORD IT REPEATS from an unknown category to its first token, since the drop '
        + 'list is persisted in every run row and exists to say which word, not to store prose',
      fn: async () => {
        const screened = screenOne({
          overrides: { category: 'altered whiskers, and then a whole sentence about the cat on the sill', },
        },);
        expect(screened.dropped,).toEqual(['unknown-category (altered)',],);
      },
    },),
    it({
      name: 'BOUNDS A LONG SINGLE TOKEN to the first characters of the limit, so no field can carry '
        + 'prose through by leaving the spaces out',
      fn: async () => {
        const screened = screenOne({ overrides: { category: 'x'.repeat(OVERLONG_LENGTH,), }, },);
        expect(screened.dropped,).toEqual([`unknown-category (${'x'.repeat(DROPPED_WORD_LIMIT,)})`,],);
      },
    },),
    it({
      name: 'BOUNDS AN UNKNOWN VERDICT the same way, since the verdict field is the other one a '
        + 'model writes free text into',
      fn: async () => {
        const screened = screenOne({
          overrides: {},
          verdict: 'catastrophic failure of the whole rendering',
        },);
        expect(screened.verdict,).toBe('uncertain',);
        expect(screened.dropped,).toEqual(['unknown-verdict (catastrophic)',],);
      },
    },),
    it({
      name:
        'RECORDS an unknown verdict rather than quietly reading it as uncertain: a voice casting a word '
          + 'this version never offered is a protocol failure, while uncertain is a state a voice may '
          + 'legitimately be in, and reading one as the other hides an instrument defect',
      fn: async () => {
        const screened = screenOne({
          overrides: {},
          verdict: 'catastrophic',
        },);
        expect(screened.verdict,).toBe('uncertain',);
        expect(screened.dropped,).toEqual(['unknown-verdict (catastrophic)',],);
        // AND THE FINDING SURVIVES, because a mis-cast verdict is not a reason
        // to discard evidence that anchors.
        expect(screened.findings,).toHaveLength(1,);
      },
    },),
    it({
      name: 'KEEPS a known verdict without adding anything to the drop list',
      fn: async () => {
        const screened = screenRenderingAudit({
          report: {
            verdict: 'no-defect-found',
            findings: [],
          },
          sourceText: SOURCE_TEXT,
          candidateText: CANDIDATE_TEXT,
        },);
        expect(screened.verdict,).toBe('no-defect-found',);
        expect(screened.dropped,).toEqual([],);
      },
    },),
    describe({
      name: quotesRequired.name,
      children: [
        it({
          name: 'RESTS every category of the vocabulary on the sides its anchoring rule names: an omission '
            + 'on the original alone, an unsupported addition on the candidate alone, each of the rest on both',
          fn: async () => {
            // EVERY CATEGORY THE VOCABULARY HOLDS IS ASKED, so a category added
            // there and given no anchoring rule fails here, by the rule's own
            // refusal, before any run meets it.
            expect(RENDERING_AUDIT_CATEGORIES.map(function sidesOf(category,): readonly [
              string,
              ReturnType<typeof quotesRequired>,
            ] {
              return [
                category,
                quotesRequired({ category, },),
              ];
            },),).toEqual([
              [
                'omission',
                ORIGINAL_ALONE,
              ],
              [
                'unsupported-addition',
                CANDIDATE_ALONE,
              ],
              [
                'altered-actor',
                BOTH_SIDES,
              ],
              [
                'altered-referent',
                BOTH_SIDES,
              ],
              [
                'altered-polarity',
                BOTH_SIDES,
              ],
              [
                'altered-modality',
                BOTH_SIDES,
              ],
              [
                'altered-time',
                BOTH_SIDES,
              ],
              [
                'altered-number',
                BOTH_SIDES,
              ],
              [
                'altered-relation',
                BOTH_SIDES,
              ],
              [
                'altered-identity',
                BOTH_SIDES,
              ],
              [
                'broken-structure',
                BOTH_SIDES,
              ],
            ],);
          },
        },),
      ],
    },),
  ],
},);
