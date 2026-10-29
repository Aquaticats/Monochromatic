/**
 Tests the reference attestation (class thirty-seven, 2026-09-16): the
 focused question, the word-for-word verification of both quotes, the merge
 of several voices into one detail under a quorum, the lines the sheets
 carry, the overlap test, the claim screen that rejects an addition claim on
 an attested detail before the panel, and the stage over a scripted bench.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { IssueClaim, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  CITED_REFERENCE_CANDIDATE_RULE,
  CITED_REFERENCE_RULE,
} from '@monochromatic-dev/translation-repair-slice-reading';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  attestationVerdicts,
  attestCitedReferences,
  type AttestedDetail,
  attestedDetailLines,
  attestedDetailsOverlapping,
  buildReferenceAttestMessages,
  compacted,
  isReferenceAttestWire,
  keptAttestations,
  mergedAttestations,
  quoteIsIn,
  REFERENCE_ATTEST_RESPONSE_FORMAT,
  screenAttestedAdditions,
} from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the stage.
 */
const l = tagged({ tag: 'reference-attest-test', },);

/**
 Four seats, so two voices are half the heard bench.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/**
 Invented original: says the cat has a younger brother, nothing about his habits.
 */
const SOURCE_TEXT = '喵喵来自小镇。\n她有一个弟弟。\n她很聪明。\n';

/**
 Invented archive: adds that the brother also sits by the stove, which the cited
 page says, and adds that she wears a bell, which nothing says.
 */
const ARCHIVE_TEXT = 'Mittens is from a small town.\nShe has a younger brother who also sits by the stove.\nShe wears a bell.\nShe is very clever.\n';

/**
 Reference lines as `citedReferenceBlock` renders them.
 */
const REFERENCE_CONTEXT = '- reference 1 https://cats.example/posts/in-memory-of-mittens ("In memory of Mittens"): Mittens had a younger brother who also sat by the stove. Both loved the sun.\n- reference 2 https://cats.example/about: could not be fetched (error)';

/**
 One well-formed attested item.
 */
const BROTHER_ITEM = {
  archiveQuote: 'She has a younger brother who also sits by the stove.',
  reference: 1,
  referenceQuote: 'Mittens had a younger brother who also sat by the stove.',
} as const;

/**
 The brother detail as the merge produces it.
 */
const BROTHER_DETAIL: AttestedDetail = {
  ...BROTHER_ITEM,
  voices: 2,
  heard: 4,
};

/**
 Claim fixture on the archive side.

 @param category - claimed category

 @param quotedText - archive words the claim quotes

 @returns One claim with a single target span

 @example
 ```ts
 const claim = targetClaim({ category: 'accuracy/addition', quotedText: 'also sits by the stove', },);
 ```
 */
function targetClaim(
  {
    category,
    quotedText,
  }: {
    readonly category: IssueClaim['category'];
    readonly quotedText: string;
  },
): IssueClaim {
  return {
    category,
    severity: 'minor',
    summary: `The translation adds "${quotedText}".`,
    spans: [{
      side: 'target',
      nodeId: 'p1',
      nodeHash: 'sha256-cat',
      startOffset: 0,
      endOffset: quotedText.length,
      quotedText,
    },],
  };
}

/**
 Scripted client answering each seat from a table.

 @param replies - reply per seat id

 @param prompts - sink for every prompt seen

 @returns Client

 @example
 ```ts
 const client = scriptedClient({ replies: { [seat]: { attested: [], }, }, prompts: [], },);
 ```
 */
function scriptedClient(
  {
    replies,
    prompts,
  }: {
    readonly replies: Readonly<Record<string, unknown>>;
    readonly prompts: string[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      prompts.push(JSON.stringify(request.messages,),);
      /**
       Scripted reply for this seat.
       */
      const value = replies[request.modelId] ?? { attested: [], };
      if (!request.validate(value,))
        throw new Error('scripted reply failed validator',);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas not used',);
    },
  };
}

await describe({
  name: 'reference attestation (class thirty-seven)',
  children: [
    it({
      name: 'ASKS the one question with the three fenced texts and reads a list of quoted items',
      fn: async () => {
        /**
         Request as built.
         */
        const messages = buildReferenceAttestMessages({
          sourceText: SOURCE_TEXT,
          archiveText: ARCHIVE_TEXT,
          referenceContext: REFERENCE_CONTEXT,
        },);
        expect(messages[0]?.content,).toContain('character for character',);
        expect(messages[1]?.content,).toContain('ARCHIVE RENDERING',);
        expect(messages[1]?.content,).toContain(REFERENCE_CONTEXT,);
        expect(isReferenceAttestWire({ attested: [BROTHER_ITEM,], },),).toBe(true,);
        expect(isReferenceAttestWire({ attested: [], },),).toBe(true,);
        expect(isReferenceAttestWire({ attested: [{ archiveQuote: 'a', reference: '1', referenceQuote: 'b', },], },),).toBe(false,);
        expect(isReferenceAttestWire({ attested: 'none', },),).toBe(false,);
        expect(REFERENCE_ATTEST_RESPONSE_FORMAT.json_schema.name,).toBe('reference_attest',);
      },
    },),
    it({
      name: 'REFUSES an attestation reply that is not a record, and an item that is not a record or whose quote is not a string, while the well-formed shapes are accepted',
      fn: async () => {
        expect(isReferenceAttestWire({ attested: [BROTHER_ITEM,], },),).toBe(true,);
        expect(isReferenceAttestWire(null,),).toBe(false,);
        expect(isReferenceAttestWire({ attested: [42,], },),).toBe(false,);
        expect(isReferenceAttestWire({ attested: [{ archiveQuote: 'a', reference: 1, referenceQuote: 'b', },], },),).toBe(true,);
        expect(isReferenceAttestWire({ attested: [{ archiveQuote: 1, reference: 1, referenceQuote: 'b', },], },),).toBe(false,);
        expect(isReferenceAttestWire({ attested: [{ archiveQuote: 'a', reference: 1, referenceQuote: 1, },], },),).toBe(false,);
      },
    },),
    it({
      name: 'VERIFIES both quotes word for word, whitespace ignored, and discards an item whose quote is not found',
      fn: async () => {
        expect(quoteIsIn({ quote: 'younger  brother who', text: 'a younger brother who also', },),).toBe(true,);
        expect(quoteIsIn({ quote: '', text: 'anything', },),).toBe(false,);
        expect(compacted({ text: '喵喵 的弟弟也睡 box。', },),).toBe('喵喵的弟弟也睡box。',);
        // One run: a voice spaced the reference's Chinese around its Latin tokens.
        expect(quoteIsIn({ quote: '喵喵 的弟弟也睡 box 。', text: '……喵喵的弟弟也睡box。……', },),).toBe(true,);
        /**
         Three items: one true, one paraphrased, one pointing at the unfetched
         page, kept as the stage keeps them, from every item's verdict.
         */
        const verified = keptAttestations({
          verdicts: attestationVerdicts({
            modelId: ROSTER[0],
            items: [
              BROTHER_ITEM,
              {
                archiveQuote: 'She has a little brother who sits by the stove too.',
                reference: 1,
                referenceQuote: BROTHER_ITEM.referenceQuote,
              },
              {
                archiveQuote: 'She wears a bell.',
                reference: 2,
                referenceQuote: 'She wore a bell.',
              },
            ],
            archiveText: ARCHIVE_TEXT,
            referenceContext: REFERENCE_CONTEXT,
          },),
        },);
        expect(verified,).toHaveLength(1,);
        expect(verified[0]?.item.archiveQuote,).toBe(BROTHER_ITEM.archiveQuote,);
      },
    },),
    it({
      name: 'VERIFIES NO QUOTE THAT SHOWS A READER NOTHING, though the text holds the same invisible characters: a '
        + 'zero-width space, a Hangul filler, or both among spaces quote no words, so an item quoting them on both '
        + 'sides is discarded',
      fn: async () => {
        /**
         Archive carrying each invisible character inside the brother sentence.
         */
        const archiveText = 'Mittens is from a small town.\nShe has a younger\u{200B}\u{3164}brother who also sits by the stove.\n';
        /**
         Reference line whose page carries the same characters.
         */
        const referenceContext = '- reference 1 https://cats.example/posts/in-memory-of-mittens ("In memory of Mittens"): '
          + 'Mittens had a younger\u{200B}\u{3164}brother who also sat by the stove.';
        /**
         Quotes that show a reader nothing.
         */
        const invisible = [
          '\u{200B}',
          '\u{3164}',
          ' \u{200B}\u{3164} ',
        ];
        expect(invisible.map(function found(quote,): boolean {
          return quoteIsIn({ quote, text: archiveText, },);
        },),).toEqual([
          false,
          false,
          false,
        ],);
        expect(keptAttestations({
          verdicts: attestationVerdicts({
            modelId: ROSTER[0],
            items: invisible.map(function itemQuoting(quote,) {
              return {
                archiveQuote: quote,
                reference: 1,
                referenceQuote: quote,
              };
            },),
            archiveText,
            referenceContext,
          },),
        },),).toEqual([],);
      },
    },),
    it({
      name: 'VERIFIES A QUOTE WHOSE MARKS DIFFER FROM THE TEXT\'S (ledger B24): a straight apostrophe against the '
        + 'archive\'s curly one, English quotes against the reference\'s corner brackets',
      fn: async () => {
        expect(quoteIsIn({ quote: 'the cat\'s "bell"', text: 'She found the cat’s “bell” by the door.', },),).toBe(true,);
        expect(quoteIsIn({ quote: '她说“铃铛”', text: '她说「铃铛」在门边。', },),).toBe(true,);
        expect(quoteIsIn({ quote: 'the dog\'s "bell"', text: 'She found the cat’s “bell” by the door.', },),).toBe(false,);
      },
    },),
    it({
      name: 'READS THE REFERENCE QUOTE IN ONE REFERENCE LINE AT A TIME (ledger B25): an item naming a reference that '
        + 'does not state it takes the number of the one that does, one naming a reference that states it keeps its '
        + 'number, and a quote running from one line into the next is found in neither',
      fn: async () => {
        /**
         Two fetched pages; only the second has the stove.
         */
        const twoPages = '- reference 1 https://cats.example/a ("A"): Mittens had a younger brother.\n'
          + '- reference 2 https://cats.example/b ("B"): Mittens had a younger brother who also sat by the stove.';
        /**
         Items as one voice answered them, kept as the stage keeps them.
         */
        const verified = keptAttestations({
          verdicts: attestationVerdicts({
            modelId: ROSTER[0],
            items: [
              {
                ...BROTHER_ITEM,
                reference: 1,
              },
              {
                archiveQuote: 'She has a younger brother',
                reference: 2,
                referenceQuote: 'Mittens had a younger brother',
              },
              {
                archiveQuote: 'She wears a bell.',
                reference: 1,
                referenceQuote: 'a younger brother. - reference 2',
              },
            ],
            archiveText: ARCHIVE_TEXT,
            referenceContext: twoPages,
          },),
        },);
        expect(verified.map(function numberOf(entry,): number {
          return entry.item
            .reference;
        },),).toEqual([2, 2,],);
        expect(verified.map(function quoteOf(entry,): string {
          return entry.item
            .archiveQuote;
        },),).toEqual([BROTHER_ITEM.archiveQuote, 'She has a younger brother',],);
      },
    },),
    it({
      name: 'MERGES overlapping quotes from distinct voices into one detail and drops one short of the quorum',
      fn: async () => {
        /**
         Two voices on the brother, quoting different spans of the same sentence,
         one voice alone on the bell.
         */
        const details = mergedAttestations({
          verified: [
            {
              modelId: ROSTER[0],
              item: BROTHER_ITEM,
            },
            {
              modelId: ROSTER[1],
              item: {
                ...BROTHER_ITEM,
                archiveQuote: 'younger brother who also sits by the stove',
              },
            },
            {
              modelId: ROSTER[0],
              item: {
                archiveQuote: 'She wears a bell.',
                reference: 1,
                referenceQuote: 'Both loved the sun.',
              },
            },
          ],
          archiveText: ARCHIVE_TEXT,
          heard: 4,
          needed: 2,
        },);
        expect(details,).toHaveLength(1,);
        expect(details[0]?.archiveQuote,).toBe(BROTHER_ITEM.archiveQuote,);
        expect(details[0]?.voices,).toBe(2,);
        expect(details[0]?.heard,).toBe(4,);
        /**
         Same voice twice is one voice.
         */
        const oneVoice = mergedAttestations({
          verified: [
            {
              modelId: ROSTER[0],
              item: BROTHER_ITEM,
            },
            {
              modelId: ROSTER[0],
              item: BROTHER_ITEM,
            },
          ],
          archiveText: ARCHIVE_TEXT,
          heard: 4,
          needed: 2,
        },);
        expect(oneVoice,).toHaveLength(0,);
      },
    },),
    it({
      name: 'MERGES a later, wider quote over an earlier, narrower one from a different voice, keeping whichever placement covers more of the detail',
      fn: async () => {
        const widerSecond = mergedAttestations({
          verified: [
            {
              modelId: ROSTER[0],
              item: {
                ...BROTHER_ITEM,
                archiveQuote: 'town.\nShe has a younger brother',
              },
            },
            {
              modelId: ROSTER[1],
              item: BROTHER_ITEM,
            },
          ],
          archiveText: ARCHIVE_TEXT,
          heard: 4,
          needed: 2,
        },);
        expect(widerSecond,).toEqual([BROTHER_DETAIL,],);
      },
    },),
    it({
      name: 'THROWS on an item whose archive quote is absent from the archive or empty once compacted, which verification never keeps, rather than dropping it unseen; the verified item beside it merges',
      fn: async () => {
        /**
         Merges one item alone against the archive, one voice being enough.

         @param archiveQuote - archive words the item claims

         @returns Details the merge kept

         @example
         ```ts
         const details = mergeAlone({ archiveQuote: 'She wears a bell.', },);
         ```
         */
        function mergeAlone({ archiveQuote, }: { readonly archiveQuote: string; },): readonly unknown[] {
          return mergedAttestations({
            verified: [{
              modelId: ROSTER[0],
              item: {
                ...BROTHER_ITEM,
                archiveQuote,
              },
            },],
            archiveText: ARCHIVE_TEXT,
            heard: 4,
            needed: 1,
          },);
        }
        expect(mergeAlone({ archiveQuote: BROTHER_ITEM.archiveQuote, },),).toHaveLength(1,);
        expect(function mergesAbsent() {
          mergeAlone({ archiveQuote: 'a feather toy on a string', },);
        },).toThrow('whose archive quote shows a reader nothing or is absent from the archive text',);
        expect(function mergesEmpty() {
          mergeAlone({ archiveQuote: ' \n ', },);
        },).toThrow('whose archive quote shows a reader nothing or is absent from the archive text',);
      },
    },),
    it({
      name: 'THROWS on an item whose archive quote shows a reader nothing though the archive holds those '
        + 'characters, which verification never keeps, rather than placing a detail of no words',
      fn: async () => {
        /**
         Archive carrying a zero-width space and a Hangul filler side by side.
         */
        const archiveText = 'Mittens is from a small town.\nShe has a younger\u{200B}\u{3164}brother.\n';
        expect([
          '\u{200B}',
          '\u{3164}',
        ].map(function refusalOf(archiveQuote,): string {
          return String(caught(function merge(): unknown {
            return mergedAttestations({
              verified: [{
                modelId: ROSTER[0],
                item: {
                  ...BROTHER_ITEM,
                  archiveQuote,
                },
              },],
              archiveText,
              heard: 4,
              needed: 1,
            },);
          },),);
        },),).toEqual([
          `Error: unreachable: an attestation from ${ROSTER[0]} citing reference 1, whose archive quote shows a `
            + 'reader nothing or is absent from the archive text it is merged against, was never verified against '
            + 'that text',
          `Error: unreachable: an attestation from ${ROSTER[0]} citing reference 1, whose archive quote shows a `
            + 'reader nothing or is absent from the archive text it is merged against, was never verified against '
            + 'that text',
        ],);
      },
    },),
    it({
      name: 'RENDERS one attested line per detail that the rules name',
      fn: async () => {
        /**
         Lines for the sheets.
         */
        const lines = attestedDetailLines({ details: [BROTHER_DETAIL,], },);
        expect(lines,).toHaveLength(1,);
        expect(lines[0],).toContain('- attested: the ARCHIVE\'s "She has a younger brother who also sits by the stove." is stated by reference 1',);
        expect(lines[0],).toContain('2 of 4 voices',);
        expect(CITED_REFERENCE_RULE,).toContain('attested',);
        expect(CITED_REFERENCE_CANDIDATE_RULE,).toContain('attested',);
      },
    },),
    it({
      name: 'FINDS the detail a claim quote overlaps, by containment or by intersecting spans, and none otherwise',
      fn: async () => {
        expect(attestedDetailsOverlapping({
          quote: 'who also sits by the stove',
          text: ARCHIVE_TEXT,
          details: [BROTHER_DETAIL,],
        },),).toHaveLength(1,);
        expect(attestedDetailsOverlapping({
          quote: 'younger brother who also sits by the stove.\nShe wears',
          text: ARCHIVE_TEXT,
          details: [BROTHER_DETAIL,],
        },),).toHaveLength(1,);
        expect(attestedDetailsOverlapping({
          quote: 'She wears a bell.',
          text: ARCHIVE_TEXT,
          details: [BROTHER_DETAIL,],
        },),).toHaveLength(0,);
        expect(attestedDetailsOverlapping({
          quote: '',
          text: ARCHIVE_TEXT,
          details: [BROTHER_DETAIL,],
        },),).toHaveLength(0,);
      },
    },),
    it({
      name: 'FINDS NO DETAIL for a claim quote that shows a reader nothing, though the detail\'s quote and the text '
        + 'hold the same zero-width space and Hangul filler',
      fn: async () => {
        /**
         Detail whose archive quote carries both invisible characters.
         */
        const padded: AttestedDetail = {
          ...BROTHER_DETAIL,
          archiveQuote: 'She has a younger\u{200B}\u{3164}brother',
        };
        expect([
          '\u{200B}',
          '\u{3164}',
          ' \u{200B}\u{3164} ',
        ].map(function overlapped(quote,) {
          return attestedDetailsOverlapping({
            quote,
            text: 'Mittens is from a small town.\nShe has a younger\u{200B}\u{3164}brother.\n',
            details: [padded,],
          },);
        },),).toEqual([
          [],
          [],
          [],
        ],);
      },
    },),
    it({
      name: 'FINDS the overlap a quote contains, REFUSES one whose quote is nowhere in the text, and REFUSES one whose detail quote is nowhere in that text either, even though the claim quote is there',
      fn: async () => {
        expect(attestedDetailsOverlapping({
          quote: 'who also sits by the stove',
          text: ARCHIVE_TEXT,
          details: [BROTHER_DETAIL,],
        },),).toEqual([BROTHER_DETAIL,],);
        expect(attestedDetailsOverlapping({
          quote: 'a feather toy on a string',
          text: ARCHIVE_TEXT,
          details: [BROTHER_DETAIL,],
        },),).toEqual([],);
        expect(attestedDetailsOverlapping({
          quote: 'She wears a bell',
          text: 'She wears a bell by the door. Nothing else is said here.',
          details: [BROTHER_DETAIL,],
        },),).toEqual([],);
      },
    },),
    it({
      name: 'REJECTS an addition claim on an attested detail before the panel and leaves every other claim alone',
      fn: async () => {
        /**
         Three claims: the attested brother, the unattested bell, a
         mistranslation on the brother.
         */
        const screened = screenAttestedAdditions({
          claims: [
            targetClaim({
              category: 'accuracy/addition',
              quotedText: 'who also sits by the stove',
            },),
            targetClaim({
              category: 'accuracy/addition',
              quotedText: 'She wears a bell.',
            },),
            targetClaim({
              category: 'accuracy/mistranslation',
              quotedText: 'younger brother',
            },),
          ],
          attested: [BROTHER_DETAIL,],
          targetText: ARCHIVE_TEXT,
        },);
        expect(screened.claims,).toHaveLength(2,);
        expect(screened.claims
          .map(function quoteOf(claim,): string {
            return claim.spans[0]?.quotedText ?? '';
          },),).toEqual([
            'She wears a bell.',
            'younger brother',
          ],);
        expect(screened.issues,).toHaveLength(1,);
        expect(screened.issues[0]?.status,).toBe('rejected',);
        expect(screened.issues[0]?.issueId.startsWith('adjudicated/',),).toBe(true,);
        expect(screened.issues[0]?.claims[0]?.claim.category,).toBe('accuracy/addition',);
        expect(screened.findings[0],).toContain('reference-attested',);
        expect(screened.findings[0],).toContain('reference 1',);
        /**
         Nothing attested: the screen is a pass-through.
         */
        const untouched = screenAttestedAdditions({
          claims: [targetClaim({
            category: 'accuracy/addition',
            quotedText: 'who also sits by the stove',
          },),],
          attested: [],
          targetText: ARCHIVE_TEXT,
        },);
        expect(untouched.claims,).toHaveLength(1,);
        expect(untouched.issues,).toHaveLength(0,);
      },
    },),
    it({
      name: 'ATTESTS over a scripted bench: two of four voices verify the brother, one voice alone invents a bell, the lines say so',
      fn: async () => {
        /**
         Prompts seen.
         */
        const prompts: string[] = [];
        /**
         The stage over four seats.
         */
        const attestation = await attestCitedReferences({
          client: scriptedClient({
            prompts,
            replies: {
              [ROSTER[0]]: { attested: [BROTHER_ITEM,], },
              [ROSTER[1]]: { attested: [
                {
                  ...BROTHER_ITEM,
                  archiveQuote: 'a younger brother who also sits by the stove',
                },
                {
                  archiveQuote: 'She wears a bell.',
                  reference: 1,
                  referenceQuote: 'wore a bell',
                },
              ], },
              [ROSTER[2]]: { attested: [{
                archiveQuote: 'She wears a bell.',
                reference: 1,
                referenceQuote: 'Both loved the sun.',
              },], },
            },
          },),
          modelIds: ROSTER,
          // WHOLE BENCH, PINNED (ledger X2): this case scripts seats by name, and
          // the window picks its seats by a hash of the prompt, so rewording the
          // fixture would change which scripted seat is heard.
          fanOut: 'whole-bench',
          sourceText: SOURCE_TEXT,
          archiveText: ARCHIVE_TEXT,
          referenceContext: REFERENCE_CONTEXT,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(prompts.length >= 2,).toBe(true,);
        expect(prompts[0],).toContain('ARCHIVE RENDERING',);
        expect(attestation.details,).toHaveLength(1,);
        expect(attestation.details[0]?.archiveQuote,).toBe(BROTHER_ITEM.archiveQuote,);
        expect(attestation.details[0]?.voices,).toBe(2,);
        expect(attestation.lines[0],).toContain('is stated by reference 1',);
        expect(attestation.findings.join('\n',),).toContain('discarded 1 of 4 items',);
      },
    },),
    it({
      name: 'VERIFIES an archive quote copied across a note between translators in the archive, because the sheet shows the archive without the note and the quote is read in the text as shown',
      fn: async () => {
        /**
         Archive whose brother sentence is cut by a note between translators.
         */
        const notedArchive = 'Mittens is from a small town.\nShe has a younger brother{/* 这句不太对劲 */}\nwho also sits by the stove.\nShe wears a bell.\nShe is very clever.\n';
        /**
         The same item as the sheet showed it, quoted across the cut.
         */
        const acrossTheNote = {
          ...BROTHER_ITEM,
          archiveQuote: 'She has a younger brother\nwho also sits by the stove.',
        };
        /**
         Prompts seen.
         */
        const prompts: string[] = [];
        const attestation = await attestCitedReferences({
          client: scriptedClient({
            prompts,
            replies: {
              [ROSTER[0]]: { attested: [acrossTheNote,], },
              [ROSTER[1]]: { attested: [acrossTheNote,], },
            },
          },),
          modelIds: ROSTER,
          fanOut: 'whole-bench',
          sourceText: SOURCE_TEXT,
          archiveText: notedArchive,
          referenceContext: REFERENCE_CONTEXT,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(prompts.some(function showsNote(prompt,): boolean {
          return prompt.includes('这句不太对劲',);
        },),).toBe(false,);
        expect(attestation.details,).toHaveLength(1,);
        expect(attestation.details[0]?.voices,).toBe(2,);
      },
    },),
    it({
      name: 'REJECTS an addition claim that overlaps an attested detail quoted across a note between translators, '
        + 'the claim extending past the detail so that neither quote holds the other',
      fn: async () => {
        /**
         Archive text the claim and the detail are placed in, a note inside the detail.
         */
        const notedArchive = 'Mittens is from a small town.\nShe has a younger brother{/* 这句不太对劲 */}\nwho also sits by the stove.\nShe wears a bell.\n';
        /**
         Detail as the attestation produced it from the text as shown.
         */
        const acrossTheNote: AttestedDetail = {
          ...BROTHER_DETAIL,
          archiveQuote: 'She has a younger brother\nwho also sits by the stove.',
        };
        const screened = screenAttestedAdditions({
          claims: [
            targetClaim({
              category: 'accuracy/addition',
              quotedText: 'by the stove.\nShe wears a bell.',
            },),
          ],
          attested: [acrossTheNote,],
          targetText: notedArchive,
          original: SOURCE_TEXT,
        },);
        expect(screened.claims,).toHaveLength(0,);
        expect(screened.issues,).toHaveLength(1,);
      },
    },),
  ],
},);
