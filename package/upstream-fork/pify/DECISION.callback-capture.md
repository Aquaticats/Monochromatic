# Decision: callback capture in the multiArgs collector

## Context

`pify`'s `multiArgs` mode synthesizes a node-style callback that must capture
 a variadic result list:
 the wrapped function invokes the callback with a
 leading error argument plus any number of trailing result arguments
 (`callback(err, a, b, c, ...)`),
 and the promise resolves with that list.
 The wrapped functions are arbitrary node-style functions,
 so the callback's variadic signature is dictated by an external convention
 and cannot be reshaped the way this fork reshapes its own call surface.

The repository's lint stack bans every obvious capture mechanism:

- `no-restricted-syntax/no-rest-params`
 (`package/oxlint-plugin/no-restricted-syntax/src/rule/no-rest-params.ts`)
 bans rest parameters in function declarations and expressions,
 and `no-restricted-syntax/no-disable-no-rest-params`
 (`package/oxlint-plugin/no-restricted-syntax/src/rule/no-disable-no-rest-params.ts`)
 bans any inline suppression of that ban.
- `no-restricted-syntax/no-arrow-function` bans arrow functions
 (which would otherwise escape the rest-parameter rule),
 with its suppression banned too.
- `no-restricted-syntax/require-destructured-params` requires destructured
 single-object parameters on function declarations with two or more inputs,
 which a node-style callback's externally dictated signature cannot take.
- oxlint's built-in `eslint/prefer-rest-params` (pedantic category,
 blocking through `denyWarnings`) flags the `arguments` object.

## Decision

Capture the result list through the `arguments` object inside one named
 collector function,
 carrying a scoped
`// oxlint-disable-next-line eslint/prefer-rest-params -- ...`
suppression that names the external-boundary mirror.

## Why the other paths lose

- A `Proxy` `apply` trap is the one seam the rules leave fully open
 ("accept an array parameter"),
 but it replaces a plain closure with a per-call proxy on the promisify hot
 path and its `bind` / `call` / `apply` forwarding needs nested bound-builtin
 getters to keep upstream's observable behavior.
- A rest parameter plus a config override would follow
 `jestMatcherApiOverride`'s precedent for externally dictated signatures,
 but it loosens a repo-authored rule in shared configuration for one package's
 one function,
 and `no-restricted-syntax/no-rest-params`'s text covers functions we control
 without the external-signature exemption its sibling
 `require-destructured-params` documents.
- Inline disabling of the rest-parameter ban is impossible outright.

## Why the scoped suppression fits

`eslint/prefer-rest-params` is a third-party heuristic whose own message
 ("use the rest parameters instead") is unsatisfiable here.
 Its suppression is not in the `no-disable-*` ban list,
 and the repository already keeps documented scoped suppressions at
 external-boundary mirrors
 (for example `package/claude-code-plugin/hook-type/src/tool-inputs-union.ts`
 mirrors Claude Code tool schemas under a named suppression).
 The suppression names the rule,
 the boundary,
 and the sibling rule that makes `arguments` the only remaining capture.

## Consequences

- The `multiArgs` collector keeps upstream `pify`'s exact runtime shape:
 a plain function capturing a fresh results array per invocation,
 including the exotic `callback.bind(...)` paths a Proxy seam would break.
- The differential oracle in `package/upstream-fork/pify.fuzz` compares
 settlements against upstream `pify` 6.1.0 with this capture in place.
- Every other callback shape in the package (error-first single result,
 non-error-first) uses plain fixed-arity closures or `resolve` itself,
 exactly like upstream.
