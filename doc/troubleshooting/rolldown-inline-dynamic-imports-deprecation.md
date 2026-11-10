# Rolldown 1.1.5 `inlineDynamicImports` through tsdown 0.22.4 emits an unsuppressible deprecation warning

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

A tsdown public `build()` call that passes:

```ts
outputOptions: {
  inlineDynamicImports: true,
}
```

emits this warning even when the build uses `logLevel: 'silent'`:

```text
WARN  inlineDynamicImports option is deprecated, please use codeSplitting: false instead.
```

The build still succeeds and produces one chunk.
This matters for cli-git trust because build diagnostics must not contaminate its JSONL command streams.

## Root cause

Tsdown 0.22.4 merges consumer `outputOptions` into its Rolldown output configuration.
At tag `v0.22.4`,
commit `434cfb3f563addffb88882b128b7a390f3434e94`,
`src/features/rolldown.ts:302-321` contains:

```ts
const outputOptions: OutputOptions = await mergeUserOptions(
  {
    format: cjsDts ? 'es' : format,
    // ...
    codeSplitting: config.exe ? false : undefined,
  },
  config.outputOptions,
  [format, { cjsDts }],
)
return outputOptions
```

Rolldown 1.1.5 then interprets the deprecated alias.
At tag `v1.1.5`,
commit `f09947ab017d6df74299f691853dcfc4f4f0f86e`,
`package/rolldown/src/utils/bindingify-output-options.ts:208-246` contains:

```ts
if (codeSplitting === false) {
  // ...
  return {
    inlineDynamicImports: true,
    advancedChunks: undefined,
  };
} else if (codeSplitting == null) {
  if (inlineDynamicImportsOption != null) {
    logger.warn(
      '`inlineDynamicImports` option is deprecated, please use `codeSplitting: false` instead.',
    );
    inlineDynamicImports = inlineDynamicImportsOption;
  }
}
```

The important semantic fact is explicit in that branch:
`codeSplitting: false` sets the binding's `inlineDynamicImports` value to `true`.
The new option therefore preserves the required one-bundle behavior rather than weakening it.

Rolldown also keeps a regression test for the warning.
`package/rolldown/tests/utils/bindingify-code-splitting.test.ts:83-94` contains:

```ts
test('codeSplitting: undefined with inlineDynamicImports shows deprecation warning', async () => {
  // ...
  await bundle.generate({
    inlineDynamicImports: true,
  });
  expect(consoleSpy).toHaveBeenCalledWith(
    '`inlineDynamicImports` option is deprecated, please use `codeSplitting: false` instead.',
  );
});
```

## Verification

Verified with:

- tsdown `0.22.4`;
- Rolldown `1.1.5`;
- tsdown commit `434cfb3f563addffb88882b128b7a390f3434e94`;
- Rolldown commit `f09947ab017d6df74299f691853dcfc4f4f0f86e`;
- Node `26.4.0`.

The fixture contains one literal dynamic import:

```ts
// /tmp/agent/tsdown-inline-fixture.ts
export async function loadValue(): Promise<unknown> {
  return await import('./tsdown-inline-value.ts');
}
```

### Failing diagnostic catalog

This invocation emits the warning despite `logLevel: 'silent'` and prints `1` for the output count:

```sh
node --input-type=module --eval "import { build } from 'tsdown'; const outputs = await build({ config: false, entry: '/tmp/agent/tsdown-inline-fixture.ts', write: false, clean: false, dts: false, logLevel: 'silent', outputOptions: { inlineDynamicImports: true } }); console.log(outputs[0].chunks.length); await Promise.all(outputs.map((output) => output[Symbol.asyncDispose]()));"
```

Observed output:

```text
WARN  inlineDynamicImports option is deprecated, please use codeSplitting: false instead.
1
```

### Clean catalog

The supported equivalent emits no warning and still prints one output:

```sh
node --input-type=module --eval "import { build } from 'tsdown'; const outputs = await build({ config: false, entry: '/tmp/agent/tsdown-inline-fixture.ts', write: false, clean: false, dts: false, logLevel: 'silent', outputOptions: { codeSplitting: false } }); console.log(outputs[0].chunks.length); await Promise.all(outputs.map((output) => output[Symbol.asyncDispose]()));"
```

Observed output:

```text
1
```

Cli-git's maintained TypeScript builder fixtures additionally verify that a literal relative dynamic import appears in
the tracked source graph and leaves no runtime local import in the sole executable bundle.

## Verified workarounds

Use Rolldown's current spelling through tsdown:

```ts
outputOptions: {
  codeSplitting: false,
}
```

Tradeoff:
this is a Rolldown-specific replacement for Rollup's older option spelling.
It is correct for cli-git because cli-git already requires Rolldown through tsdown and rejects every output shape except
one JavaScript chunk.

## What does not work

- `logLevel: 'silent'` does not suppress this warning.
  The verification invocation demonstrates that the warning comes from Rolldown option normalization rather than
  tsdown's normal build reporting.
- Setting both options does not provide compatibility without noise.
  `package/rolldown/src/utils/bindingify-output-options.ts:209-214` explicitly warns that
  `inlineDynamicImports` is ignored when `codeSplitting: false` is present.
- Keeping the deprecated spelling merely because the original cli-git decision named it produces avoidable stderr
  output.
  The binding code proves the replacement has the same inline-dynamic-import effect.

## Upstream filing decision

No `.out-of-scope/` entry covers Rolldown or tsdown.
Searches across open and closed Rolldown issues and pull requests for
`inlineDynamicImports deprecated codeSplitting false` found no matching report.

The six constraints resolve as follows:

1.  **Is it really upstream's fault?
    ** No.
    This is an intentional,
    tested deprecation with a precise replacement.
2.  **Can upstream fix it?
    ** Not applicable as a defect.
    Removing the warning would defeat the documented migration.
3.  **Are they supporting this use case?
    ** Yes.
    Rolldown supports single-bundle dynamic imports through `codeSplitting: false`,
    and tsdown exposes Rolldown `outputOptions`.
4.  **Would the repo welcome our contribution?
    ** The Rolldown `CONTRIBUTING.md` links its public contribution guide;
    no repository policy inspected for this diagnosis prohibited outside contributions.
    This does not override constraint 1.
5.  **Will they likely fix it?
    ** No fix is warranted because current behavior is deliberate and covered by an upstream test.
6.  **Have we prototyped a minimal fix compatible with their architecture?
    ** No upstream patch is appropriate.
    The verified consumer-side option migration is the complete fix.

Nothing should be filed upstream.
The retained artifact is intentionally not fileable:

~~~md
Title: Do not file: `inlineDynamicImports` deprecation is working as designed

Rolldown 1.1.5 intentionally warns and directs consumers to `codeSplitting: false`.
Source and tests confirm that the replacement sets the same internal inline-dynamic-import behavior.
There is no upstream defect to report.
~~~
