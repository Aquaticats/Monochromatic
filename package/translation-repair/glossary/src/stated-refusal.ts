//region Stated refusal
// A REFUSAL A COMMAND STATES IN ITS OWN WORDS, and may therefore repeat.
//
// Every entry point goes through `reportingRefusals`, which drops the
// message of any class that has not declared it quote-free. That was the right
// default and it broke something real: twelve throw sites across ten CLIs
// carried usage lines, missing-key lines and probe refusals, and every one of
// them turned into `refused by Error` plus a stack. Those are the messages an
// operator needs MOST, because they are the ones that say what to type next.
//
// WHAT MAY GO IN ONE. Sentences we wrote, arguments the operator supplied
// (flags, paths, draw names), environment variable NAMES, and numbers this
// process computed. Every part is either authored here or handed in by the
// person reading the output.
//
// WHAT MAY NOT. Anything read out of a corpus file, a run file, a model's
// answer or a provider's response body. `model-catalog.ts` had one message
// interpolating an HTTP `statusText`, which is the provider's wording rather
// than ours; the status code says the same thing, so the message was narrowed
// to the code rather than the marker widened to admit it. That direction is the
// point: a message that cannot meet the rule gets rewritten, not exempted.
//
// This is deliberately NOT the same claim as "audited and found clean". A class
// carries the marker because of how its message is BUILT, which stays true as
// the code changes, rather than because someone read it once.

/**
 A refusal whose message this package wrote, and may therefore repeat.

 SEPARATE FROM A FAULT. `reportingRefusals` reports one of these as the
 command declining to run, with no frames and no talk of a bug, because
 nothing here is broken: a usage line is an answer, not a crash.

 @example
 ```ts
 throw new StatedRefusalError({
   says: 'name at least one log file: spend-report <path> [<path> ...]',
 },);
 ```
 */
export class StatedRefusalError extends Error {
  /**
   Declares this message safe to forward: every part of it is either a
   sentence written here or a value the operator handed in.
   */
  readonly messageNamesOnly: true = true;

  /**
   @param says - whole message, composed only of authored words, operator
   arguments, environment variable names and numbers computed here

   @param cause - failure the refusal answers, kept for whoever inspects the
   refusal and never repeated in the message, since a caught failure's own
   words are nothing this package wrote
   */
  constructor(
    {
      says,
      cause,
    }: {
      readonly says: string;
      readonly cause?: unknown;
    },
  ) {
    super(
      says,
      // Conditional spread keeps cause absent when none was supplied.
      ...((cause === undefined) ? [] : [{ cause, },]),
    );
    this.name = 'StatedRefusalError';
  }
}

//endregion Stated refusal
