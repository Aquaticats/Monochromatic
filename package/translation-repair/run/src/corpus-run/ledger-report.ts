import { reportingRefusals, } from './cli-refusal.ts';
import type { CommandLineOf, } from './command-lines.ts';
import { reportLedger, } from './ledger-report-run.ts';
import { resolveRunsDir, } from './run-config.ts';

//region Ledger report
// WHAT EACH MODEL WROTE, AND WHAT THE JUDGES SAID ABOUT IT. Spends no quota and
// touches no model.
//
// WITHOUT A MODEL NAMED it reports the whole ledger: who wrote how much, how
// often it was chosen, and the two ballot faults counted apart.
//
// WITH `--model <id>` it prints that seat's candidates and the reasons judges
// gave for choosing them. That is the question a standing cannot answer: a low
// share means a seat was rarely picked as the best of several, which is not the
// same as writing something wrong, and only the text says which it was.
//
// PRINTS CORPUS WORDING when a model is named, because the candidate text IS
// the evidence. A run directory already holds it. Do not paste this output
// anywhere public.
//
// THE SUMMARY VIEW PRINTS NO WORDING AT ALL, and `ledger-directory.ts` is what
// keeps that true when a file will not read.
//
// THIS FILE IS ONLY THE WIRING: the command line, and the runs directory the
// environment names. `ledger-report-run.ts` holds the procedure and
// `ledger-report-print.ts` the printers.

/**
 Reads the runs directory this process was pointed at, reports its ledger and
 leaves the exit code the report returns.

 @param line - the report's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await main({ line, },);
 ```
 */
async function main({ line, }: { readonly line: CommandLineOf<'ledger-report'>; },): Promise<void> {
  process.exitCode = await reportLedger({
    line,
    runsDir: await resolveRunsDir(),
  },);
}

// NOT WRAPPED IN A CATCH. Every failure this command raises now names itself safely:
// the reads go through `readRunJson`, which refuses without quoting the file it
// could not parse, and the listing re-raises with a code rather than a path.
// Wrapping them under one class name would replace a message that says what
// happened with `Error`, which is what `verify-published.ts` learned by doing it.
if (import.meta.main)
  await reportingRefusals({
    what: 'ledger-report',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Ledger report
