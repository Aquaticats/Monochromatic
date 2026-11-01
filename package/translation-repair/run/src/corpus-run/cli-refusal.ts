import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { RunJsonUnreadableError, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { taskRunnerVerdict, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  RUN_SEATS,
  seatReportLines,
  type SeatTally,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { readCommandLine, } from './command-line.ts';
import {
  COMMAND_LINES,
  type CommandLineOf,
  type CommandName,
} from './command-lines.ts';
import { COMMAND_OWNERS, } from './command-owners.ts';
import { runEndReports, } from './run-end-report.ts';

//region CLI refusal
// Turns ANY failure out of a CLI body into a report that quotes nothing.
//
// The half of that defect that mattered most is closed: a run file no longer
// prints itself when it will not parse. Verifying that at the boundary showed
// the other half still open. The refusal is safe now, but it is still UNCAUGHT,
// and a bundled CLI is one line of minified JavaScript, so Node echoes about
// three thousand characters of it around a correct one-line message.
//
// THIS USED TO CATCH ONLY `RunJsonUnreadableError`, and said so: catching every
// `Error` would hide the stack of a genuine programming fault, trading a rare
// ugly report for a permanently undiagnosable one. That reasoning was right
// about the cost and wrong about the choice, because the two are separable.
//
// A measurement supplied the fact the old note did not have. A `YAMLParseError`
// message embeds a source code frame, measured on five failure shapes and
// present in all five, and Node renders a cause chain. Re-throwing therefore
// published a page's front matter, which names a person, through a printer that
// never agreed to. Re-throwing is not neutral; it delegates the decision to
// whatever prints next.
//
// SO EVERYTHING IS CAUGHT, AND THE STACK IS KEPT ANYWAY. What is dropped is the
// message line and the cause chain, which is where text travels; what is kept
// is the frames, which name files inside our own `dist`. A fault stays
// diagnosable, and a class that declares its message quote-free still gets to
// say it in full.
//
// EVERY corpus-run entry point was put through this, thirty-eight of them,
// after measuring the four cells it turns on. Running the source on a throwaway
// fixture under node's type stripping:
//
//   bare, marked error      exit 1, 708 bytes. A stack dump, and Node also
//                           spills the error's own fields as an object literal.
//   wrapped, marked error   exit 4, 193 bytes. The message, and what to do.
//   bare, unmarked error    exit 1, 554 bytes. Class, message and stack.
//   wrapped, unmarked error exit 5, 662 bytes. Class and frames, no message.
//
// So this is an outright win on a refusal, and on a fault it trades the message
// for a code that separates a fault from a refusal from the command's own
// verdict. Unwrapped, all three are `1`. The lost message is the policy in
// `refusal-text.ts` failing closed, and the lever that gets it back is marking
// more of our own classes, not unwrapping the commands.
//
// A VERDICT SURVIVES because `process.exitCode` is set only in the catch here,
// which was measured before the change rather than read off this file:
// `verify-published` still exits `2` on a run it cannot check, byte-identical.
//
// THE FIRST THING IT DOES, before the command line is read, is ask the task
// runner guard (`task-runner-guard.ts`) whether the process may run the command:
// a process that holds provider keys and was not started by the command's own
// task is refused as stated, and so every entry file hands in the environment
// it runs in, since only a real process holds one.
//
// `ledger-report.ts` is wrapped like the rest, but almost nothing reaches this
// from it. It reads a whole directory and reports the files that refused as a
// shortfall inside its own output, which is a better answer than stopping, and
// its closing comment records why. This catches only what escapes that.

/**
 Exit code a CLI leaves behind when a run file would not read.

 FOUR, uniform across every command here, because 1 through 3 already carry
 each CLI's OWN verdicts and those differ per command: 1 means an absent
 ledger to one reader and a disagreeing published tree to another. A gate
 reading THIS code learns the same thing whichever reader it ran, which is
 that the run was never examined.
 */
const COULD_NOT_READ = 4;

/**
 Exit code a CLI leaves behind when it failed for a reason nobody planned for.

 SEPARATE FROM FOUR, because they ask different things of whoever reads them.
 Four says a named file would not read and the run is intact. This says the
 command itself broke, which is a bug report rather than a re-run.
 */
const UNEXPECTED_FAULT = 5;

/**
 Exit code a CLI leaves behind when it declined in its own words.

 ABOVE THE FAULT CODE RATHER THAN BELOW IT, because a stated refusal is the
 mildest of the three: nothing broke and nothing was half-read. A usage line,
 an unset key, a control that did not hold. Codes one through three stay free
 for each command's own verdicts, which this must never be read as.
 */
const REFUSED_AS_STATED = 6;

/**
 Renders a caught value's stack frames, without its message or its cause.

 THE FRAMES ARE THE SAFE HALF. Each names a file and a position inside our own
 built output, so they locate a fault precisely and carry no text that was
 read. The message line is dropped because an error built by interpolation can
 embed whatever it was given, and the cause chain is dropped because a parser's
 cause is exactly the thing that quotes.

 THE FRAMES BEGIN WHERE THE STACK'S HEADER ENDS, not at any line that starts
 with `at `: the header is the message as it was when the error was built,
 and a message holding a line break and then `at ` would otherwise print that
 line of itself. A header that does not hold the message (rewritten after the
 stack was recorded) has no end this can tell, so no frames are printed.

 @param error - caught value, of unknown type by construction

 @returns Frame lines, or a note saying why there are none

 @example
 ```ts
 console.error(framesOf({ error, },),);
 ```
 */
function framesOf({ error, }: { readonly error: unknown; },): string {
  if (!Error.isError(error,))
    return '  (no frames: the thrown value is not an Error)';

  /**
   Stack as the runtime recorded it, message line included.
   */
  const { stack, } = error;

  if ((typeof stack) !== 'string')
    return '  (no frames: this error recorded no stack)';

  /**
   The message the header was written with; searched for, never printed.
   */
  const { message, } = error;

  /**
   Where the header's message begins, which is nowhere for an empty message.
   */
  const messageAt = stack.indexOf(message,);
  if (messageAt === (-1))
    return '  (no frames: the stack\'s header does not hold the error\'s message, so where its frames begin is unknown)';

  /**
   Where the header ends: after the message, which is where it begins for an
   empty one, leaving only the class name before the frames.
   */
  const headerEnd = messageAt + message.length;

  return stack
    .slice(headerEnd,)
    .split('\n',)
    .filter(function isFrame(line,): boolean {
      return line
        .trimStart()
        .startsWith('at ',);
    },)
    .join('\n',);
}

/**
 Prints the seat report to stderr when its scope ends, after whatever the
 command said, so the closing lines of every command name any seat that
 produced nothing usable. Nothing at all when no seat was asked, so
 a command that never built a client prints nothing extra.

 @param seats - tally to render

 @returns Async disposable printing the end reports and then the seat report on dispose

 @example
 ```ts
 await using _report = printingSeatReport({ seats: RUN_SEATS, },);
 ```
 */
function printingSeatReport({ seats, }: { readonly seats: SeatTally; },): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      // THE END REPORTS FIRST (NeuralWatt's plan energy), then the seat report, so the
      // closing lines of every command stay the seats.
      for (const line of await runEndReports())
        console.error(line,);
      for (const line of seatReportLines({ tally: seats, },))
        console.error(line,);
    },
  };
}

/**
 Runs a CLI body, reporting a refusal this package wrote rather than crashing.

 @param what - command name as an operator would type it, which starts the
 line and names the declaration its command line is read against
 (`command-lines.ts`)

 @param argv - process arguments, read whole before the body starts, so an
 argument the command does not read is refused rather than ignored (ledger
 B75)

 @param env - environment of the process the command runs in, handed in by
 the entry file because only a real process holds one; read before anything
 else, so a process that holds provider keys and does not name the command as
 the one it means to start refuses before its command line is read
 (`task-runner-guard.ts`)

 @param run - CLI body to run, given the command line read

 @param seats - tally to print when the command ends; defaults to the
 run-wide one `createRunClient` counts into, and tests pass their own

 @example
 ```ts
 if (import.meta.main)
   await reportingRefusals({ what: 'score-verify', argv: process.argv, env: process.env, run: main, },);
 ```
 */
export async function reportingRefusals<const Command extends CommandName>(
  {
    what,
    argv,
    env,
    run,
    seats = RUN_SEATS,
  }: {
    readonly what: Command;
    readonly argv: readonly string[];
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly run: (input: { readonly line: CommandLineOf<Command>; },) => Promise<void>;
    readonly seats?: SeatTally;
  },
): Promise<void> {
  /**
   Prints the seat report when this scope ends, whatever happened inside it:
   under the refusal line on a refusal, alone on a clean run, so a seat that
   produced nothing usable is in the closing lines of every command.
   */
  await using _report = printingSeatReport({ seats, },);

  try {
    // FIRST, before the command line is read and before the body touches a
    // file, a clone or the network: a process that holds provider keys runs a
    // command only when it names that command as the one it means to start,
    // which the command's task does for it.
    /**
     Whether this process may run the command.
     */
    const verdict = taskRunnerVerdict({
      env,
      command: what,
      owner: COMMAND_OWNERS[what],
    },);
    if (!verdict.allowed)
      throw new StatedRefusalError({ says: verdict.says, },);

    await run({
      line: readCommandLine({
        command: what,
        spec: COMMAND_LINES[what],
        argv,
      },),
    },);
  } catch (error) {
    // TO STDERR, unlike the reports these commands normally print. A reader
    // piping stdout to a file is collecting a report, and this says there is no
    // report to collect.
    console.error(`${what}: ${refusalText({ error, },)}`,);

    // NOTHING FURTHER TO SAY. The `console.error` line is the whole report: the command
    // stated why it declined, and no frames are worth printing because there is
    // no bug to locate.
    if (error instanceof StatedRefusalError) {
      process.exitCode = REFUSED_AS_STATED;
      return;
    }

    if (error instanceof RunJsonUnreadableError) {
      console.error(
        '  Nothing was read past that file, so this run was not examined. Re-run the pass to '
          + 'rewrite it, or name a run directory that has it.',
      );
      process.exitCode = COULD_NOT_READ;
      return;
    }

    console.error(
      '  This is a fault in the command rather than in the run. The stack frames printed with it name the '
        + 'built files it stopped in.',
    );
    console.error(framesOf({ error, },),);
    process.exitCode = UNEXPECTED_FAULT;
  }
}

//endregion CLI refusal
