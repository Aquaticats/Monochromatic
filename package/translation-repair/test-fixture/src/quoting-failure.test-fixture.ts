//region Quoting failure
// A FAILURE WHOSE OWN MESSAGE QUOTES A CREDENTIAL, for cases that prove a log
// line, a finding or a stored record repeats a caught value's class and never
// its words. The shape is the one a runtime `fetch` rejection takes for a
// header it cannot send: an unmarked built-in class whose message holds the
// header's value. Cat-themed invention; the value is no real key.

/**
 Stand-in credential the failure's message quotes.
 */
export const WHISKER_KEY = 'whisker-key-7421';

/**
 Builds an unmarked built-in failure whose message quotes the stand-in
 credential, so a line holding that credential has repeated the message.

 @returns A `TypeError` the way a runtime raises one for an unsendable header

 @example
 ```ts
 const failure = quotingFailure();
 ```
 */
export function quotingFailure(): TypeError {
  return new TypeError(`Invalid value "Bearer ${WHISKER_KEY}" for header "authorization"`,);
}

//endregion Quoting failure
