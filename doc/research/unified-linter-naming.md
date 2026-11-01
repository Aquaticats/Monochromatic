# Unified linter naming research

Read-only research from 2026-09-23 for the unified Rust and Markdown linter
([`doc/handover/unified-linter.md`](../handover/unified-linter.md)).
Registry availability is a snapshot of that day;
anyone can claim a crates.io name at any time,
and reserving one is an external publish that needs the user's authorization.

## Ecosystem precedent

- ESLint:
   binary `eslint`,
   config `eslint.config.{js,mjs,cjs,ts,mts,cts}`,
   directives `eslint-disable`,
   `eslint-enable`,
   `eslint-disable-line`,
   `eslint-disable-next-line` with a reason after `--`,
   rules `plugin/rule`,
   languages `plugin/language` such as `markdown/commonmark`;
   official language plugins `@eslint/markdown`,
   `@eslint/json`,
   `@eslint/css`.
- oxlint:
   binary `oxlint`,
   config `.oxlintrc.json` or `oxlint.config.ts`,
   directives `oxlint-disable` and siblings,
   also accepting `eslint-*` for migration;
   crates `oxc_*`.
- Biome:
   binary `biome`,
   config `biome.json`,
   suppressions `// biome-ignore lint/<group>/<rule>: <reason>`,
   GritQL plugins,
   crates `biome_<lang>_<role>` such as `biome_markdown_analyze`.
- Rome (archived 2023):
   `rome-ignore` suppressions,
   crates `rome_*`.
- Clippy and Dylint:
   suppression through rustc attributes;
   config `clippy.toml` and `dylint.toml`.
- deno lint:
   `// deno-lint-ignore <codes>` with the reason after `--`;
   plugin rules `<plugin>/<rule>`.
- Ruff:
   `# noqa` and `# ruff: disable[...]`;
   config `ruff.toml`;
   crates `ruff_*`.
- rslint:
   two unrelated tools now share the name and the directive prefix
   (`rslint/rslint`,
   dormant since 2023,
   and `web-infra-dev/rslint`,
   active),
   a cautionary precedent.
- selene:
   `-- selene: allow(lint)`;
   config `selene.toml`.
- stylelint:
   `stylelint-disable` family with the reason after ` -- `;
   rules `namespace/rule`.
- markdownlint:
   `<!-- markdownlint-disable -->` family;
   rule numbers with aliases such as `MD001` / `heading-increment`.
- rumdl:
   `rumdl-disable`,
   also accepting `markdownlint-*`;
   same name on crates.io,
   PyPI,
   and npm.
- Vale:
   `<!-- vale off -->`;
   config `.vale.ini`.

Patterns:

- The directive prefix is the tool's own name everywhere except Clippy,
   Dylint,
   and Ruff,
   so choosing the name chooses the directive.
- `<name>-disable|enable|disable-line|disable-next-line` plus ` -- reason` is shared by ESLint,
   oxlint,
   stylelint,
   rslint,
   rumdl,
   and today's `rust-linter`.
- Crate families lead with the tool name,
   then language or role.

## Repository facts

- Linter crates use the `monochromatic-` prefix
   (`monochromatic-rust-linter`,
   `-core`,
   `-pattern`,
   `-plugin-builtin`);
   other publishable crates such as `forbidden-strings` do not.
- `rust-linter-disable` appears only inside the linter's own crates;
   `markdown-lint` has no directives;
   no Markdown file uses `markdownlint-disable`,
   so the prefix can change without an alias.
- `#[allow(tool::rule)]` would need the nightly-only `register_tool`;
   if that ever stabilizes,
   the tool name must be a valid Rust identifier.
- No candidate appears in either forbidden-strings appendix.
- In-repository clashes:
   `mono` is the cli-git policy namespace (`cli-git.config.ts:24`,
   `:44`);
   `chroma` is the OKLCH colour term in 61 places.
- Rename size:
   73 tracked files mention `rust-linter`,
   58 mention `markdown-lint`,
   18 mention `rust-linter.toml`.

## Candidates

- `sumilint`:
   sumi is the ink of sumi-e monochrome painting.
  Free on crates.io (`sumilint` and `monochromatic-sumilint`),
   npm,
   and GitHub (zero repositories).
  Opaque to readers who do not know the word.
  8-character binary,
   valid identifier,
   no stutter with the `monochromatic-` prefix.
- `monolint`:
   one linter,
   and mono from Monochromatic.
  First readings collide with an existing monorepo linter (npm `monolint`,
   `flaviostutz/monolint`),
   Mono (.NET),
   and the repository's own `mono/` policy namespace.
  Free on crates.io,
   taken on npm.
- `monochromatic-lint`:
   descriptive fallback,
   free everywhere,
   18-character binary,
   36-character directive prefix,
   not a valid identifier.
- `achrolint`:
   achromatic plus lint;
   free everywhere;
   obscure spelling.
- `grisaille`:
   painting in greys;
   a dictionary word;
   hard spelling;
   does not say lint.
- `mochro`:
   contraction of Monochromatic;
   reads as a misspelled Mocha or mochi.
- `inklint`:
   reads as a linter for inkle's ink language.
- `aquatint`:
   etching technique and a pun on Aquaticat;
   "tint" suggests colour and sits one letter from "lint".
- `unilint`:
   taken on npm;
   a same-concept repository exists;
   no Monochromatic tie.
- `greylint`:
   one letter from the active linter `graylint`.
- `chromalint`:
   contradicts monochrome and clashes with the repository's `chroma` term and two large products.

Screened out as taken:
`omnilint`,
`polylint`,
`lintel`,
`graylint`,
`sumi`,
`chiaro`,
`tenebra`,
`mezzotint`,
`inkwash`.

## Ranking

`sumilint` > `monolint` > `monochromatic-lint` > `achrolint` > `grisaille` >
`mochro` > `inklint` > `aquatint` > `unilint` > `greylint` > `chromalint`.

- `sumilint` over `monolint`:
   same length and shape,
   but `monolint`'s first reading is an existing linter,
   Mono,
   and the repository's own namespace.
- `monolint` over `monochromatic-lint`:
   shorter binary and directive,
   valid identifier.
- `monochromatic-lint` over `achrolint`:
   neither collides,
   so legibility decides.
- `achrolint` over `grisaille`:
   says lint,
   easier spelling.
- `grisaille` over `mochro`:
   on-theme first reading against a misspelling.
- `mochro` over `inklint`:
   a near-miss beats a confidently wrong reading.
- `inklint` over `aquatint`:
   says lint and names a monochrome medium.
- `aquatint` over `unilint`:
   free on registries and personal to the owner.
- `unilint` over `greylint`:
   collides only with dormant repositories.
- `greylint` over `chromalint`:
   `chromalint` contradicts the theme and collides with large products.

## Caveats

- PyPI and trademark registries were not checked.
- Typeability and pronunciation are judgments;
   only lengths were measured.
