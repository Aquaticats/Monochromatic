// The safety-sweep interval the shipped IDE build was compiled with, read from the policy source of a disposable
// copy so a second build can be made with another interval and compared against it.
// What: the `SAFETY_SWEEP` declaration of src/refresh_policy.rs, as the text that declares it and the whole
// seconds it names.
// Why: the comparison build is the shipped build with this one number rewritten, so the reader must find the same
// text a replacement rewrites; the repository lint bans the pattern that did it.
// Tested in refresh-policy-sweep.test.ts.
import { isAsciiDigit } from './scan-characters.ts';

/** Text every declaration of the sweep interval opens with. */
const declarationStart = 'pub const SAFETY_SWEEP: Duration = Duration::from_secs(';

/** Text that closes the declaration after its number of seconds. */
const declarationEnd = ');';

/** The sweep interval one policy source declares. */
export type SafetySweepDeclaration = {
  /** The declaration exactly as the source spells it, which a replacement rewrites. */
  readonly text: string;
  /** Whole seconds between safety rereads. */
  readonly seconds: number;
};

/**
 Sweep interval the policy source declares.
 
 @param policyText - Whole text of the policy source.
 
 @returns The declaration and its seconds, or undefined when the source declares no complete one.
 
 @example
 ```ts
 safetySweepDeclaration('pub const SAFETY_SWEEP: Duration = Duration::from_secs(1);')?.seconds; // 1
 ```
 */
export function safetySweepDeclaration(policyText: string): SafetySweepDeclaration | undefined {
  let start = policyText.indexOf(declarationStart);
  // A declaration without a number is not one a build can be measured with, so the search resumes at the next.
  while (start >= 0) {
    let cursor = start + declarationStart.length;
    const secondsStart = cursor;
    while (isAsciiDigit(policyText[cursor] ?? '')) cursor++;
    if (cursor > secondsStart && policyText.startsWith(declarationEnd, cursor)) {
      return {
        text: policyText.slice(start, cursor + declarationEnd.length),
        seconds: Number(policyText.slice(secondsStart, cursor)),
      };
    }
    start = policyText.indexOf(declarationStart, start + 1);
  }
  return undefined;
}
