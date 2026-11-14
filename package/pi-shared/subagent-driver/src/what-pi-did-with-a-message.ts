/**
 What Pi made of a message the driver sent: whether it took it, whether it refused it, what it said it did with it, and what the
 feed keeps of its answer.

 One place for these four readings, because three senders ask them: the wake that sends an agent back to work it left running, the
 attempt after a model error, and the ask of an agent whose last answer said nothing.

 @module
 */

import type { CommandAnswer, } from './driver-types.ts';
import { fieldsOf, } from './read-pi-fields.ts';

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

 @param piWorks - whether Pi was working when the answer was read, which is the state a refusal is judged by

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
  return piWorks ? { ending: 'at-work', } : { ending: 'refused', };
}
