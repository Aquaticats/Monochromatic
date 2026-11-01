/**
 Tests that a claim's summary and quotes cannot open a line of their own on
 a sheet or log line whose items are told apart by the lines that open them
 (ledger L14(d)).

 WHY. The panel sheet renders each quote raw after `- evidence (SIDE): `, and
 the panel, checker, editor and introduced-defect probe sheets, both human
 grading sheets and the filing log lines render each claim summary raw. Over
 every artifact 349 of 45,860 quotes and 3 of 23,714 summaries carry a line
 break, so their later lines stood as unlabelled lines between items, and a
 later line reading like `CLAIM 2` renumbers every ballot after it. A log
 line split the same way loses its chunk tag on every line after the first.

 WHAT EACH KEEPS. A summary is one sentence of prose, so its line breaks fold
 to spaces. A panel quote is evidence the claim is checked against, so its
 later lines stay, indented under the evidence item. The grading sheet folds
 its quotes as the repair sheet already did.

 Fixtures are cat-themed invention; the quote is not taken from the fixture
 documents on purpose, so a line it opens can only have come from the quote.

 @module
 */

import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import { formatRepairSheet, } from '@monochromatic-dev/translation-repair-assembly';
import {
  DEFAULT_PRECISION_BAR,
  DEFAULT_SAMPLE_SEED,
  formatGradingSheet,
  type GradingCandidate,
} from '@monochromatic-dev/translation-repair-house-rule';
import {
  buildAdjudicationMessages,
  buildIntroducedDefectMessages,
  buildResolutionMessages,
} from '@monochromatic-dev/translation-repair-adjudication';
import {
  type AdjudicatedIssue,
  aggregateClaims,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  buildEditorMessages,
  describeClaimFiling,
  describeIssueFiling,
} from '../dist/final/node/index.mjs';

/**
 Invented original.
 */
const SOURCE_TEXT = '咪咪在中午打盹。';

/**
 Invented archive rendering.
 */
const TARGET_TEXT = 'Mittens hunts at noon.';

/**
 Opening of the summary's second line, which no fixture document carries.
 */
const SUMMARY_TAIL = 'ZQSUMMARYTAIL 2';

/**
 Summary spanning two lines.
 */
const SUMMARY = `Napping is rendered as hunting.\n${SUMMARY_TAIL}`;

/**
 The summary folded onto one line, which every sheet must carry.
 */
const FOLDED_SUMMARY = `Napping is rendered as hunting. ${SUMMARY_TAIL}`;

/**
 Opening of the quote's second line, which no fixture document carries.
 */
const QUOTE_TAIL = 'ZQQUOTETAIL';

/**
 Quote spanning two lines.
 */
const QUOTE = `hunts\n${QUOTE_TAIL}`;

/**
 Accepted issue carrying the two-line summary and quote.
 */
const ISSUE: AdjudicatedIssue = {
  issueId: 'issue/nap',
  status: 'accepted',
  severity: 'major',
  claims: [
    {
      claimId: 'claim/nap',
      claim: {
        category: 'accuracy/mistranslation',
        severity: 'major',
        summary: SUMMARY,
        spans: [
          {
            side: 'target',
            nodeId: 'block/1',
            nodeHash: hashContent({ content: TARGET_TEXT, },),
            startOffset: TARGET_TEXT.indexOf('hunts',),
            endOffset: TARGET_TEXT.indexOf('hunts',) + QUOTE.length,
            quotedText: QUOTE,
          },
        ],
      },
    },
  ],
  tallies: {},
};

/**
 The claim itself, as the critic stage files it.
 */
const CLAIM = ISSUE.claims[0]?.claim;

/**
 Grading candidate carrying the same summary and quote.
 */
const CANDIDATE: GradingCandidate = {
  entryId: 'mittens',
  band: 'small',
  issueId: ISSUE.issueId,
  category: 'accuracy/mistranslation',
  severity: 'major',
  summary: SUMMARY,
  sourceQuotes: ['打盹',],
  targetQuotes: [QUOTE,],
  sourceAnchor: 'quoted',
};

/**
 Every message of an exchange, joined.

 @param messages - exchange to read

 @returns Joined text

 @example
 ```ts
 const text = joined({ messages, },);
 ```
 */
function joined({ messages, }: { readonly messages: readonly ChatMessage[]; },): string {
  return messages
    .map(function textOf(message,) {
      return messageText({ message, },);
    },)
    .join('\n',);
}

/**
 Whether any line of a text opens with the given words.

 @param text - sheet or log text

 @param opening - words a line may not open with

 @returns Whether such a line exists

 @example
 ```ts
 const forged = opensLine({ text, opening: SUMMARY_TAIL, },);
 ```
 */
function opensLine({ text, opening, }: { readonly text: string; readonly opening: string; },): boolean {
  return text
    .split('\n',)
    .some(function opens(line,) {
      return line.startsWith(opening,);
    },);
}

/**
 Every sheet and log line carrying the claim, by name.
 */
const RENDERED: readonly { readonly name: string; readonly text: string; }[] = [
  {
    name: 'panel',
    text: joined({
      messages: buildAdjudicationMessages({
        sourceText: SOURCE_TEXT,
        targetText: TARGET_TEXT,
        clusters: aggregateClaims({ claims: (CLAIM === undefined) ? [] : [CLAIM,], },).clusters,
      },).messages,
    },),
  },
  {
    name: 'checker',
    text: joined({
      messages: buildResolutionMessages({
        sourceText: SOURCE_TEXT,
        patchedText: TARGET_TEXT,
        issues: [ISSUE,],
      },).messages,
    },),
  },
  {
    name: 'editor',
    text: joined({
      messages: buildEditorMessages({
        sourceText: SOURCE_TEXT,
        targetText: TARGET_TEXT,
        envelopes: [
          {
            envelopeId: 'envelope/nap',
            startOffset: 0,
            endOffset: TARGET_TEXT.length,
            baseText: TARGET_TEXT,
            baseHash: hashContent({ content: TARGET_TEXT, },),
            issueIds: [ISSUE.issueId,],
          },
        ],
        issues: [ISSUE,],
      },).messages,
    },),
  },
  {
    name: 'introduced-defect probe',
    text: joined({
      messages: buildIntroducedDefectMessages({
        sourceText: SOURCE_TEXT,
        baselineText: TARGET_TEXT,
        regions: [
          {
            envelopeId: 'envelope/nap',
            issueIds: [ISSUE.issueId,],
            before: TARGET_TEXT,
            editorAfter: 'Mittens naps at noon.',
          },
        ],
        issues: [ISSUE,],
      },).messages,
    },),
  },
  {
    name: 'grading sheet',
    text: formatGradingSheet({
      sample: [CANDIDATE,],
      seed: DEFAULT_SAMPLE_SEED,
      bar: DEFAULT_PRECISION_BAR,
      corpusSha: 'feedfac',
      drawDigest: 'digest/mittens',
    },),
  },
  {
    name: 'repair sheet',
    text: formatRepairSheet({
      sample: [CANDIDATE,],
      seed: DEFAULT_SAMPLE_SEED,
      corpusSha: 'feedfac',
      drawDigest: 'digest/mittens',
    },),
  },
  {
    name: 'claim filing log line',
    text: (CLAIM === undefined)
      ? ''
      : describeClaimFiling({
        sliceIndex: 1,
        claimId: 'claim/nap',
        claim: CLAIM,
        filers: {},
      },),
  },
  {
    name: 'issue filing log line',
    text: describeIssueFiling({
      sliceIndex: 1,
      issue: ISSUE,
      filers: {},
    },),
  },
];

await describe({
  name: 'a claim\'s summary and quotes open no line of their own (ledger L14(d))',
  children: [
    ...RENDERED.map(function toCase({ name, text, },) {
      return it({
        name: `the ${name} FOLDS the summary onto one line`,
        fn: async () => {
          expect({
            folded: text.includes(FOLDED_SUMMARY,),
            opened: opensLine({ text, opening: SUMMARY_TAIL, },),
          },).toEqual({
            folded: true,
            opened: false,
          },);
        },
      },);
    },),
    ...RENDERED.map(function toCase({ name, text, },) {
      return it({
        name: `the ${name} lets no quote open a line`,
        fn: async () => {
          expect(opensLine({ text, opening: QUOTE_TAIL, },),).toBe(false,);
        },
      },);
    },),
    it({
      name: 'the panel KEEPS a quote\'s later lines, indented under the evidence item',
      fn: async () => {
        /**
         Panel sheet text.
         */
        const panel = RENDERED.find(function isPanel({ name, },) {
          return name === 'panel';
        },)?.text ?? '';
        expect(panel.includes(`- evidence (TRANSLATION): hunts\n  ${QUOTE_TAIL}`,),).toBe(true,);
      },
    },),
    it({
      name: 'the grading sheet FOLDS a quote as the repair sheet does',
      fn: async () => {
        /**
         Grading sheet text.
         */
        const grading = RENDERED.find(function isGrading({ name, },) {
          return name === 'grading sheet';
        },)?.text ?? '';
        expect(grading.includes(`“hunts ${QUOTE_TAIL}”`,),).toBe(true,);
      },
    },),
  ],
},);
