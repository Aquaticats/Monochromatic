//region Structural ballot
// ONE JUDGE'S OR GATE VOICE'S SCHEMA-VALID REPLY BODY, naming only the
// choice: no finding supports or opposes it and nothing is dropped. The keys are
// the reply's own (`assessment`, `dropped`, `unsupported`, `verdict`; ledger B762),
// which the ballot a reader builds from them stores as `reason`, `dropped`,
// `unsupported` and `choice`.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Several consolidate and lane-contest
// tests kept their own copy of this builder; all now import it from here.

/**
 Builds one ballot body.

 @param choice - candidate this judge or gate voice names

 @returns Reply body a judge or gate voice would return

 @example
 ```ts
 const body = ballot({ choice: 'consolidated', },);
 ```
 */
export function ballot({ choice, }: { readonly choice: string; },): string {
  return JSON.stringify({
    assessment: 'the original supports it',
    dropped: [],
    unsupported: [],
    verdict: choice,
  },);
}

//endregion Structural ballot
