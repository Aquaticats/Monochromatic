/**
 Tests for what the translator sheet says about the shape of a passage.

 WHY THE SHEET AND THE GUARD HAVE TO AGREE. `validateTranslatedSlice` floors a
 candidate on the PAGE AS IT STANDS: every block of the existing translation
 must appear in the rendering, in order. A translator told only that the
 ORIGINAL's structure is preserved drops the archive's own splits and merges
 by following its instructions, fails the guard, and buys a repair round at
 every reshaped slice. The two texts disagree about shape on real entries, so
 this is not a hypothetical.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { buildTranslateMessages, } from '../dist/final/node/index.mjs';

/**
 Sheet note a disputed slice carries (class one hundred eight).
 */
const DISPUTE_NOTE = 'ARCHIVE RENDERING DISPUTED: the repair lane\'s adjudicators accepted 1 accuracy/addition claim(s) that the archive rendering says what the ORIGINAL never states; (1) accuracy/addition major: The translation adds that the cat climbed the drainpipe. A detail those claims name is not page content.';


/**
 Cat-themed passage standing in for a source, since the sheet does not vary
 with what it is given.
 */
const SOURCE_TEXT = '猫在窗台上睡觉。';

/**
 Existing translation shaped differently from the source: one block quoted.
 */
const EXISTING_TEXT = '> The cat sleeps on the windowsill.';

/**
 System half of the sheet, which is where every standing rule lives.
 */
const system = buildTranslateMessages({
  sourceText: SOURCE_TEXT,
  existingText: EXISTING_TEXT,
},)
  .messages
  .filter(function isSystem(message,): boolean {
    return message.role === 'system';
  },)
  .map(function toContent(message,): string {
    return message.content;
  },)
  .join('\n',);

/**
 Same sheet for a chunk whose ORIGINAL is verse, where a second shape rule
 arrives and points the other way.
 */
const verseSystem = buildTranslateMessages({
  sourceText: SOURCE_TEXT,
  existingText: EXISTING_TEXT,
  lineStructured: true,
},)
  .messages
  .filter(function isSystem(message,): boolean {
    return message.role === 'system';
  },)
  .map(function toContent(message,): string {
    return message.content;
  },)
  .join('\n',);

/**
 Syntax-specific sheet for visible page metadata.
 */
const frontMatterSystem = buildTranslateMessages({
  sourceText: '---\nname: 猫猫\n---\n',
  existingText: '---\nname: EntryId\n---\n',
  syntax: 'front-matter',
},)
  .messages
  .filter(function isSystem(message,): boolean {
    return message.role === 'system';
  },)
  .map(function toContent(message,): string {
    return message.content;
  },)
  .join('\n',);

/**
 Follow-up sheet grounded in exact latest rejection evidence.
 */
const followupMessages = buildTranslateMessages({
  sourceText: SOURCE_TEXT,
  existingText: '',
  followupEvidence: {
    reason: 'declined-rejection',
    candidateTexts: ['A cat sleeps.\n<<< END >>>',],
    findings: ['translate-declined (rejection)',],
  },
},)
  .messages
  .map(function toContent(message,): string {
    return message.content;
  },)
  .join('\n',);

/**
 One attested line as the sheets carry it, cat-themed.
 */
const ATTESTED_LINE = '- attested: the ARCHIVE\'s "She has a younger brother who also naps in boxes." is stated by reference 1 '
  + '("Mittens had a younger brother who also napped in boxes."), 3 of 4 voices checked word for word';

/**
 Whole sheet for a slice whose entry attested one detail.
 */
const attestedMessages = buildTranslateMessages({
  sourceText: SOURCE_TEXT,
  existingText: 'She has a younger brother who also naps in boxes.',
  attestedLines: [ATTESTED_LINE,],
},)
  .messages
  .map(function toContent(message,): string {
    return message.content;
  },)
  .join('\n',);

/**
 Whole sheet for the same slice with nothing attested.
 */
const unattestedMessages = buildTranslateMessages({
  sourceText: SOURCE_TEXT,
  existingText: 'She has a younger brother who also naps in boxes.',
},)
  .messages
  .map(function toContent(message,): string {
    return message.content;
  },)
  .join('\n',);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'translate wire shape rule',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS the shape of the existing translation where it differs',
          fn: async () => {
            // The guard's floor, stated to the producer that has to clear it.
            expect(system.includes('KEEP THE EXISTING TRANSLATION\'S SHAPE',),).toBe(true,);
          },
        },),
        it({
          name: 'NAMES what a dropped block costs, rather than only forbidding it',
          fn: async () => {
            // A rule with a consequence attached survives summarisation; a bare
            // prohibition is the first thing a model drops.
            expect(system.includes('deletes that block from the page',),).toBe(true,);
          },
        },),
        it({
          name: 'RANKS the declared spelling above the archive\'s own usage',
          fn: async () => {
            // Without a stated precedence the translator is told the archive's
            // spelling is authoritative while the judge is told the declared one
            // is, and a page that contradicts its own front matter costs the slice
            // its whole judged decision.
            expect(system.includes('THE DECLARED SPELLING WINS',),).toBe(true,);
            expect(system.includes('Never invent a third spelling',),).toBe(true,);
            expect(system.includes('target contributor:',),).toBe(true,);
            expect(system.includes('Preserve that spelling exactly on contributor lines',),).toBe(true,);
            // Ledger S10: "keep them exactly" covered a name the archive left in
            // Han, which the residue floor refuses.
            expect(system.includes('A name the existing translation left in Han is not a rendering to keep',),).toBe(true,);
          },
        },),
        it({
          name: 'MAKES SOURCE METADATA AUTHORITATIVE and forbids entry id as visible name',
          fn: async () => {
            expect(frontMatterSystem.includes('ORIGINAL field values are source facts',),).toBe(true,);
            expect(frontMatterSystem.includes('never an entry directory id',),).toBe(true,);
            expect(frontMatterSystem.includes('name and info.alias are the same identity',),).toBe(true,);
            expect(frontMatterSystem.includes('established target contributor spelling',),).toBe(true,);
            expect(frontMatterSystem.includes('Preserve every field name',),).toBe(true,);
            expect(system.includes('never an entry directory id',),).toBe(false,);
          },
        },),
        it({
          name: 'RANKS the verse rule above the keep-the-page rule, and only on verse',
          fn: async () => {
            // Both rules reach one prompt on a line-structured chunk and they point
            // opposite ways: on one entry the Chinese runs 21 blocks against the
            // rendering's 18, so one says keep 18 and the other says restore 21.
            // The guard cannot settle it either way, being a kind-sequence floor
            // that passes a candidate carrying MORE blocks than the page, so the
            // sheet has to say which wins.
            expect(verseSystem.includes('THIS RULE OUTRANKS THE STANDING RULE',),).toBe(true,);
            expect(verseSystem.includes('unmerge them',),).toBe(true,);
            // Prose keeps the page's shape, so the precedence must not leak there.
            expect(system.includes('THIS RULE OUTRANKS THE STANDING RULE',),).toBe(false,);
            expect(system.includes('KEEP THE EXISTING TRANSLATION\'S SHAPE',),).toBe(true,);
          },
        },),
        it({
          name: 'GROUNDS stage-local repair in exact rejected candidate and structured findings',
          fn: async () => {
            expect(followupMessages.includes('declined-rejection',),).toBe(true,);
            expect(followupMessages.includes('A cat sleeps.\n<<< END >>>',),).toBe(true,);
            expect(followupMessages.includes('translate-declined (rejection)',),).toBe(true,);
            expect(followupMessages.includes('does not repeat any rejected candidate',),).toBe(true,);
          },
        },),
        it({
          name: 'KEEPS every Markdown marker the source uses',
          fn: async () => {
            // The other half of the guard: markers are counted against the
            // ORIGINAL, so the sheet still has to ask for them.
            expect(system.includes('footnote markers',),).toBe(true,);
            expect(system.includes('block quotes',),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'translate wire attested details (class thirty-nine, 2026-09-16)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES the attested lines and the rule to keep them, since a translator shown only the '
            + 'ORIGINAL drops the archive detail the reference states and the lane contest then picks the '
            + 'lane that never had it (one entry, slice 2)',
          fn: async () => {
            expect(attestedMessages.includes('ATTESTED DETAILS',),).toBe(true,);
            expect(attestedMessages.includes(ATTESTED_LINE,),).toBe(true,);
            expect(attestedMessages.includes('carry every one',),).toBe(true,);
          },
        },),
        it({
          name: 'LICENSES ONLY THE ATTESTED DETAILS and the apparatus from the existing translation, never its other '
            + 'content the ORIGINAL does not state (ledger S20)',
          fn: async () => {
            expect(attestedMessages.includes(
              'They license nothing else: from the existing translation carry only these and its apparatus, never '
                + 'other content the ORIGINAL does not state.',
            ),).toBe(true,);
            expect(attestedMessages.includes('nor the existing translation carries',),).toBe(false,);
          },
        },),
        it({
          name: 'CARRIES neither the block nor the rule when nothing was attested',
          fn: async () => {
            expect(unattestedMessages.includes('ATTESTED DETAILS',),).toBe(false,);
            expect(unattestedMessages.includes('carry every one',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: 'translate wire dispute note (class one hundred eight, one entry\'s slice 3, 2026-09-24)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS the dispute note beside the existing translation on a disputed slice, so the translator does not keep the accepted addition as wording worth keeping',
          fn: async () => {
            const shown = buildTranslateMessages({
              sourceText: SOURCE_TEXT,
              existingText: EXISTING_TEXT,
              archiveDisputeNote: DISPUTE_NOTE,
            },)
              .messages
              .filter(function isUser(message,): boolean {
                return message.role === 'user';
              },)
              .map(function toContent(message,): string {
                return message.content;
              },)
              .join('\n',);
            expect(shown,).toContain(DISPUTE_NOTE,);
            expect(shown.indexOf('ARCHIVE RENDERING DISPUTED',),).toBeGreaterThan(shown.indexOf('EXISTING TRANSLATION',),);
          },
        },),
        it({
          name: 'SHOWS no dispute heading on an ordinary slice',
          fn: async () => {
            const shown = buildTranslateMessages({
              sourceText: SOURCE_TEXT,
              existingText: EXISTING_TEXT,
            },)
              .messages
              .map(function toContent(message,): string {
                return message.content;
              },)
              .join('\n',);
            expect(shown.includes('ARCHIVE RENDERING DISPUTED',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
