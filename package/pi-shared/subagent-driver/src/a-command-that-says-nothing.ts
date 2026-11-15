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

 What it can do to one, from the same two documents and Pi's own `abort`: `{"type":"abort"}` ends the operation in flight, kills
 the tool's process tree through its abort signal and answers once the session is idle, which is the same call the run's wall
 clock makes (`put-the-run-away.ts`); `{"type":"abort_bash"}` cancels only a `bash` command a client sent over RPC and leaves a
 tool the agent ran alone, so it cannot end this one; `steer` is queued until the current turn's tool calls finish, so it cannot
 reach the agent while the command runs; and `prompt` starts a turn, which an idle Pi takes and a busy one answers with a refusal.

 ## The two times

 The first is `SILENT_NOTICE_SECONDS`: one feed line and one summary entry, naming the command and how long it has said nothing.
 The second is `SILENT_ENDING_SECONDS`: that command is ended and the agent is told. Both are options of the run
 (`--silent-seconds` and `--silent-end-seconds`), and `driver-constants.ts` carries the measurement each value comes from.

 The silence is written down once per stretch, not once per look: a line every look would bury the feed it is meant to explain.

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
 Tells whether a command that said nothing is past the point where the driver ends it.

 Two things are asked: the run has said nothing for at least the second stated time, and the command has run at least as long as
 the later of that time and the bound it declared for itself, plus the grace that lets Pi's own kill land first.

 @param silentMilliseconds - how long the run has said nothing

 @param silentEndMilliseconds - the second stated time, in milliseconds

 @param graceSeconds - the grace past a declared bound

 @param command - the command in flight, when one is

 @returns whether the driver ends this command now

 @example
 ```ts
 const due = aSilentCommandIsDueToEnd({
   silentMilliseconds: 3_700_000,
   silentEndMilliseconds: 3_600_000,
   graceSeconds: 60,
   command: { tool: 'bash', args: {}, milliseconds: 3_700_000 },
 },);
 ```
 */
export function aSilentCommandIsDueToEnd({
  silentMilliseconds,
  silentEndMilliseconds,
  graceSeconds,
  command,
}: {
  readonly silentMilliseconds: number;
  readonly silentEndMilliseconds: number;
  readonly graceSeconds: number;
  readonly command?: SilentCommand
},): boolean {
  if ((command === undefined) || (silentMilliseconds < silentEndMilliseconds)) {
    return false;
  }
  /**
   How long this command may run: the second stated time, or its own declared bound plus the grace, whichever is later.
   */
  const dueAt = (command.boundSeconds === undefined)
    ? silentEndMilliseconds
    : Math.max(
      silentEndMilliseconds,
      (command.boundSeconds + graceSeconds) * MILLISECONDS_PER_SECOND,
    );
  return command.milliseconds >= dueAt;
}

/**
 Keeps a command as a feed line and a summary hold it: its arguments cut to the length one line carries, and the bound it declared
 when it declared one.

 @param command - the command in flight

 @returns the command as it is written down

 @example
 ```ts
 const kept = keptCommand({ command: { tool: 'bash', args: { command: 'rg -l x' }, milliseconds: 700_000 }, },);
 ```
 */
export function keptCommand({ command, }: { readonly command: SilentCommand },): SilentCommand {
  return {
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
 What the agent is told when the command it was sitting in is ended: which command, after how long, and that a command which may
 run long needs a bound of its own.

 @param command - the command the driver ended

 @param silentMilliseconds - how long the run had said nothing

 @returns the message the agent's next turn begins with

 @example
 ```ts
 const said = endedCommandMessageOf({ command, silentMilliseconds: 3_600_000, },);
 ```
 */
export function endedCommandMessageOf({
  command,
  silentMilliseconds,
}: {
  readonly command: SilentCommand;
  readonly silentMilliseconds: number
},): string {
  /**
   The minutes the run had said nothing, which is the figure a reader of a long run thinks in, and nothing under a minute where
   the milliseconds are the whole figure.
   */
  const minutes = silentMilliseconds / (MILLISECONDS_PER_SECOND * SECONDS_PER_MINUTE);
  return [
    `The driver ended the command you were running: ${command.tool} \`${preview({
      value: command.args,
      length: FEED_PREVIEW_CHARACTERS.toolArguments,
    },)}\``,
    `It had said nothing for ${String(silentMilliseconds,)} ms`
      + `${(minutes < 1) ? '' : ` (${minutes.toFixed(1,)} minutes)`}, and had run `
      + `${String(command.milliseconds,)} ms when the driver ended it.`,
    (command.boundSeconds === undefined)
      ? 'It declared no bound of its own.'
      : `It declared a bound of ${String(command.boundSeconds,)} seconds for itself and was still going `
        + `${String(SILENT_BOUND_GRACE_SECONDS,)} seconds past that bound.`,
    'Its answer ends as aborted where the driver stopped it, and that answer is what you have of the command.',
    'A command that may run long needs a bound of its own: give the bash tool its `timeout` argument, in seconds, and the driver '
      + 'leaves the command alone until that bound has passed. A bound buys time rather than silence, because a command whose own '
      + 'bound could not end it is ended by the driver a minute past it, and nothing outlives the run\'s wall clock.',
    'Go on from here: read what the command left, and give a bound to any command you run again that may take long.',
  ].join('\n',);
}

/**
 Writes a stretch of silence down the first time it is seen, so a reader of the feed learns what the run is sitting in without
 waiting for that to end.

 @param watch - watch request

 @param command - the command in flight, when one is

 @param silentMilliseconds - how long the run has said nothing

 @param waited - where the driver stands with this run's silence, which this fills
 */
function noticeTheSilence({
  watch,
  command,
  silentMilliseconds,
  startedAt,
  waited,
}: {
  readonly watch: WatchRequest;
  readonly command?: SilentCommand;
  readonly silentMilliseconds: number;
  readonly startedAt: number;
  readonly waited: SilenceWaitedFor
},): void {
  waited.noticedAt = startedAt;
  watch.record
    .noteSilence({
      silentMilliseconds,
      ...(command === undefined ? {} : { command: keptCommand({ command, }), }),
    },);
}

/**
 Why one command did not go through, for the line that says whether the agent was told.

 @param answer - how waiting for Pi's answer ended

 @param what - which command this was, for the message

 @returns the reasons, empty when the command went through

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
 Ends the command a silence is sitting in, the way the wall clock ends one, and sends the agent a message of its own saying what
 was ended, after how long, and that a command which may run long needs a bound of its own.

 @param watch - watch request

 @param command - the command to end

 @param silentMilliseconds - how long the run has said nothing

 @returns why the run cannot go on, or that it goes on

 @example
 ```ts
 const after = await endWhatSaidNothing({ watch, command, silentMilliseconds, },);
 ```
 */
async function endWhatSaidNothing({
  watch,
  command,
  silentMilliseconds,
}: {
  readonly watch: WatchRequest;
  readonly command: SilentCommand;
  readonly silentMilliseconds: number
},): Promise<RunProgress> {
  /**
   Pi's answer to the abort, which ends the operation in flight and the command with it.
   */
  const aborted = await watch.connection
    .send({
      command: { type: 'abort', },
      deadline: watch.deadline,
    },);
  /**
   What the agent is told, which arrives as a turn of its own: the abort leaves Pi idle, so a prompt starts that turn rather than
   queueing behind the command.
   */
  const told = await watch.connection
    .send({
      command: {
        type: 'prompt',
        message: endedCommandMessageOf({
          command,
          silentMilliseconds,
        },),
      },
      deadline: watch.deadline,
    },);
  /**
   Why the abort or the message did not go through, when one did not.
   */
  const problems = [
    ...problemsOf({
      answer: aborted,
      what: 'the abort',
    },),
    ...problemsOf({
      answer: told,
      what: 'the message',
    },),
  ];
  watch.record
    .noteSilenceEnded({
      silentMilliseconds,
      command: keptCommand({ command, }),
      ended: true,
      told: takenByPi(told,),
      ...((problems.length === 0) ? {} : { failure: problems.join('; ',), }),
    },);
  // A message Pi refused woke nobody, so the agent still does not know what became of its command and the run cannot go on as one
  // whose agent was told.
  return refusedByPi(told,) ? 'prompt-rejected' : 'still-running';
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
   The command the run is sitting in, when it is sitting in one: the one started last and not yet ended.
   */
  const command = watch.record
    .commandsInFlight()
    .at(-1,);
  /**
   When this stretch of silence started at.
   */
  const startedAt = watch.record
    .silenceStartedAt();
  if (waited.noticedAt !== startedAt) {
    noticeTheSilence({
      watch,
      ...(command === undefined ? {} : { command, }),
      silentMilliseconds,
      startedAt,
      waited,
    },);
  }
  /**
   Whether this silence is past the point where the driver ends the command.
   */
  const dueToEnd = (command !== undefined) && aSilentCommandIsDueToEnd({
    silentMilliseconds,
    silentEndMilliseconds: watch.options
      .silentEndSeconds
      * MILLISECONDS_PER_SECOND,
    graceSeconds: SILENT_BOUND_GRACE_SECONDS,
    command,
  },);
  if ((!dueToEnd) || (command === undefined)) {
    return 'still-running';
  }
  return await endWhatSaidNothing({
    watch,
    command,
    silentMilliseconds,
  },);
}
