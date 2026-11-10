# CAC 7.0.0 declared option strings that look numeric lose their exact argv spelling

## Symptom

CAC 7.0.0 changes declared option values before a consumer sees them:

- `--policy 001` becomes number `1`;
- `--policy +2` becomes number `2`;
- adding `{ type: [String] }` produces `['1']` and `['2']`,
  after the leading characters are already lost.

For cli-git,
this changes policy identifiers.
The incumbent parser returns exact strings `001` and `+2`.
A later `String(value)` cannot recover the input.

Open upstream issue [`cacjs/cac#165`][issue-165] reports the same behavior in CAC 7.0.0.
Its 2026-06-17 comment confirms that `type: [String]` does not preserve a leading plus.

## Root cause

The verified candidate is npm `cac@7.0.0`,
release tag `v7.0.0`,
commit `77f602fcb2d1e75d24f5ecd94d5bf667acaa857a`.
CAC inlines `mri@1.2.0`,
tag `v1.2.0`,
commit `e73e9f9d5b02124d14ac17dac2c4801687d3e99a`.

### CAC omits MRI string metadata

CAC constructs MRI configuration with aliases and boolean names only.
It has no `string` member at `src/utils.ts:42-81`:

```ts
// cacjs/cac src/utils.ts:42-48
interface MriOptions {
  alias: {
    [k: string]: string[]
  }
  boolean: string[]
}

export function getMriOptions(options: Option[]): MriOptions {
  const result: MriOptions = { alias: {}, boolean: [] }
```

`CAC.mri` passes this configuration to MRI at `src/cac.ts:263-282`:

```ts
// cacjs/cac src/cac.ts:263-282
private mri(
  argv: string[],
  /** Matched command */ command?: Command,
): ParsedArgv {
  const cliOptions = [
    ...this.globalCommand.options,
    ...(command ? command.options : []),
  ]
  const mriOptions = getMriOptions(cliOptions)

  // Double-dash extraction omitted from this excerpt.
  let parsed = mri(argv, mriOptions)
```

### MRI converts untyped option values

MRI 1.2.0 applies unary plus to any option value that is not listed as a string or boolean.
The conversion is at `src/index.js:5-11`:

```js
// lukeed/mri src/index.js:5-11
function toVal(out, key, val, opts) {
  var x, old=out[key], nxt=(
    !!~opts.string.indexOf(key) ? (val == null || val === true ? '' : String(val))
    : typeof val === 'boolean' ? val
    : !!~opts.boolean.indexOf(key) ? (val === 'false' ? false : val === 'true' || (out._.push((x = +val,x * 0 === 0) ? x : val),!!val))
    : (x = +val,x * 0 === 0) ? x : val
  );
```

Unary plus converts `001` to `1` and `+2` to `2`.
That conversion discards the original characters.

### CAC transforms after MRI

CAC runs configured `type` transforms only after MRI returns.
The call is at `src/cac.ts:323-331`:

```ts
// cacjs/cac src/cac.ts:323-331
for (const key of Object.keys(parsed)) {
  if (key !== '_') {
    const keys = key.split('.')
    setDotProp(options, keys, parsed[key])
    setByType(options, transforms)
  }
}
```

`setByType` first wraps the already converted value in an array and then maps `String`
at `src/utils.ts:128-142`:

```ts
// cacjs/cac src/utils.ts:128-142
export function setByType(
  obj: { [k: string]: any },
  transforms: { [k: string]: any },
): void {
  for (const key of Object.keys(transforms)) {
    const transform = transforms[key]

    if (transform.shouldTransform) {
      obj[key] = [obj[key]].flat()

      if (typeof transform.transformFunction === 'function') {
        obj[key] = obj[key].map(transform.transformFunction)
      }
    }
  }
}
```

`String(1)` is `'1'`,
not the original `'001'`.
The earlier hypothesis that `{ type: [String] }` could preserve exact argv was therefore wrong.
The source order and artifact probe disprove it.

## Verification

### Version and artifact

The npm tarball was downloaded from the registry on 2026-08-14.

- npm integrity:
  `sha512-tixWYgm5ZoOD+3g6UTea91eow5z6AAHaho3g0V9CNSNb45gM8SmflpAc+GRd1InC4AqN/07Unrgp56Y94N9hJQ==`;
- measured SHA-512:
  `b62c566209b9668383fb783a51379af757a8c39cfa0001da868de0d15f4235235be3980cf1299f96901cf8645dd489c2e00a8dff4ed49eb829e7a63de0df6125`;
- npm provenance binds the artifact to `cacjs/cac`,
  tag `v7.0.0`,
  and commit `77f602fcb2d1e75d24f5ecd94d5bf667acaa857a`.

### Runnable artifact harness

Download and extract the pinned artifact into a disposable directory:

```sh
work="$(mktemp --directory)"
curl --fail --location --silent --show-error \
  https://registry.npmjs.org/cac/-/cac-7.0.0.tgz \
  --output "${work}/cac.tgz"
tar --extract --file "${work}/cac.tgz" --directory "${work}"
```

Save this harness as `${work}/probe.mjs`:

```js
// probe.mjs
import { cac } from './package/dist/index.js'

function parse(value, type) {
  const cli = cac('probe')
  cli.option(
    '--policy <id>',
    'Policy identifier',
    type === undefined ? undefined : { type },
  )
  return cli.parse(
    ['node', 'probe', '--policy', value],
    { run: false },
  ).options.policy
}

console.log(JSON.stringify({
  text: parse('alpha'),
  leadingZero: parse('001'),
  leadingPlus: parse('+2'),
  stringArrayLeadingZero: parse('001', [String]),
  stringArrayLeadingPlus: parse('+2', [String]),
}))
```

Run it from the disposable directory:

```sh
cd -- "${work}"
node probe.mjs
```

CAC 7.0.0 returns:

```json
{"text":"alpha","leadingZero":1,"leadingPlus":2,"stringArrayLeadingZero":["1"],"stringArrayLeadingPlus":["2"]}
```

### Working catalog

- `alpha` remains exact because unary plus produces `NaN`.
- Post-`--` tokens remain exact because CAC removes them before MRI parsing and stores them under `options['--']`.
- Plain positional tokens remain exact in the tested one-level command forms.

### Failing catalog

- Leading zero:
  `001` becomes `1`.
- Leading plus:
  `+2` becomes `2`.
- Explicit array transform:
  `{ type: [String] }` yields `['1']` or `['2']`,
  not the original strings.
- Repeated option values:
  each numeric-looking occurrence is converted before CAC forms the result array.

### Upstream prototype

A fresh disposable clone at the pinned tag was created at
`~/temp/agent/upstream-prototype.lgbktubA`.
The [prototype patch](cac-option-value-coercion.patch) adds MRI string metadata for every nonboolean declared option and
adds four upstream-style assertions.

The same assertion harness ran before and after the patch in a network-disabled,
read-only Node 24.18.0 container with 2 GiB memory and 2 CPUs.

Pre-patch result:

```text
exit=1
AssertionError [ERR_ASSERTION]: Expected values to be strictly deep-equal
```

Post-patch result:

```text
exit=0
{"leadingZero":"001","leadingPlus":"+2","stringArrayLeadingZero":["001"],"stringArrayLeadingPlus":["+2"]}
```

Evidence hashes:

- patch SHA-256:
  `b3362641520169a1068f172ecdd6e0e2e9b4f8663bc8061bec2f276c221eb0d2`;
- pre-patch stderr SHA-256:
  `71e32f3bd24eb8af985b0a44ce9e6a805df98819fc71854feef692e8dd60b4b7`;
- post-patch stdout SHA-256:
  `a181e42962cf3e3aa6f9d33ae81b006874d0bda3202f18e3751eb8075435d185`.

The patch intentionally addresses numeric coercion only.
It does not fix dash-led declared values,
lone `-`,
or kebab-case boolean metadata.

## Verified workarounds

### Preserve selected values before CAC

Scan only the owned option region before CAC:

1.  Capture exact separated and joined `--policy` values.
2.  Replace each captured value with a non-dash,
    nonnumeric placeholder.
3.  Let CAC route and validate the command.
4.  Restore captured strings in the application result.

The cli-git management prototype matched the incumbent on all 44 catalog cases.
The catalog included `001`,
`+2`,
`-x`,
`--all` as a policy value,
joined values,
repetition,
unknown options,
and post-`--` pathspecs.

Tradeoff:
cli-git still owns a value-aware argv scan.
Adding CAC does not remove lexical parser responsibility,
and every new exact-value option must join that scanner.

### Read exact values from `rawArgs`

CAC stores the provided argv array on `cli.rawArgs`.
A consumer can find a known option and read the following token directly.

Tradeoff:
this requires owned handling for aliases,
joined values,
repetition,
`--`,
missing values,
and option arity.
It is suitable for a narrow known option,
not as a replacement for cli-git's Git-region parser.

### Carry the source patch in a fork

The prototype patch preserves declared nonboolean option strings before CAC transforms.

Tradeoff:
this changes CAC's historical numeric coercion and may be breaking for consumers that depend on inferred numbers.
A fork also transfers release,
security,
and merge maintenance to the consumer.
It does not solve cli-git's other CAC parity failures.

## What does not work

- Shell quotes do not help.
  The shell removes quoting syntax before Node receives argv,
  so CAC sees the same string for `001` and `"001"`.
- `{ type: [String] }` does not preserve the input.
  It converts the already damaged number and wraps the result in an array.
- `String(parsedValue)` does not restore a leading zero or plus.
- `allowUnknownOptions()` does not change MRI value conversion.
- Reading `options['--']` helps only tokens after `--`.
  It does not affect declared option values before the separator.
- Open PR [`cacjs/cac#163`][pr-163] contains the same string-metadata direction,
  but the fix is absent from CAC 7.0.0 and remains unmerged.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` contains no CAC,
argv-parser,
or CLI-parser exemption.

1.  **Is it really upstream's fault?**
    Yes.
    CAC declares `<value>` options but omits MRI string metadata,
    so its inlined parser changes exact argv before returning it.
2.  **Can upstream fix it?**
    Yes.
    The prototype adds one MRI option category and focused tests without changing CAC's architecture.
3.  **Are they supporting this use case?**
    Yes.
    CAC documents required option values and `{ type: [String] }` transforms.
    Issue `#165` is an upstream report for this exact surface.
4.  **Would the repository welcome our contribution?**
    Yes.
    `README.md` gives a pull-request workflow,
    `.github/ISSUE_TEMPLATE.md` requests reproductions and possible solutions,
    and recent external PRs have merged.
    No `CONTRIBUTING.md`,
    pull-request template,
    security policy,
    or AI-assisted contribution ban was found.
5.  **Will they likely fix it?**
    Soft yes under the filing rule.
    No maintainer has rejected the behavior or declared numeric coercion a non-goal.
    Issue `#165` and PR `#163` remain open and unassigned,
    so response timing is unknown.
6.  **Have we prototyped a minimal compatible fix?**
    Yes.
    The linked patch fails the assertion harness before the change and passes after it.
    It also adds upstream-style regression cases.

The duplicate search covered open and closed issues and pull requests using numeric coercion,
string option,
leading zero,
leading plus,
MRI,
and option type terms.
Issue [`#165`][issue-165] is the matching issue.
PR [`#163`][pr-163] is the matching candidate fix direction.
Do not open a new issue.

The following additive comment is ready for human review but was not posted:

~~~md
I reproduced this on the published `cac@7.0.0` artifact and traced it through the pinned source.

`getMriOptions()` in `src/utils.ts` supplies `alias` and `boolean`, but not MRI's `string` list. MRI 1.2.0 therefore reaches its numeric fallback in `toVal()` before CAC applies `type: [String]`. That explains why `[String]` produces an array containing the already changed value.

I also tested the `string`-metadata direction from #163 against the v7.0.0 tag with these cases:

- `--policy 001` stays `'001'`;
- `--policy +2` stays `'+2'`;
- `{ type: [String] }` produces `['001']` and `['+2']`.

The same assertion harness failed before the source change and passed after it. The minimal source change adds `string: string[]` to `MriOptions`, initializes it, and adds each nonboolean option's canonical name. MRI expands aliases from the existing alias map.

I can provide the two-hunk source-and-test patch if maintainers want to continue this direction. It changes historical inferred-number behavior, so it may need release-note treatment.
~~~

[issue-165]: https://github.com/cacjs/cac/issues/165
[pr-163]: https://github.com/cacjs/cac/pull/163
