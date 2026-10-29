# Decision: no Rust migration for file-enforcer

Superseded 2026-09-16 by
[`doc/decision/monorepo-manager-all-rust.md`](../../../doc/decision/monorepo-manager-all-rust.md):
the monorepo manager must ship as one file,
so file-enforcer's functionality is rewritten in Rust inside it.
The reasons below did not consider single-file shipping;
they still describe the TypeScript implementation until the Rust tool replaces it.

Records why file-enforcer stays TypeScript and what to optimize instead.
Prompted by a "rewrite the CPU-bound dev tooling in Rust" survey of the monorepo.
The short answer:
 the no-op steady state is neither subprocess-bound nor parse-bound in this repo's
actual config,
 the avoidable cost is recomputation a cache removes in TypeScript,
 and the consumer
surface is TypeScript config modules that a native core cannot host.

## Question

Should file-enforcer be rewritten in Rust on the theory that it is a CPU-bound batch tool
(walk the tree,
 recompute derived files) of the kind that benefits from a native rewrite?

## Verdict

No. Three independent reasons,
 any one of which is sufficient:

1.  The consumer surface is a TypeScript builder API,
     not a CLI boundary.
2.  The no-op cost is recomputation,
     removable with a staleness cache in TypeScript.
3.  The residual after that cache is file IO,
     already overlapped,
     which a rewrite does not improve.

## What the no-op run actually does

"No-op" means file-enforcer runs,
 finds every derived file already in sync,
 and writes nothing.
Tracing the code path:

-   No subprocesses fire.
     `src/pipeline/exec.ts` runs external commands only when an action is
    triggered (install a package,
     run a generator).
     A fully-synced run spawns nothing.
-   No mtime or stat gate precedes the transform.
     `writeIfChanged` (`src/io/write.ts`,
     the core of
    `overwrite` / `overwriteEach`) always computes the candidate `content`,
     reads the existing dest
    via `readExisting`,
     and string-compares `existing === content`.
     It skips the write on equality,
    not the recompute.
-   Reads are parallelized (`src/io/cat.ts`,
     `Promise.all` over glob matches).
-   The read cache is in-memory and per-process (`src/io/cache.ts`).
     A one-shot `mise run` starts
    cold and re-reads sources from disk.

So the no-op run is:
 read every source,
 run every builder's transform to produce candidate output,
read every dest,
 compare,
 discover equality,
 write nothing.
The recompute happens every time;
 the content-diff is the only thing standing between "recomputed"
and "written".

## CPU versus IO in that run

It depends on the builder,
 and in this repo it leans IO:

-   The transforms in `src/pipeline/transform.ts` are cheap:
     `dedup` (split / Set / join),
    `getJsonProperty` (JSON.
    parse + dot-prop).
-   The root `file-enforcer.config.ts` is dominated by string concatenation.
     `CLAUDE.md` is
    `${await cat(['./AGENTS.md'])}` embedded in a template;
     `cat(['./mise.no-env.toml'])` reads TOML
    as text and never parses it.
     That path is file reads plus concatenation:
     IO-leaning,
     trivial CPU.
-   The CPU share rises only for per-package parse-heavy builders:
     the TOML round-trip in
    `src/pipeline/toml.ts` (`parseTomlEdit` then `tomlStringify`) and index generation in
    `src/package/mise.generate-index.ts`,
     multiplied across the package count.

Even at that end,
 the bottleneck is recomputation volume,
 not a single hot kernel.

## Why Rust is the wrong tool here

-   Config-as-TypeScript is the consumer contract.
     `file-enforcer.config.ts` and per-package configs
    import `overwrite`,
     `cat`,
     `ensurePackage`,
     `editTomlKey` and call them as TypeScript functions,
    with `await`,
     `.map`,
     and template literals carrying real logic.
     This is the repo's deliberate
    "switch from config-as-data to TypeScript when conf needs logic" architecture.
     A Rust core would
    force every config into an FFI or subprocess boundary,
     discarding the builder API that is the
    package's entire value.
-   The avoidable cost is recompute,
     and that is an algorithmic fix in any language.
     Eliminating it
    in TypeScript captures the same win without the rewrite.
-   After a staleness cache,
     the residual is file IO.
     It is already overlapped via `Promise.all`,
     and
    a native rewrite does not materially shorten disk reads.
-   A rewrite also adds native build and distribution complexity to a tool consumed in-process by the
    repo's TypeScript tooling.

## What to do instead

1.  Add a persisted staleness manifest.
     Map each managed dest to a hash (or mtime set) of its sources
    plus a hash of the dest.
     On run,
     stat or hash the inputs and the dest;
     when all match the
    manifest,
     skip the transform,
     the dest read,
     and the compare for that builder.
     This removes the
    recompute cost that one-shot runs pay today,
     because the in-memory `writeTimestamps`
    (`src/tracker.ts`) do not survive across separate process invocations.
2.  Reuse the watch-mode mtime infrastructure as a build-skip gate.
     `src/watch/watch-filter.ts`
    already stats files and compares `mtimeMs` against recorded write timestamps to classify echoes
    versus external edits.
     Watch mode can extend that signal to skip unchanged builders,
     not only to
    suppress echoes.
3.  Optimize a specific builder if a profile names one.
     The TOML path already documents batching:
    parse once with `parseTomlEdit`,
     chain `tomlSet`,
     stringify once.
     Apply that where a builder
    re-parses the same source repeatedly.
4.  Measure before optimizing.
     Profile a real no-op run across the monorepo and confirm where the
    time goes;
     the analysis above predicts IO and recompute,
     not parsing,
     but the manifest decision
    should rest on numbers from this tree.

## When to revisit

Reconsider a partial port only if,
 after the staleness cache lands,
 profiling shows the no-op run is
still dominated by transform CPU,
 and that CPU isolates into a hot kernel with a stable boundary that
does not cross the TypeScript builder API (for example,
 a standalone file scanner invoked as a step).
Port that kernel,
 not the builder API.
 A whole-tool rewrite is not justified at any measurement seen
so far.
