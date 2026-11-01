import {
  type CorpusPin,
  listCorpusPeople,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { BenchmarkEntry, } from '@monochromatic-dev/translation-repair-archive/ts';
import type { bandOf, } from '@monochromatic-dev/translation-repair-insertion/ts';
import { seedRecallEntry, } from './recall-benchmark-entry.ts';

//region Recall benchmark choose
// WHICH ENTRIES THE BENCHMARK RUNS, filled in corpus order so the selection
// is deterministic for a given pin.

/**
 Entries drawn from each size band. Nine entries keeps a run inside a few
 hours at the measured per-entry cost while still covering every band.
 */
export const ENTRIES_PER_BAND = 3;

/**
 Bands in report order.
 */
const BANDS = [
  'small',
  'medium',
  'large',
] as const;

/**
 What the selection settled on.

 @example
 ```ts
 const { chosen, perBand, } = await chooseRecallEntries({ pin, },);
 ```
 */
export type RecallChoice = {
  /**
   Seeded entries chosen, in corpus order.
   */
  readonly chosen: readonly BenchmarkEntry[];

  /**
   How many entries each band contributed, keyed by every band name the type
   holds, so every band has its count (ledger B77).
   */
  readonly perBand: Readonly<Record<ReturnType<typeof bandOf>, number>>;
};

/**
 Chooses seeded entries across the size bands, up to the per-band limit.

 @param pin - corpus clone and commit the entries are read at

 @returns The chosen entries and each band's count

 @example
 ```ts
 const { chosen, perBand, } = await chooseRecallEntries({ pin, },);
 ```
 */
export async function chooseRecallEntries(
  { pin, }: { readonly pin: CorpusPin; },
): Promise<RecallChoice> {
  /**
   Every person id at the pinned commit.
   */
  const people = await listCorpusPeople({ pin, },);

  /**
   Encoder measuring page-source bytes for banding.
   */
  const sizer = new TextEncoder();

  /**
   Seeded entries chosen per band, filled in corpus order so the selection is
   deterministic for a given pin.
   */
  const chosen: BenchmarkEntry[] = [];

  /**
   How many entries each band has contributed so far.
   */
  const perBand: Record<ReturnType<typeof bandOf>, number> = {
    small: 0,
    medium: 0,
    large: 0,
  };
  for (const id of people) {
    if (chosen.length >= (ENTRIES_PER_BAND * BANDS.length))
      break;

    /* oxlint-disable no-await-in-loop -- corpus reads are sequential git shows and this selection runs once at setup */
    /**
     This id's seeding outcome, carrying its band when it is usable.
     */
    const outcome = await seedRecallEntry({
      id,
      sizer,
      pin,
    },);
    /* oxlint-enable no-await-in-loop */
    if (outcome.kind === 'skipped')
      continue;
    if (perBand[outcome.band] >= ENTRIES_PER_BAND)
      continue;
    chosen.push(outcome.entry,);
    perBand[outcome.band] += 1;
  }

  return {
    chosen,
    perBand,
  };
}

//endregion Recall benchmark choose
