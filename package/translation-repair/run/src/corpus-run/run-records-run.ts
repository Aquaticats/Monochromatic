import { _neutralizeControlCharacters as neutralizeControlCharacters, } from '@monochromatic-dev/logger-core/ts';

import type { CommandLineOf, } from './command-lines.ts';
import {
  refuseRepeatedLogs,
  streamLogLines,
} from './report-log-read.ts';

//region Run records run
// The whole procedure of `run-records`: print the records a pass kept in its run folder the way the console showed
// them. A pass keeps its records as `log/<day>/*.log.jsonl` (`typography/src/run-log.ts`), and the raw files are the
// wrong thing to read on a terminal: a message can carry a control character, and `jq --raw-output '.message'` prints
// it to the terminal as the sequence it is (and exits with an error on a record holding a lone surrogate).
//
// ONE LINE FOR EACH RECORD, `[level] [ISO instant] message`, the message's control characters (C0 except the line feed
// and the tab, DEL and C1) written as `\uXXXX` by the logger's own function for it, which is the console sink's. A line
// that is no record (a cut or foreign line) is printed as it stands but for the same escaping. The debug and trace
// records, which the console showed only for a verbose run, are left out unless `--verbose` is given.
//
// SPENDS NO QUOTA AND TOUCHES NO MODEL. It reads files.

/**
 Prints the records of the named run folders and record files.

 @param line - command line, read whole by `reportingRefusals`

 @throws {@link StatedRefusalError} when a log is named twice or by two spellings that reach one record file, or one
 will not read

 @example
 ```ts
 await printRunRecords({ line, },);
 ```
 */
export async function printRunRecords({ line, }: { readonly line: CommandLineOf<'run-records'>; },): Promise<void> {
  /**
   Run folders and record files named on the command line.
   */
  const paths = line.positionals;

  // BEFORE ANY RECORD IS PRINTED: a log named twice would be printed twice.
  await refuseRepeatedLogs({ paths, },);

  for await (const record of streamLogLines({
    paths,
    verbose: line.switched('verbose',),
  },))
    console.log(neutralizeControlCharacters(record,),);
}

//endregion Run records run
