/**
 Opening sentence of each wording reminder.

 Each reminder starts with one sentence that names its trigger. The decision emits that
 sentence, and the once-per-turn scan in `reminder-history.ts` looks for the same sentence in
 the transcript, so both read it from here and cannot drift apart.

 @module
 */

/**
 Opening sentence of the reminder for an uncertainty phrase.

 @param phrase - phrase that matched

 @returns the sentence the reminder opens with

 @example
 ```ts
 uncertaintyLead('probably'); // 'Your response contains uncertain language ("probably").'
 ```
 */
function uncertaintyLead(phrase: string,): string {
  return `Your response contains uncertain language ("${phrase}").`;
}

/**
 Opening sentence of the reminder for an uncited categorical dismissal.

 @param phrase - dismissal phrase that matched

 @returns the sentence the reminder opens with

 @example
 ```ts
 dismissalLead("project doesn't use"); // 'Your response contains an uncited categorical dismissal ("project doesn\'t use").'
 ```
 */
function dismissalLead(phrase: string,): string {
  return `Your response contains an uncited categorical dismissal ("${phrase}").`;
}

/**
 Opening sentence of the reminder for a trailing question to the user.

 @param sentence - the question sentence that matched

 @returns the sentence the reminder opens with

 @example
 ```ts
 questionLead('Want me to commit?'); // 'Your response ends with a question to the user ("Want me to commit?").'
 ```
 */
function questionLead(sentence: string,): string {
  return `Your response ends with a question to the user ("${sentence}").`;
}

export {
  dismissalLead,
  questionLead,
  uncertaintyLead,
};
