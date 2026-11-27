/**
 Taking a run from a started Pi to the reason it ended: what Pi starts with, the first prompt, and the watch that follows it.

 @module
 */

import { join, } from 'node:path';
import type { ConnectionToPi, } from './connection-to-pi.ts';
import {
  type StartThatEndedWithoutARun,
  type WaitThatEndedAStart,
} from './a-start-that-ended-without-a-run.ts';
import { START_UP_ANSWER_MILLISECONDS, } from './driver-constants.ts';
import type {
  CommandAnswer,
  DriverOptions,
  EndReason,
  PiExit,
  PiRecord,
  RecordOfPiRun,
  RunProgress,
} from './driver-types.ts';
import { millisecondsLeftUntil, } from './deadline.ts';
import { endReasonOfPromptWait, } from './end-reasons.ts';
import { preview, } from './previews-of-pi-messages.ts';
import { maskingIsArmedIn, } from './secret-values-handed-to-pi.ts';
import { readOrdinaryFile, } from './read-ordinary-file.ts';
import {
  fieldsOf,
  recordsIn,
  textOf,
} from './read-pi-fields.ts';
import type { WatchRequest, } from './watch-request.ts';
import { watchUntilEnded, } from './watch-pi-run.ts';

/**
 Tells whether Pi refused the prompt itself.

 @param answer - how waiting for Pi's answer ended

 @returns whether Pi answered with a refusal
 */
function answeredRefusal(answer: CommandAnswer,): boolean {
  if (answer.ending !== 'answered') {
    return false;
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record } = answer;
  return record.success === false;
}

/**
 Pi's answer data, or nothing when Pi answered nothing.

 @param answer - how waiting for the answer ended

 @returns the answer's fields, or an empty object when Pi said nothing
 */
function answeredDataOf(answer: CommandAnswer,): PiRecord {
  if (answer.ending !== 'answered') {
    return {};
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record } = answer;
  return fieldsOf(record.data,);
}

/**
 The three asks a run makes while Pi starts, each named so a reader of a line that ended a start can follow it to the command that
 ran it.

 They are what issue 6556's TimeoutError named none of: a start that ended on one of these said `prompt-rejected` and stopped.
 */
const STATE_ASK = 'Pi\'s answer to `get_state`, the first of the three answers a run waits for while Pi starts';
const COMMANDS_ASK = 'Pi\'s answer to `get_commands`, the second of the three answers a run waits for while Pi starts';
const PROMPT_ASK = 'Pi\'s answer to the run\'s first prompt, the third of the three answers a run waits for while Pi starts';

/**
 Says how Pi ended, in the words a line about a start that ended without a run needs.

 @param exit - how Pi ended

 @returns the ending, in words a reader needs no log for
 */
function exitWordsOf(exit: PiExit,): string {
  if (exit.code !== undefined) {
    return `Pi exited with status ${String(exit.code,)}`;
  }
  if (exit.signal !== undefined) {
    return `Pi left on ${exit.signal}`;
  }
  return 'Pi left';
}

/**
 Says one wait of a start that ended without a run: the ask, what ended it, and how long it had waited.

 @param ask - the ask, in words a reader can follow

 @param answer - how waiting for that ask ended

 @param boundMilliseconds - the silence the ask is given, which a wait Pi went or the run's deadline ended did not reach

 @param waitedMilliseconds - how long the wait had been waiting when it ended

 @returns the wait, as the line and the summary report it
 */
function waitThatEnded({
  ask,
  answer,
  boundMilliseconds,
  waitedMilliseconds,
}: {
  readonly ask: string;
  readonly answer: CommandAnswer;
  readonly boundMilliseconds: number;
  readonly waitedMilliseconds: number;
},): WaitThatEndedAStart {
  if (answer.ending === 'pi-exited') {
    return {
      wait: `${ask}, which ended when Pi went`,
      waitedMilliseconds,
    };
  }
  if (answer.ending === 'deadline') {
    return {
      wait: ask,
      bound: `the run\'s own wall clock, which ended it before the ${String(
        boundMilliseconds,
      )} ms this wait is given`,
      waitedMilliseconds,
    };
  }
  return {
    wait: ask,
    bound: `${String(boundMilliseconds,)} ms`,
    waitedMilliseconds,
  };
}

/**
 Notes the model and commands Pi starts with, which show at a glance what the run is using.

 @param watch - what the run needs

 @returns the command list Pi answered with, of any shape, which is what says whether the masking extension is loaded, how long the
 state ask and the command-list ask each had waited, and whether the state was answered at all
 */
async function noteStartingState({
  connection,
  record,
  deadline,
  startUpAnswerMilliseconds = START_UP_ANSWER_MILLISECONDS,
}: WatchRequest,): Promise<{
  readonly commands: unknown;
  readonly answer: CommandAnswer;
  readonly state: CommandAnswer;
  readonly stateMilliseconds: number;
  readonly commandsMilliseconds: number;

  /**
   The session file Pi named in its own state, when it named one, which is where Pi keeps what this run writes.
   */
  readonly sessionFile?: string;
}> {
  /**
   When the state ask began, which is what the line about a start that never answered it reports.
   */
  const stateAskedAt = Date.now();
  /**
   Pi's state, waited for only as long as the run has left.
   */
  const stateAnswer: CommandAnswer = await connection.send({
    command: { type: 'get_state', },
    answerMilliseconds: startUpAnswerMilliseconds,
    deadline,
  },);
  /**
   How long that wait had been waiting when it ended.
   */
  const stateMilliseconds = Date.now() - stateAskedAt;
  /**
   Pi's state, when it answered.
   */
  const state = answeredDataOf(stateAnswer,);
  /**
   The session file Pi says it is keeping, which is what the run masks once it is over.
   */
  const sessionFile = textOf(state.sessionFile,);
  record.note({
    kind: 'state',
    model: fieldsOf(state.model,)
      .id,
    thinkingLevel: state.thinkingLevel,
    ...((sessionFile === '') ? {} : { sessionFile, }),
  },);
  /**
   When the command-list ask began.
   */
  const commandsAskedAt = Date.now();
  /**
   How waiting for Pi's commands ended: a command list that came back is what says whether the masking of secret values is in
   place, and a wait that gave nothing is a start whose ending the run's own bounds name.
   */
  const answer: CommandAnswer = await connection.send({
    command: { type: 'get_commands', },
    answerMilliseconds: startUpAnswerMilliseconds,
    deadline,
  },);
  /**
   How long that wait had been waiting when it ended.
   */
  const commandsMilliseconds = Date.now() - commandsAskedAt;
  /**
   Pi's commands.
   */
  const { commands, } = answeredDataOf(answer,);
  record.note({
    kind: 'commands',
    names: recordsIn(commands,)
      .map(function nameOf(command,) {
        return command.name;
      },),
  },);
  return {
    commands,
    answer,
    state: stateAnswer,
    stateMilliseconds,
    commandsMilliseconds,
    ...((sessionFile === '') ? {} : { sessionFile, }),
  };
}

/**
 Notes that the masking could not be confirmed, which is a run that sends no prompt.

 @param watch - what the run needs

 @param handedOver - how many values the driver handed Pi

 @param commands - the command list Pi answered with, of any shape, which is empty when it answered none

 @param answered - whether Pi answered its command list at all
 */
function noteMaskingNotArmed({
  watch,
  handedOver,
  commands,
  answered,
}: {
  readonly watch: WatchRequest;
  readonly handedOver: number;
  readonly commands: unknown;
  readonly answered: boolean;
},): void {
  watch.record
    .note({
      kind: 'masking_not_armed',
      handedOver,
      answered,
      commands: recordsIn(commands,)
        .map(function nameOf(command,) {
          return command.name;
        },),
    },);
}

/**
 Where a run stands once its first prompt is sent.
 */
type ProgressAfterPrompt = {
  readonly endReason: RunProgress;

  /**
   How long the prompt's wait had been waiting when it ended, which a line about a start that ended there reports.
   */
  readonly waitedMilliseconds: number;

  /**
   How waiting for the prompt ended, which says whether the bound, Pi's exit, or the run's deadline ended it.
   */
  readonly answer: CommandAnswer;
};

/**
 Sends the first prompt and notes what Pi made of it.

 @param connection - running Pi

 @param record - run's record

 @param options - options of this run

 @returns why the run is already over, or that it goes on
 */
async function sendFirstPrompt({
  connection,
  record,
  options,
  deadline,
  startUpAnswerMilliseconds = START_UP_ANSWER_MILLISECONDS,
}: WatchRequest,): Promise<ProgressAfterPrompt> {
  /**
   The prompt, which must be an ordinary file: a named pipe would wait for a writer.
   */
  const prompt = await readOrdinaryFile(options.promptFile,);
  /**
   When the prompt was sent.
   */
  const askedAt = Date.now();
  /**
   Pi's answer, waited for within the run's own deadline and, until it comes, under the bound a starting Pi gets rather than
   the one an answer owed mid-run gets.
   */
  const answer = await connection.send({
    command: {
      type: 'prompt',
      message: prompt,
    },
    answerMilliseconds: startUpAnswerMilliseconds,
    deadline,
  },);
  /**
   How long that wait had been waiting when it ended.
   */
  const waitedMilliseconds = Date.now() - askedAt;
  /**
   What Pi did with the prompt, or what the driver's own bound named instead.
   */
  const { disposition, } = notePromptAnswer({
    answer,
    record,
    chars: prompt.length,
  },);
  if (disposition === 'handled') {
    record.countHandledPrompt();
  }
  if ((answer.ending === 'answered') && (!answeredRefusal(answer,))) {
    // Pi took the prompt, which is where work may begin: a Pi that leaves after this is never started again.
    record.countPromptTaken();
    return {
      endReason: 'still-running',
      waitedMilliseconds,
      answer,
    };
  }
  return {
    endReason: endReasonOfPromptWait({
      answer,
      passedDeadline: millisecondsLeftUntil(deadline,) === 0,
    },),
    waitedMilliseconds,
    answer,
  };
}

/**
 Notes what Pi made of the prompt on the feed, including the bound or the compaction that ended the wait when Pi answered
 nothing.

 @param answer - how waiting for Pi's answer ended

 @param record - run's record

 @param chars - characters of the prompt sent

 @returns what Pi said it did with the prompt, which is nothing when Pi said nothing at all
 */
function notePromptAnswer({
  answer,
  record,
  chars,
}: {
  readonly answer: CommandAnswer;
  readonly record: RecordOfPiRun;
  readonly chars: number
},): { readonly disposition: unknown } {
  if (answer.ending !== 'answered') {
    record.note({
      kind: 'prompt_sent',
      success: false,
      // A wait the driver ended names its own bound or the compaction it waited on, which is why Pi refused nothing.
      error: answer.problem,
      chars,
    },);
    return { disposition: undefined, };
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record: answered } = answer;
  /**
   What Pi did with the prompt.
   */
  const { disposition, } = fieldsOf(answered.data,);
  record.note({
    kind: 'prompt_sent',
    success: answered.success ?? false,
    disposition,
    error: answered.error ?? null,
    chars,
  },);
  return { disposition, };
}


/**
 Why the run is over, and what a start that ended without a run has to say about it.
 */
export type Ending = {
  readonly endReason: EndReason;

  /**
   The session file Pi named, when the run got as far as asking it, which is what the run masks once it is over.
   */
  readonly sessionFile?: string;

  /**
   What the start has to say when it ended without a run, which is absent for a run whose prompt Pi took.
   */
  readonly startThatEndedWithoutARun?: StartThatEndedWithoutARun;
};

/**
 Says what a start that never got Pi's command list has to say, which is why no prompt was sent.

 @param starting - what Pi answered while it started

 @param startUpAnswerMilliseconds - the bound each of the three answers is given

 @param handedOver - how many values of this machine's secret environment the driver handed Pi

 @returns the note for the summary and the line the start leaves
 */
function commandsNoteOf({
  starting,
  startUpAnswerMilliseconds,
  handedOver,
  exit,
}: {
  readonly starting: Awaited<ReturnType<typeof noteStartingState>>;
  readonly startUpAnswerMilliseconds: number;
  readonly handedOver: number;
  readonly exit: PiExit;
},): StartThatEndedWithoutARun {
  /**
   Every ask that went unanswered before the start ended, in the order they were made.
   */
  const waits: WaitThatEndedAStart[] = [
    ...((starting.state.ending === 'answered')
      ? []
      : [waitThatEnded({
        ask: STATE_ASK,
        answer: starting.state,
        boundMilliseconds: startUpAnswerMilliseconds,
        waitedMilliseconds: starting.stateMilliseconds,
      },)]),
    waitThatEnded({
      ask: COMMANDS_ASK,
      answer: starting.answer,
      boundMilliseconds: startUpAnswerMilliseconds,
      waitedMilliseconds: starting.commandsMilliseconds,
    },),
  ];
  /**
   What ended the last of them, which says what to do about it.
   */
  const last = starting.answer;
  if (last.ending === 'pi-exited') {
    return {
      waits,
      what: `${exitWordsOf(exit,)} before it answered its command list, so no prompt was sent`,
      next: 'Read what Pi printed to its standard error, which is where an extension that fails to load, a credential store it cannot read, and a module it cannot find all say so, and start the run again: this is the failure a start is made again after. The run\'s stderr.log holds those lines, and its feed.jsonl holds each start the driver made.',
    };
  }
  return {
    waits,
    what: `Pi never answered its command list, so the driver could not tell whether the extension that masks this machine\'s secret values was loaded, and it sent no prompt (${String(
      handedOver,
    )} values had been handed to Pi to mask)`,
    next: `Start the run again when the host is quieter: Pi is loading its session and its extensions while these three answers wait, and a host under load can spend longer than the bound on them. If the same wait runs out on a quiet host, run \`pi --mode rpc\` by hand in the run\'s worktree with the provider and model the run names, and watch its standard error: an extension that fails to load, a credential store it cannot read, and a session larger than its model\'s window each show there. The run\'s feed.jsonl holds the lines this start wrote, and its stderr.log holds what Pi printed.`,
  };
}

/**
 Says what a start whose Pi answered a command list without the masking in place has to say.

 @param starting - what Pi answered while it started

 @returns the note for the summary and the line the start leaves
 */
function maskingNoteOf({
  starting,
  handedOver,
}: {
  readonly starting: Awaited<ReturnType<typeof noteStartingState>>;
  readonly handedOver: number;
},): StartThatEndedWithoutARun {
  return {
    waits: [{
      wait: 'the command list Pi answers `get_commands` with, which says whether the extension that masks this machine\'s secret values is loaded',
      waitedMilliseconds: starting.commandsMilliseconds,
    },],
    what: `Pi answered its command list and no command in it armed the masking with the ${String(
      handedOver,
    )} values the driver handed it, so the driver sent no prompt`,
    next: 'Read the run\'s stderr.log: an extension that did not load, a run given `--no-extensions`, and the masking extension\'s entry missing beside the driver each show there. Start the run again once the extension loads, or pass `--pi-argument` for the extension folder that holds it.',
  };
}

/**
 Says what a start ended by its prompt ask has to say, which is the third of the three answers or the bound that replaced it.

 @param answer - how waiting for the prompt ended

 @param waitedMilliseconds - how long that wait had been waiting

 @param startUpAnswerMilliseconds - the bound the prompt is given

 @returns the note for the summary and the line the start leaves
 */
function promptNoteOf({
  answer,
  waitedMilliseconds,
  startUpAnswerMilliseconds,
  exit,
}: {
  readonly answer: CommandAnswer;
  readonly waitedMilliseconds: number;
  readonly startUpAnswerMilliseconds: number;
  readonly exit: PiExit;
},): StartThatEndedWithoutARun {
  /**
   The prompt's own wait, as the line reports it.
   */
  const waited = waitThatEnded({
    ask: PROMPT_ASK,
    answer,
    boundMilliseconds: startUpAnswerMilliseconds,
    waitedMilliseconds,
  },);
  if (answer.ending === 'pi-exited') {
    return {
      waits: [waited,],
      what: `${exitWordsOf(exit,)} before it took the prompt, so the run never began`,
      next: 'Read what Pi printed to its standard error, which is where a load failure, a refused provider and a missing module all say why: the run\'s stderr.log holds those lines, and its feed.jsonl holds each start the driver made.',
    };
  }
  if (answer.ending === 'deadline') {
    return {
      waits: [waited,],
      what: 'the run\'s own wall clock ended the wait for its first prompt, so the run never began',
      next: 'Give the run a longer `--wall-seconds`, which bounds Pi\'s start-up as well as the run, or start it when the host is quieter: the feed\'s last lines say what Pi was doing when the clock ran out.',
    };
  }
  return {
    waits: [waited,],
    what: 'Pi answered its state and its command list with the masking in place, and then took no prompt, so the run never began',
    next: `Start the run again when the host is quieter: loading the session Pi was given is part of what this wait covers, and a session larger than its model\'s window makes Pi compact before it takes the prompt. If the same wait runs out on a quiet host, read the run\'s feed.jsonl for a compaction line and its stderr.log for what Pi printed.`,
  };
}

/**
 Takes a run from a started Pi to its end. A failure in the driver itself still ends in a summary and a stopped Pi.

 @param watch - what the run needs

 @returns why the run ended, and what a start that ended without a run has to say

 @example
 ```ts
 const ending = await endReasonOfRun(watch,);
 ```
 */
export async function endReasonOfRun(watch: WatchRequest,): Promise<Ending> {
  try {
    if (millisecondsLeftUntil(watch.deadline,) === 0) {
      return { endReason: 'wall-clock', };
    }
    /**
     The bound each of the three answers a run waits for while Pi starts is given.
     */
    const startUpAnswerMilliseconds = watch.startUpAnswerMilliseconds ?? START_UP_ANSWER_MILLISECONDS;
    /**
     Pi's own state and command list, and whether the masking of secret values is in place before any work begins.
     */
    const starting = await noteStartingState(watch,);
    /**
     The session file Pi named, which is what the run masks once it is over.
     */
    const namedSession = starting.sessionFile;
    /**
     How many values the driver handed Pi, when this run handed any.
     */
    const handedOver = watch.secretValuesHandedOver;
    if ((handedOver !== undefined) && (starting.answer
      .ending
      !== 'answered')) {
      // No prompt without the answer that says the masking is in place. The run ends for the reason that kept the answer from
      // coming, which is what a reader has to fix.
      noteMaskingNotArmed({
        watch,
        handedOver,
        commands: starting.commands,
        answered: false,
      },);
      return {
        endReason: endReasonOfPromptWait({
          answer: starting.answer,
          passedDeadline: millisecondsLeftUntil(watch.deadline,) === 0,
        },),
        ...(namedSession === undefined ? {} : { sessionFile: namedSession, }),
        startThatEndedWithoutARun: commandsNoteOf({
          starting,
          startUpAnswerMilliseconds,
          handedOver,
          exit: watch.connection.exitDetails(),
        },),
      };
    }
    if ((handedOver !== undefined) && (!maskingIsArmedIn({
      commands: starting.commands,
      count: handedOver,
    },))) {
      noteMaskingNotArmed({
        watch,
        handedOver,
        commands: starting.commands,
        answered: true,
      },);
      return {
        endReason: 'masking-not-armed',
        ...(namedSession === undefined ? {} : { sessionFile: namedSession, }),
        startThatEndedWithoutARun: maskingNoteOf({
          starting,
          handedOver,
        },),
      };
    }
    /**
     Where the run stands once its first prompt is sent, which may already be its end.
     */
    const { endReason, waitedMilliseconds, answer, } = await sendFirstPrompt(watch,);
    if (endReason !== 'still-running') {
      return {
        endReason,
        ...(namedSession === undefined ? {} : { sessionFile: namedSession, }),
        startThatEndedWithoutARun: promptNoteOf({
          answer,
          waitedMilliseconds,
          startUpAnswerMilliseconds,
          exit: watch.connection.exitDetails(),
        },),
      };
    }
    return {
      endReason: await watchUntilEnded(watch,),
      ...(namedSession === undefined ? {} : { sessionFile: namedSession, }),
    };
  }
  catch (error) {
    watch.record
      .noteDriverFailure(String(error,),);
    return { endReason: 'driver-failed', };
  }
}
