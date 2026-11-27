/**
 Taking a run from a started Pi to the reason it ended: what Pi starts with, the first prompt, and the watch that follows it.

 @module
 */

import { join, } from 'node:path';
import type { ConnectionToPi, } from './connection-to-pi.ts';
import type { StartThatEndedWithoutARun, } from './a-start-that-ended-without-a-run.ts';
import {
  commandsNoteOf,
  maskingNoteOf,
  promptNoteOf,
} from './notes-of-a-start-that-ended-without-a-run.ts';
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
 Notes the model and commands Pi starts with, which show at a glance what the run is using.

 @param connection - the running Pi

 @param record - the run's feed and counts

 @param deadline - when the run must be over

 @param startUpAnswerMilliseconds - the bound each of the three answers is given

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
          exit: watch.connection
            .exitDetails(),
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
    const {
      endReason,
      waitedMilliseconds,
      answer,
    } = await sendFirstPrompt(watch,);
    if (endReason !== 'still-running') {
      return {
        endReason,
        ...(namedSession === undefined ? {} : { sessionFile: namedSession, }),
        startThatEndedWithoutARun: promptNoteOf({
          answer,
          waitedMilliseconds,
          startUpAnswerMilliseconds,
          exit: watch.connection
            .exitDetails(),
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
