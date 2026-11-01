import {
  type CorpusPin,
  isMissingCorpusObject,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { SizedEntry, } from '@monochromatic-dev/translation-repair-insertion/ts';
import type { CorpusPair, } from './pass-entry-contract.ts';

//region Pass eligibility
// Which entries a pass may work on, and which it cannot, said out loud.
//
// SPLIT OUT OF THE PASS so the one decision that shrinks a run's population is
// testable against a throwaway clone. Until this module the pass stepped past
// EVERY corpus read failure as if it were the expected missing side of an
// incomplete pair, wrote no line about it, and ranked its bands over whatever
// was left: a clone that had gone away read as a corpus with no entries.

/**
 One entry a pass could not pair, with the side that was absent.

 @example
 ```ts
 const gap: IncompleteEntry = { id: 'whiskers', side: 'target', detail: 'a41fc60:people/whiskers/page.en.md (missing-object)', };
 ```
 */
export type IncompleteEntry = {
  /**
   Corpus entry.
   */
  readonly id: string;

  /**
   Which page was absent at the pin.
   */
  readonly side: 'source' | 'target';

  /**
   What the read said, in its own words, which name the path and never its
   content.
   */
  readonly detail: string;
};

/**
 What a pass found when it walked its entries.

 @example
 ```ts
 const { eligible, settled, incomplete, } = await collectEligiblePairs({ ids, done, pin, },);
 ```
 */
export type PassEligibility = {
  /**
   Complete unsettled pairs, in walk order.
   */
  readonly eligible: readonly CorpusPair[];

  /**
   Already-settled entries reduced to their sizes, which band ranking needs.
   */
  readonly settled: readonly SizedEntry[];

  /**
   Entries with one side absent at the pin, which reach no pool.
   */
  readonly incomplete: readonly IncompleteEntry[];

  /**
   Entries whose original and English pages the pin both holds, pending or
   settled, in walk order: the population a pass's closing line counts over.
   A settled entry whose English page the pin lacks is sized for ranking and
   is no pair here.
   */
  readonly paired: readonly string[];
};

/**
 Walks the entries a pass was asked about and sorts them into pairs it can
 work on, pairs already settled, and entries missing a side.

 ONLY A MISSING OBJECT IS STEPPED PAST. Any other read failure, an unreadable
 clone, an unknown commit, a spawn failure, an oversized page, propagates:
 those are faults in the run's setup, not facts about the corpus, and a pass
 that swallowed them would report a smaller corpus than exists.

 @param ids - entries to walk, in the order they are walked

 @param done - entries a previous run already settled

 @param pin - clone and commit to read at

 @returns Pairs, settled sizes, the entries that could not be paired, and
 every entry the pin holds as a complete pair

 @throws {@link CorpusReadError} for any read failure other than a missing
 object at the pin

 @example
 ```ts
 const { eligible, incomplete, } = await collectEligiblePairs({ ids: people, done, pin, },);
 ```
 */
export async function collectEligiblePairs(
  {
    ids,
    done,
    pin,
  }: {
    readonly ids: readonly string[];
    readonly done: ReadonlySet<string>;
    readonly pin: CorpusPin;
  },
): Promise<PassEligibility> {
  /**
   Encoder measuring page-source byte size once per entry.
   */
  const sizer = new TextEncoder();

  /**
   Complete unsettled pairs.
   */
  const eligible: CorpusPair[] = [];

  /**
   Already-settled entries reduced to their sizes.
   */
  const settled: SizedEntry[] = [];

  /**
   Entries with a side absent at the pin.
   */
  const incomplete: IncompleteEntry[] = [];

  /**
   Entries with both sides at the pin, pending or settled.
   */
  const paired: string[] = [];

  /* oxlint-disable no-await-in-loop -- corpus reads are sequential git shows; the list is small and this runs once at setup */
  for (const id of ids) {
    /**
     Original zh page text for this entry, or the reason it is absent.
     */
    const source = await readSide({
      pin,
      id,
      side: 'source',
    },);
    if (source.kind === 'absent') {
      incomplete.push(source.gap,);
      continue;
    }

    /**
     Page-source size deciding this entry's band.
     */
    const sourceBytes = sizer.encode(source.text,)
      .length;
    if (done.has(id,)) {
      settled.push({
        id,
        sourceBytes,
      },);
      // A FINISHED ENTRY IS A COMPLETE PAIR ONLY WHERE THE PIN HOLDS ITS
      // ENGLISH PAGE TOO, read for that alone: its source size in `settled` is
      // all the ranking needs, and no line names its absence, as none did
      // before.
      if ((await readSide({
        pin,
        id,
        side: 'target',
      },)).kind === 'read')
        paired.push(id,);
      continue;
    }

    /**
     Translated en page text for this entry, or the reason it is absent.
     */
    const target = await readSide({
      pin,
      id,
      side: 'target',
    },);
    if (target.kind === 'absent') {
      incomplete.push(target.gap,);
      continue;
    }
    eligible.push({
      id,
      sourceText: source.text,
      targetText: target.text,
    },);
    paired.push(id,);
  }
  /* oxlint-enable no-await-in-loop */
  return {
    eligible,
    settled,
    incomplete,
    paired,
  };
}

/**
 One side of an entry as read, or the fact that it is absent at the pin.
 */
type SideRead = {
  /**
   The page is there.
   */
  readonly kind: 'read';

  /**
   Its text.
   */
  readonly text: string;
} | {
  /**
   The page is not at the pin.
   */
  readonly kind: 'absent';

  /**
   Entry and side that were absent.
   */
  readonly gap: IncompleteEntry;
};

/**
 Reads one side of an entry, naming absence rather than throwing it.

 @param pin - clone and commit to read at

 @param id - corpus entry

 @param side - which page

 @returns Text, or the named absence

 @throws {@link CorpusReadError} for any failure other than a missing object

 @example
 ```ts
 const source = await readSide({ pin, id, side: 'source', },);
 ```
 */
async function readSide(
  {
    pin,
    id,
    side,
  }: {
    readonly pin: CorpusPin;
    readonly id: string;
    readonly side: IncompleteEntry['side'];
  },
): Promise<SideRead> {
  /**
   Page this side is kept at.
   */
  const relPath = (side === 'source') ? `people/${id}/page.md` : `people/${id}/page.en.md`;
  try {
    return {
      kind: 'read',
      text: await readCorpusFile({
        pin,
        relPath,
      },),
    };
  }
  catch (error) {
    // A MISSING SIDE IS AN ORDINARY STATE of this corpus (`tdor` at the pinned
    // commit) and is named rather than swallowed; anything else propagates.
    if (!isMissingCorpusObject(error,))
      throw error;
    return {
      kind: 'absent',
      gap: {
        id,
        side,
        detail: error.message,
      },
    };
  }
}

//endregion Pass eligibility
