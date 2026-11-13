# Optique removal handover

> References below this note that no longer resolve in the tree.
> Each entry gives the value that identifier has now:
> `package/cli/mutation-test/package.json` is now `package/mutation-test/cli/package.json`
> (2026-10-29 `113633b259`).

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/watch-restart` is now `package/watch-restart/cli`;
> `package/dev-script/backup-path` is now `package/backup-path/cli`;
> `package/dev-script/task-util` is now `package/monorepo-task/core-exec`;
> `package/build-tool/css` is now `package/css/cli`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/token-count` is now `package/llm/token-count`;
> `package/module/image-diff` is now `package/llm/image-diff`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/fy` is now `package/fy/cli`;
> `package/cli/git-clone-size` is now `package/git-clone-size/cli`;
> `package/cli/mutation-test` is now `package/mutation-test/cli`;
> `package/cli/mvm` is now `package/mvm/cli`;
> `package/cli/vmsync` is now `package/vmsync/cli`.

## Status

Planning and user grilling are in progress.
No parser implementation or consumer migration is authorized until the user confirms shared understanding.

Keep this handover current when requirements,
decisions,
evidence,
rejected designs,
or the next action change.

## User intent

- Replace Optique with a repository-owned parser based on Valibot.
- Migrate existing Optique consumers,
  except `package/git-policy/cli/`.
- Include the paused consumer under `package-paused/dev-script/inference-canary/`.
- Do not expand this migration to handwritten parsers that do not use Optique.
- Preserve historical troubleshooting records,
  marking superseded recommendations where appropriate rather than deleting durable evidence.
- Settle observable parser results and grammar before choosing implementation architecture.
- Ask one decision question at a time.
- Demonstrate every concept in an option comparison with a concrete sketch.

## Settled scope

The active workspace has thirteen non-cli-git manifests declaring an Optique dependency.
`package/cli/mutation-test/package.json` has no matching source import and appears to contain stale dependencies.

Current non-cli-git source imports occur in twenty-one TypeScript files across these consumers:

- `package/build-tool/css/`
- `package/claude-code-plugin/source/`
- `package/cli/fy/`
- `package/cli/git-clone-size/`
- `package/cli/mvm/`
- `package/cli/vmsync/`
- `package/dev-script/backup-path/`
- `package/dev-script/task-util/`
- `package/dev-script/watch-restart/`
- `package/module/image-diff/`
- `package/module/token-count/`
- `package/pi-plugin/spawn/`

`package/test-fixture/oxlint-no-restricted-syntax/` also imports Optique as a lint fixture.
The migration must replace that fixture without changing the rule it tests.

`package/git-policy/cli/` is explicitly out of scope,
including concurrent work replacing its parser.
Do not edit,
move,
or absorb that work into this migration.
Optique catalog and lockfile entries may therefore remain while cli-git still needs them.
Since cli-git dropped Optique (2026-07-15 `be3c522375` dropped it from the management facade),
the entries remain only for the thirteen manifests listed here and the paused consumer,
so no outside dependency holds them open.
Success is no Optique dependency or import outside cli-git,
not zero repository-wide string matches.

Re-measured 2026-11-10 at `ac6f5822b1`:
same thirteen manifests,
same twenty-one consumer files,
and `package/git-policy/cli/` holds no Optique reference in its manifest or source.
`package/mutation-test/cli/` still declares both packages with zero imports,
so it is removable without waiting on the parser design.

## Current result-shape proposal

The user prefers a command grammar whose structured options precede a required `--` separator.
Every token after the separator is captured without option interpretation:

```text
image-tool compare
  [-p, --provider voyage|gemini]
  [-t, --tag <TAG> [<TAG> ...]]
  [--json]
  -- <1> <2> [<3> ...]

image-tool cmp ...    hidden alias for compare
```

The proposed successful result is:

```ts
{
  commandPath: ['compare'],
  trailingArguments: ['a.png', 'b.png'],
  provider: 'voyage',
  tags: ['release', 'nightly'],
  json: true,
}
```

The proposed static shape is:

```ts
type CompareArgs = Readonly<{
  commandPath: readonly ['compare'];
  trailingArguments: readonly [string, string, ...string[]];
  provider?: 'voyage' | 'gemini';
  json?: true;
  tags?: readonly [string, ...string[]];
}>;
```

Valibot `1.4.2` can represent the non-empty tuple tail directly:

```ts
const trailingArgumentsSchema = v.tupleWithRest(
  [
    v.string(),
    v.string(),
  ],
  v.string(),
);
```

This proposal deliberately avoids assigning semantic field names to tail positions.
The parser should preserve tail order and accept dash-led values after `--` verbatim.
The command alias `cmp` should canonicalize to `commandPath: ['compare']` and remain absent from generated help.

## Settled result-shape decisions

- `provider` is omitted when `--provider` is absent.
  The parser does not choose a provider or implement the command's business policy.
- `trailingArguments` preserves a schema-constrained tail after `--`.
- Commands accept arguments only through a required `--` separator.
  Options occur before it;
  every later token is preserved as tail data without option interpretation.
- The shared parser has no named-positional feature.
  Commands without arguments require no separator.
- Each command's syntax-level Valibot schema defines its tail cardinality.
- Presence flags use absent-or-`true` rather than always-present booleans.
- Collection-valued option output uses an optional non-empty tuple.
  For example,
  absent `--tag` omits `tags`,
  while one valid occurrence can produce `tags?: readonly [string, ...string[]]`.
- Repeating `--tag` is invalid.
  Output cardinality does not imply repeated option occurrences.
- One `--tag` occurrence consumes one or more plain argv tokens until the next option or final `--`.
  A dash-led token ends the collection and is interpreted as an option.
  Dash-led tag values are not supported by this syntax.
- Every declared option may occur at most once,
  regardless of whether repeated occurrences use the same spelling or different aliases.
  Duplicate scalar,
  presence,
  and collection-valued options are usage errors;
  no occurrence silently overrides or merges another.
- Valued options accept only separate value tokens.
  Joined long values such as `--provider=voyage` and attached short values such as `-pvoyage` are unknown options.
  Scalar and variadic options use the same separation rule.
- Short aliases must occupy exact argv tokens.
  Clusters such as `-jq` and `-jqp` are unknown options even when each character names a declared alias.
- Long options match exact declared aliases only.
  Unambiguous prefixes and fuzzy corrections are not accepted syntax;
  they may appear only as non-executing diagnostic suggestions.
- Result cardinality preserves syntax:
  zero-or-one uses `value?: T`,
  zero-or-many uses `values?: readonly [T, ...T[]]`,
  and one-or-many uses `values: readonly [T, ...T[]]`.
- Generated help uses display-only metavars for tail positions and rest values.
  Metavars improve usage and missing-value diagnostics but never become result keys.
- A commandless CLI is a root leaf and has no `commandPath` property.
  The parser never invents a program-name path absent from argv.
- Subcommands may nest to arbitrary declared depth,
  as in `cat calico meow --loud`.
- Intermediate command nodes are routing-only.
  Only the innermost leaf command may declare options or a trailing-argument tail.
- Command-bearing results flatten canonical command tokens into a readonly non-empty `commandPath` tuple.
  Leaf options and `trailingArguments` remain top-level properties.
  A single-level command uses `commandPath: readonly ['compare']`;
  nested syntax can use `commandPath: readonly ['calico', 'meow']`.
- Hidden aliases canonicalize their own path segment before output.
- Each leaf may apply a whole-result Valibot schema after field parsing
  to validate relationships across supplied options and trailing arguments.
  This schema may transform values while preserving declared field layout,
  but may not rename,
  nest,
  remove,
  merge,
  add,
  or default result fields.
  It may not consult external state.
  Failed relationship checks are parser failures with status 2.
- Syntax schemas are synchronous only.
  Async Valibot schemas are rejected during definition validation;
  async work begins after successful parsing.
- `--help` and `-h` produce contextual help at the root,
  every intermediate route,
  and every leaf.
  They are exact reserved parser controls and never appear
  in an application success value or reach the application handler.
- A control-position help spelling is exclusive:
  combining it with an option,
  another command token after it,
  or a trailing-argument separator is a usage error.
  Command tokens before it select the help context and do not count as a combination.
- Literal `--help` and `-h` tokens after the trailing-argument separator remain data under the settled separator rule.
- Contextual help writes to stdout and settles with status 0.
  Parser usage and validation failures write every independently recoverable diagnostic
  followed by full contextual help to stderr and settle with status 2.
  Grammar issues are ordered by argv token index,
  tail issues follow option-region issues,
  and independent Valibot issues preserve schema order.
  Recovery proceeds only from known token boundaries and never reinterprets input to manufacture cascades.
  Failure help uses the deepest canonically resolved command route,
  even when argv used aliases.
  Neither path invokes the application handler.
  Application failures retain their own status and output policy.

## Unresolved result-shape decisions

No known result-shape decision remains open.
Other grammar and diagnostic decisions remain open.

## Architecture discussion

Three implementation models were demonstrated but not selected:

- CLI descriptors containing Valibot schemas.
- A Valibot result schema plus separate token bindings.
- CLI metadata attached to Valibot schemas.

The grammar and successful result interface are now sufficiently settled to revisit the module seam.
A thin wrapper that merely calls `v.parse` has no reason to exist.
The owned module must earn its keep by compiling and enforcing argv grammar around Valibot validation.

Candidate owned responsibilities are:

- validate and compile command-tree definitions once;
- route exact canonical commands and aliases into a `commandPath`;
- enforce routing-only intermediate nodes and leaf-only options;
- scan exact option tokens,
  separate values,
  variadic option values,
  duplicate occurrences,
  and the required trailing separator;
- assemble syntax-preserving result cardinality and omission;
- invoke synchronous Valibot schemas at the correct token boundaries;
- map Valibot issues to argv indices,
  spellings,
  metavars,
  and deterministic aggregated diagnostics;
- generate contextual help for every route;
- gate application dispatch and implement stdout/status 0 help plus stderr/status 2 failures.

Valibot remains the value and relationship validation engine behind this grammar seam.
It is not the whole parser.

The user accepted this value proposition and module seam.
`compileCli(definition)` is the deep core:
it validates and compiles grammar,
precomputes routing and help,
and returns pure parsing and rendering behavior.
A process adapter such as `runCli` owns argv and stream integration plus application dispatch gating.
The application callback receives only successful syntax results.

## Speculative representations beyond the first three

The user suspects a materially better definition model than all three demonstrated options.
That suspicion is credible.
The first comparison treated a handwritten whole-result schema as inevitable,
which created duplication that the compiler may be able to eliminate.
No representation below is accepted yet.

### Leading speculation: grammar algebra with derived schema

Make an owned grammar algebra the only source of argv structure.
Each field constructor contributes:

- token grammar;
- help metadata;
- occurrence cardinality;
- a synchronous Valibot value schema;
- its inferred output field type.

`compileCli` derives the leaf result type and synthesizes whole-result Valibot validation.
The user never repeats the assembled object shape.

```ts
const meow = leaf({
  options: {
    volume: valueOption({
      names: ['--volume'],
      metavar: 'LEVEL',
      description: 'Volume from 0 to 10.',
      schema: v.pipe(
        v.string(),
        v.transform(Number,),
        v.number(),
        v.minValue(0,),
        v.maxValue(10,),
      ),
    },),
    tags: variadicOption({
      names: ['--tag'],
      metavar: 'TAG',
      description: 'One or more tags.',
      itemSchema: v.string(),
    },),
    loud: presenceOption({
      names: ['--loud'],
      description: 'Use loud output.',
    },),
  },
  trailingArguments: trailing({
    items: [
      {
        metavar: 'FIRST-SOUND',
        schema: v.string(),
      },
      {
        metavar: 'SECOND-SOUND',
        schema: v.string(),
      },
    ],
    rest: {
      metavar: 'ADDITIONAL-SOUND',
      schema: v.string(),
    },
  },),
},).check(
  v.check(
    function loudHasVolume(result,) {
      return result.loud !== true
        || result.volume !== undefined;
    },
    '--loud requires --volume.',
  ),
);

const catGrammar = program({
  name: 'cat',
  commands: {
    calico: route({
      aliases: [
        {
          name: 'c',
          hidden: true,
        },
      ],
      commands: {
        meow,
      },
    },),
  },
},);

export type CatCliResult = CliOutput<typeof catGrammar>;

export const catCli: CompiledCli<CatCliResult> = compileCli(catGrammar,);
```

The intended derived leaf member is:

```ts
type MeowCliResult = Readonly<{
  commandPath: readonly ['calico', 'meow'];
  volume?: number;
  tags?: readonly [string, ...string[]];
  loud?: true;
  trailingArguments: readonly [string, string, ...string[]];
}>;
```

The compiler can synthesize a Valibot object schema internally from public constructors:

- scalar value option becomes an optional entry whose output is `v.InferOutput` of its value schema;
- presence option becomes an optional literal-`true` entry;
- variadic option maps its item schema over a grammar-enforced non-empty tuple;
- trailing items and rest schemas derive the exact output tuple;
- canonical route nodes derive the literal `commandPath` tuple;
- `.check(...)` contributes shape-preserving whole-result validation without a duplicate object schema.

This is not arbitrary parser-combinator composition.
It is a closed owned algebra that compiles into one immutable grammar AST.
The internal AST may resemble descriptor-first design,
but callers no longer maintain a parallel whole-result schema.

Why it may beat the first three designs:

- unlike descriptor plus `resultSchema`,
  it does not repeat field layout;
- unlike schema plus bindings,
  grammar cardinality and output cardinality come from one field node;
- unlike Valibot metadata,
  it does not inspect arbitrary schema wrappers or depend on beta metadata;
- unlike an Optique-compatible algebra,
  it exposes only repository-approved grammar forms.

### Variant: result-first grammar field wrappers

Another possibility is to make owned field wrappers feel schema-first without attaching metadata to arbitrary schemas:

```ts
const meow = cliLeaf({
  volume: cliValue({
    names: ['--volume'],
    schema: volumeSchema,
    metavar: 'LEVEL',
  },),
  tags: cliVariadic({
    names: ['--tag'],
    itemSchema: v.string(),
    metavar: 'TAG',
  },),
  loud: cliPresence({
    names: ['--loud'],
  },),
  trailingArguments: cliTrailing({
    items: [firstSound, secondSound],
    rest: additionalSound,
  },),
},);
```

Each wrapper is an owned grammar node containing a Valibot schema,
not itself a Valibot `BaseSchema`.
`compileCli` recognizes only these owned nodes.

Do not implement grammar nodes as custom Valibot schemas without a source prototype.
The installed Valibot `1.4.2` `BaseSchema` protocol exposes `~run` and `~standard` as internal members.
Depending on that protocol would recreate the coupling rejected in the metadata design.

This variant may simply be a naming and layout form of the leading grammar algebra.
Its value is worth testing through real declarations before treating it as a separate architecture.

### Variant: compiled argv codec

The compiled result could expose a schema-like codec from argv to syntax output:

```ts
const catCli = compileCli(catGrammar,);

const parsed = catCli.parse(argv,);

// Speculative only:
const argvSchema: StandardSchemaV1<readonly string[], CatCliResult> = catCli.schema;
```

This would make the entire grammar composable as a schema from `readonly string[]` to the inferred result.
It could support consumer pipelines after syntax parsing.

Risks:

- help requests and aggregated provenance diagnostics are not ordinary value-validation failures;
- a Valibot-compatible implementation may require internal schema protocol members;
- schema composition could bypass the process adapter's help and status contracts;
- exposing both `.parse` and `.schema` may create two public seams.

Treat the codec as an optional adapter only if a prototype can use
public Standard Schema contracts without weakening parser controls.
It is not the leading core representation.

### Variant: event grammar as an internal implementation

The scanner could emit a provenance-preserving event stream before result assembly:

```ts
type CliEvent =
  | Readonly<{ kind: 'command'; tokenIndex: number; canonical: string; }>
  | Readonly<{ kind: 'option'; tokenIndex: number; key: string; spelling: string; }>
  | Readonly<{ kind: 'value'; tokenIndex: number; optionKey: string; value: string; }>
  | Readonly<{ kind: 'separator'; tokenIndex: number; }>
  | Readonly<{ kind: 'trailing'; tokenIndex: number; value: string; }>;
```

A reducer would apply field schemas,
assemble the syntax result,
and aggregate diagnostics using original token provenance.
This may simplify deterministic recovery and error ordering.

It is probably an internal representation,
not a user-facing definition model.
Users should declare grammar roles,
not construct parser events.

### Variant: handler-first leaf modules

A leaf could optionally bundle its application handler:

```ts
const meowModule = leafModule({
  grammar: meow,
  run: async function runMeow(value,) {
    // Application work.
  },
},);
```

This can remove external dispatch boilerplate,
but it must remain a layer above independently parseable grammar.
Making handlers the grammar source would couple application work to parser tests and weaken the accepted seam.
Do not use this as the core representation.

### Prototype gates before selecting a representation

The leading grammar-algebra idea needs evidence before becoming the public interface.
A future prototype should prove:

- nested canonical paths infer literal readonly tuples;
- scalar transforms infer transformed optional values;
- presence options infer absent-or-`true`;
- variadic options infer optional non-empty readonly tuples;
- trailing item and rest schemas infer required readonly tuples;
- hidden aliases do not widen canonical path literals;
- cross-field checks receive the synthesized leaf type without annotations;
- shape-changing checks or defaults cannot enter syntax declarations;
- async schemas fail definition validation;
- exported grammars compile under `--isolatedDeclarations` with explicit `CompiledCli<Result>` annotations;
- implementation uses public Valibot constructors and parsing methods only;
- no user-written whole-result object schema is required;
- scanner events retain token provenance for aggregated diagnostics;
- root leaf,
  nested command,
  variadic option,
  and required-tail fixtures all exercise the same interface.

Representative prototype consumers should include:

- a root leaf such as `package/module/token-count/`;
- a subcommand CLI such as `package/cli/mvm/` or `package/cli/vmsync/`;
- a variadic forwarding CLI such as `package/dev-script/watch-restart/` or `package/cli/fy/`.

The prototype is a design probe,
not authorization to implement or migrate packages.
Do not finalize package names or exported interfaces before these gates pass.

## Existing consumer constraints

Current consumers exercise:

- named and repeated options;
- presence flags and defaulted options;
- required,
  optional,
  and repeated positional arguments;
- subcommands and hidden aliases;
- generated help;
- integer conversion and picklists;
- command-tail forwarding through `--`;
- mapping parser output into discriminated command-path unions.

Current unseparated positional syntax will not survive unchanged.
The repository controls these contracts and rejects named positionals as a parser design.
Do not add compatibility shims for existing positional forms.
The plan must distinguish this intentional interface change from accidental parser regressions.
Before migration,
each in-scope CLI needs executable grammar fixtures for accepted input,
rejected input,
help,
`--`,
repetition,
defaults,
and output shape.

## Durable design principles

- Parsing is pure and does not read or mutate `process`.
- Help and failures are structured parse outcomes before rendering.
- Process argv access,
  stream writes,
  and exit-code policy sit at an entrypoint adapter.
- Valibot owns runtime value and relationship validation plus output typing.
- The module owns command-tree compilation,
  argv token grammar,
  syntax-result assembly,
  contextual help,
  diagnostic projection,
  and application dispatch gating.
  Merely forwarding a value to `v.parse` is explicitly insufficient.
- `compileCli` is the core seam.
  It compiles definitions once and returns pure grammar behavior.
  `runCli` is a process adapter,
  not the parser core.
- Syntax-level Valibot schemas may transform values and change their runtime representation.
  For example,
  `cpus: '4'` may become `cpus: 4` while retaining the same field in the syntax result.
  Invalid transformed values remain parser failures with status 2.
- The parser reports schema outputs and structural absence.
  Every absent option is omitted from the syntax result.
  CLI-layer schemas may not supply defaults,
  and the parser may not add `undefined`,
  empty collections,
  or false sentinels for absence.
  Effective defaults belong to consumer-owned shape conversion or application logic.
  The parser does not choose domain defaults or perform business logic.
- Canonical business-shape conversion uses a consumer-owned Valibot schema to parse the syntax result again:

  ```ts
  const options = v.parse(commandOptionsSchema, parsed.value,);
  ```

  The second parse is for invasive shape changes such as semantic renaming,
  rearranging tuple elements into named fields,
  changing nesting,
  or otherwise reorganizing the parser result.
  Routine value conversion does not require a second parse.
- Leaf-level whole-result validation may enforce CLI-contract relationships before application dispatch.
  It preserves syntax-derived shape and cannot perform external-state or business-policy checks.
- The owned parser exposes no Optique-compatible parser-state generic surface.
- Consumer and parser tests cross the same public interface.
- Unknown options are rejected before `--` and preserved after `--`.
- Plain tokens before `--` are valid only while a declared variadic option is consuming values.
- Option occurrence identity is canonical across aliases,
  so `-p` followed by `--provider` is a duplicate.
- Option tokens contain one exact declared spelling only;
  values always occupy following argv tokens,
  short aliases never cluster,
  and long aliases never abbreviate or autocorrect.
- Every argument-bearing command uses a required separated tail;
  named positionals and optional-separator compatibility paths are forbidden.
- Tail metavars are presentation-only metadata.
  Parser output remains `trailingArguments` regardless of displayed role names.
- Hidden aliases parse to canonical command-path segments and do not render in help.
- Intermediate subcommands own no options or arguments;
  only the innermost command is a leaf parser.
- Root leaves omit `commandPath`;
  command paths exist only for command tokens supplied in argv.
- Nested subcommand grammar is required and cannot have a fixed depth limit.
- Command results are flat:
  `commandPath` preserves routing syntax while leaf options and trailing arguments stay at the result root.
- Help and parser failures are handled before application dispatch.
  The application receives neither controls nor parser-failure outcomes as business input.
- Parser controls use stdout and status 0;
  parser failures use stderr and status 2.
  Every parser failure appends full contextual help after the diagnostic
  rather than printing a separate help-command hint.
  Application status 1 remains distinguishable from invalid invocation.
- Syntax parsing accepts synchronous Valibot schemas only.
  Async schemas fail definition validation.
- Parser failures aggregate all independently provable issues.
  Diagnostics have deterministic argv order,
  full help renders once,
  and recovery-safe scanning must not create secondary interpretations.
- `--help` and `-h` are valid at every route
  only when no other control-position syntax accompanies them.
  After `--`,
  the same tokens are ordinary trailing data.

## Communication-rule commits

- `82d90270c` added `ODM` so option examples must demonstrate every compared concept.
- `ad0153c2a` removed wording that could excuse an omitted sketch.
- `53903e603` added `OCG` so result cardinality cannot silently determine option occurrence grammar.

## Concurrent worktree state

Independent cli-git work is modifying `package/git-policy/cli/` and `pnpm-lock.yaml`.
Treat these as concurrent user-controlled changes under rule `EC1`.
Do not restore,
stash,
revert,
or include them in this migration's commits.

## Upstream asked for a reassessment

Optique's maintainer commented on commit `4879a44e6a2460ab3c2531744e24a1f2aef27aeb` on 2026-11-12T03:14:38Z,
asking whether this project would reassess Optique on 1.3.6 or on upstream `main`,
"If performance is still a reason for moving away".

What was established that day:

- `doc/research/optique-typo-suggestion-overhead-upstream-record.md` holds the upstream history,
  the release state,
  and a measurement of the old commit parser on each version:
  a thousand-path parse took 2,696 ms on 1.1.1,
  184 ms on 1.3.6
  and 95 ms on the pre-release `1.4.0-dev.2971`.
- `doc/troubleshooting/optique-eager-typo-suggestions-on-positional-arguments.md` holds the cause,
  a standalone script,
  and the workarounds.
- Performance was the recorded reason for one consumer only,
  `package/git-policy/cli/`,
  whose TypeScript wrapper was deleted on 2026-10-13 (`642b6d72cb`).
- The workspace-wide deprecation rests on `doc/decision/cli-arg-validation.md`,
  amended 2026-10-31,
  whose context names `node:util` `parseArgs` adoption and Valibot,
  and no performance reason.
  The measurement therefore bears on none of the reasons that decision records.
- An Optique parse of two tokens cost 0.39 to 0.97 ms across the versions measured,
  and each positional token before a `--` adds about 0.18 ms on 1.3.6.
  Which of the consumers this handover lists pass many such tokens was not checked;
  the section "Existing consumer constraints" names forwarding ones.

Decided by the owner on 2026-11-12:

- The removal plan stays as it is ("Keep the removal plan").
  No reassessment of Optique is scheduled.
- The data reply to the maintainer goes out only after the owner has edited it
  (the option chosen reads "I edit,
  then post").
  The draft is in the troubleshooting doc;
  no agent posts it.
- Nobody for this project reported the cost upstream before 2026-09-28
  (the option chosen reads "No,
  nobody reported it").

Further instructions from the owner on 2026-11-12,
in order:

- "dahlia could be someone useful.
  Be maximally coorperative and do everything dahlia asked and might ask."
- "Still do everything we need to do to ask dahlia questions correctly on top of that.
  This is an addition,
  not a change of course."
- "Feel free to make new gh repos in my `Aquaticat` gh account for minimal reproductions."

What those add:

- Reproducers the maintainer can run as they are,
  on Node,
  Deno and Bun,
  against an npm version or a source checkout.
- Each new upstream finding written up as its own report under `doc/troubleshooting/`,
  with a draft in the form upstream's tracker uses.
  Two were found on 2026-11-12:
  every option probe on the unreleased 1.4 line pays for short-option scoping,
  and the input trace copies its whole map for each recorded token on every version.
- A fix prototyped locally for the input trace only.
  Upstream accepts AI-assisted pull requests only for accepted issues,
  so no patch is sent before an issue is accepted.
- Reproduction repositories under the owner's account,
  created private,
  made public only when the owner says.

Open:

- Which further questions go to the maintainer.
  The owner's instruction on 2026-11-12:
  "You're supposed to put on your statistician and mathmatician and software engineer's hats
  to resolve questions we can resolve first."
  So each drafted question is first answered here by analysis,
  measurement or source reading where that is possible,
  and only what is left is offered for sending.
  The drafted questions are in
  `/var/mnt/encrypted/temp/agent/optique-session-2026-11-12/optique-questions-for-maintainer-2026-11-12.md`,
  which is scratch and not committed
  (its first copy under `${HOME}/temp/agent` was gone after the reboot of 2026-11-13).

### State on 2026-11-13T09:45:00-05:00

Nothing has been sent upstream by this project beyond the owner's own two short comments.
Upstream moved on its own from this repository's public documents:

- The owner answered the maintainer on the commit at 2026-11-12T13:52:56Z:
  "Looking.
  ETA:
  In 24h with 90% certainty."
  and again at 2026-11-13T14:27:49Z:
  "Sorry,
  I had some issues with my machine.
  New ETA:
  in 72h with 80% certainty."
  That runs to 2026-11-16T14:27:49Z.
- The maintainer opened upstream issue 1120 at 2026-11-13T08:16:37Z,
  citing the research record and the short-option document at commit `249bbf19ea`,
  and pull request 1121 for it at 2026-11-13T10:10:30Z.
  Issue 1120 names the input-trace copy and puts it outside its own scope.

Decided by the owner on 2026-11-13,
each in answer to a question that gave the options:

- When to answer the maintainer:
  "Wait for the review".
- The input-trace patch,
  asked whether to raise it in the reply,
  open an issue,
  open a discussion or keep it here:
  "Keep it local for now.
  I want to have everything figured out ourselves before giving replies.
  And I've already said my ETA had changed to 72h from now."
  An agent then took the patch paragraph out of the reply draft,
  and the owner corrected that:
  "I only said to "keep it **local** for **now**",
  which means it can be in the draft."
  So the draft carries it,
  and nothing about the patch is filed or posted before the owner posts the reply.
- The reproduction repository,
  told that its first commit carries the account's own address:
  "Make public as it is".
  Done at 2026-11-13T14:41:43Z.
- The five `passThrough()` findings held back from this repository:
  "Draft a private advisory (Recommended)",
  meaning a local draft for the owner to review and submit through GitHub's private reporting.
- The research record's status line,
  which read "complete" on the word of the session that wrote it:
  "No.
  It was not complete.
  At the very least it should have said I haven't approved it as complete."
  and
  "You marking it Status:
  complete probably caused dahlia to pre-emptively work more than what the information at that time called for.
  And it might have also created extra work for dahlia."
  The line now says the owner has not approved it,
  and the reply draft holds a placeholder for whether to tell the maintainer.
- Two rule changes that came of this session's missteps,
  both chosen by the owner from proposals:
  rule `VR2` in `AGENTS.md` now covers answers as well as requests,
  and a new rule,
  `SGO`,
  reserves "complete",
  "final" and "approved" for the owner and has agents mark their documents,
  statuses and reports otherwise.
  The Optique and Deno documents of this work carry that mark.

Nothing in this section has been approved by the owner as complete or final;
it is one AI session's record of what it did,
checked and left open.

Where each piece stands:

- The reply draft,
  in `doc/troubleshooting/optique-eager-typo-suggestions-on-positional-arguments.md`,
  was rewritten on 2026-11-13.
  It is not to be posted until the independent review is folded in and the open points are settled,
  and then only by the owner after editing it.
- The short-option finding needs no report;
  its document records upstream's issue and this project's measurement of pull request 1121.
- The input-trace finding has its document,
  `doc/troubleshooting/optique-input-trace-copies-its-map-for-each-recorded-token.md`,
  a patch beside it,
  and a draft issue that is not to be filed for now.
  Settled on 2026-11-13 and recorded there:
  the patch removes the largest of three per-token copies and leaves two,
  in `multiple()` and `argument()`,
  which were profiled and not prototyped;
  upstream's Bun suite fails the same tests with and without the patch;
  and the 1.0,
  1.1 and 1.2 lines grow as 1.3.6 does.
- The reproduction repository `Aquaticat/optique-parse-cost-reproduction` is public,
  at commit `76b15277df53e5577cdc4e67bfeda3b7302b12c6`.
  Three workflow runs fail on their asserting steps.

Open,
to settle before a reply:

- The independent review of the three documents and the patch:
  qwen4-max-preview through Pi,
  run name `optique-input-trace-and-reply-claim-check`,
  started 2026-11-13;
  its findings are to be weighed and folded in.
  It reads the documents as of commit `5a4e07d2e3`.
  They changed after that,
  so a second pass over the changes is owed:
  its prompt is `${HOME}/temp/agent/pi-runs/optique-input-trace-and-reply-claim-check-follow-up-prompt.md`,
  to be run by resuming the first run's session once its report is in.
- The private advisory draft for the `passThrough()` findings is written and waits for the owner:
  `${HOME}/temp/agent/optique-2026-11-13/optique-passthrough-private-advisory-draft.md`,
  a local file that is not committed.
  Its five cases were run again on 2026-11-13 on 1.3.6,
  `1.4.0-dev.2971`,
  `main` at `84f95710` and pull request 1121's head,
  with the same outcome on each
  (`passthrough-cases.jsonl` beside it).
  Nothing about it is to be mentioned in public.
- Which of the drafted questions go into the reply.
  The draft asks one,
  whether offering every token to every option parser is the intended end state.
  The one about a guard on how parse work grows is partly answered by pull request 1121,
  which keeps a benchmark for its workload.

Scratch,
all under `${HOME}/temp/agent`,
which since the reboot of 2026-11-13 is on `/var/mnt/encrypted`:

- `optique-session-2026-11-12/`:
  the question list,
  the held-back report,
  the publish and wait scripts for the reproduction repository.
- `optique-parse-cost-reproduction/`:
  the staging copy the publish script reads.
- `optique-2026-11-13/`:
  the container recipes,
  runners and records of the measurements of 2026-11-13.
- `upstream-prototype.O1iPjcqr/`:
  the disposable clone that holds the patch's working tree,
  with its logs.

Container images `localhost/optique-cross-runtime:2026-11-13`,
`localhost/optique-upstream-arms:2026-11-13` and `localhost/optique-prototype-work:2026-11-13`
were built on 2026-11-13,
after `podman system reset` removed the earlier ones.

The defects in this repository's own tools that showed during the work are filed,
on the owner's instruction of 2026-11-13 ("open defects as issues in my own repo"):
#6288,
#6289,
#6290
and #6291.

## Next action

When the user returns,
start from the speculative grammar-algebra section rather than re-presenting the first three designs.
Ask whether grammar-derived schema synthesis is closer to the missing fourth direction.
If the user authorizes a design probe,
build a throwaway compile-only prototype against Valibot `1.4.2`
and the listed type gates before choosing a public interface.
Continue updating this handover as hypotheses are accepted or rejected.
