# Jackspeak 4.2.3, type-flag 4.5.0, and Argue 3.1.0 lose partial Git argv semantics

Note 2026-11-09T16:47:44-05:00:
`package/git-policy/cli/src/parser/argv.unit.test.ts` left the tree on 2026-10-13 in `642b6d72cb`.

## Symptom

A cli-git parser declares only policy-relevant Git options.
Unknown options must remain forwardable,
known value options must consume dash-led values,
and exact option spelling must survive.
The three tested npm parsers handle an ordinary closed schema but diverge on those partial-grammar cases.

Jackspeak rejects valid unknown-option shapes.
Its one-line message is the following segments joined by one space:

```text
Unknown option '-q'.
To specify a positional argument starting with a '-', place it at the end of the command after '--', as in '-- -q'
```

Type-flag changes token roles:

- `-m -a` becomes message `''` plus declared flag `-a`,
  rather than message value `-a`;
- terminal `-m` becomes message `''`,
  rather than a missing-value failure;
- `-all` becomes short group `-a -l -l`,
  rather than one unknown exact token;
- `-q path` separates unknown flag `q` from positional `path`,
  rather than retaining the current tentative unknown-option pair.

Argue changes option boundaries:

- `-- -a` still parses `-a` as a flag;
- both undeclared `--a` and undeclared `-all` match aliases declared only as `-a` and `--all`;
- unknown options and positionals remain in one process-global leftover array without token roles.

These are package capability gaps for cli-git's partial Git grammar.
They do not imply that ordinary closed-schema CLI use is broken.

## Root cause

### Jackspeak restores strict unknown-option rejection after `util.parseArgs`

Jackspeak calls Node's parser with `strict: false` and token output
(`~/temp/agent/jackspeak-2026-08-14/src/index.ts:708-716`):

```ts
const result = parseArgs({
  args,
  options: toParseArgsOptionsConfig(this.#configSet),
  // always strict, but using our own logic
  strict: false,
  allowPositionals: this.#allowPositionals,
  tokens: true,
})
```

It then rejects every option token absent from its config
(`~/temp/agent/jackspeak-2026-08-14/src/index.ts:721-761`):

```ts
for (const token of result.tokens) {
  // ...
  const my = this.#configSet[token.name]
  if (!my) {
    throw new Error(
      `Unknown option '${token.rawName}'. ` +
        `To specify a positional argument starting with a '-', ` +
        `place it at the end of the command after '--', as in ` +
        `'-- ${token.rawName}'`,
```

The parser therefore cannot return cli-git's unknown-option facts.
Catching the error recovers no successful parse result.
Declaring every current Git option would replace a deliberately partial grammar
with a complete version-specific grammar.

### Type-flag settles a pending value before classifying the next flag

Type-flag's iterator calls a pending value callback with `undefined` before it dispatches a newly encountered flag
(`~/temp/agent/type-flag-2026-08-14/src/argv-iterator.ts:95-123`):

```ts
const parsedFlag = parseFlagArgv(argvElement);

if (parsedFlag) {
  triggerValueCallback();

  if (!onFlag) {
    continue;
  }

  const [flagName, flagValue, isAlias] = parsedFlag;

  if (isAlias) {
    for (let j = 0; j < flagName.length; j += 1) {
      triggerValueCallback();
      // ...
```

The stable 4.5.0 consumer turns that absent callback value into an empty string
(`~/temp/agent/type-flag-2026-08-14/src/type-flag.ts:91-111`):

```ts
const getFollowingValue = (
  value?: string | boolean,
  valueIndex?: Index,
) => {
  // ...
  values.push(
    applyParser(parser, value || ''),
  );
};
```

`getFlag` uses the same iterator and the same empty-string fallback
(`~/temp/agent/type-flag-2026-08-14/src/get-flag.ts:29-60`),
so extracting one flag at a time does not preserve `-m -a` or distinguish missing value from explicit empty value.

Short tokens are always iterated as alias groups.
That is useful for conventional `-abc` parsing,
but cli-git's generic parser recognizes exact names
and delegates command-specific cluster handling to separate scanners.

Unknown flags are stored in a name-keyed object while positional arguments are stored separately
(`~/temp/agent/type-flag-2026-08-14/src/type-flag.ts:120-140`).
That output cannot retain the exact interleaved token sequence on its own.

### Argue removes prefix information and never terminates option scanning

Argue stores one mutable argv array at module scope
(`~/temp/agent/argue-cli-2026-08-14/src/argv.ts:1-24`):

```ts
const ARGV_START_INDEX = 2

export const argv = process.argv.slice(ARGV_START_INDEX)

export function setArgs(...args: string[]) {
  argv.splice(0, argv.length)
  argv.push(...args)
}
```

Its option matcher excludes bare `--` from option syntax,
but the main loop continues beyond it.
It also removes either one or two leading dashes before a reader sees a name
(`~/temp/agent/argue-cli-2026-08-14/src/options.ts:12-22`):

```ts
function isOption(arg: string) {
  return /^--?[^-].*/.test(arg)
}

function removePrefix(arg: string) {
  return arg.replace(/^--?/, '')
}
```

The loop applies that normalized name and advances past non-options without entering a terminated state
(`~/temp/agent/argue-cli-2026-08-14/src/options.ts:94-106`):

```ts
while (arg) {
  if (isOption(arg)) {
    [arg, optionEqValue] = splitOption(arg)
    optionResult = readOption(removePrefix(arg), read, options)

    if (optionResult) {
      remove()
      Object.assign(options, optionResult)
    }
  }

  next()
}
```

A custom reader can count repeated flags and collect ordered values.
It cannot distinguish `-a` from `--a` after `removePrefix` has discarded the evidence.
Unknown options remain exact in the leftover array,
but separating them from positionals requires cli-git's own unknown-consumption heuristic.

## Verification

### Versions and artifacts

The test used:

- `jackspeak@4.2.3`,
   tag `v4.2.3`,
  commit `a58b42f39e2fb04b28b8169005a5ddbc3302730e`;
- `type-flag@4.5.0`,
   tag `v4.5.0`,
  commit `6e0c46911ea64c829459a27bfaf1b45e8e335869`;
- `argue-cli@3.1.0`,
   tag `v3.1.0`,
  commit `45db68f4acce979d0ba725ae83e320a0e906165a`.

Each downloaded tarball matched npm's SHA-512 integrity.
The complete provenance and integrity records are in
`doc/audit/tech-cli-git-command-line-argument-parser-replac-vet-2026-08-14.md`.

### Minimal harness

The following program is the executed reduced reproduction.
Save it as `~/temp/agent/npm-cli-parsers-partial-git-argv-minimal-2026-08-14.mjs`
after extracting the exact tarballs under the candidate mount paths used by the command.

```js
// ~/temp/agent/npm-cli-parsers-partial-git-argv-minimal-2026-08-14.mjs
import { jack } from '/candidate/jackspeak/dist/esm/index.min.js';
import { typeFlag } from '/candidate/type-flag/dist/index.mjs';
import { readOptions, rest, setArgs } from '/candidate/argue-cli/dist/index.js';

const capture = (fn) => {
  try {
    return { returned: fn() };
  } catch (error) {
    return { threw: error instanceof Error ? error.message : String(error) };
  }
};

const jackspeak = jack({ allowPositionals: true })
  .flagList({ all: { short: 'a' } })
  .optList({ message: { short: 'm' } });

const runTypeFlag = (args) => typeFlag({
  all: { type: [Boolean], alias: 'a' },
  message: { type: [String], alias: 'm' },
}, [...args]);

const names = new Set(['all', 'a']);
const countAll = (option, _read, options) => (
  names.has(option)
    ? { all: (typeof options.all === 'number' ? options.all : 0) + 1 }
    : null
);
const runArgue = (args) => {
  setArgs(...args);
  return { options: readOptions(countAll), leftovers: rest() };
};

console.log(JSON.stringify({
  positive: {
    jackspeak: jackspeak.parseRaw(['-a', '-m', 'msg', 'path']),
    typeFlag: runTypeFlag(['-a', '-m', 'msg', 'path']),
    argueCli: runArgue(['-a', 'path']),
  },
  jackspeakUnknown: capture(() => jackspeak.parseRaw(['-q', 'path'])),
  typeFlagDashValue: runTypeFlag(['-m', '-a']),
  typeFlagMissingValue: runTypeFlag(['-m']),
  typeFlagShortGroup: runTypeFlag(['-all']),
  argueTerminator: runArgue(['--', '-a']),
  argueLongShortAlias: runArgue(['--a']),
  argueShortLongAlias: runArgue(['-all']),
}, null, 2));
```

The exact bounded invocation was:

```sh
# doc/troubleshooting/npm-cli-parsers-partial-git-argv.md
podman run \
  --memory=2g \
  --cpus=2 \
  --pids-limit=128 \
  --ulimit nofile=1024:1024 \
  --rm \
  --network none \
  --read-only \
  --cap-drop=ALL \
  --security-opt=no-new-privileges \
  --tmpfs /tmp:rw,size=64m \
  --env HOME=/tmp/home \
  --volume "${HOME}/temp/agent/npm-cli-parsers-partial-git-argv-minimal-2026-08-14.mjs:/probe/minimal.mjs:ro,Z" \
  --volume "${HOME}/temp/agent/jackspeak-artifact-2026-08-14/extracted/package:/candidate/jackspeak:ro,Z" \
  --volume "${HOME}/temp/agent/type-flag-artifact-2026-08-14/extracted/package:/candidate/type-flag:ro,Z" \
  --volume "${HOME}/temp/agent/argue-cli-artifact-2026-08-14/extracted/package:/candidate/argue-cli:ro,Z" \
  docker.io/library/node:24-slim \
  node /probe/minimal.mjs
```

The command exited zero on Node 24.18.0 and Linux x64.
The harness SHA-256 is
`2c3aeb1ea7b8e4739e9635ea43998a744ded1c8619b705b879150040d46dcd4b`.
The output SHA-256 is
`cc184f117955d6479348efa4983ce1529bd1b62e00420188e4d4014d854e0123`.

### Working catalog

All three packages handled these configured cases in the complete matrix:

- one ordinary boolean flag;
- one ordinary string value;
- one positional;
- repeated known flags and values when configured through available APIs;
- a known option after a positional;
- a lone dash;
- joined equals values.

Jackspeak and type-flag honored `--` termination.
Jackspeak and Argue consumed dash-led declared values and failed on missing declared values.
Type-flag returned unknown joined-option facts.
Argue retained exact unknown input in its leftover array.

### Failing catalog

Jackspeak failed both unknown forms:

- `-q path`;
- `--unknown=value path`.

Type-flag failed these exact roles:

- `-m -a`;
- terminal `-m`;
- exact token `-all`;
- interleaved unknown option and following plain token.

Argue failed these exact boundaries:

- `-- -a`;
- declared `-a` versus undeclared `--a`;
- declared `--all` versus undeclared `-all`;
- separate unknown-option and positional result roles.

## Verified workarounds

### Retain cli-git's owned parser

The current parser explicitly models all failing cases
and keeps command-specific short-cluster logic outside the generic exact-name scan.
Its direct unit catalog is
`package/git-policy/cli/src/parser/argv.unit.test.ts`.
The prior CAC assessment also exercised a 52-case management parity catalog before rejecting that migration shape:
`doc/audit/tech-cli-git-parser-migration-to-cac-vet-2026-08-14-a48b54e2.md`.

`package/git-policy/cli/src/parser/argv.unit.test.ts` was removed on 2026-10-13 in `642b6d72cb`
(refactor(*):
delete the TypeScript Git policy wrapper,
its tasks,
bins and trust workflow);
no
file in the working tree carries it now.

Tradeoff:
cli-git continues to own and test generic token-role logic.
That ownership is the behavior these packages do not jointly provide.

### Use the packages only for closed grammars matching their contracts

The positive control proves each package can parse its declared ordinary options.
Jackspeak fits strict closed schemas.
Type-flag fits typed value extraction when dash-led values follow its documented rules.
Argue fits sequential global parsing when prefix normalization and mutable state are acceptable.

Tradeoff:
this does not replace cli-git's shared partial-Git parser.
Using one only for management commands leaves the shared parser
and handwritten command dispatch or result adaptation in place.

## What does not work

### Catching Jackspeak's unknown-option error

The throw contains a message and cause,
not a successful token stream.
Stripping the unknown token and retrying loses its relationship to a following value and introduces a new parser loop.

### Using type-flag `getFlag` repeatedly

`getFlag` shares `argvIterator` and the same `implicitValue || ''` fallback.
It therefore cannot recover arbitrary dash-led values or distinguish missing from explicitly empty input.
Repeated reparsing also does not reconstruct the original interleaving after tokens are removed.

### Using only Argue custom readers

Custom readers recover counts and ordered known values.
They receive names only after one or two dashes have been removed,
and `readOptions` has no terminated state.
A pre-split fixes `--`,
but exact prefix handling still needs inspection before Argue,
and unknown-role classification still needs inspection after it.

### Treating type-flag beta negative-number support as the stable fix

Pull request <https://github.com/privatenumber/type-flag/pull/71> adds selected negative-number values to
5.0.0 beta.
It intentionally lets a token that resolves to defined aliases win as flags.
It does not make arbitrary dash-led strings such as `-a` values,
change stable 4.5.0,
or fix missing-value and exact-token requirements.

## Upstream filing decision

No matching `.out-of-scope/` exemption exists.
The search inspected every file in `.out-of-scope/` and found no entry for these packages or this parser class.

Duplicate searches covered open and closed issues and pull requests:

- Jackspeak query `unknown option pass through` found none.
- Type-flag queries `missing value` and `negative value` found pull request 71,
  which was read with its comments and changed-file patches.
- Argue queries `double dash terminator` and `option prefix short long` found none.

### Jackspeak constraints

1.  Is it really upstream's fault?
    No.
    Strict rejection is the package's stated design and matches its implementation.
2.  Can upstream fix it?
    Yes,
    by adding a separate permissive token result,
    but that would be a new mode rather than correction of the tested contract.
3.  Are they supporting this use case?
    No.
    The README says unrecognized configs throw and presents Jackspeak as strict.
4.  Would the repository welcome the contribution?
    No contribution policy or AI-assisted filing ban was found in the README,
    repository root,
    or `.github/` workflows.
5.  Will they likely fix it?
    Not applicable after constraints 1 and 3 fail.
6.  Has a minimal fix been prototyped?
    No.
    The automatic prototype gate does not apply because constraints 1 through 5 do not all hold or softly hold.

Upstream artifact:
nothing to add.
Do not file an issue that asks a strict parser to become cli-git's partial Git parser.

### Type-flag constraints

1.  Is it really upstream's fault?
    No for stable 4.5.0.
    Its README explicitly documents empty missing string values and dash-led value tradeoffs.
2.  Can upstream fix it?
    Yes,
    but arbitrary dash-led values require a new token-role rule beyond beta pull request 71.
3.  Are they supporting this use case?
    No.
    The package is a focused typed flag parser,
    not a partial external-grammar parser preserving exact unknown token order.
4.  Would the repository welcome the contribution?
    No contribution policy or AI-assisted filing ban was found in the README,
    repository root,
    or `.github/` configuration.
5.  Will they likely fix it?
    The maintainer has actively changed beta tokenization,
    but the current documented stable behavior is intentional.
6.  Has a minimal fix been prototyped?
    No.
    The automatic prototype gate does not apply because constraints 1 and 3 fail.

Upstream artifact:
nothing to add to pull request 71.
Its negative-number implementation and limits are already documented in the thread.

### Argue constraints

1.  Is it really upstream's fault?
    No established defect was found.
    Prefix normalization and continued scanning are architectural behavior,
    while exact partial-Git spelling is not a documented contract.
2.  Can upstream fix it?
    Yes,
    by exposing raw spelling to readers and adding explicit terminator state.
3.  Are they supporting this use case?
    No.
    The README documents sequential application CLI parsing,
    not a partial external grammar with ordered unknown-option facts.
4.  Would the repository welcome the contribution?
    Issue templates exist and no contribution or AI-assisted filing ban was found in the README,
    repository root,
    or `.github/` files.
5.  Will they likely fix it?
    No direct maintainer signal exists for this use case.
6.  Has a minimal fix been prototyped?
    No.
    The automatic prototype gate does not apply because constraints 1 and 3 fail.

Upstream artifact:
nothing to add.
Do not file a cli-git-specific feature request without upstream support for the underlying use case.
