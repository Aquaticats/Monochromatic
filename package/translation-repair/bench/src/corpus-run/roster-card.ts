import { fetchTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  type CommandLineOf,
  reportingRefusals,
} from '@monochromatic-dev/translation-repair-run/ts';
import { printRosterCard, } from './roster-card-print.ts';

//region Roster card
// Prints the card fragment for one model off a provider's live listing:
// `roster-card-ask.ts` reads the command line, `roster-card-listing.ts`
// fetches the listing, `roster-card-render.ts` renders the card and
// `roster-card-print.ts` is the procedure.
//
// usage: mise run roster-card -- <synthetic|hyper|openrouter|bedrock|neuralwatt|deepseek|xiaomi> <served id>

/**
 Prints the card over the process's own environment, the live transport and
 today's date. Not `async`: it hands the promise on, so a run that refuses
 before its first call leaves no continuation behind it that nothing runs.

 @param line - the card's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await printCard({ line, },);
 ```
 */
function printCard({ line, }: { readonly line: CommandLineOf<'roster-card'>; },): Promise<void> {
  return printRosterCard({
    line,
    env: process.env,
    transport: fetchTransport,
    now: new Date(),
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'roster-card',
    argv: process.argv,
    env: process.env,
    run: printCard,
  },);

//endregion Roster card
