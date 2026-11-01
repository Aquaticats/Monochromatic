import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';

//region Window trial witness
// The one check no unit test can make: did the window reach the LIVE judges?
//
// `translate-slice.unit.test.ts` and `window-trial-slice.unit.test.ts` both pin
// the forwarding against a synthetic client, which proves the code path exists
// and is wired. Neither can prove that in THIS run, with this roster and this
// prompt renderer, the label went out on the wire. A wide arm that quietly
// judged narrow evidence would not fail; it would return a clean null, and a
// clean null is exactly what the trial is trying to distinguish from a real one.
//
// So the run watches its own first purchase, then stops watching. The cost is
// one wrapper and a few kilobytes of retained sheets.

/**
 Opening of the label the wide arm's sheets carry.

 SPELLED OUT HERE RATHER THAN IMPORTED FROM THE PROMPT, deliberately. This
 checks the wire, not the code, and a check that reads its expectation from the
 thing under test cannot fail. Renaming the label in `translate-judge.ts` should
 stop this run on its first slice and make someone look.
 */
export const WINDOW_LABEL = 'SURROUNDING ORIGINAL';

/**
 Live run disagreeing with what the arms claim they sent.

 @example
 ```ts
 throw new WindowEvidenceError({ found: 0, expected: 6, },);
 ```
 */
export class WindowEvidenceError extends Error {
  /**
   Declares this message safe to forward: it names the window and two counts.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming how many sheets carried the window against how many
   the arms bought should have.

   @param found - sheets carrying {@link WINDOW_LABEL}

   @param expected - sheets that should carry it

   @example
   ```ts
   new WindowEvidenceError({ found: 0, expected: 6, },);
   ```
   */
  public constructor(
    {
      found,
      expected,
    }: {
      readonly found: number;
      readonly expected: number;
    },
  ) {
    super(
      `${String(found,)} of the judge sheets carried ${WINDOW_LABEL} where `
        + `${String(expected,)} should have. The wide arm is the only thing this `
        + `trial varies, so if it did not carry the window then all three arms saw `
        + `the same evidence and every row bought after this point would report a `
        + `false null. Nothing has been spent beyond the first slice.`,
    );
    this.name = 'WindowEvidenceError';
  }
}

/**
 Client recording every sheet it sends, alongside the record.

 @example
 ```ts
 const witness = witnessSheets({ client, },);
 ```
 */
export type SheetWitness = {
  /**
   Stand-in passed to the arms in place of the real client.
   */
  readonly client: ModelClient;

  /**
   Every sheet sent through it, joined per exchange, in order.
   */
  readonly sheets: readonly string[];
};

/**
 Wraps a client so what it sends can be read back.

 PASSES EVERY CALL STRAIGHT THROUGH. This observes, it never substitutes: a
 witness that answered on the model's behalf would make the first slice's rows
 unlike every later row, and those rows are kept.

 @param client - real client every call is forwarded to

 @returns Wrapper plus growing record of what went out

 @example
 ```ts
 const witness = witnessSheets({ client, },);
 await runSliceArms({ client: witness.client, ... },);
 ```
 */
export function witnessSheets(
  { client, }: ForeignBorrowed<{ readonly client: ModelClient; }>,
): SheetWitness {
  /**
   Sheets seen so far, which the caller reads after the first purchase.
   */
  const sheets: string[] = [];

  return {
    sheets,
    client: {
      chatText: client.chatText,
      quotas: client.quotas,
      chatJson: async function forward<ValueT,>(
        request: ForeignBorrowed<ChatJsonRequest<ValueT>>,
      ): Promise<ChatJsonOutcome<ValueT>> {
        sheets.push(request.messages
          .map(function toContent(message,): string {
            // A VISION MESSAGE CARRIES PARTS, and what a witness records is
            // what the model was ASKED. The picture is not recoverable from a
            // sheet and is not what a window trial compares, so one shared
            // reader names it rather than rendering it.
            return messageText({ message, },);
          },)
          .join('\n',),);
        return await client.chatJson(request,);
      },
    },
  };
}

/**
 Refuses a run whose wide arms did not carry the window.

 COUNTS RATHER THAN CHECKING PRESENCE, because a partial forward is the failure
 that hides: one judge of six seeing the window would move a rate a little in
 the direction the trial expects, and a presence check would pass it.

 @param sheets - everything the witness saw

 @param expected - sheets that should carry the window, wide arms times judges

 @throws WindowEvidenceError when the counts disagree

 @example
 ```ts
 assertWindowReachedJudges({ sheets: witness.sheets, expected: 6, },);
 ```
 */
export function assertWindowReachedJudges(
  {
    sheets,
    expected,
  }: {
    readonly sheets: readonly string[];
    readonly expected: number;
  },
): void {
  /**
   Sheets that carried it.
   */
  const found = sheets
    .filter(function carries(sheet,): boolean {
      return sheet.includes(WINDOW_LABEL,);
    },)
    .length;
  if (found !== expected)
    throw new WindowEvidenceError({
      found,
      expected,
    },);
}

//endregion Window trial witness
