# Generalize ssg-test into a publishable SSG (draft)

Status:
 draft.
 Supersedes
[`TODO.generalize-ssg.md`](./TODO.generalize-ssg.md),
 whose defects are catalogued in
[`doc/audit/generalize-ssg-plan.md`](../../../doc/audit/generalize-ssg-plan.md).
This rewrite is built on the decided i18n architecture
([`doc/planning/module-i18n-compose.md`](../../../doc/planning/module-i18n-compose.md))
and the real path-model and publishability constraints.

Goal:
 extract the site-specific data from `ssg-test` so the build pipeline becomes a reusable,
publishable static site generator,
 and move the Aquaticat content into a sibling `site-aquaticat` package.

## Scope and honest non-goals

The previous plan advertised arbitrary user-configurable languages.
That contradicts the chosen i18n architecture and is removed here.

- Supported locales are a closed set:
   `ca`,
   `en`,
   `zh`.
  The set is determined by the grammar renderers shipped in `@monochromatic-dev/i18n-compose-core`
  (`defineCatalanLocale`,
   `defineEnglishLocale`,
   `defineChineseLocale`).
  A site selects a subset of these.
   Adding a new language is not a config option;
  it requires adding a locale renderer to `module-i18n-compose` first
  (`doc/planning/module-i18n-compose.md:1104`,
   `:322-328`).
- UI label keys are SSG-owned (the site chrome:
   language chooser,
   search box,
   theme toggle,
   and so on).
  The SSG ships default label values for `ca`,
   `en`,
   and `zh`.
- The author-facing frontmatter schema stays fixed at `{ title, description, tags }` for v1.
  Schema extension is a future non-goal,
   recorded here so the limitation is explicit,
   not silent.
- "General-purpose" means general for content,
   site identity,
   and branding,
   across the supported locale set.
  It does not mean general for language mechanics.

## Prerequisite: complete the i18n migration first

Phase 6 of `doc/planning/module-i18n-compose.md` (migrate `ssg-test` off `typesafe-i18n` onto
`@monochromatic-dev/i18n-compose-core`) is a hard prerequisite,
 not part of this plan.
The config and i18n design below assume it is done:
 `siteName` and `siteDescription` are `label` keys,
not `typesafe-i18n` translation fields,
 and there is no codegen step.

Integration point for this plan:
 the SSG defines the `Label` union as the SSG UI keys plus `siteName`
and `siteDescription`.
 The SSG ships default UI label values per locale.
 The site author supplies
`siteName` and `siteDescription` values (and optional UI overrides) through `site.config.ts`.
At build time the SSG composes per-locale label records and calls `createI18n` with the site's selected locales.

## Phase 1: config schema and loader (valibot)

The SSG looks for `site.config.ts` in the current working directory by convention.

- [ ] Create `src/config/schema.ts` with a valibot schema (the workspace standard;
       see `src/lib/content.ts:21`
      and the `config-schemas.ts` convention in `package/pi-plugin/advisor`).
      Do not use zod;
       it is not a dependency and was migrated away (`doc/migration/zod-to-valibot.md`).
- [ ] Export `SupportedLocale` (`'ca' | 'en' | 'zh'`) and the UI label key union from the package.
- [ ] Create `src/config/load.ts` that resolves and imports `site.config.ts` from `process.cwd()`
      and validates it with the schema.
- [ ] Export `defineConfig` from the package entry point for type-safe config authoring.

Schema shape:

```ts
import * as v from 'valibot';

export type SupportedLocale = 'ca' | 'en' | 'zh';

export type UiLabelKey =
  | 'chooseALang' | 'searchPlaceholder' | 'noResults' | 'page'
  | 'postNotInLang' | 'redirectingToLangChooser' | 'themeToggle' | 'langSwitcher';

export const siteConfigSchema = v.object({
  url: v.string(),
  contentDir: v.optional(v.string(), 'content'),
  locales: v.array(v.picklist(['ca', 'en', 'zh'])),
  defaultLocale: v.picklist(['ca', 'en', 'zh']),
  site: v.record(
    v.picklist(['ca', 'en', 'zh']),
    v.object({ name: v.string(), description: v.string() }),
  ),
  labelOverrides: v.optional(
    v.record(v.picklist(['ca', 'en', 'zh']), v.partial(v.record(/* UiLabelKey */ v.string(), v.string()))),
  ),
  tickerQuotes: v.array(v.string()),
  favicon: v.object({
    source: v.optional(v.string(), 'public/favicon.svg'),
    backgroundColor: v.object({ r: v.number(), g: v.number(), b: v.number() }),
  }),
  brand: v.optional(v.object({ primaryLight: v.string(), primaryDark: v.string() })),
  repository: v.optional(v.object({ host: v.literal('github'), slug: v.string() })),
});
```

Validation rules to enforce in the loader,
 not just the schema:

- [ ] `defaultLocale` must be a member of `locales`.
- [ ] `site` must have an entry for every member of `locales`.
- [ ] Every key in `labelOverrides` and `site` must be a member of `locales` (no stray locales).

### Values that remain SSG defaults

Standard web conventions and internal details:

- Favicon output paths,
   icon dimensions (180px apple,
   192/512px PWA),
   maskable safe zone (409px),
  SVG render density (384px).
- Default UI label values for `ca`,
   `en`,
   `zh` (overridable via `labelOverrides`).
- Default typography (the bundled fonts).
   Fonts as a config option are a v1 open question;
   see "Open decisions".
- The dev server config (shipped as a default;
   see Phase 6).

Note the contrast with the previous plan:
 pipeline source paths for cache invalidation are no longer
classified as a safe-to-leave default,
 because they were broken in the consumer scenario.
The path model below replaces them.

## Phase 2: project-root versus package-root path model

The previous plan left `dist`,
 `public`,
 the fonts output dir,
 and the cache pipeline glob as cwd-relative
module constants.
 When the SSG runs as a dependency,
 the cache glob
(`PIPELINE_GLOB = 'src/{lib,components,client}/**/*.ts'`,
 `src/build.ts:73`) matches nothing in the
consumer's cwd,
 so the cache never invalidates on an SSG change.
 Fix the model explicitly.

- [ ] Introduce two resolved roots:
  - `projectRoot` is `process.cwd()`:
     holds `site.config.ts`,
     `contentDir`,
     `public/`,
     `dist/`,
     `.cache/`.
  - `packageRoot` is resolved from `import.meta` (the installed SSG location):
     holds the SSG's own styles,
    default fonts,
     default dev-server config,
     and the SSG's `package.json` (for the version fingerprint).
- [ ] Replace the cwd-relative cache fingerprint with inputs that are correct for a dependency:
  1. the SSG package version (read from `packageRoot/package.json`),
  2. a hash of the consumer's content under `projectRoot/<contentDir>`,
  3. a hash of `site.config.ts`,
  4. a hash of any consumer i18n source,
      if introduced later.
- [ ] Confirm the cache invalidates on i18n changes.
       The current glob omits `src/i18n/`
      (`doc/planning/module-i18n-compose.md:139`);
       after the migration the SSG's i18n is package-internal,
      so the SSG-version input covers it.
       Verify this holds rather than assuming it.

## Phase 3: thread config through the pipeline

Replace each hardcoded site-specific constant with a value read from the validated config.

- [ ] `src/build.ts`:
   remove `const SITE_URL = 'https://aquati.cat'` (`:64`) and
      `const CONTENT_DIR = 'src/content'` (`:67`);
   read `config.url` and `config.contentDir`.
      Load and validate config at the top of the build.
- [ ] `src/build/render.ts`:
   remove `SVG_SOURCE` (`:10`) and `BACKGROUND` (`:12-18`);
      read `config.favicon.source` and `config.favicon.backgroundColor`.
- [ ] Footer ticker:
       parameterize `TICKER_QUOTES`;
       pass `config.tickerQuotes` to the footer renderer.
- [ ] `src/lib/rss.ts`:
   confirm `siteUrl` is already a parameter (it is,
   `:44,51`);
   thread `config.url`.
- [ ] Branding (closes the half-fixed token claim in the prior plan):
  - [ ] `src/style/tokens.ts:120-121` defines `--primary-light: #bf97e3` and `--primary-dark: #4e318f`.
        Source these from `config.brand` when present,
     default to the current values otherwise.
  - [ ] `src/template/head.ts:151-163` hardcodes the same two colors as `theme-color` meta tags.
        These are not CSS custom properties,
     so they need their own threading from `config.brand`.
- [ ] Site identity strings flow through the i18n label tables,
       not direct interpolation:
      `head.ts` `${title} | ${siteName}`,
       the RSS channel title and description,
       and the language picker
      all resolve `siteName` and `siteDescription` via `i18n.label(locale, 'siteName')` after the migration.

## Phase 4: environment-coupling hardening

`src/lib/git-dates.ts` assumes this exact repository.
 A published SSG must degrade gracefully.

- [ ] `findMiseMonorepoRootCached()` (`git-dates.ts:97,262`,
   from the private `module-fs-path`)
      assumes a mise monorepo root.
   Make the root resolution optional with a `projectRoot` fallback,
      so the date pipeline works when the consumer is not in a mise monorepo.
- [ ] `getGithubSlug()` derives `owner/repo` from the `origin` remote for source and edit links.
      Source the slug from `config.repository.slug` when provided;
   fall back to remote detection;
      emit no source link when neither is available.
- [ ] Date resolution order:
       git commit dates when in a git repo,
       else frontmatter date or file mtime.
      The current shallow-clone path uses `gh`;
       gate it behind "GitHub remote detected" so non-GitHub
      consumers do not invoke it.

## Phase 5: separate content into site-aquaticat

Create `package/webapp-content/site-aquaticat/` containing only Aquaticat content,
 config,
 and assets.

- [ ] `package.json` depending on the renamed SSG via `workspace:*`,
   with `[tools]` for `pagefind`
      and `caddy` if the dev/search tasks need them (the SSG pipeline uses both).
- [ ] `mise.toml` whose build and dev tasks invoke the SSG bin (see Phase 6),
   not `bun src/build.ts`.
- [ ] `site.config.ts` with the Aquaticat values (`url`,
   `locales: ['ca','en','zh']`,
   `defaultLocale`,
      per-locale `site` names and descriptions,
   `tickerQuotes`,
   `favicon`,
   and `brand` for the purple).
- [ ] Move all MDX content for every active locale:
       `content/ca/`,
       `content/en/`,
       `content/zh/`.
      The previous plan omitted `ca`.
- [ ] Move the co-located content images:
       `src/content/*.{avif,jpg,png}` sit beside,
       not under,
       the locale
      directories,
       and the previous plan did not list them.
- [ ] Move `public/` (favicon source,
       static images,
       `resume-no-pii.pdf`).
- [ ] Licenses:
       site-aquaticat carries `CC-BY-SA-4.0` for content (plus `LGPL-3.0-or-later` for any code).
      The SSG package drops to `LGPL-3.0-or-later` only;
       today it is `LGPL-3.0-or-later AND CC-BY-SA-4.0`.
- [ ] Verify the site builds and serves from the new package.

## Phase 6: invocation model (bin and dev server)

The previous plan never said how a consumer triggers a build whose entry lives in `node_modules`.

- [ ] Add a `bin` entry to the SSG `package.json` (modeled on `package/mvm/cli/package.json:9`),
      for example `{ "mono-ssg": "dist/final/node/index.mjs" }`,
       with subcommands `build`,
       `dev`,
       `serve`.
- [ ] The bin entry point has `#!/usr/bin/env node` as its first line
      (`AGENTS.md` "Adding new packages" rule 4);
       parse args with `@optique/core` (catalog),
       as `cli-git` does.
- [ ] The CLI loads `site.config.ts` from cwd,
       resolves `projectRoot` and `packageRoot`,
       runs the pipeline.
- [ ] Ship a default dev-server config from `packageRoot` (the current `Caddyfile` is generic SSG-output
      serving:
       try_files,
       hashed-asset cache headers).
       The `dev`/`serve` subcommands use it so consumers do
      not copy a `Caddyfile`;
       allow an override path in config.
- [ ] site-aquaticat's mise tasks call `mono-ssg build` and `mono-ssg dev`.

## Phase 7: rename and relocate

Resolve the name-to-path mismatch the previous plan introduced (`@monochromatic-dev/ssg` under
`package/webapp-content/ssg/` violates the path-encodes-name convention).
 See "Open decisions" for the
naming choice;
 the steps below are common to whichever option is picked.

- [ ] Move the directory to the chosen path and set `package.json` `name` to the matching convention name.
- [ ] Add an `exports` field for the public API (`defineConfig`,
       `SupportedLocale`,
       the label key union)
      and the `bin` field.
- [ ] Update the functional reference at `mise.toml:619`
      (`package/ssg/aquati.cat/node_modules/.bin`) and the rationale comments at
      `pnpm-workspace.yaml:243,309,313`.
- [ ] Remove `src/content/` from the SSG (now in site-aquaticat).

## Phase 8: publish preparation

- [ ] Bundle the private dependencies into the published output.
       The SSG depends on three `private: true`
      packages:
       `module-logger`,
       `module-fs-path`,
       `module-hyperscript`.
       tsdown/rolldown bundles them,
       but
      verify the bundle does not break `module-fs-path`'s monorepo-root logic (covered in Phase 4).
      `module-const`,
       `module-or-throw`,
       and `module-i18n-compose` are already publishable.
- [ ] Set `package.json` `files`,
       `main`/`exports`,
       `bin`,
       `license` (`LGPL-3.0-or-later`),
       and `repository`;
      remove `private: true`;
       add `publishConfig` mirroring `package/i18n-compose/core/package.json`.
- [ ] Add a `README.md`:
       what it is,
       the supported locale set,
       how to author `site.config.ts`,
      the CLI commands,
       and the config reference.
- [ ] Add a `prepublishOnly` or mise build task.
- [ ] Verify a clean install plus build works from site-aquaticat.
- [ ] Lint with zero errors;
       confirm tests pass;
       add config-loader and schema-validation tests.

## Phase 9: verification at the user boundary

- [ ] Build site-aquaticat through the bin (not `bun src/build.ts`),
       serve `dist/`,
       and confirm rendered
      output,
       RSS,
       search index,
       and favicons are correct,
       not merely that the build starts.
- [ ] Confirm the closed locale set:
       a config selecting only `['en']` builds an English-only site;
      a config naming an unsupported locale fails validation with a clear error.
- [ ] On a throwaway non-git directory (per "Verify on a throwaway,
       not against real state"),
       confirm graceful
      degradation:
       dates fall back,
       no source links,
       no mise-monorepo assumption,
       build still succeeds.

## Acceptance criteria

- [ ] No `typesafe-i18n` references remain (covered by the prerequisite migration).
- [ ] No arbitrary-language config;
       `locales` is a subset of `ca`/`en`/`zh`.
- [ ] All site-specific values come from `site.config.ts`:
       url,
       content dir,
       site names/descriptions,
      ticker quotes,
       favicon source and background,
       brand colors (both CSS custom properties and theme-color),
      repository slug.
- [ ] Cache invalidates on SSG version change,
       content change,
       and config change.
- [ ] The SSG runs as a dependency via its bin from an arbitrary `projectRoot`.
- [ ] The package publishes (private deps bundled,
       `private: true` removed,
       license corrected).
- [ ] git-dates degrades gracefully outside this repo.
- [ ] Content for all three locales plus co-located images live in site-aquaticat;
       the SSG has no content.

## Open decisions (for review)

1. Package name and location.
    The SSG is a build tool with a bin,
    so the path-encoded naming convention applies.
   - Option A:
      `package/cli/ssg`,
      name `@monochromatic-dev/cli-ssg`.
      Pros:
      matches the build-tool category and
     the bin convention;
      consistent with `cli-git`.
      Cons:
      `cli-ssg` is a slightly unusual public npm name.
   - Option B:
      `package/module/ssg`,
      name `@monochromatic-dev/module-ssg`.
      Pros:
      convention-consistent.
     Cons:
      it is a CLI plus library,
      not a plain module;
      less apt than A.
   - Option C:
      keep `package/webapp-content/ssg`,
      name `@monochromatic-dev/webapp-content-ssg`.
     Pros:
      smallest move.
      Cons:
      categorizes a general-purpose tool as site content,
      beside site-aquaticat.
   - Ranking:
      A > B > C.
      A wins over B because the bin makes it a CLI,
      which is exactly the `cli/` category;
     B wins over C because C miscategorizes a tool as content.
      This is a non-measurable preference
     (what public name you want),
      so it is left for you to decide rather than assumed.
2. Fonts as config.
    v1 ships the current typefaces as SSG defaults and subsets them against the consumer's
   content.
    Whether to expose font overrides in config now or defer is an open call.
3. Un-private versus bundle.
    Bundling the three private deps is the lighter path;
    un-privating and publishing
   them is heavier but cleaner if other packages will be published later.
