/**
 What Pi made of a message the driver sent: whether it took it, whether it refused it, what it said it did with it, and what the
 feed keeps of its answer.

 One place for these four readings, because three senders ask them: the wake that sends an agent back to work it left running, the
 attempt after a model error, and the ask of an agent whose last answer said nothing.

 @module
 */

import type { CommandAnswer, } from './driver-types.ts';
import {
  fieldsOf,
  textOf,
} from './read-pi-fields.ts';

/**
 Tells whether Pi took a message.

 @param answer - how waiting for Pi's answer ended

 @returns whether Pi answered that it took it

 @example
 ```ts
 const taken = takenByPi({ ending: 'answered', record: { success: true }, },);
 ```
 */
export function takenByPi(answer: CommandAnswer,): boolean {
  if (answer.ending !== 'answered') {
    return false;
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record, } = answer;
  return record.success === true;
}

/**
 Tells whether Pi refused the message itself, which is no answer and no turn started.

 @param answer - how waiting for Pi's answer ended

 @returns whether Pi answered with a refusal

 @example
 ```ts
 const refused = refusedByPi({ ending: 'answered', record: { success: false } },);
 ```
 */
export function refusedByPi(answer: CommandAnswer,): boolean {
  if (answer.ending !== 'answered') {
    return false;
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record, } = answer;
  return record.success === false;
}

/**
 What Pi said it did with a prompt, or nothing when Pi said nothing.

 @param answer - how waiting for Pi's answer ended

 @returns what Pi said, or nothing

 @example
 ```ts
 const disposition = dispositionOf({ ending: 'answered', record: { data: { disposition: 'handled' } }, },);
 ```
 */
export function dispositionOf(answer: CommandAnswer,): unknown {
  if (answer.ending !== 'answered') {
    return undefined;
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record, } = answer;
  /**
   What Pi did with the prompt.
   */
  const { disposition, } = fieldsOf(record.data,);
  return disposition;
}

/**
 What the feed says of a message: Pi's own word for a refusal, or the bound that ended the wait.

 @param answer - how waiting for Pi's answer ended

 @returns what the feed keeps

 @example
 ```ts
 const said = outcomeTextOf({ ending: 'silence', problem: 'no answer' },);
 ```
 */
export function outcomeTextOf(answer: CommandAnswer,): unknown {
  if (answer.ending !== 'answered') {
    return answer.problem;
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record, } = answer;
  return record.error ?? null;
}

/**
 The words a refusal uses when Pi had begun a turn as the message arrived.

 Pi 1.5.2 throws "Agent is already processing. Specify streamingBehavior ('steer' or 'followUp') to queue the message." from
 `prompt` when it is streaming (`dist/core/agent-session.js`), and the RPC mode answers the command with that text as its error;
 the package's own stand-in says "agent is already running". Both name the same fact, and a refusal that names it is no refusal of
 the message: the same message goes through once Pi rests.
 */
const WORDS_OF_A_WORKING_PI: readonly string[] = [
  'already processing',
  'already running',
];

/**
 Tells whether a refusal says Pi was at work as the message arrived.

 @param said - what Pi said when it refused

 @returns whether the words name a Pi that had begun a turn

 @example
 ```ts
 const working = refusalNamesPiAtWork('Agent is already processing. Specify streamingBehavior ...');
 ```
 */
export function refusalNamesPiAtWork(said: string,): boolean {
  /**
   The words in one case, so a provider or a Pi that capitalises differently is still read.
   */
  const lowered = said.toLowerCase();
  return WORDS_OF_A_WORKING_PI.some(function namesWork(words,) {
    return lowered.includes(words,);
  },);
}

/**
 What a message the driver sent Pi came to, read from the state Pi was left in after it was sent.

 `taken` is Pi's word that it has the message; `at-work` is a refusal Pi gave while it was working, which is no answer about the
 message at all, because Pi refuses every prompt handed to it at work and takes the same message once it rests; `refused` is
 Pi's own refusal while it rested; and `unanswered` is the driver's bound running out, or Pi leaving, before any answer.

 Every send the driver makes reads this, so an at-work refusal is never an ending and never a loss: it is the same rule for the
 ask of an agent whose answer said nothing, the wake that sends an agent back to work it left, a message from the inbox, and the
 prompt of an attempt after a model error.
 */
export type MessageToPi =
  | { readonly ending: 'taken' }
  | { readonly ending: 'at-work' }
  | { readonly ending: 'refused' }
  | { readonly ending: 'unanswered'; readonly problem: string };

/**
 Reads what a message the driver sent Pi came to.

 @param answer - how waiting for Pi's answer ended

 @param piWorks - whether Pi was working when the answer was read, which is one of the two things a refusal is judged by

 @returns what the message came to

 @example
 ```ts
 const cameTo = messageToPiCameTo({ answer: { ending: 'answered', record: { success: false } }, piWorks: true },);
 ```
 */
export function messageToPiCameTo({
  answer,
  piWorks,
}: {
  readonly answer: CommandAnswer;
  readonly piWorks: boolean
},): MessageToPi {
  if (answer.ending !== 'answered') {
    return {
      ending: 'unanswered',
      problem: answer.problem,
    };
  }
  if (takenByPi(answer,)) {
    return { ending: 'taken', };
  }
  /**
   What Pi said for refusing, which names a Pi at work in the words Pi 1.5.2 uses for it.
   */
  const said = textOf(answer.record
    .error,);
  if (piWorks || refusalNamesPiAtWork(said,)) {
    return { ending: 'at-work', };
  }
  return { ending: 'refused', };
}
