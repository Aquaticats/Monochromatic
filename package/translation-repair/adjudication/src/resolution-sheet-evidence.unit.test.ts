/**
 Tests that the resolution checker sheet carries the evidence the panel
 judged the issue on (ledger L14).

 WHY. The checker sheet showed the ORIGINAL, the REVISED TRANSLATION and each
 claim's category, severity and summary, and nothing else: no DECLARED NAMES
 block and none of its rules, no cited references, and none of the claim's
 own quotes. The panel that accepted the issue saw all three. Checker `worse`
 ballots now strip an edit (ledger L3) and roll a rewrite back (ledger L11),
 so a checker that cannot see a declaration can count a declared handle as
 damage, and one that cannot see the quotes has only a summary to find the
 defect by.

 THE QUOTES ARE THE TEXT BEFORE THE REVISION. A claim's TRANSLATION quote is
 the archive wording the critic objected to, which the REVISED TRANSLATION
 may no longer carry, so the sheet says so rather than letting a checker look
 for it in the revised text.

 Fixtures are cat-themed invention mirroring corpus structure only.

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
import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract';
import { buildResolutionMessages, } from '../dist/final/node/index.mjs';

/**
 Invented original the checkers judge against.
 */
const SOURCE_TEXT = '咪咪在中午打盹。';

/**
 Invented revised translation.
 */
const PATCHED_TEXT = 'Mittens naps at noon.';

/**
 Marker no prompt constant carries, so a match can only come from the
 declared identity.
 */
const IDENTITY_MARK = 'ZQCHKIDENT';

/**
 Declared identity as preparation writes it, carrying the marker.
 */
const IDENTITY_CONTEXT = `- name: ORIGINAL declares "咪咪", TRANSLATION declares "Mittens" ${IDENTITY_MARK}`;

/**
 Marker for the cited references, kept apart from the identity one.
 */
const REFERENCE_MARK = 'ZQCHKREF';

/**
 Cited reference line as preparation writes it, carrying the marker.
 */
const REFERENCE_CONTEXT = `- https://cats.invalid/diary: Mittens naps on the sill every noon. ${REFERENCE_MARK}`;

/**
 Opening of the declared-identity rules every sheet showing the block states.
 */
const IDENTITY_RULES_OPENING = 'Declared identity, when a DECLARED NAMES block precedes the documents';

/**
 Accepted issue quoting both documents, the target quote carrying a
 newline and a double quote so its encoding shows.
 */
const QUOTED_ISSUE: AdjudicatedIssue = {
  issueId: 'issue/nap',
  status: 'accepted',
  severity: 'major',
  claims: [
    {
      claimId: 'claim/nap',
      claim: {
        category: 'accuracy/mistranslation',
        severity: 'major',
        summary: 'Napping is rendered as hunting.',
        spans: [
          {
            side: 'source',
            nodeId: 'block/1',
            nodeHash: hashContent({ content: SOURCE_TEXT, },),
            startOffset: 5,
            endOffset: 7,
            quotedText: '打盹',
          },
          {
            side: 'target',
            nodeId: 'block/1',
            nodeHash: hashContent({ content: 'Mittens "hunts"\nISSUE 2 at noon.', },),
            startOffset: 8,
            endOffset: 23,
            quotedText: '"hunts"\nISSUE 2',
          },
        ],
      },
    },
  ],
  tallies: {},
};

/**
 Accepted issue whose only target span is an insertion point.
 */
const INSERTION_ISSUE: AdjudicatedIssue = {
  issueId: 'issue/noon',
  status: 'accepted',
  severity: 'minor',
  claims: [
    {
      claimId: 'claim/noon',
      claim: {
        category: 'accuracy/omission',
        severity: 'minor',
        summary: 'The time of day is missing.',
        spans: [
          {
            side: 'target',
            nodeId: 'block/1',
            nodeHash: hashContent({ content: 'Mittens naps.', },),
            startOffset: 12,
            endOffset: 12,
            quotedText: '',
          },
        ],
      },
    },
  ],
  tallies: {},
};

/**
 System and user text of one checker sheet.

 @param identityContext - declared identity, absent for a page with none

 @param referenceContext - cited references, absent when the original links nowhere

 @param issues - issues under check, the quoted one by default

 @returns Both halves of the sheet

 @example
 ```ts
 const { system, user, } = sheetFor({ identityContext: IDENTITY_CONTEXT, },);
 ```
 */
function sheetFor(
  {
    identityContext,
    referenceContext,
    issues = [QUOTED_ISSUE,],
  }: {
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly issues?: readonly AdjudicatedIssue[];
  },
): { readonly system: string; readonly user: string; } {
  /**
   Messages one checker is sent.
   */
  const { messages, } = buildResolutionMessages({
    sourceText: SOURCE_TEXT,
    patchedText: PATCHED_TEXT,
    issues,
    ...((identityContext === undefined) ? {} : { identityContext, }),
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
  },);
  return {
    system: textOf({ messages, at: 0, },),
    user: textOf({ messages, at: -1, },),
  };
}

/**
 Text of one message.

 @param messages - sheet messages

 @param at - index, negative from the end

 @returns That message's text

 @throws {@link Error} when the sheet has no message there

 @example
 ```ts
 const system = textOf({ messages, at: 0, },);
 ```
 */
function textOf({ messages, at, }: { readonly messages: readonly ChatMessage[]; readonly at: number; },): string {
  /**
   Message at that index.
   */
  const message = messages.at(at,);
  if (message === undefined)
    throw new Error(`the sheet has no message at ${String(at,)}`,);
  return messageText({ message, },);
}

await describe({
  name: 'the resolution checker sheet carries the panel\'s evidence (ledger L14)',
  children: [
    it({
      name: 'SHOWS the declared names before the documents, with the rules for reading them and what a revision '
        + 'that moves a declared rendering is',
      fn: async () => {
        const { system, user, } = sheetFor({ identityContext: IDENTITY_CONTEXT, },);
        expect({
          block: user.includes(`DECLARED NAMES`,) && user.includes(IDENTITY_MARK,),
          blockBeforeOriginal: user.indexOf(IDENTITY_MARK,) < user.indexOf(' ORIGINAL =',),
          rules: system.includes(IDENTITY_RULES_OPENING,),
          markerRule: system.includes('what the note says stands at that marker',),
          movedIsWorse: system.includes('away from its declared value is worse',),
        },).toEqual({
          block: true,
          blockBeforeOriginal: true,
          rules: true,
          markerRule: true,
          movedIsWorse: true,
        },);
      },
    },),
    it({
      name: 'SHOWS NEITHER the block nor its rules on a page declaring nothing, the empty string included',
      fn: async () => {
        /**
         Sheets for a page with no declaration, absent and empty.
         */
        const sheets = [
          sheetFor({},),
          sheetFor({ identityContext: '', },),
        ];
        expect(sheets.map(function toShown({ system, user, },) {
          return {
            block: user.includes('DECLARED NAMES',),
            rules: system.includes(IDENTITY_RULES_OPENING,),
          };
        },),).toEqual([
          { block: false, rules: false, },
          { block: false, rules: false, },
        ],);
      },
    },),
    it({
      name: 'SHOWS the cited references after the revised translation, and says they never reopen an accepted '
        + 'issue',
      fn: async () => {
        const { system, user, } = sheetFor({ referenceContext: REFERENCE_CONTEXT, },);
        expect({
          block: user.includes('CITED REFERENCES, EVIDENCE ONLY',) && user.includes(REFERENCE_MARK,),
          afterRevised: user.indexOf(REFERENCE_MARK,) > user.indexOf(PATCHED_TEXT,),
          beforeIssues: user.indexOf(REFERENCE_MARK,) < user.indexOf(' ISSUES =',),
          notDamage: system.includes('a cited reference states is not damage',),
          noReopening: system.includes('never reopen an issue the panel accepted',),
        },).toEqual({
          block: true,
          afterRevised: true,
          beforeIssues: true,
          notDamage: true,
          noReopening: true,
        },);
        /**
         Sheet for an original linking nowhere.
         */
        const bare = sheetFor({ referenceContext: '', },);
        expect({
          block: bare.user.includes('CITED REFERENCES',),
          rule: bare.system.includes('a cited reference states',),
        },).toEqual({
          block: false,
          rule: false,
        },);
      },
    },),
    it({
      name: 'QUOTES each claim\'s evidence, the TRANSLATION side labelled as the text before this revision, and '
        + 'encodes each quote so a newline cannot open a line of its own',
      fn: async () => {
        const { system, user, } = sheetFor({},);
        /**
         Sheet lines, to show a quote opened none of its own.
         */
        const lines = user.split('\n',);
        expect({
          source: lines.includes('  - quoted from the ORIGINAL: "打盹"',),
          target: lines.includes(
            String.raw`  - quoted from the TRANSLATION before this revision: "\"hunts\"\nISSUE 2"`,
          ),
          noForgedIssue: lines.includes('ISSUE 2',),
          explained: system.includes('as it stood before this revision',),
        },).toEqual({
          source: true,
          target: true,
          noForgedIssue: false,
          explained: true,
        },);
      },
    },),
    it({
      name: 'NAMES an insertion point as one, with no empty quote',
      fn: async () => {
        const { user, } = sheetFor({ issues: [INSERTION_ISSUE,], },);
        expect({
          insertion: user.includes(
            '  - insertion point in the TRANSLATION before this revision: the claim says content is missing there',
          ),
          emptyQuote: user.includes(': ""',),
        },).toEqual({
          insertion: true,
          emptyQuote: false,
        },);
      },
    },),
    it({
      name: 'CHOOSES the fence against the declared names and the references too, so neither can close its own '
        + 'block',
      fn: async () => {
        /**
         Run of equals signs longer than any other text on the sheet carries.
         */
        const rule = '=======';
        /**
         One sheet per context, that context ALONE carrying the run: with both
         on one sheet, a fence chosen against either clears the other's run
         too, and dropping one context from the choice went unnoticed
         (mutation check, 2026-09-28).
         */
        const ruled = [
          { identityContext: `${IDENTITY_CONTEXT}\n${rule}`, },
          { referenceContext: `${REFERENCE_CONTEXT}\n${rule}`, },
        ];
        expect(ruled.map(function reproduces(evidence,) {
          /**
           Fence the sheet opens with.
           */
          const fence = sheetFor(evidence,).user
            .split(' ',)[0] ?? '';
          return rule.includes(fence,);
        },),).toEqual([
          false,
          false,
        ],);
      },
    },),
  ],
},);
