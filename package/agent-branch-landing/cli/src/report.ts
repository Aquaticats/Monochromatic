/**
 Where a landing says what it found. What a landing prints is its result, which a coordinator pastes into a handover, so it goes
 to standard output as plain text and not through a logger; a warning that is not a result goes to standard error, as
 `doc/decision/cli-output-and-exit-codes.md` asks.

 @module
 */

/**
 Somewhere to say what a landing found.
 */
export type Report = {
  /**
   One line of result, on standard output.
   */
  readonly line: (text: string) => void;

  /**
   One line of warning, on standard error, kept apart from the result so a caller piping the result still sees it.
   */
  readonly warning: (text: string) => void;
};

/**
 The report a command writes when it runs for a person or an agent: results on standard output, warnings on standard error.
 */
export const reportToTheTerminal: Report = {
  line: function line(text: string,): void {
    process.stdout
      .write(`${text}\n`,);
  },
  warning: function warning(text: string,): void {
    process.stderr
      .write(`${text}\n`,);
  },
};
