import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  runLogger,
  sliceTagged,
} from '@monochromatic-dev/translation-repair-typography/ts';

//region Pass entry logger
// THE ONE PLACE A PASS BUILDS THE LOGGER OF AN ENTRY. Every stage of an entry writes through a logger built from its id,
// and two things about that logger are decided here so no site can forget either: it is ROOTED ON THE RUN LOG (`l:
// runLogger`), so the `round:` lines, the `SLICE-COST` lines, the consolidate driver's lines and every entry-level warning
// and error are records of the run folder and not lines of the console alone (ledger B1110), and it is slice-tagged, so
// a line written inside an attempt names the attempt, lane and slice (ledger B963).

/**
 The logger a pass hands the work of one entry.

 @param entryId - entry the lines belong to, which is the tag every line carries

 @returns Logger writing `[entry] [attempt n] [lane slice n] message` through the installed run log, or the default
 logger while none is installed

 @example
 ```ts
 const el = entryLoggerOf({ entryId: entry.id, },);
 ```
 */
export function entryLoggerOf({ entryId, }: { readonly entryId: string; },): Logger {
  return sliceTagged({
    l: tagged({
      tag: entryId,
      l: runLogger,
    },),
  },);
}

//endregion Pass entry logger
