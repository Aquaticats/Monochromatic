import { idListFlag, } from './command-flags.ts';
import type { ReadsFlag, } from './command-line-types.ts';

//region Entry filter
// Restricts a pass to named corpus entries.
//
// WHY THIS EXISTS. The pass orders entries so coverage fills evenly, which is
// right for accumulation and wrong when one specific entry is the evidence for
// an open question. `Toka_ls`, the entry whose editor fabricated three lines,
// sat at position 22 of 71: roughly fourteen hours away at the measured
// per-entry cost, for a question a single entry answers.
//
// RUN IT INTO A THROWAWAY RUNS DIRECTORY. Hand-picking an entry into the main
// pass would put a deliberately chosen document into a pool that later draws
// treat as a natural accumulation, which is exactly the bias a seeded stratified
// draw exists to avoid. Point `TRANSLATION_REPAIR_RUNS_DIR` somewhere disposable
// and the accumulation stays honest.

/**
 Reads the entry allowlist from command-line arguments.

 An EMPTY SET MEANS EVERY ENTRY, which keeps the ordinary pass untouched: the
 flag is absent, the set is empty, and no filtering happens. That is why this
 returns a set rather than an optional list; a caller cannot forget to handle
 absence, because absence and "no restriction" are the same value.

 @param line - the pass's command line, read whole by `reportingRefusals`,
 which refuses `--only` written with nothing after it (ledger B75)

 @returns Ids to run, empty when unrestricted

 @example
 ```ts
 const onlyIds = readOnlyIds({ line, },);
 ```
 */
export function readOnlyIds(
  { line, }: { readonly line: ReadsFlag<'only'>; },
): ReadonlySet<string> {
  // A flag that parsed to nothing would silently run the WHOLE corpus, which is
  // the opposite of what was asked and expensive to discover afterwards, so
  // the shared reader refuses it.
  return new Set(idListFlag({
    asked: line.flag('only',),
    naming: 'entry id',
  },),);
}

//endregion Entry filter
