import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  type CorpusPair,
  type EntryOutcome,
  tallyCaughtEntry,
} from '@monochromatic-dev/translation-repair-slice-cache/ts';
import {
  type EntryProgress,
  shipInterimOnStop,
} from './pass-entry-interim.ts';
import type {
  EntryFlight,
  PassStop,
} from './pass-stop.ts';

//region Pass entry caught
// WHAT AN ENTRY DOES WITH THE ERROR THAT ENDED IT, split from `pass-entry.ts` at its line budget: ship what it has, then
// file the stop. The order is the point: the page is shipped before the tally, so the tally of a stop that ends the run
// is printed after the page the entry left (B1220, B1247).

/**
 Ships the interim page of an entry an error ended and files the stop.

 EVERY STOP SHIPS WHAT THE ENTRY HAS (the owner's answer of 2026-10-29, "Ship interim, resumable", and "no matter what, we
 must ship"; B1183, B1220) and leaves the entry open: the tally still files the stop as it did, no artifact is written,
 and a later run repeats the entry and replaces the page. Before the slicing exists the archive ships as the page.

 @param error - what ended the entry

 @param progress - what the entry had finished

 @param entry - corpus pair the entry was read from

 @param publishDir - root of the mirrored tree

 @param interimDir - directory the interim record is written into

 @param tip - repository head the pass runs at

 @param pipelineDigest - built pipeline that ships the page

 @param corpusSha - corpus commit the entry was read at

 @param attempt - number of the entry's attempt that stopped

 @param wallClock - clock the lines' times are read on

 @param stop - what the pass shares with its signal handler, absent where no stop signal is sent

 @param flight - the entry's place in that stop, told once the page is shipped

 @param durationMs - time the entry took

 @param aborted - whether the entry's deadline had aborted when the error was caught

 @param endRun - told of the error when it ends the run

 @param l - entry logger

 @returns The scheduling disposition of the stop

 @example
 ```ts
 return await settleCaughtEntry({ error, progress, entry, publishDir, interimDir, tip, pipelineDigest, corpusSha, attempt, wallClock, durationMs, aborted, l, },);
 ```
 */
export async function settleCaughtEntry(
  {
    error,
    progress,
    entry,
    publishDir,
    interimDir,
    tip,
    pipelineDigest,
    corpusSha,
    attempt,
    wallClock,
    stop,
    flight,
    durationMs,
    aborted,
    endRun,
    l,
  }: {
    readonly error: unknown;
    readonly progress: EntryProgress;
    readonly entry: CorpusPair;
    readonly publishDir: string;
    readonly interimDir: string;
    readonly tip: string;
    readonly pipelineDigest: PipelineDigest;
    readonly corpusSha: string;
    readonly attempt: number;
    readonly wallClock: () => Date;
    readonly stop?: PassStop;
    readonly flight?: EntryFlight;
    readonly durationMs: number;
    readonly aborted: boolean;
    readonly endRun?: (args: { readonly error: unknown; },) => void;
    readonly l: Logger;
  },
): Promise<EntryOutcome> {
  await shipInterimOnStop({
    error,
    progress,
    entryId: entry.id,
    sourceText: entry.sourceText,
    targetText: entry.targetText,
    publishDir,
    interimDir,
    tip,
    pipelineDigest,
    corpusSha,
    attempt,
    wallClock,
    ...((stop === undefined) ? {} : { stopSignal: stop.signal, }),
    ...((flight === undefined) ? {} : { flight, }),
    l,
  },);
  return tallyCaughtEntry({
    entryId: entry.id,
    error,
    attempt,
    wallClock,
    durationMs,
    aborted,
    ...((endRun === undefined) ? {} : { endRun, }),
  },);
}

//endregion Pass entry caught
