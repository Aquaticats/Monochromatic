# Oxlint plugin test support

Dev-only helpers shared by the oxlint plugin fixture tests.

The package owns common fixture-path resolution,
oxlint JSON capture,
temp fixture copying,
and rule-code de-duplication.
It is private because the helpers support this monorepo's tests rather than a published runtime plugin.

## Exports

- `OXLINT_PLUGIN_TEST_ROOT`: the workspace root, which is the working directory of every oxlint run.
- `fixturePackageRoot`, `fixtureSourceRoot` and `fixtureConfigPath`: locate a package under `package/test-fixture`,
  its `src` directory, and a config file inside it.
- `resolveFixtureTarget`: resolves a relative fixture path against a source root and keeps an absolute path as is.
- `createTempFixtureFile`: copies a fixture into a disposable temp directory,
  and returns its path with an async disposer that removes the directory.
- `runOxlintFixture`: runs oxlint over one target and returns the diagnostics of one plugin.
- `uniqueRuleCodes`: the sorted, de-duplicated rule codes of a diagnostic list.
- `OxlintOutputParseError`: the error `runOxlintFixture` throws when oxlint prints text instead of JSON.

## `runOxlintFixture`

`runOxlintFixture` starts the `oxlint` executable on the `PATH`
with `--format json`,
inside `OXLINT_PLUGIN_TEST_ROOT`,
and parses the JSON it prints.
It returns the diagnostics whose rule code starts with the requested prefix,
in the order oxlint emitted them.

Oxlint exits non-zero when it reports findings.
That is not a failure here:
the runner reads the output either way.

```ts
// example.unit.test.ts
const diagnostics = await runOxlintFixture({
  codePrefix: 'tsdoc(',
  configFlag: '--config',
  fixtureConfig: fixtureConfigPath({ fixturePackageName: 'oxlint-tsdoc', fileName: '.oxlintrc.fixture.json', }),
  target: resolveFixtureTarget({ fixtureSourceRoot, fixturePath: 'invalid/file.ts', }),
});
```

### Required options

- `codePrefix`: the diagnostic code prefix to keep, including the opening parenthesis,
  such as `'tsdoc('`.
  The type is `` `${string}(` ``,
  so a prefix without the parenthesis does not compile.
  Diagnostics of other plugins and of oxlint's built-in rules are dropped.
- `configFlag`: the spelling of the config flag passed to oxlint, either `'-c'` or `'--config'`.
  Use the spelling the plugin's fixture setup accepts.
- `fixtureConfig`: the absolute path of the oxlint config the run loads.
- `target`: the absolute path of the one source file to lint.

### Optional options

- `threads`: a number passed to oxlint as `--threads <n>`.
  Leave it out to use oxlint's default.
  Set it for plugins whose tests are sensitive to resource use.
- `fix`: asks oxlint to write fixes back to `target`.
  Leave it out to only lint,
  which leaves the target untouched even when a rule offers a fix.
  `'fixes'` passes `--fix`.
  `'fixes-and-suggestions'` passes `--fix` and `--fix-suggestions`.
  The run returns the diagnostics left after the fixes were applied,
  and a test reads the target back to check the rewritten source.
  Pass a temp copy from `createTempFixtureFile`, never a checked-in fixture.
- `environment`: variables added to the environment of the oxlint process,
  such as a fault injection loaded through `NODE_OPTIONS`.
  They are merged over the environment of the test process,
  so variables that are not listed keep their current values.
  Leave it out to run oxlint with the environment of the test process unchanged.

### Failures

A diagnostic with no rule code makes the run throw an `Error`,
whatever the `codePrefix` is.
Oxlint reports a plugin that threw,
and a source it could not parse,
as such a diagnostic.
Keeping only the diagnostics of one plugin would drop it silently,
and a fixture expecting no findings would pass while the plugin crashed.
The error message names the target and lists every message oxlint gave.
This applies to fix runs too,
because a plugin that crashes fixes nothing,
and a test reading the target back to show it was left alone would otherwise pass through the crash.

When oxlint prints text instead of JSON,
the run throws an `OxlintOutputParseError`.
This happens when oxlint cannot load a plugin or read its config,
for example when a plugin's `createOnce` throws.
The message names the target and the config,
and includes the command line,
the raw standard output and the raw standard error.
The JSON parse failure is its `cause`.

When the failure thrown by the process exposes no standard output as a string,
the run throws an `Error` whose `cause` is that original failure.

## Probe plugins

`src/harness-probe-plugin.test-fixture.ts` and `src/harness-probe-create-once-plugin.test-fixture.ts` are oxlint plugins
that misbehave on demand.
`src/run-oxlint-fixture.unit.test.ts` uses them to show the runner reports,
stays quiet,
fails on a plugin that throws,
applies fixes,
and passes `environment` through.
Oxlint sets up every rule of a plugin when it loads the plugin,
so a rule that throws in `createOnce` has a plugin of its own.
