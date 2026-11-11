/**
 Taking a run from a started Pi to the reason it ended: what Pi starts with, the first prompt, and the watch that follows it.

 @module
 */

import { join, } from 'node:path';
import type { ConnectionToPi, } from './connection-to-pi.ts';
import type {
  CommandAnswer,
  DriverOptions,
  EndReason,
  PiRecord,
  ProviderAndModel,
  RecordOfPiRun,
  RunProgress,
} from './driver-types.ts';
import { millisecondsLeftUntil, } from './deadline.ts';
import { endReasonOfPromptWait, } from './end-reasons.ts';
import { preview, } from './previews-of-pi-messages.ts';
import { readOrdinaryFile, } from './read-ordinary-file.ts';
import {
  fieldsOf,
  recordsIn,
} from './read-pi-fields.ts';
import { promptAfterAProviderSwitch, } from './switch-provider-on-a-spent-quota.ts';
import type { WatchRequest, } from './watch-request.ts';
import { watchUntilEnded, } from './watch-pi-run.ts';

/**
 What the first prompt of a Pi says: the task the run was started with, or the work a provider switch left unfinished.
 */
export type PromptToSend =
  | {
    readonly kind: 'the-prompt-file'
  }
    | {
      readonly kind: 'after-a-provider-switch';

    /**
     What the model said when its message ended in the error the switch answers.
     */
      readonly problem: string;

    /**
     The provider the run left.
     */
      readonly from: string;

    /**
     The provider and model this Pi runs on.
     */
      readonly to: ProviderAndModel
    };

/**
 The task this run was started with, which is the first prompt of a Pi the run has not moved.
 */
export const THE_PROMPT_FILE: PromptToSend = Object.freeze({ kind: 'the-prompt-file', },);

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

 @param connection - running Pi

 @param record - run's record
 */
async function noteStartingState({
  connection,
  record,
  deadline,
}: WatchRequest,): Promise<void> {
  /**
   Pi's state, waited for only as long as the run has left.
   */
  const state = answeredDataOf(await connection.send({
    command: { type: 'get_state', },
    deadline,
  },),);
  record.note({
    kind: 'state',
    model: fieldsOf(state.model,)
      .id,
    thinkingLevel: state.thinkingLevel,
  },);
  /**
   Pi's commands.
   */
  const { commands, } = answeredDataOf(await connection.send({
    command: { type: 'get_commands', },
    deadline,
  },),);
  record.note({
    kind: 'commands',
    names: recordsIn(commands,)
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
};

/**
 The text of the first prompt this Pi is given.

 @param prompt - what the prompt says

 @param promptFile - the file the task was written to

 @returns the prompt's text
 */
async function firstPromptText({
  prompt,
  promptFile,
}: {
  readonly prompt: PromptToSend;
  readonly promptFile: string
},): Promise<string> {
  if (prompt.kind === 'the-prompt-file') {
    // The prompt, which must be an ordinary file: a named pipe would wait for a writer.
    return await readOrdinaryFile(promptFile,);
  }
  return promptAfterAProviderSwitch({
    problem: prompt.problem,
    from: prompt.from,
    to: prompt.to,
  },);
}

/**
 Sends the first prompt and notes what Pi made of it.

 @param watch - what the run needs

 @param prompt - what this Pi's first prompt says

 @returns why the run is already over, or that it goes on
 */
async function sendFirstPrompt({
  watch,
  prompt,
}: {
  readonly watch: WatchRequest;
  readonly prompt: PromptToSend
},): Promise<ProgressAfterPrompt> {
  /**
   What the run needs, read once for the send and the notes about it.
   */
  const {
    connection,
    record,
    options,
    deadline,
  } = watch;
  /**
   The prompt, read before Pi is given anything so that one the run cannot read fails it here.
   */
  const text = await firstPromptText({
    prompt,
    promptFile: options.promptFile,
  },);
  /**
   Pi's answer, waited for within the run's own deadline.
   */
  const answer = await connection.send({
    command: {
      type: 'prompt',
      message: text,
    },
    deadline,
  },);
  /**
   What Pi did with the prompt, or what the driver's own bound named instead.
   */
  const { disposition, } = notePromptAnswer({
    answer,
    record,
    chars: text.length,
  },);
  if (disposition === 'handled') {
    record.countHandledPrompt();
  }
  if ((answer.ending === 'answered') && (!answeredRefusal(answer,))) {
    return { endReason: 'still-running', };
  }
  return {
    endReason: endReasonOfPromptWait({
      answer,
      passedDeadline: millisecondsLeftUntil(deadline,) === 0,
    },),
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
 Why the run is over.
 */
export type Ending = {
  readonly endReason: EndReason;
};

/**
 Takes a run from a started Pi to its end. A failure in the driver itself still ends in a summary and a stopped Pi.

 @param watch - what the run needs

 @param prompt - what this Pi's first prompt says

 @returns why the run ended

 @example
 ```ts
 const ending = await endReasonOfRun({ watch, prompt: THE_PROMPT_FILE, },);
 ```
 */
export async function endReasonOfRun({
  watch,
  prompt,
}: {
  readonly watch: WatchRequest;
  readonly prompt: PromptToSend
},): Promise<Ending> {
  try {
    if (millisecondsLeftUntil(watch.deadline,) === 0) {
      return { endReason: 'wall-clock', };
    }
    await noteStartingState(watch,);
    /**
     Where the run stands once the prompt is sent, which may already be its end.
     */
    const { endReason, } = await sendFirstPrompt({
      watch,
      prompt,
    },);
    if (endReason !== 'still-running') {
      return { endReason, };
    }
    return { endReason: await watchUntilEnded(watch,), };
  }
  catch (error) {
    watch.record
      .noteDriverFailure(String(error,),);
    return { endReason: 'driver-failed', };
  }
}
