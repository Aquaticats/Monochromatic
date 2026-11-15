/**
 What a run that says nothing needs: a line in the feed the first time, and past a second stated time the end of the command it is
 sitting in, with a message of its own that tells the agent what was ended.

 Nothing watched a command that never returns. A Pi run found on 2026-11-15 had sat inside one `bash` search of a folder holding
 several hundred gigabytes for 15.6 hours (its `tool_start` at 1,284,897 ms, its `tool_end` at 57,600,250 ms with
 `Command aborted`), the abort arriving only from the run's wall clock, and its branch never moved. Three more runs had written no
 feed line for 105, 378 and 724 minutes, each inside one command, and a `SIGTERM` to the process that held it brought all three
 back within a minute.

 ## What a run says, and what it does not

 Every record Pi writes reaches the driver as one call to the run's record, so a run that says nothing is a run Pi has written
 nothing about. That is exactly what the record's activity clock measures, and it needs no second clock of its own: a command's
 output arrives as `tool_execution_update` records, which Pi's bash tool emits at most every 100 ms while output comes, and a
 model's reply arrives as `message_update` deltas. A build printing progress or a search printing matches is therefore never
 silent, however long it runs, while a command whose output stopped, or whose own process went and left a descendant holding the
 pipe, is.

 What the driver can observe of a command in flight is what Pi's protocol carries, from `docs/json.md` and `docs/rpc-commands.md`
 in the installed Pi:

 - `tool_execution_start` `{toolCallId, toolName, args}`, whose `args` hold the bash tool's `command` and its optional `timeout`
   in seconds;
 - `tool_execution_update` `{toolCallId, toolName, args, partialResult}`, one per chunk of output;
 - `tool_execution_end` `{toolCallId, toolName, result, isError, durationMs}`.

 Nothing reports "the command is still working", so silence is all the driver has to go on, and it cannot go looking for the
 command's process: no record names it.

 What it can do, from the same two documents and Pi's own source: `{"type":"abort"}` ends the operation in flight, kills every
 tool's process tree through the run's abort signal and answers once the session is idle, which is the same call the run's wall
 clock makes (`put-the-run-away.ts`); `{"type":"abort_bash"}` cancels only a `bash` command a client sent over RPC, because it
 aborts the `_bashAbortControllers` that `executeBash` fills, and a tool the agent ran is not in that set; `steer` is queued until
 the current turn's tool calls finish, so it cannot reach the agent while a command runs; and `prompt` starts a turn, which an idle
 Pi takes and a busy one answers with a refusal. Nothing the protocol offers ends one call of a tool that the agent started, so
 the only end here is of the whole operation, and every rule below answers to that.

 ## The two times

 The first is `SILENT_NOTICE_SECONDS`: one feed line and one summary entry, naming every command in flight and how long each has
 said nothing. The second is `SILENT_ENDING_SECONDS`: the operation they are in is ended and the agent is told. Both are options of
 the run (`--silent-seconds` and `--silent-end-seconds`), and `driver-constants.ts` carries the measurement each value comes from.

 The silence is written down once per stretch, not once per look: a line every look would bury the feed it is meant to explain.

 ## Which call is which

 A call is told by its `toolCallId`, which Pi writes on `tool_execution_start` and on `tool_execution_end`, so two calls of one
 tool in flight at once are told apart and their ends may arrive in any order. An end with no identifier, or with one no start
 carried, falls back to the last call of that tool name still open, and that fallback can take the wrong one when two calls of
 that tool are in flight; the identifier is what removes that case, and it is what the feed's line and the summary name.

 ## When the operation is due

 The operation is due when the run has said nothing for the second stated time and every call in flight has outlived what it
 declared: a call that gave the bash tool a `timeout` gets that bound plus `SILENT_BOUND_GRACE_SECONDS`, and a call that declared
 none gets the stated time. A call beside a peer with a longer bound waits for that peer, because the end is of the operation and
 takes the peer with it; an unbounded call beside a two-hour one is therefore ended after two hours.

 ## A long honest command, and a hang

 The driver cannot tell them apart from silence and does not pretend to: a whole-suite run that prints nothing looks like a hang,
 so the second time sits above every honest silence measured (the longest stretch with no command open was 1,205 s, and the longest
 command that declared no bound of its own was 5,512 s).

 An agent that knows a command may run long says so in advance, by giving the bash tool its `timeout` argument in seconds. Pi
 enforces that bound itself, killing the command's process tree when it passes, and this driver does not end the command before
 the bound has passed. A bound buys time, never immunity: a command whose own bound could not end it, which is a descendant
 holding the pipe after the command's process went, is ended by the driver `SILENT_BOUND_GRACE_SECONDS` after that bound, and the
 message says the bound had passed and the command was still going. A bound longer than the run's wall clock changes nothing about
 when the run ends, because the wall clock stays the outer bound of every wait.

 ## What a failed recovery leaves the run with

 Each step of ending one answers three ways: `done`, `refused`, or `unanswered`. Pi refusing any step is an answer of no, and the
 run ends `prompt-rejected`. A step nobody answers is neither a yes nor a no, and the run ends `silent-command-unanswered`: its
 agent was never told what became of its command, so no reader may take it for a run that finished its turn. A session that does
 not settle after the abort is never sent the message at all, because a session still at work is not one whose next turn is the
 agent's. Either way the run does not end `idle`, and the summary's entry says which step answered what.

 @module
 */

import {
  FEED_PREVIEW_CHARACTERS,
  MILLISECONDS_PER_SECOND,
  SECONDS_PER_MINUTE,
  SILENT_BOUND_GRACE_SECONDS,
} from './driver-constants.ts';
import type {
  CommandAnswer,
  RecoveryAnswer,
  RunProgress,
  SilentCommand,
} from './driver-types.ts';
import { fieldsOf, } from './read-pi-fields.ts';
import { preview, } from './previews-of-pi-messages.ts';
import type { WatchRequest, } from './watch-request.ts';

/**
 Where the driver stands with the silence of one run, kept for as long as the watch of that run lasts.
 */
export type SilenceWaitedFor = {
  /**
   When the stretch of silence that has been written down started at, or nothing while no line is owed.

   A stretch is told from the next one by that instant rather than by the silence being short again: a run that speaks and falls
   silent once more between two looks is a new silence, and is named.
   */
  noticedAt?: number;
};

/**
 Reads the bound a command declared for itself.

 Only the bash tool's own `timeout` argument counts. It is the bound Pi enforces by killing the command's process tree, and it is
 one number in the arguments rather than a word a filter can mistake for something else. A `timeout` written in the command's own
 shell line is not read here, which is why the message that follows an ended command names the argument.

 @param args - the arguments Pi reported for the tool start

 @returns the bound as the field a silent command carries, absent when the command declared none

 @example
 ```ts
 const declared = declaredBoundOf({ command: 'mise run test', timeout: 5400 },);
 ```
 */
export function declaredBoundOf(args: unknown,): { readonly boundSeconds?: number } {
  /**
   The bound the tool start carries, of any shape.
   */
  const { timeout, } = fieldsOf(args,);
  // Absence is the field not being there, the way every other optional field of this package says it.
  return (((typeof timeout) === 'number') && Number.isFinite(timeout,)
    && (timeout > 0)) ? { boundSeconds: timeout, } : {};
}

/**
 How long one command may run before the driver may end the operation it belongs to.

 @param command - the command in flight

 @param silentEndMilliseconds - the second stated time, in milliseconds

 @param graceSeconds - the grace past a declared bound

 @returns the milliseconds the command may run

 @example
 ```ts
 const dueAt = dueAtOf({ command, silentEndMilliseconds: 3_600_000, graceSeconds: 60, },);
 ```
 */
export function dueAtOf({
  command,
  silentEndMilliseconds,
  graceSeconds,
}: {
  readonly command: SilentCommand;
  readonly silentEndMilliseconds: number;
  readonly graceSeconds: number
},): number {
  return (command.boundSeconds === undefined)
    ? silentEndMilliseconds
    : Math.max(
      silentEndMilliseconds,
      (command.boundSeconds + graceSeconds) * MILLISECONDS_PER_SECOND,
    );
}

/**
 Tells whether the operation a silent run sits in is past the point where the driver ends it.

 The end the protocol offers is of the whole operation (`abort`), so the question is about that operation and not about one call:
 the run has said nothing for at least the second stated time, and every call still in flight has run at least as long as the
 later of that time and the bound it declared for itself, plus the grace that lets Pi's own kill land first.

 A call that declared no bound of its own is therefore held to the stated time, and a call whose peer declared a longer bound
 waits for that peer: ending the operation ends them all. The message the agent gets names every call that was in flight.

 @param silentMilliseconds - how long the run has said nothing

 @param silentEndMilliseconds - the second stated time, in milliseconds

 @param graceSeconds - the grace past a declared bound

 @param commands - every call still in flight

 @returns whether the driver ends the operation now

 @example
 ```ts
 const due = theOperationIsDueToEnd({
   silentMilliseconds: 7_300_000,
   silentEndMilliseconds: 3_600_000,
   graceSeconds: 60,
   commands: [{ tool: 'bash', args: {}, milliseconds: 7_300_000, boundSeconds: 7_200 }],
 },);
 ```
 */
export function theOperationIsDueToEnd({
  silentMilliseconds,
  silentEndMilliseconds,
  graceSeconds,
  commands,
}: {
  readonly silentMilliseconds: number;
  readonly silentEndMilliseconds: number;
  readonly graceSeconds: number;
  readonly commands: readonly SilentCommand[]
},): boolean {
  if ((commands.length === 0) || (silentMilliseconds < silentEndMilliseconds)) {
    return false;
  }
  return commands.every(function pastItsOwnLimit(command,) {
    return command.milliseconds >= dueAtOf({
      command,
      silentEndMilliseconds,
      graceSeconds,
    },);
  },);
}

/**
 Keeps a command as a feed line and a summary hold it: its arguments cut to the length one line carries, its own identifier, and
 the bound it declared when it declared one.

 @param command - the command in flight

 @returns the command as it is written down

 @example
 ```ts
 const kept = keptCommand({ command: { tool: 'bash', args: { command: 'rg -l x' }, milliseconds: 700_000 }, },);
 ```
 */
export function keptCommand({ command, }: { readonly command: SilentCommand },): SilentCommand {
  return {
    ...(command.toolCallId === undefined ? {} : { toolCallId: command.toolCallId, }),
    tool: command.tool,
    args: preview({
      value: command.args,
      length: FEED_PREVIEW_CHARACTERS.toolArguments,
    },),
    milliseconds: command.milliseconds,
    ...(command.boundSeconds === undefined ? {} : { boundSeconds: command.boundSeconds, }),
  };
}

/**
 Keeps every command a silence sat in, as a feed line and a summary hold them.

 @param commands - the commands in flight

 @returns the commands as they are written down

 @example
 ```ts
 const kept = keptCommands([{ tool: 'bash', args: { command: 'rg -l x' }, milliseconds: 700_000 }],);
 ```
 */
export function keptCommands({ commands, }: { readonly commands: readonly SilentCommand[] },): readonly SilentCommand[] {
  return commands.map(function kept(command,) {
    return keptCommand({ command, },);
  },);
}

/**
 What one command of a silence is, for the message the agent reads: its name and what it was asked to do, and the bound it
 declared when it declared one.

 @param command - the command that was in flight

 @returns what the message says of it

 @example
 ```ts
 const line = namedCommandOf({ command, },);
 ```
 */
function namedCommandOf({ command, }: { readonly command: SilentCommand },): string {
  /**
   The bound this command declared, in the words of the message the agent reads.
   */
  const bound = (command.boundSeconds === undefined)
    ? 'no bound of its own'
    : `a bound of ${String(command.boundSeconds,)} seconds of its own`;
  return `${command.tool} \`${preview({
    value: command.args,
    length: FEED_PREVIEW_CHARACTERS.toolArguments,
  },)}\` (${bound}, running ${String(command.milliseconds,)} ms)`;
}

/**
 What the agent is told when the operation it was sitting in is ended: every command that was in flight, after how long, and that a
 command which may run long needs a bound of its own.

 The end the protocol offers is of the whole operation, so every call in flight ended with it, and a call that declared a longer
 bound than its peer was ended when the driver had waited for that peer.

 @param commands - every command that was in flight when the operation was ended

 @param silentMilliseconds - how long the run had said nothing

 @returns the message the agent's next turn begins with

 @example
 ```ts
 const said = endedCommandMessageOf({ commands: [command], silentMilliseconds: 3_600_000, },);
 ```
 */
export function endedCommandMessageOf({
  commands,
  silentMilliseconds,
}: {
  readonly commands: readonly SilentCommand[];
  readonly silentMilliseconds: number
},): string {
  /**
   The minutes the run had said nothing, which is the figure a reader of a long run thinks in, and nothing under a minute where
   the milliseconds are the whole figure.
   */
  const minutes = silentMilliseconds / (MILLISECONDS_PER_SECOND * SECONDS_PER_MINUTE);
  /**
   What one call was, or what each of several was, as the message's verbs agree with.
   */
  const several = commands.length > 1;
  return [
    `The driver ended the operation you were in, after it had said nothing for ${String(silentMilliseconds,)} ms`
      + `${(minutes < 1) ? '' : ` (${minutes.toFixed(1,)} minutes)`}. `
      + `${several ? 'Every command in flight ended with it' : 'The command in flight ended with it'}:`,
    ...commands.map(function line(command,) {
      return `- ${namedCommandOf({ command, },)}`;
    },),
    'Their answers end as aborted where the driver stopped them, and those answers are what you have of them.',
    'A command that may run long needs a bound of its own: give the bash tool its `timeout` argument, in seconds, and the driver '
      + 'waits for it. The driver ends the operation only when every command in flight has outlived what it declared, so a command '
      + 'with a longer bound holds the ones beside it, and a command whose own bound could not end it is ended a minute past that '
      + 'bound. Nothing outlives the run\'s wall clock.',
    'Go on from here: read what the commands left, and give a bound to any command you run again that may take long.',
  ].join('\n',);
}

/**
 Writes a stretch of silence down the first time it is seen, so a reader of the feed learns what the run is sitting in without
 waiting for that to end.

 @param watch - watch request

 @param commands - every call in flight, when any is

 @param silentMilliseconds - how long the run has said nothing

 @param waited - where the driver stands with this run's silence, which this fills
 */
function noticeTheSilence({
  watch,
  commands,
  silentMilliseconds,
  startedAt,
  waited,
}: {
  readonly watch: WatchRequest;
  readonly commands: readonly SilentCommand[];
  readonly silentMilliseconds: number;
  readonly startedAt: number;
  readonly waited: SilenceWaitedFor
},): void {
  waited.noticedAt = startedAt;
  watch.record
    .noteSilence({
      silentMilliseconds,
      ...((commands.length === 0) ? {} : { commands: keptCommands({ commands, }), }),
    },);
}

/**
 What Pi answered one step of ending a silent command with: an answer that did it, a refusal, or no answer at all.

 @param answer - how waiting for Pi's answer ended

 @returns what that step got

 @example
 ```ts
 const step = answerOf({ answer: { ending: 'silence', problem: 'no answer' }, },);
 ```
 */
function answerOf({ answer, }: { readonly answer: CommandAnswer },): RecoveryAnswer {
  if (answer.ending !== 'answered') {
    return 'unanswered';
  }
  return (answer.record
    .success
    === false) ? 'refused' : 'done';
}

/**
 Why one step of ending a silent command did not go through, in Pi's own words or its silence.

 @param answer - how waiting for Pi's answer ended

 @param what - which step this was, for the message

 @returns the reasons, empty when the step went through

 @example
 ```ts
 const problems = problemsOf({ answer: { ending: 'silence', problem: 'no answer' }, what: 'the abort' },);
 ```
 */
function problemsOf({
  answer,
  what,
}: {
  readonly answer: CommandAnswer;
  readonly what: string
},): readonly string[] {
  if (answer.ending !== 'answered') {
    return [`Pi answered nothing to ${what}: ${answer.problem}`];
  }
  /**
   What Pi said of it.
   */
  const { record, } = answer;
  if (record.success !== false) {
    return [];
  }
  /**
   Pi's own word for the refusal, which is absent only when it gave none.
   */
  const reason = (record.error === undefined) ? 'no reason given' : preview({
    value: record.error,
    length: FEED_PREVIEW_CHARACTERS.eventDetail,
  },);
  return [`Pi refused ${what}: ${reason}`];
}

/**
 Tells whether Pi took a message.

 @param answer - how waiting for Pi's answer ended

 @returns whether Pi answered that it took it

 @example
 ```ts
 const taken = takenByPi({ ending: 'answered', record: { success: true } },);
 ```
 */
function takenByPi(answer: CommandAnswer,): boolean {
  return (answer.ending === 'answered') && (answer.record
    .success
    === true);
}

/**
 Tells whether Pi refused a message itself, which is no answer and no turn started.

 @param answer - how waiting for Pi's answer ended

 @returns whether Pi answered with a refusal

 @example
 ```ts
 const refused = refusedByPi({ ending: 'answered', record: { success: false } },);
 ```
 */
function refusedByPi(answer: CommandAnswer,): boolean {
  return (answer.ending === 'answered') && (answer.record
    .success
    === false);
}

/**
 Ends the operation a silence is sitting in, the way the wall clock ends one, and sends the agent a message of its own saying what
 was ended, after how long, and that a command which may run long needs a bound of its own.

 Every step is asked and answered separately. The abort ends the operation; the settle is Pi's own record that the session is idle,
 which its answer already implies and which this reads anyway; the message is the turn the agent goes on in. A step that is refused
 is an answer of no, and a step that is answered with nothing is neither: both leave the agent unable to go on, and the run ends
 with a reason that says which it was.

 @param watch - watch request

 @param commands - every command in flight in the operation to end

 @param silentMilliseconds - how long the run has said nothing

 @returns why the run cannot go on, or that it goes on

 @example
 ```ts
 const after = await endWhatSaidNothing({ watch, commands, silentMilliseconds, },);
 ```
 */
async function endWhatSaidNothing({
  watch,
  commands,
  silentMilliseconds,
}: {
  readonly watch: WatchRequest;
  readonly commands: readonly SilentCommand[];
  readonly silentMilliseconds: number
},): Promise<RunProgress> {
  /**
   Pi's answer to the abort, which ends the operation in flight and every command with it.
   */
  const aborted = await watch.connection
    .send({
      command: { type: 'abort', },
      deadline: watch.deadline,
    },);
  /**
   What that step got.
   */
  const abort = answerOf({ answer: aborted, },);
  /**
   Whether the session was idle afterwards, as Pi's own records say: a message sent to a session still at work is a message the
   agent never reads as its next turn.
   */
  const settled = (abort === 'done') && (!watch.record
    .isRunning());
  /**
   What the agent is told, which arrives as a turn of its own: this is asked only of a session the abort left idle, because a
   prompt to a busy one is refused before it is ever the agent's.
   */
  const told = (abort === 'done') && settled
    ? await watch.connection
      .send({
        command: {
          type: 'prompt',
          message: endedCommandMessageOf({
            commands,
            silentMilliseconds,
          },),
        },
        deadline: watch.deadline,
      },)
    : undefined;
  /**
   What the message step got, when it was asked.
   */
  const message = (told === undefined) ? undefined : answerOf({ answer: told, },);
  /**
   Why a step that was asked did not go through, in the words the summary carries.
   */
  const problems = [
    ...problemsOf({
      answer: aborted,
      what: 'the abort',
    },),
    ...((told === undefined)
      ? []
      : problemsOf({
        answer: told,
        what: 'the message',
      },)),
  ];
  watch.record
    .noteSilenceEnded({
      silentMilliseconds,
      commands: keptCommands({ commands, }),
      abort,
      settled,
      ...((message === undefined) ? {} : { message, }),
      ...((problems.length === 0) ? {} : { problems, }),
    },);
  return endOf({
    abort,
    settled,
    ...((message === undefined) ? {} : { message, }),
  },);
}

/**
 Why a run cannot go on, from what its silent command's ending got: nothing stands between it and the next look when the abort
 ended the operation, the session settled and the agent was told; a refusal anywhere is Pi answering no to the driver; and any
 other step that went unanswered is a run whose agent was never told what became of its command.

 @param abort - what the abort got

 @param settled - whether the session was idle after it

 @param message - what the recovery message got, when it was asked

 @returns why the run cannot go on, or that it goes on

 @example
 ```ts
 const end = endOf({ abort: 'done', settled: true, message: 'done', },);
 ```
 */
function endOf({
  abort,
  settled,
  message,
}: {
  readonly abort: RecoveryAnswer;
  readonly settled: boolean;
  readonly message?: RecoveryAnswer
},): RunProgress {
  if ((abort === 'done') && settled
    && (message === 'done')) {
    return 'still-running';
  }
  return ((abort === 'refused') || (message === 'refused')) ? 'prompt-rejected' : 'silent-command-unanswered';
}

/**
 Does what a run's silence needs: nothing, the line that names it, or the end of the command the run is sitting in.

 Called on every look at a run's limits, so the line goes out the first time the silence passes the first stated time, and the
 command is ended once it passes the second. A silence with no command in flight is a model that has answered nothing, which the
 run's own retry and wall clock already bound, so it is written down and nothing more.

 @param watch - watch request

 @param waited - where the driver stands with this run's silence, which this fills and clears

 @returns why the run cannot go on, or that it goes on

 @example
 ```ts
 const afterSilence = await aSilenceInTheRun({ watch, waited, },);
 ```
 */
export async function aSilenceInTheRun({
  watch,
  waited,
}: {
  readonly watch: WatchRequest;
  readonly waited: SilenceWaitedFor
},): Promise<RunProgress> {
  /**
   The first stated time, in milliseconds.
   */
  const noticeMilliseconds = watch.options
    .silentSeconds
    * MILLISECONDS_PER_SECOND;
  /**
   How long the run has said nothing.
   */
  const silentMilliseconds = watch.record
    .silentMilliseconds();
  if (silentMilliseconds < noticeMilliseconds) {
    return 'still-running';
  }
  /**
   Every call the run is sitting in, which is more than one when a tool started another or Pi started two at once.
   */
  const commands = watch.record
    .commandsInFlight();
  /**
   When this stretch of silence started at.
   */
  const startedAt = watch.record
    .silenceStartedAt();
  if (waited.noticedAt !== startedAt) {
    noticeTheSilence({
      watch,
      commands,
      silentMilliseconds,
      startedAt,
      waited,
    },);
  }
  /**
   Whether this silence is past the point where the driver ends the operation every call in flight shares.
   */
  const dueToEnd = theOperationIsDueToEnd({
    silentMilliseconds,
    silentEndMilliseconds: watch.options
      .silentEndSeconds
      * MILLISECONDS_PER_SECOND,
    graceSeconds: SILENT_BOUND_GRACE_SECONDS,
    commands,
  },);
  if (!dueToEnd) {
    return 'still-running';
  }
  return await endWhatSaidNothing({
    watch,
    commands,
    silentMilliseconds,
  },);
}
