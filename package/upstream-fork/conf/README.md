## module-conf-fork

TypeScript fork of [`conf`](https://github.com/sindresorhus/conf),
 in-repo so its behavior is owned,
 tested,
 and mutation-tested here instead of trusted from a third-party package.
 `conf` stores app configuration in a JSON file with defaults,
 schema validation,
 dot-notation access,
 optional encryption,
 migrations,
 file watching,
 and change events.

### Attribution

Derived from [`conf`](https://github.com/sindresorhus/conf) by
[Sindre Sorhus](https://sindresorhus.com),
 released upstream under the [MIT License](LICENSES/MIT.txt).
 Store semantics,
 the on-disk and encryption wire formats,
 validation behavior,
 migration bookkeeping,
 and error message texts come from `conf`'s GitHub source at fork time
 (commit `83e267178f`, version-labeled 15.1.0).
 npm `conf` 15.1.0 predates that commit and differs observably,
 so the fuzz sidecar's differential oracle pins the forked snapshot
 instead of the npm release.
 Upstream's copyright and permission notice are preserved verbatim in
`LICENSES/MIT.txt`;
 this fork's own code is licensed `LGPL-3.0-or-later AND MIT`
as declared in `package.json`.

### Usage

```ts
// package/upstream-fork/conf/example.ts
import { createConf, } from '@monochromatic-dev/upstream-fork-conf';

const config = createConf<{ theme: string, }>({
  projectName: 'foo',
  defaults: {
    theme: 'light',
  },
});

config.set({
  key: 'theme',
  value: 'dark',
});
config.get('theme'); // => 'dark'
```

### Behavior

#### Store

`createConf(options)` resolves one config file and returns a frozen store
 object bound to it.
 `path` names the file,
 `store` reads or replaces everything,
 `size` counts items,
 and the store iterates as `[key, value]` pairs.
 Writes are atomic replacements:
 readers see the old or new content,
 never a partial write.

#### Items

`get` reads one key or dotted path with an optional default,
`set` places one item or many,
`has` probes,
`appendToArray` grows array values,
`reset` restores keys to their defaults,
`delete` removes one key,
 and `clear` resets everything to defaults.
 Writes re-read the file first,
 so changes another process made survive the write.

#### Defaults and schema

`defaults` seeds missing items;
`schema` validates the store as JSON Schema draft-2020-12 through ajv,
 applying schema `default` values and reporting violations as
`SchemaViolationError`.
 `rootSchema` adds root keywords and `ajvOptions` tunes ajv.
 Values in `defaults` overwrite schema defaults.

#### Encryption

`encryptionKey` obfuscates the config file with `aes-256-cbc`,
 `aes-256-gcm`,
 or `aes-256-ctr`.
 The wire format matches upstream `conf` exactly:
 initialization vector,
 `:` separator,
 ciphertext,
 and the GCM authentication tag,
 keyed by PBKDF2-SHA512 over the initialization vector.

#### Migrations

`migrations` maps versions or semver ranges to handlers that run when
 `projectVersion` upgrades.
 Bookkeeping lives under the reserved `__internal__` key and never surfaces
 through the store.
 A failed step restores the store to its pre-step snapshot and throws
`MigrationFailedError`.

#### Watching and cache

`watch` reports file changes made by other processes to `onDidChange` and
 `onDidAnyChange` subscribers.
 `cache` keeps the store in memory between writes,
 dropping it on every write and on watched changes.

### Deviations from upstream conf

#### Factory instead of a class

`createConf(options)` returns a frozen object instead of exposing a `Conf`
 class,
 because repository lint bans classes for long-lived stateful objects.
 The `store` getter/setter survives as a property accessor,
 and `_closeWatcher` is renamed `closeWatcher`.

#### Call shapes

Members with multiple inputs take one destructured object
(`get({ key, defaultValue })`,
 `set({ key, value })`,
 `set({ values })`,
 `appendToArray({ key, value })`,
 `reset({ keys })`,
 `onDidChange({ key, callback })`).
 Single-input members keep their positional key
(`get(key)`,
 `has(key)`,
 `delete(key)`,
 `onDidAnyChange(callback)`).
 Repository lint bans rest parameters and multi-positional-parameter
 declarations outright.

#### Change callbacks

`onDidChange` and `onDidAnyChange` callbacks receive one change object
(`{ newValue?, oldValue? }` and `{ newValue, oldValue }`) instead of
 positional pairs,
 and are not invoked with a `this` argument.

#### Error types

Invalid input throws typed errors
(`InvalidKeyError`,
 `MissingValueError`,
 `ReservedKeyError`,
 `UnsupportedValueTypeError`,
 `NonArrayValueError`,
 `InvalidCallbackError`,
 `InvalidEncryptionAlgorithmError`,
 `MissingProjectNameError`,
 `MissingProjectVersionError`,
 `InvalidSchemaError`,
 `RootSchemaPropertiesError`,
 `SchemaViolationError`,
 `DecryptionFailedError`,
 `InvalidAuthenticationTagError`,
 `MigrationFailedError`) instead of bare `TypeError` or `Error`.
 Each class extends the same base upstream throws and carries upstream's
 message text verbatim,
 so migrating callers see identical diagnostics.

#### In-repo dependencies

The atomic write,
 debounce,
 per-user config directory resolution,
 and byte conversion helpers live in this package instead of upstream's
`atomically`,
 `debounce-fn`,
 `env-paths`,
 and `uint8array-extras` dependencies,
 leaving those off the trust surface of the write path.
 `dot-prop`,
 `semver`,
 `ajv`,
 `ajv-formats`,
 and `json-schema-typed` stay as dependencies:
 their path grammar,
 semver-range predicate semantics,
 and JSON Schema coverage are behavior-defining and would be reimplementations
 with drift risk.

#### Atomic write

Writes stage a randomly named sibling in the config file's own directory and
 rename it over the target,
 which keeps the rename on one filesystem and makes upstream's cross-device
 fallback unnecessary.
 Crash durability is rename-based;
 upstream's fsync pass is not reproduced.
 The Snap direct-write workaround (sindresorhus/conf#82) is preserved.

#### ajv-formats interop

Upstream unwraps one extra `.default` layer from `ajv-formats` for its build's
 CommonJS/ESM interop (ajv-validator/ajv#2047).
 This repository's Node module resolution returns the plugin directly,
 verified by direct invocation,
 so the fork imports it plainly.

#### Reserved-key scan

The `__internal__`-key scan over multi-item `set` payloads walks breadth-first
 with a visited set,
 so cyclic payloads terminate instead of exhausting the call stack as
 upstream's recursion would.

#### Store access copies

Internal key placement and removal build a top-level copy of the freshly-read
 store instead of writing through the caller's object.
 Observable store state is unchanged;
 the copies satisfy the repository's readonly-parameter contract.

### Layout

`conf.ts` owns the factory and the `Conf` type lives in `conf-type.ts`.
 `conf-crud.ts` and `conf-events.ts` hold the item and subscription methods,
 `store-file.ts` the read/write pipeline,
 `encryption.ts` the wire format,
 `schema.ts` validation,
 `migrate.ts` migration execution,
 `watcher.ts` file watching,
 `store-access.ts` key access,
 `file-io.ts` the synchronous filesystem boundary (see
`DECISION.sync-api.md`),
 and `options.ts`/`prepare-options.ts` configuration.
 Test files sit beside each module as `<stem>.unit.test.ts`,
 so mutation testing selects each module's tests automatically.
 `test-support.ts` provides disposable directory,
 home,
 and encryption fixtures.

### Testing

```bash
# package/upstream-fork/conf/mise.toml

# Unit tests against the built dist
mise run //package/upstream-fork/conf:buildAndTest

# Same suite inside a capped podman container with a disposable HOME
mise run //package/upstream-fork/conf:test:container

# Container-isolated oxc mutation testing
mise run //package/upstream-fork/conf:test:mutation

# Property fuzzing and the upstream differential oracle
mise run //package/upstream-fork/conf.fuzz:fuzz

# Coverage-reachability gate over this package's src
mise run //package/upstream-fork/conf.fuzz:fuzz:coverage
```
