# Optique argument parsing observes readonly buffers but invokes parser capabilities

## Symptom

`prefer-readonly-parameter-type/prefer-readonly-parameter-types` reported opaque effects for every
`@optique/core` `parseSync` and `runParserSync` call receiving a readonly argument array.

Treating the complete call as opaque incorrectly attributed parser and option capabilities to the argument buffer.
Treating the complete call as observational would instead hide real parser and callback effects.

## Source audit

The audit used `@optique/core` `1.1.1` at commit
`b8d39082fdeb37bb16c68b2dc2396d4c9c45b1d5`.

Audited source and shipped files:

- `packages/core/src/internal/parser.ts`,
  digest `b8ad8e789978a25980f9f46b442e7117f36feaede399fe74e0f1a59411787376`;
- shipped `dist/internal/parser.js`,
  digest `138e40f7f4c2bb88c3e44bdf5ed23caf61c553540b60a3aae23d201888ca8671`;
- `packages/core/src/facade.ts`,
  digest `cf01245fd9322d8a4eca0a718f4e126acf0671d2c79eeadb3c29d3ef2dda5d65`;
- shipped `dist/facade.js`,
  digest `b964c1f6b330b37e855f965a89a9b64e2a4dd5e41b4d93e674cfe540cbba20cc`.

`parseSync` installs its `readonly string[]` argument as `ParserContext.buffer`.
The loop replaces context values returned by `parser.parse` and compares or reads buffers without writing the
supplied array.
Completion invokes `parser.complete`.
Parser methods remain capabilities because custom parsers can change their own or captured state.

`runParserSync` delegates to `runParser`.
The facade preserves the readonly argument buffer while reading configuration,
invoking parser methods,
and potentially invoking configured output or error callbacks.
Configuration objects can also expose getters or proxies.

The same commit's `@optique/run` `runSync` implementation delegates to `runImpl` or `runWithSync`.
It reads `RunOptions` fields and can invoke configured `onExit`,
`stdout`,
and `stderr` callbacks.
The authored `packages/run/src/run.ts` digest is
`64cc52ec2b2b34bc849d55fd1aba66573c485a156e19b2216dd69b3b87b32663`.
The shipped `dist/run.js` digest is
`0ff2e1dd543f6b45d507ae52d33effdf830bf15323c1b22b0bab82d52f304978`.

## Resolution

The exact package-major catalog records:

- argument buffers as observational;
- parser objects as opaque and their `parse` and `complete` properties as invoked;
- parse options as opaque;
- run options as opaque,
  with top-level `onError`,
  `stderr`,
  and `stdout` properties recorded as invoked capabilities.

This separation permits readonly CLI argument arrays without claiming that parser or option objects are pure.
Unknown versions and unmatched call identities remain fail-closed.

The watch-restart CLI keeps a narrow local `@mutates options` contract around `runSync`.
A package-wide entry would need to model every `RunOptions` field,
context capability,
and callback path rather than only this call site's three optional callbacks.

## Verification

Focused catalog tests resolve both installed declaration identities and assert that only parser and options positions
are opaque.
The Git policy CLI lint then accepts direct readonly argument buffers while retaining diagnostics for unrelated
provider boundaries.

## Upstream filing decision

No upstream issue was filed.
Optique already declares argument buffers readonly and its implementation honors that contract.
The missing piece was effect granularity in this project's semantic catalog,
not a defect in Optique.

## Sources

- [Optique `parseSync` source][parse-source]
- [Optique `runParserSync` source][facade-source]
- [Optique `runSync` source][run-source]

[parse-source]: https://github.com/dahlia/optique/blob/b8d39082fdeb37bb16c68b2dc2396d4c9c45b1d5/packages/core/src/internal/parser.ts
[facade-source]: https://github.com/dahlia/optique/blob/b8d39082fdeb37bb16c68b2dc2396d4c9c45b1d5/packages/core/src/facade.ts
[run-source]: https://github.com/dahlia/optique/blob/b8d39082fdeb37bb16c68b2dc2396d4c9c45b1d5/packages/run/src/run.ts
