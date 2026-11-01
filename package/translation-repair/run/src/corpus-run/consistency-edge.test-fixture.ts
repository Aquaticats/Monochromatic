/**
 Small pages and scripted ballots for the cases that probe the consistency
 pass at the edges of a quote: markup around it, a second place in its slice,
 a comment inside the repeated wording, and a stored record that was altered.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure';
import type { ArtifactConsistency, } from '@monochromatic-dev/translation-repair-artifact';
import {
  ballotText,
  firstPlaceWins,
  otherSheetBehaviour,
  type SeatBehaviour,
  type SeatScript,
} from './consistency-page.test-fixture.ts';
import {
  REFRAIN_QUOTES,
  SENTENCE_QUOTES,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-quotes.test-fixture.ts';
import {
  isRefrainSheet,
  sheetKindOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-sheet-reading.test-fixture.ts';

//region Consistency edge fixture

/**
 How many places of the cat page's refrain stand before the fourth.
 */
const REFRAIN_PLACES_BEFORE_THE_FOURTH = 3;

/**
 A page of two sections that repeat one sentence, the second inside a longer one.

 @returns Preparation of the page

 @example
 ```ts
 const prepared = sentencePage();
 ```
 */
export function sentencePage(): PreparedDocumentPair {
  return prepareDocumentPair({
    sourceText: '## 一\n\n窗台上的阳光让小猫睡着了。\n\n## 二\n\n傍晚的时候，窗台上的阳光让小猫睡着了，小鱼干也笑了。\n',
    targetText: '## One\n\nThe sunny window put the kitten to sleep.\n\n## Two\n\nIn the evening, the sunlight on the sill '
      + 'lulled the kitten to sleep, and the fish snacks smiled too.\n',
  },);
}

/**
 A page whose two paragraphs repeat one sentence and share one slice.

 @returns Preparation of the page

 @example
 ```ts
 const prepared = sharedSlicePage();
 ```
 */
export function sharedSlicePage(): PreparedDocumentPair {
  return prepareDocumentPair({
    sourceText: '窗台上的阳光让小猫睡着了。\n\n傍晚的时候，窗台上的阳光让小猫睡着了，小鱼干也笑了。\n',
    targetText: 'The sunny window put the kitten to sleep.\n\nIn the evening, the sunlight on the sill '
      + 'lulled the kitten to sleep, and the fish snacks smiled too.\n',
  },);
}

/**
 Every seat answers one ballot about a repeated sentence, whatever the sheet.

 @param quotes - what the ballot quotes, a label and a quote each

 @param choice - label the ballot chooses

 @param keep - labels the ballot keeps

 @returns Script for the scripted client

 @example
 ```ts
 const behave = everySeatQuotes({ quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat naps', },], choice: 'OCCURRENCE 1', keep: [], },);
 ```
 */
export function everySeatQuotes(
  {
    quotes,
    choice,
    keep,
  }: {
    readonly quotes: readonly Record<string, string>[];
    readonly choice: string;
    readonly keep: readonly string[];
  },
): SeatScript {
  return function answers({ sheet, }: { readonly sheet: string; },): SeatBehaviour {
    if (sheetKindOf({ sheet, },) !== 'ballot')
      return otherSheetBehaviour({ sheet, },);
    return {
      kind: 'answers',
      text: ballotText({
        quotes,
        choice,
        keep,
      },),
    };
  };
}

/**
 Every seat answers the refrain's sheet with the fourth place already worded
 as the first place's pattern asks, and the sentence's sheet as the first place
 winning.

 @param sheet - the sheet the seat was shown

 @returns Behaviour of a seat that answers it

 @example
 ```ts
 const behaviour = fourthAlreadyFollows({ sheet, },);
 ```
 */
export function fourthAlreadyFollows({ sheet, }: { readonly sheet: string; },): SeatBehaviour {
  return isRefrainSheet({ sheet, },)
    ? {
      kind: 'answers',
      text: ballotText({
        quotes: [
          ...REFRAIN_QUOTES.slice(
            0,
            REFRAIN_PLACES_BEFORE_THE_FOURTH,
          ),
          {
            label: 'OCCURRENCE 4',
            quote: 'So this cat must be a warm little treasure.',
            slot: 'warm',
          },
        ],
        choice: 'OCCURRENCE 1',
        keep: [],
      },),
    }
    : firstPlaceWins({ sheet, },);
}

/**
 Every seat answers the sentence's sheet with a second quote that runs on into
 the refrain's line, and the refrain's sheet as the first place winning.

 @param sheet - the sheet the seat was shown

 @returns Behaviour of a seat that answers it

 @example
 ```ts
 const behaviour = sentenceQuoteSpansTheRefrain({ sheet, },);
 ```
 */
export function sentenceQuoteSpansTheRefrain({ sheet, }: { readonly sheet: string; },): SeatBehaviour {
  return (isRefrainSheet({ sheet, },) || (sheetKindOf({ sheet, },) !== 'ballot'))
    ? firstPlaceWins({ sheet, },)
    : {
      kind: 'answers',
      text: ballotText({
        quotes: [
          ...SENTENCE_QUOTES.slice(
            0,
            1,
          ),
          {
            label: 'OCCURRENCE 2',
            quote: 'the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too. '
              + 'So this cat has to be a sweet little darling',
          },
        ],
        choice: 'OCCURRENCE 1',
        keep: [],
      },),
    };
}

/**
 Every seat answers the refrain's sheet with a second quote that carries the
 subject's name inside the slot's line, and the sentence's sheet as the first
 place winning.

 @param sheet - the sheet the seat was shown

 @returns Behaviour of a seat that answers it

 @example
 ```ts
 const behaviour = slotWithTheName({ sheet, },);
 ```
 */
export function slotWithTheName({ sheet, }: { readonly sheet: string; },): SeatBehaviour {
  return isRefrainSheet({ sheet, },)
    ? {
      kind: 'answers',
      text: ballotText({
        quotes: [
          ...REFRAIN_QUOTES.slice(
            0,
            1,
          ),
          {
            label: 'OCCURRENCE 2',
            quote: 'So this cat is surely Mittens, a lazy little treasure.',
            slot: 'lazy',
          },
          ...REFRAIN_QUOTES.slice(2,),
        ],
        choice: 'OCCURRENCE 1',
        keep: [],
      },),
    }
    : firstPlaceWins({ sheet, },);
}

/**
 The preparation with the text of every block of every slice's original replaced
 by other characters of the same length, so the offsets of a place read other
 words in the block shown than in the whole original.

 @param prepared - preparation of the page

 @returns The same preparation with its blocks' text blanked

 @example
 ```ts
 const blanked = withBlankedBlocks({ prepared: preparedCatPage(), },);
 ```
 */
export function withBlankedBlocks(
  { prepared, }: { readonly prepared: PreparedDocumentPair; },
): PreparedDocumentPair {
  return {
    ...prepared,
    slices: prepared.slices
      .map(function blanked(slice,) {
        return {
          ...slice,
          source: {
            ...slice.source,
            nodes: slice.source
              .nodes
              .map(function wrong(node,) {
                return {
                  ...node,
                  text: '＊'.repeat(node.text
                    .length,),
                };
              },),
          },
        };
      },),
  };
}

/**
 The outcome of every place of every group, group by group.

 @param consistency - the section

 @returns One list of outcomes per group, in page order

 @example
 ```ts
 const outcomes = outcomesByGroup({ consistency, },); // [['chosen', 'replaced',],]
 ```
 */
export function outcomesByGroup(
  { consistency, }: { readonly consistency: ArtifactConsistency; },
): readonly (readonly string[])[] {
  return consistency.groups
    .map(function outcomesOfGroup(group,): readonly string[] {
      return group.occurrences
        .map(function outcomeOf(occurrence,): string {
          return occurrence.outcome;
        },);
    },);
}

/**
 Files a run left in the consistency cache, parsed.

 @param dir - entry cache directory

 @returns Path and parsed content of every stored gather

 @example
 ```ts
 const stored = await storedGathers({ dir, },);
 ```
 */
export async function storedGathers(
  { dir, }: { readonly dir: string; },
): Promise<readonly {
  readonly path: string;
  readonly parsed: Record<string, unknown>
}[]> {
  /**
   File names the pass stored gathers under.
   */
  const names = (await readdir(dir,))
    .filter(function isGather(name,): boolean {
      return name.startsWith('consistency.',) && name.endsWith('.json',);
    },)
    .toSorted();
  return await Promise.all(names.map(async function parsedFile(name,): Promise<{
    readonly path: string;
    readonly parsed: Record<string, unknown>;
  }> {
    /**
     Where the file stands.
     */
    const path = join(
      dir,
      name,
    );
    /**
     Content of the file, which a gather stores as one object.
     */
    const parsed: unknown = JSON.parse(await readFile(
      path,
      'utf8',
    ),);
    if (!isJsonRecord(parsed,))
      throw new Error(`the stored gather ${name} is no object`,);
    return {
      path,
      parsed,
    };
  },),);
}

/**
 Rewrites every stored first-round gather's record with a change a case names.
 The run-off, writer and choice records, which carry a `kind`, are left as stored.

 @param dir - entry cache directory

 @param change - new record from the stored one

 @example
 ```ts
 await alterStored({ dir, change: (record) => ({ ...record, quorum: 0, }), },);
 ```
 */
export async function alterStored(
  {
    dir,
    change,
  }: {
    readonly dir: string;
    readonly change: (record: Record<string, unknown>,) => unknown;
  },
): Promise<void> {
  for (const stored of await storedGathers({ dir, },)) {
    /**
     The record the envelope holds.
     */
    const { record, } = stored.parsed;
    if (!isJsonRecord(record,))
      throw new Error(`the stored gather ${stored.path} holds no record`,);
    if (record.kind !== undefined)
      continue;
    /* oxlint-disable no-await-in-loop -- a handful of files rewritten one after another */
    await writeFile(
      stored.path,
      JSON.stringify({
        ...stored.parsed,
        record: change(record,),
      },),
      'utf8',
    );
    /* oxlint-enable no-await-in-loop */
  }
}

//endregion Consistency edge fixture
