# Done

AI-powered task aggregator that surfaces what to do next based on location and focus context.

## How it runs

Client JS is bundled by tsdown as a separate build step (`mise run build:js:client`).
In development,
 `mise run dev:site` uses `mise watch` to restart the process on any source change.

```sh
mise run build:js:client  # bundle client scripts
node src/server.ts        # start server
mise run dev:site         # development (auto-restart on src/ change via mise watch)
```

## Architecture overview

Single h3 server process handling both page routes (HTML) and API routes (JSON).

1. **CSS**:
    h-css generates rule strings from typed declaration records;
    no separate compiler runs
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

- `@monochromatic-dev/hyperscript-core`:
   h-css,
   h-html,
   h-dom factories

This package runs only within the Monochromatic monorepo.

## Decision: no ORM

Drizzle ORM was evaluated as the strongest TypeScript ORM candidate for this project.
It was rejected because the database layer relies heavily
on SQLite-specific features that Drizzle cannot model natively:

- **FTS5 virtual tables** for full-text search
- **Triggers** that keep the FTS index in sync with the tasks table
- **`json_each()`** table-valued function for querying JSON arrays stored in columns
- **Partial indexes** (`WHERE due_date IS NOT NULL`,
   etc.)

Adopting Drizzle would mean writing some queries with the query builder and others as raw `sql` template escape hatches:
 two query styles in one codebase.
That is worse than consistently using raw `@tursodatabase/database` everywhere.

The current approach uses named SQL constants extracted to the top of each file,
 which keeps query text readable and scannable without introducing a second abstraction layer.

## CSS framework comparison variants

Done's UI is built with h-css (typed TypeScript composition;
 see `PHILOSOPHY.css.md`).
Sibling packages re-implement the same product against alternative CSS frameworks so the
approaches can be diffed side by side at the same scale of UI.

- `../variant-postcss/`:
   PostCSS `@mixin` + `@apply` (via `@monochromatic-dev/css-cli`)

These comparison packages exist purely as reference.
 They inherit product changes from
this package;
 they are not intended to be shipped,
 and may be deleted once their
comparison value is exhausted.
 See each variant's README for the per-file differences.

## Further reading

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
- `PHILOSOPHY.css.md` (repo root):
   Why h-css over `@apply`,
   CSS-in-JS,
   and external CSS files
- `package/hyperscript/core/README.md`:
   h-css API reference
