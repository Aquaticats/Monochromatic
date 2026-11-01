# Done (PostCSS `@apply` comparison variant)

PostCSS-pipeline implementation of `../app`,
 kept as a reference point for
comparing CSS frameworks at full-app scale.
Same product spec,
 AI prompts,
 database schema,
 and HTTP routes;
the only axis that differs is the CSS authoring approach.

See `../app/README.md` for the canonical product description.
This README only documents what this variant changes.

## Why this variant exists

`../app` is the canonical implementation and uses h-css (typed TypeScript composition;
see `PHILOSOPHY.css.md`).
This `done-postcss` package implements the same UI using the PostCSS `@mixin` + `@apply`
pipeline (via `@monochromatic-dev/css-cli`) so reviewers can diff the two approaches
side by side at the same scale.

The package is kept buildable while the comparison is useful.
It is not part of the shipped product and should not be extended with `done`-only features.

## Differences from `../app`

### CSS authoring

- `done` writes mixins as TypeScript functions returning `CssDeclarations`
  in `src/client/mixins.ts`,
   composed via object spread.
  `done-postcss` writes them in `src/client/mixins.css` with `@define-mixin` /
  `@apply` syntax.
- `done` builds global styles inside TypeScript (`styles.ts` plus per-layer modules).
  `done-postcss` writes them in `src/client/styles.css` and imports the compiled
  output with `with { type: 'text' }`.
- Component-level Shadow DOM styles in `done` are `$({ rule, decls, children })` calls
  with typed value constructors (`cssRem`,
   `cssVar`,
   `cssInt`).
  In `done-postcss` they are ``css(`@apply --foo; ...`)`` template strings expanded
  at runtime by `build-tool-css`.

### Build pipeline

- `done` has no `build:css` task and no startup CSS step;
   h-css emits CSS inside the
  TypeScript bundles at build time,
   or inline at SSR time.
- `done-postcss` has a `mise run build:css` task that compiles `styles.css` into
  `dist/css/`,
   and `src/server.ts` calls `buildCSS()` at startup to refresh the
  compiled output.

### Dependencies

- `done` depends on the workspace's shared `@monochromatic-dev/hyperscript-core`.
- `done-postcss` additionally depends on `@monochromatic-dev/css-cli` and
  `postcss`,
   plus the `@monochromatic-dev/rolldown-plugin-import-attributes`
  devDependency that lets client code import pre-built CSS via `with { type: 'text' }`.

### tsdown config

- `done`'s `tsdown.client.config.ts` registers no extra plugins.
- `done-postcss`'s config registers `importAttributesPlugin()` so the client bundle
  can inline the pre-built CSS string.

## How it runs

Client JS is bundled by tsdown as a separate build step (`mise run build:js:client`).
CSS compilation (build-css) still runs at server startup.
In development,
 `mise run dev:site` uses `mise watch` to restart the process on any source change.

```sh
mise run build:js:client  # bundle client scripts
node src/server.ts        # start server (compiles CSS at startup)
mise run dev:site         # development (auto-restart on src/ change via mise watch)
```

## Architecture overview

Single h3 server process handling both page routes (HTML) and API routes (JSON).

1. **CSS**:
    `@monochromatic-dev/css-cli` resolves `@import` and expands `@mixin`/`@apply` into plain CSS
2. **Client JS**:
    tsdown bundles one entry per page (inbox,
    in-progress,
    task-details,
    search,
    settings) via `mise run build:js:client`
3. **Server**:
    h3 `H3` route registration plus h3 static serving for built client assets from `dist/client/`
4. **Database**:
    SQLite (@tursodatabase/database) with FTS5 full-text search,
    initialized via side-effect import at startup
5. **Client**:
    Vanilla TypeScript with custom elements;
    reads server-embedded JSON from `<script id="page-data">`,
    builds DOM imperatively

## Monorepo dependencies

- `@monochromatic-dev/css-cli`:
   CSS `@mixin`/`@apply` expansion pipeline
- `@monochromatic-dev/hyperscript-core`:
   h-html,
   h-dom factories (h-css is unused here)

This package runs only within the Monochromatic monorepo.

## Lifecycle

This package is reference scaffolding.
 It exits in one of two ways:

- **Mirrored away**:
   kept as long as the side-by-side comparison with `../app` is
  useful for evaluating h-css or for benchmarking against future framework variants
  (`done-tailwind`,
   `done-vanilla-extract`,
   etc.).
- **Retired**:
   deleted once its comparison value is exhausted.
  Record the rejection reason in a decision doc under `doc/decision/`.

Until retired,
 mirror behaviour changes from `../app` only when the comparison's
coverage of the PostCSS surface needs to stay in sync.
Do not extend this variant with features that have not landed in `../app` first.

## Decision: no ORM

Inherited from `../app`.
 Drizzle ORM was evaluated as the strongest TypeScript ORM
candidate and rejected because the database layer relies heavily on SQLite-specific
features that Drizzle cannot model natively:

- **FTS5 virtual tables** for full-text search
- **Triggers** that keep the FTS index in sync with the tasks table
- **`json_each()`** table-valued function for querying JSON arrays stored in columns
- **Partial indexes** (`WHERE due_date IS NOT NULL`,
   etc.)

The current approach uses named SQL constants extracted to the top of each file.

## Further reading

- `../app/README.md`:
   Canonical product description
- `PHILOSOPHY.css.md` (repo root):
   Why h-css replaces this PostCSS approach
- `package/css/cli/README.md`:
   `@mixin`/`@apply` pipeline details
- `PLAN.md`:
   Implementation plan with DB schema,
   AI prompts,
   and deployment details
- `SPEC.md`:
   Product specification (task model,
   screens,
   sync targets)
- `FRAMEWORK_EVALUATION.md`:
   Historical evaluation of vanilla TS plus Bun before the Node migration
