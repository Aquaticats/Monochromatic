# Package structure review

Review date:
 2026-02-17

> Stale as of 2026-05-13:
>  several proposed moves have already happened
> (`dev-script/`,
>  `test-fixture/`,
>  `webapp-*`,
>  and `desktop-daemon/` now exist).
> Treat the package list and proposed migration below as historical input,
> not current state.
>  Re-audit with the current `package/` tree before acting.

## Current layout

```text
package/
  build/
    backup-path      @monochromatic-dev/build-backup-path
    css              @monochromatic-dev/build-css
    ensure-dependencies  @monochromatic-dev/ensure-dependencies
    time             @monochromatic-dev/build-time
  config/
    dprint           @monochromatic-dev/config-dprint
    tsdown           @monochromatic-dev/config-tsdown
    typescript       @monochromatic-dev/config-typescript
  fixture/
    test-css-imported        @monochromatic-dev/test-css-imported
    test-css-imported-no-exports  @monochromatic-dev/test-css-imported-no-exports
    test-css-importing       @monochromatic-dev/test-css-importing
    test-css-importing-filepath  @monochromatic-dev/test-css-importing-filepath
  module/
    es               @monochromatic-dev/module-es
    nvim-mcp         @monochromatic-dev/nvim-mcp
  site/
    ai-tree          ai-tree
    astro-test       @monochromatic-dev/site-astro-test
    bun-test         @monochromatic-dev/bun-test
    done             @monochromatic-dev/site-done
    exa-search       @monochromatic-dev/site-exa-search
    rss              @monochromatic-dev/site-rss
  style/
    monochromatic    @monochromatic-dev/style-monochromatic
```

## Structural constraint

All packages must live exactly two levels deep:
 `package/<category>/<name>`.
Category names are hyphenated multi-word when needed for clarity.
The `package.json` "name" field follows `@monochromatic-dev/<category>-<name>`.

## Findings

### Naming inconsistencies in `package.json` "name" fields

The convention `@monochromatic-dev/<category>-<name>` is used for most packages,
 but several break it:

- **`ai-tree`**:
   bare name,
   no scope or category prefix
- **`@monochromatic-dev/bun-test`**:
   missing category prefix
- **`@monochromatic-dev/ensure-dependencies`**:
   missing `build-` prefix
- **`@monochromatic-dev/nvim-mcp`**:
   missing category prefix
- **`test-css-*` packages**:
   use `test-` prefix instead of matching their directory category

### Misplaced packages

- **`bun-test`** lives in `site/` but is an experiment for the `done` app,
   not a standalone site.
  Belongs alongside `done` in the same webapp category.
- **`nvim-mcp`** lives in `module/` but is a standalone MCP server binary,
   not a reusable library.
  Belongs in a dedicated `mcp/` category.

### Category problems

- **`build/`** is too broad;
   mixes source transformers (`css`) with developer workflow scripts (`backup-path`,
   `ensure-dependencies`) and a near-empty stub (`time`).
- **`module/`** is vague in a JS ecosystem where "module" means everything.
  Holds both a general-purpose library (`es`) and an MCP server binary (`nvim-mcp`).
- **`site/`** is the worst offender;
   conflates full server apps (`done`,
   `rss`),
   single-page tools (`exa-search`),
   a static documentation site (`astro-test`),
   an AI tool (`ai-tree`),
   and a build-tool test harness (`bun-test`).
- **`fixture/`** contains only test fixtures but uses a non-descriptive name.
- **`style/`** is adequate for now but undersells the category if it grows
  to include multiple stylesheet packages (resets,
   utility sheets,
   theme packages).

### Package flagged for removal

- **`time`** (`build-time`):
   contains only `console.log(new Date().toISOString())`.
  Its `package.json` repository.
  directory points to `build/file-enforcer`,
   a path that does not exist.
  Appears to be an abandoned placeholder.
  Recommend removing it.

### Missing descriptions

These `package.json` files have empty or absent `description` fields:

- `config-dprint`
- `config-tsdown`

## Proposed categories

- **`build-tool/`**:
   packages that transform or compile source code as part of a build pipeline
- **`config/`**:
   shared tool configuration presets (oxlint,
   TypeScript,
   tsdown,
   dprint,
   etc.)
- **`dev-script/`**:
   standalone developer automation scripts not directly part of a build pipeline
- **`library/`**:
   reusable code meant to be imported by other packages (replaces the vague "module")
- **`mcp/`**:
   Model Context Protocol servers
- **`stylesheet/`**:
   CSS packages meant to be imported by applications (resets,
   design tokens,
   mixins,
   themes)
- **`test-fixture/`**:
   packages that exist solely as inputs for automated tests
- **`webapp-content/`**:
   content-driven or documentation websites (static site generators,
   blogs)
- **`webapp-productivity/`**:
   interactive web applications for personal productivity (task managers,
   feed readers,
   etc.)
- **`webapp-search/`**:
   web applications centered on search and information retrieval

Experiments and PoCs live as regular siblings in the category of the package they are experimenting for,
 distinguished by their `package.json` description (e.g. "Experimental companion to done").
New categories are created as needed when a hypothetical non-experiment package would warrant one.

## Proposed migration

### Packages to rename (`package.json` "name" field)

- `ai-tree` -> `@monochromatic-dev/webapp-search-ai-tree`
- `@monochromatic-dev/bun-test` -> `@monochromatic-dev/webapp-productivity-bun-test`
- `@monochromatic-dev/ensure-dependencies` -> `@monochromatic-dev/dev-script-ensure-dependencies`
- `@monochromatic-dev/nvim-mcp` -> `@monochromatic-dev/mcp-nvim`
- `@monochromatic-dev/test-css-imported` -> `@monochromatic-dev/test-fixture-css-imported`
- `@monochromatic-dev/test-css-imported-no-exports` -> `@monochromatic-dev/test-fixture-css-imported-no-exports`
- `@monochromatic-dev/test-css-importing` -> `@monochromatic-dev/test-fixture-css-importing`
- `@monochromatic-dev/test-css-importing-filepath` -> `@monochromatic-dev/test-fixture-css-importing-filepath`
- `@monochromatic-dev/build-backup-path` -> `@monochromatic-dev/backup-path-cli`
- `@monochromatic-dev/build-css` -> `@monochromatic-dev/css-cli`
- `@monochromatic-dev/build-time` -> flagged for removal
- `@monochromatic-dev/module-es` -> `@monochromatic-dev/library-es`
- `@monochromatic-dev/style-monochromatic` -> `@monochromatic-dev/stylesheet-monochromatic`
- `@monochromatic-dev/site-astro-test` -> `@monochromatic-dev/webapp-content-astro-test`
- `@monochromatic-dev/site-done` -> `@monochromatic-dev/done-app`
- `@monochromatic-dev/site-exa-search` -> `@monochromatic-dev/webapp-search-exa-search`
- `@monochromatic-dev/site-rss` -> `@monochromatic-dev/rss-app`

### Packages to move (directory)

- `build/backup-path` -> `backup-path/cli`
- `build/css` -> `css/cli`
- `build/ensure-dependencies` -> `dev-script/ensure-dependencies`
- `build/time` -> remove
- `fixture/test-css-imported` -> `test-fixture/css-imported`
- `fixture/test-css-imported-no-exports` -> `test-fixture/css-imported-no-exports`
- `fixture/test-css-importing` -> `test-fixture/css-importing`
- `fixture/test-css-importing-filepath` -> `test-fixture/css-importing-filepath`
- `module/es` -> `library/es`
- `module/nvim-mcp` -> `mcp/nvim`
- `site/ai-tree` -> `webapp-search/ai-tree`
- `site/astro-test` -> `webapp-content/astro-test`
- `site/bun-test` -> `webapp-productivity/bun-test`
- `site/done` -> `done/app`
- `site/exa-search` -> `webapp-search/exa-search`
- `site/rss` -> `rss/app`
- `style/monochromatic` -> `stylesheet/monochromatic`

### Fill in missing descriptions

Add meaningful `description` fields to the remaining packages listed above.

### Proposed final layout

```text
package/
  build-tool/
    css                          @monochromatic-dev/css-cli
  config/
    dprint                       @monochromatic-dev/config-dprint
    tsdown                       @monochromatic-dev/config-tsdown
    typescript                   @monochromatic-dev/config-typescript
  dev-script/
    backup-path                  @monochromatic-dev/backup-path-cli
    ensure-dependencies          @monochromatic-dev/dev-script-ensure-dependencies
  library/
    es                           @monochromatic-dev/library-es
  mcp/
    nvim                         @monochromatic-dev/mcp-nvim
  stylesheet/
    monochromatic                @monochromatic-dev/stylesheet-monochromatic
  test-fixture/
    css-imported                 @monochromatic-dev/test-fixture-css-imported
    css-imported-no-exports      @monochromatic-dev/test-fixture-css-imported-no-exports
    css-importing                @monochromatic-dev/test-fixture-css-importing
    css-importing-filepath       @monochromatic-dev/test-fixture-css-importing-filepath
  webapp-content/
    astro-test                   @monochromatic-dev/webapp-content-astro-test
  webapp-productivity/
    bun-test                     @monochromatic-dev/webapp-productivity-bun-test
    done                         @monochromatic-dev/done-app
    rss                          @monochromatic-dev/rss-app
  webapp-search/
    ai-tree                      @monochromatic-dev/webapp-search-ai-tree
    exa-search                   @monochromatic-dev/webapp-search-exa-search
```
