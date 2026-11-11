# Intentional lint suppressions

## Rule and configuration

`package/config/oxlint/src/rule/restriction.ts` enables
`no-restricted-syntax/no-nullish-union` at error severity.
It rejects `T | null` and `T | undefined` unions
because this repo models absence with optional properties
under `exactOptionalPropertyTypes`
(`no-restricted-syntax/no-optional-escape` bans the `Partial` and
sentinel workarounds).
`typescript(method-signature-style)` asks for property signatures
over method signatures.

## Why the config cannot express this package

This package mirrors one upstream shape exactly
(the same mirror-first posture
`doc/decision/pi-model-retirement.md` records):
`@deepseek-ai/dsh-credentials`
`CredentialProvider.resolve(ref): Promise<ResolvedCredential | undefined>`
(`lib/types/index.d.ts`).

Two suppressions sit on that one structural mirror
(`QwenTokenPlanIndividualCredentialsTarget` in `src/fetch-listing.ts`):

- `no-restricted-syntax/no-nullish-union`:
  `undefined` is the seam's own "reference unconfigured" answer
  and must reach the caller unchanged.
  The real `CredentialProvider` would not satisfy a property typed
  `Promise<ResolvedCredential>`,
  so the structural seam cannot narrow the answer
  without breaking the assignment it exists for.
- `typescript(method-signature-style)`:
  the mirrored declaration is a method,
  and method bivariance is what lets the real service
  satisfy the structural mirror.
  The property form fails assignability under `strictFunctionTypes`,
  measured as `TS2379` on `src/index.ts`
  ("Type 'string' is not assignable to type 'CredentialRef'")
  with `resolve: (ref: string) => ...`;
  a property typed with the branded parameter
  would force this package back onto
  the `credentialRef` runtime import
  the seam exists to avoid
  (its brand is compile-time identity,
  so the raw reference name resolves identically).

Every other absence in this package
(the optional `getCredentials` lookup,
the `credentials` property of
`QwenTokenPlanIndividualCredentialsLookupResult` in `src/fetch-listing.ts`,
the facts fields the shared core keeps optional,
and the test fixtures)
uses optional properties,
and the resolution failure half travels as
`QwenTokenPlanIndividualCredentialError` (PP4),
not as a nullish value.

`eslint/no-await-in-loop` suppressions for the mount-race retry loops are
recorded in `doc/lint/sequential-awaits.md`.
