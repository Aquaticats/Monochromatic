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
