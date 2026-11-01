import { statedRefusalMessage, } from '../stated-refusal-message.test-fixture.ts';

//region Cap argument refusal
// THE REFUSAL SENTENCE A `--cap` FLAG DRAWS WHEN WHAT WAS TYPED IS NO WHOLE
// NUMBER WRITTEN IN DIGITS, shared by every command line reader that takes a
// `--cap` flag; also the refusal message one such reader drew for what was
// typed, read off whichever reader the caller wraps.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The coverage-probe-args,
// judge-fidelity-args and rendering-audit-settled-args tests kept their own
// copy of this message; all now import it from here. coverage-probe-args
// and judge-fidelity-args also kept their own copy of `refusalOf`, each
// wrapping its own command-line reader; both now import it too.

/**
 Refusal a cap draws when it is no whole number written in digits.

 @param cap - cap as typed

 @returns The refusal's sentence

 @example
 ```ts
 const said = capNotDigits({ cap: 'fourty', },);
 ```
 */
export function capNotDigits({ cap, }: { readonly cap: string; },): string {
  return `--cap needs a whole number written in digits, at most ${String(Number.MAX_SAFE_INTEGER,)}, `
    + `and ${JSON.stringify(cap,)} is not one`;
}

/**
 Message of the stated refusal reading what was typed drew.

 @param typed - what the operator wrote after the script path

 @param askedFrom - reads a probe's arguments off the command line carrying what was typed, thrown on a refusal

 @returns The refusal's message

 @throws Error when `askedFrom` threw something other than a stated refusal

 @example
 ```ts
 const said = refusalOf({ typed: ['--damage', 'scratches',], askedFrom, },);
 ```
 */
export function refusalOf(
  {
    typed,
    askedFrom,
  }: {
    readonly typed: readonly string[];
    readonly askedFrom: (input: { readonly typed: readonly string[]; },) => unknown;
  },
): string {
  return statedRefusalMessage({
    read: function readsTyped(): void {
      askedFrom({ typed, },);
    },
  },);
}

//endregion Cap argument refusal
