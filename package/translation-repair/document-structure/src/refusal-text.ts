import {
  errorName,
  isStackOverflow,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Refusal text
// Renders a caught value as text that carries no quoted content.
//
// V8 GIVES A `JSON.parse` REFUSAL A MESSAGE THAT QUOTES THE TEXT IT REFUSED,
// ten characters of it, so forwarding a caught error's message is a disclosure
// decision and not a formatting one. This same mechanism has reached a
// terminal past an absent catch, and through a catch that repeats the
// message.
//
// NOT A SECOND `caughtValueText`. That function answers "what does this value
// say". This one answers "may this value's message be repeated". The house rule
// prefers `caughtValueText` wherever the first question is the one being asked,
// and a site forwarding a parse failure into a log or a stored record is asking
// the second.
//
// A DECLARED FIELD RATHER THAN A SYMBOL. A symbol key would refuse a forged
// marker outright, but `--isolatedDeclarations` cannot emit a computed
// property name (TS9038), and the `Error.isError` gate already rejects every
// plain object a run file could carry. What remains is an Error someone wrote
// the field onto deliberately, which is the same trust a symbol import gives.
//
// A MARKER RATHER THAN A LIST OF TRUSTED CLASSES. A central list has to be
// edited by whoever adds an error class, and the cost of forgetting is that a
// quoting message ships. A class that opts in states the property beside the
// message it builds, where the two can be read against each other.
//
// FAILS CLOSED. An unmarked class, a foreign Error, and a thrown non-Error all
// take the naming branch, so the only way to leak through here is to mark a
// class whose message quotes. Two copies of this module would not share a
// symbol, which costs a message rather than disclosing one.

/**
 Stand-in for a refusal that states no position of its own.

 Shared so a reader meets one spelling across parsers: a YAML refusal and
 an MDX one report position differently, and inventing a phrase per parser
 makes a log line say two things where it means one.
 */
export const NAMED_POSITION_UNSTATED = 'an unstated position';

/**
 Error declaring its own message free of quoted content.
 */
export type NamingError = Error & { readonly messageNamesOnly: true; };

/**
 Decides whether a caught value's message may be repeated.

 @param value - caught value, of unknown type by construction

 @returns Whether it declares its message free of quoted content

 @example
 ```ts
 if (namesWithoutQuoting(error,))
   console.error(error.message,);
 ```
 */
export function namesWithoutQuoting(value: unknown,): value is NamingError {
  return Error.isError(value,)
    && ('messageNamesOnly' in value)
    && (value.messageNamesOnly === true);
}

/**
 Renders a caught value for a log line or a stored record, quoting nothing.

 @param error - caught value, of unknown type by construction

 @returns Its message where it declares one safe, its class name otherwise

 @example
 ```ts
 console.error(`${what}: ${refusalText({ error, },)}`,);
 ```
 */
export function refusalText(
  { error, }: { readonly error: unknown; },
): string {
  if (namesWithoutQuoting(error,))
    return error.message;

  // THE ENGINE'S STACK EXHAUSTION IS NAMED IN PLAIN WORDS: its message is fixed engine text that quotes nothing, and the
  // class name `RangeError` alone does not tell a reader what ran out.
  if (isStackOverflow(error,))
    return 'refused because the engine ran out of call stack space';

  return `refused by ${errorName({ error, },)}`;
}

//endregion Refusal text
