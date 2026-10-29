# Migration: zod to valibot

Status:
 decided 2026-05-09.
 Migration not started.

## Decision

New schemas use [valibot][valibot] starting today.
Existing zod schemas migrate as files are touched for other reasons;
 no big-bang rewrite.
When the last consumer of zod is removed,
 drop zod from the catalog.

[valibot]: https://valibot.dev

## Rationale

### Zod governance smell

Two findings combine into a single concern.

**Maintainer focus shifted.
**
colinhacks' X bio reads "building @pullfrogai,
 the Zod guy";
 LinkedIn lists Pullfrog as employer.
Zod is sponsorship-funded side work;
 67 sponsors at 64% of his stated "enough to live on" goal.

**Bot owns the rejection lane.
**
`dosubot` autoclose is ~2% of all closures,
 but **92% of `NOT_PLANNED` closures**
(391 of 423 in the last 12 months).
colinhacks only NOT_PLANNED-closes 5%.
When a request gets rejected,
 the bot does it,
 often without the maintainer ever responding.
Issue [#3751][#3751] (TypeScript `isolatedDeclarations` support) is the pattern,
 not the exception:
bradzacher (TypeScript-ESLint maintainer) pushed back "still very relevant",
altano provided a workaround,
the bot pinged colinhacks who never responded,
then the bot autoclosed three months later as `NOT_PLANNED`.
Mikescops commented "still an issue" 35 minutes after autoclosure with no reopen.

[#3751]: https://github.com/colinhacks/zod/issues/3751

Bus factor on Zod is 1.
colinhacks authored 491 of 772 commits in the last 12 months;
 the next contributor sits at 12.
The project ships actively today
(164M weekly downloads,
 5.3x year-over-year growth,
 v4.4.3 released 2026-05-04,
 64 PRs merged in last 30 days),
but the rejection-lane governance and the maintainer's stated focus on a separate startup are forward risks.

### Valibot health

- **Active maintenance.
  ** 636 commits in last 12 months.
   Most recent release v1.4.0 on 2026-05-05.
  Subpackage releases ship independently and additively.
- **Adoption rising.
  ** ~10M weekly downloads,
   ~7x year-over-year.
   ~6% of Zod's volume but climbing.
- **API stability.
  ** v1.
  x is stable and additive;
   no churn-style major version on the horizon.
- **Standard Schema membership active.
  ** `~standard` property implemented,
   so consumer code stays portable.
- **Issue responsiveness.
  ** Fabian Hiller engages on threads within days;
   62 open / 598 closed issues.
- **Org transfer benign.
  ** `fabian-hiller/valibot` moved to `open-circle/valibot` on 2026-01-27 (PR #1395).
  Open Circle is Fabian's own umbrella org for Valibot and Formisch,
   not an acquisition.
- **Documented `isolatedDeclarations` guidance.
  **
  Valibot ships `v.GenericSchema<TInput, TOutput>` as the sanctioned annotation type for explicit declarations
  (analogous to `z.ZodType<T>` but officially documented as the workaround).
  Fabian engages constructively in the related threads,
  including explicitly flagging the trade-off that the workaround loses inference detail.
  Contrast with Zod,
   where [#3751][#3751] sat three months without a maintainer comment before bot-autoclosure.
  This directly addresses our use case (`isolatedDeclarations: true` is set workspace-wide).

## Risks accepted

- **Valibot bus factor 1.
  **
  Fabian Hiller authored 132 of 188 substantive commits (70%) in the last 12 months;
  the next-most-active human had 2 commits.
  If Fabian stops,
   the project stalls.
- **Funding gap.
  **
  Open Collective ~$2,428/year recurring against a $10k/month target.
  No corporate employer paying him to work on Valibot.
- **Two open RFCs without resolution** (#1441 compiler,
   #1389 v2 object input types).

We trade Zod's "huge but governance-rotting" risk for Valibot's "small but single-maintainer" risk.
The primary reason this is acceptable is Standard Schema:
if Valibot stalls,
 our consumers (h3 routes,
 frontmatter parsers) keep working
against any other Standard-Schema-implementing library,
 and a second migration is mechanical.

## Alternatives considered and rejected

### Reject: stay on Zod

Cheapest option,
 and our local pattern (`zod/mini` + concrete `z.ZodMini*` annotations under `isolatedDeclarations`) works.
Rejected because the rejection-lane governance pattern is unlikely to improve
while colinhacks' attention is on Pullfrog,
and writing the migration off as "later" tends to mean "much later" once download volume keeps growing the lock-in.

### Reject: ArkType

Type-first DSL with parse-time validation.
Rejected because:

- Adoption is much smaller (~889K weekly downloads vs Valibot's 10M);
   ecosystem support is thinner.
- DSL syntax differs sharply from `zod/mini`'s function-based shape,
   making mechanical migration impossible.
  Higher friction to adopt incrementally.

### Reject: Effect Schema (`@effect/schema`)

Part of the larger Effect ecosystem.
Rejected because:

- Pulls in the Effect runtime philosophy (Effect contexts,
   fibers,
   etc.) which we do not use elsewhere.
  Adopting just the schema piece is awkward.
- ~968K weekly downloads,
   similar ecosystem size to ArkType.

### Reject: @badrap/valita

Tiny and simple.
Rejected because:

- ~84K weekly downloads,
   smallest of the candidates.
   Sparser ecosystem and feature gaps relative to Valibot.
- Single-maintainer too,
   without Valibot's growth trajectory.

## Caveats from the due-diligence pass

A targeted "funny business" sweep on 2026-05-09 (code samples,
 issue triage,
 PR responsiveness,
 sponsor list,
security advisories,
 org-transfer integrity,
 RFCs) returned a `minor concerns` verdict.
The substantive finding worth pinning here:

### Standard Schema result-shape violation (Valibot [#1343][#1343])

Valibot's `safeParse` and the `~standard.validate` adapter return `{typed, success, output, issues}` unconditionally.
When validation partially succeeds,
 both `output` (a.
k.
a.
 `value` under the StandardSchemaV1 contract) and `issues` are populated.
The Standard Schema spec requires they be mutually exclusive.
Fabian acknowledged the spec violation on 2025-12-14 but deferred the fix because it would add ~100 bytes
to the headline 500-byte string-schema budget;
 milestoned for v1.5,
 no commitment date.

[#1343]: https://github.com/open-circle/valibot/issues/1343

**Practical guidance for the 13 migration sites:
**
discriminate on `'issues' in result` (per Fabian's recommendation),
 not `'value' in result`,
when consuming Standard Schema results from Valibot.
Watch milestone v1.5 for the upstream fix and remove the workaround once shipped.

**The "100 bytes is a big deal" framing is irrelevant for our migration calculus.
**
Measured locally on 2026-05-09 with Bun's bundler + `gzip -9` on identical `string() + parse()` source:

- `zod` (classic):
   282,977 B minified,
   63,518 B gzipped
- `zod/mini` (our current baseline):
   6,850 B minified,
   **2,753 B gzipped**
- `valibot` (today,
   spec-violating):
   1,776 B minified,
   **881 B gzipped**
- `valibot` projected with #1343 fix applied (+60-80 gzipped bytes from modeling the diff against
  `_getStandardProps.ts:33-37`):
   ~940-960 B gzipped

We shed ~1,800 gzipped bytes per schema either way (~65% reduction).
The 60-80 bytes Fabian is protecting is 4% of the migration savings.
His framing is true for his own marketing pitch (500 -> 600 = 20% inflation on the headline number)
but does not affect our trade-off.

### Bot-autoclose pattern looks like Zod's but is not

`dosubot` closes ~93% of Valibot's `NOT_PLANNED` issues,
 mirroring the Zod ratio that triggered this whole exercise.
Drill-down on four samples (#1180,
 #1183,
 #1196,
 #1232) shows the opposite dynamic:
in every case Fabian engaged constructively first (asked for repro,
 accepted feature,
 explained tradeoff)
and the bot only stale-closed months later when reporters did not respond.
The ratio is the same;
 the substance is not.
 Recorded so future health checks do not flag this as new.

### Code-quality signals are clean

Zero runtime dependencies,
 MIT.
`parse.ts` 33 lines,
 `safeParse.ts` 35 lines,
 `_getStandardProps.ts` 43 lines.
No `as any`,
 no `eval`/`Function()`,
 no network calls,
 no `process.env` reads,
 no hardcoded credentials.
EMOJI_REGEX explicitly credits `emoji-regex-xs` v1.0.0 MIT.
Two `@ts-expect-error` in `string.ts` are scoped,
 intentional internal narrowing of the dataset discriminated union.

## Migration approach

Big-bang executed on 2026-05-10 after the touch-based plan was overtaken by the
decision to drop zod entirely.
 All 14 zod consumers were migrated in a single
commit and zod was removed from the catalog.

The zod-mini to valibot translation map below applied uniformly across the surface.

### Mechanical mapping

- `z.parse(schema, x)` to `v.parse(schema, x)`
- `z.string()` to `v.string()`
- `z.object({...})` to `v.object({...})`
- `z.array(s)` to `v.array(s)`
- `z.union([a, b])` to `v.union([a, b])`
- `z.optional(s)` to `v.optional(s)`
- `z.record(k, v)` to `v.record(k, v)`
- `z.boolean()` to `v.boolean()`
- `z.number()` to `v.number()`
- `z.nullable(s)` to `v.nullable(s)`
- `z.enum(RECORD)` to `v.picklist([...values])` (literal tuple,
   not record)
- `z.uuid()` to `v.uuid()`
- `z.infer<typeof S>` to `v.InferOutput<typeof S>`
- `z.url().safeParse(x)` to `v.safeParse(v.pipe(v.string(), v.url()), x)`
- `z.ZodMini*` annotations to `v.GenericSchema<TInput, TOutput>` (documented escape hatch
  under `isolatedDeclarations`;
   loses inference detail but compiles)

### Non-mechanical idioms

**Coerce.
** Valibot has no `coerce` namespace;
 build a pipe that takes `unknown`,
transforms with the target constructor,
 then validates the post-transform type:

```ts
v.parse(
  v.pipe(
    v.unknown(),
    v.transform(Number,),
    v.number(),
  ),
  process.env.PORT ?? DEFAULT_PORT,
);
```

For `z.coerce.date()`,
 prefer a typed input union over `v.unknown()` to avoid
`no-unsafe-type-assertion` lint warnings on the transform parameter:

```ts
const coerceDateSchema = v.pipe(
  v.union([v.string(), v.number(), v.date(),],),
  v.transform(function toDate(input,) {
    return new Date(input,);
  },),
  v.date(),
);
```

**URL with constraints.
** `v.url()` only validates parseability.
 Add a
`v.check()` callback for protocol/host constraints:

```ts
v.pipe(
  v.string(),
  v.url(),
  v.check(
    function isHttpDomainUrl(s,) {
      const u = new URL(s,);
      return /^https?:$/.test(u.protocol,) && v.DOMAIN_REGEX.test(u.hostname,);
    },
    'Invalid HTTP(S) URL with valid domain',
  ),
);
```

**Async pipe.
** When a transform is async (e.g. prompts the user),
 use
`v.parseAsync` with `v.pipeAsync` and `v.transformAsync`:

```ts
await v.parseAsync(
  v.pipeAsync(
    v.nullable(v.pipe(v.string(), v.uuid(),),),
    v.transformAsync(async function promptSet(val,): Promise<string> {
      if (val !== null)
        return val;
      return notNullishOrThrow(await prompt('Set api key',),);
    },),
    v.uuid(),
  ),
  localStorage.getItem('apiKey',),
);
```

**Schema with coerced output type.
** Annotate `v.GenericSchema<TInput, TOutput>`
with explicit,
 distinct TInput and TOutput when the schema coerces (e.g. dates):

```ts
export const postFrontmatterSchema: v.GenericSchema<
  { title: string; published: string | number | Date; },
  { title: string; published: Date; }
> = v.object({...});
```

**Result-shape difference.
** `v.safeParse` returns `{success, output, issues}`,
not zod's `{success, data, error}`.
 Update result consumers accordingly.
 Per
Standard Schema [#1343][#1343],
 both `output` and `issues` may be populated when
validation partially succeeds;
 discriminate on `'issues' in result`,
 not
`'output' in result`.

**No `coerce` namespace,
 no `regexes` namespace.
** Valibot exposes regex constants
as top-level:
 `v.DOMAIN_REGEX`,
 `v.EMAIL_REGEX`,
 etc.

## Status updates

- 2026-05-09:
   decision recorded.
- 2026-05-09:
   due-diligence pass completed;
   verdict `minor concerns`;
   #1343 caveat documented above.
- 2026-05-10:
   bootstrap complete;
   `valibot` `>=1.4.0` added to `pnpm-workspace.yaml` catalog.
- 2026-05-10:
   big-bang complete.
   14 source files migrated (rss x8,
   exa-search x2,
   ssg-test x2,
   ai-tree x1,
   auto-mode/config-schemas x1).
   Five package.
  jsons swapped zod->valibot,
   two (`module/es`,
   `llm/image-diff`) had unused zod entries dropped.
   zod removed from catalog.
   PHILOSOPHY.
  `tool-choices.md` updated.
   The recount note:
   original 13 missed `auto-mode/src/config-schemas.ts`
   because it imports zod with double-quoted `"zod/mini"`
   whereas every other consumer uses single quotes.
