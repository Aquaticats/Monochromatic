## module-pify-fork

TypeScript fork of [`pify`](https://github.com/sindresorhus/pify),
 in-repo so its behavior is owned,
 tested,
 and fuzzed here instead of trusted from a third-party package.

### Attribution

Derived from [`pify`](https://github.com/sindresorhus/pify) by
[Sindre Sorhus](https://sindresorhus.com),
 released upstream under the [MIT License](LICENSES/MIT.txt).
 Member selection,
 callback semantics,
 option merge behavior,
 and error message texts come from `pify` 6.1.0.
 Upstream's copyright and permission notice are preserved verbatim in
`LICENSES/MIT.txt`;
 this fork's own code is licensed `LGPL-3.0-or-later AND MIT`
as declared in `package.json`.

### Usage

```ts
// package/upstream-fork/pify/example.ts
import { pify, } from '@monochromatic-dev/upstream-fork-pify';

// Promisify a single function.
const readFile = pify({ input: nodeFs.readFile, });
const data = await readFile({ args: ['package.json', 'utf8'], });

// Promisify all methods in a module.
const fs = pify({ input: nodeFs, });
const same = await fs.readFile({ args: ['package.json', 'utf8'], });
```

### Behavior

#### Wrapping

`pify({ input, options })` accepts a callback-style function,
 a module object,
 or a function module.
 The result is a `Proxy` view whose selected function members return promises
 settling with the callback's outcome.
 Calls run the wrapped function synchronously inside the promise constructor,
 so a synchronous throw becomes a rejection,
 and the wrapped function sees the proxy's target as `this` when the call came
 through the proxy.

#### Options

Options mirror upstream `pify`:
`multiArgs` collects every callback argument into one array
 (rejections then carry the whole list,
 error included),
`include` / `exclude` select members by string equality or `RegExp` test,
`errorFirst` toggles the leading error argument,
`promiseModule` swaps the promise constructor,
 and `excludeMain` keeps a function module's own call raw.
 Defaults match upstream exactly:
 error-first callbacks,
 `Promise`,
 and an `exclude` of `[/.+(?:Sync|Stream)$/]`.

#### Preserved upstream quirks

The differential oracle compares member selection and option handling against
 upstream `pify` 6.1.0 directly,
 so three upstream quirks are reproduced verbatim instead of fixed:

- The member-selection cache is shared across `pify` calls and keyed by target
 object only.
 The first `pify` call to read a member freezes that member's promisification
 decision for every later `pify` call on the same object,
 whatever those later calls' `include` / `exclude` options say.
- The selection cache starts as a plain object and membership is tested with
`in`,
 so keys living on `Object.prototype` (`'constructor'`,
`'toString'`,
 ...)
 skip include / exclude evaluation entirely and fall through to the `get`
 trap's `Function.prototype` comparison.
- An option key explicitly set to `undefined` overwrites its default
 (upstream spreads the caller's record after the defaults and never
 re-normalizes),
 so `pify({ input, options: { errorFirst: undefined, }, })` behaves
 non-error-first and `{ exclude: undefined, }` crashes on first member access
 exactly like upstream.

### Deviations from upstream pify

#### Call shape

`pify({ input, options })` takes one destructured object and promisified calls
 pass an `args` tuple (`readFile({ args: [...] })`).
 Repository lint bans rest parameters and multi-positional-parameter
 declarations outright.

#### Error types

Invalid input throws `InvalidInputError` instead of a bare `TypeError`.
 It extends `TypeError` and carries upstream's message text verbatim,
 so migrating callers see identical diagnostics and the fuzz sidecar's
 differential oracle compares both implementations directly.

#### Callback capture

The `multiArgs` collector captures the wrapped function's variadic result list
 through the `arguments` object under a scoped
`eslint/prefer-rest-params` suppression:
 node-style callbacks deliver results as free-form trailing arguments,
 and the repository's rest-parameter ban makes `arguments` the only closure
 capture of that list.
 See [DECISION.callback-capture.md](DECISION.callback-capture.md) for the
 full rule-by-rule derivation.

#### Type level

Result inference mirrors upstream `pify`'s `index.d.ts` modulo this fork's
`{ args }` call shape,
 with one fix:
 `errorFirst` is forwarded to the module-method result
 computation (upstream silently drops it).
 Upstream's own type quirks are kept:
 single-parameter callbacks under `errorFirst` stay `Promise<unknown>`,
 functions without a trailing callback stay `Promise<unknown>`,
 `include` / `exclude` entries type as `keyof Module` even though the runtime
 accepts `RegExp`,
 and the overload pin on `excludeMain` is preserved.

### Layout

`pify.ts` owns the proxy and overloads,
`promisify-function.ts` the wrapper factory and callback shapes,
`key-filter.ts` member selection,
`pify-options.ts` option resolution,
`type-helpers.ts` the tuple and string type helpers,
 and `errors.ts` the error class.
 Test files sit beside each module as `<stem>.unit.test.ts`,
 so mutation testing selects each module's tests automatically.

### Testing

```bash
# package/upstream-fork/pify/mise.toml

# Unit tests against the built dist
mise run //package/upstream-fork/pify:buildAndTest

# Property-based fuzz campaign (invariants plus an upstream differential oracle)
mise run //package/upstream-fork/pify.fuzz:fuzz

# Coverage-reachability gate over this package's src
mise run //package/upstream-fork/pify.fuzz:fuzz:coverage

# Container-isolated mutation testing
mise run //package/upstream-fork/pify:test:mutation
```
