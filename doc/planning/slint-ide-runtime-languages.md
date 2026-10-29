# Slint IDE runtime language coverage

## Status

Implemented on branch `feat/ide-runtime-languages` on 2026-10-05,
for the queue item "Provision measured Helix runtime language coverage" in
[the handover](../handover/slint-ide-0x.md).
The build-only runtime now ships 27 grammars instead of the initial five.
Language-server startup and subprocess project-write confinement are separate queue items,
not part of this work.

Sources of truth after this change:

- `package/ide/app/src/bin/ide-runtime.rs`:
  the `GRAMMARS` selection.
- `package/ide/app/mise.toml`:
  the `runtime` task that fetches,
  builds,
  checks notices,
  and publishes assets.
- `package/ide/app/src/syntax.rs`:
  recognition and the bundled-manifest rule.
- `package/ide/app/tests/syntax_inventory.rs`,
  `tests/syntax_injection.rs`,
  and `tests/syntax_provisioning.rs`:
  one test per language,
  per companion grammar,
  and per manifest outcome.

## Selection rule

The confirmed contract is:
coverage is the languages `tokei` finds as repository source or configuration,
intersected with what Helix supports at the pinned revision
`ba40e547426b0f9896c8bdc699a4ab11f2b37dbc`.
Documentation code fences do not add languages,
and a language Helix lacks gets no custom integration.

Three refinements were needed to apply that rule mechanically.

### A measured language can be several Helix languages

Helix recognizes files by glob and extension,
so one `tokei` language can resolve to more than one Helix language.
`tsconfig.json` is `jsonc`,
`mise.toml` is `miseconfig`,
and `.github/workflows/*.yml` is `github-action`.
Each of those reuses the grammar of its base language,
so they add highlighting rules but no grammar.
The intersection therefore lists Helix languages,
while the build selection lists grammars.

### Grammar names are not language names

The `use-grammars` selection takes grammar ids.
The `qml` language uses the `qmljs` grammar,
and `markdown.inline` uses `markdown_inline`.

### Injected grammars are optional, so they follow the same presence rule

A language's queries can inherit other query directories and inject other languages.
Neither creates a hard requirement:

- `; inherits:` names query directories,
  not grammars,
  and the `runtime` task already publishes the whole pinned `runtime/queries` tree.
- An injected language whose grammar library is absent is skipped.
  `helix-core/src/syntax.rs` (`compile_syntax_config`) returns `Ok(None)` when the library does not exist,
  and tree-house 0.4.0 `src/parse.rs` (`LayerData::parse`) returns without parsing
  when the loader has no configuration.
  The host language still paints that range with its own rules.

So an injected grammar is included only when its trigger occurs in repository content,
measured with `rg` and a positive control per pattern.
[Companion grammars](#companion-grammars) records each count.

## Measurement

### Command and rule

```sh
# run from any directory; tokei 15.0.0
tokei --hidden --exclude .git --output json /var/home/user/worktrees/ide-runtime-languages
```

This repeats the rule of the 2026-10-04 measurement recorded in
[the scope history](slint-ide-0x.md) under "Measured language scope":

- ignore files are respected,
  which excludes `node_modules`,
  `target`,
  and other build output;
- hidden files are included and `.git` is excluded;
- active,
  paused (`package-paused`),
  and deprecated (`package-deprecated`) packages are all included,
  as are documentation directories.

The root is a clean checkout of `main` at `cd54f8b64`,
so the result does not depend on untracked files in the main worktree.
Every one of the 9,559 reported paths is tracked (`git ls-files` comparison:
9,559 reported,
9,559 tracked,
0 reported but untracked).
Generated TypeScript and JSON still contribute to line counts,
so the counts rank nothing;
they only establish presence.

### Standalone languages

Files and code lines per `tokei` language:

- TypeScript:
  5,415 files,
  611,840 lines.
- JSON:
  568 files,
  387,039 lines.
- Rust:
  663 files,
  63,896 lines.
- HTML:
  152 files,
  21,349 lines.
- TOML:
  466 files,
  17,096 lines.
- YAML:
  24 files,
  12,520 lines.
- JavaScript:
  72 files,
  6,159 lines.
- Kotlin:
  70 files,
  5,892 lines.
- CSS:
  55 files,
  4,370 lines.
- SVG:
  7 files,
  3,327 lines.
- Slint:
  5 files,
  2,429 lines.
- XML:
  137 files,
  1,140 lines.
- HCL:
  1 file,
  722 lines.
- SQL:
  7 files,
  521 lines.
- QML:
  8 files,
  484 lines.
- Shell:
  5 files,
  351 lines.
- C:
  1 file,
  208 lines.
- Batch:
  2 files,
  128 lines.
- Dockerfile:
  3 files,
  33 lines.
- C++:
  1 file,
  17 lines.
- Markdown:
  1,466 files.
- MDX:
  36 files.
- Plain Text:
  395 files.

No `.tsx` or `.jsx` file is tracked (`git ls-files '*.tsx' '*.jsx'` prints nothing),
so `tokei` reports neither.

### Embedded-only languages

`tokei` reports these only inside Markdown code fences,
so by the contract they add nothing:
BASH,
CMake,
Clojure,
Dhall,
Go,
INI,
Java,
Perl,
PHP,
PlantUML,
PowerShell,
Python,
Ruby,
Swift,
Zig,
and Zsh.
Bash is selected anyway through standalone shell scripts.

### Tracked files `tokei` does not classify

`tokei` leaves 996 tracked files unclassified.
The pinned Helix loader does not recognize 866 of them (images,
audio,
fonts,
fuzz seeds,
and similar).
It recognizes the other 130:

- already covered by a selected grammar:
  `toml` 25 (lock files such as `Cargo.lock`),
  `dockerfile` 16 (`Containerfile`,
  `*.Containerfile`,
  `Containerfile.*`),
  `javascript` 6,
  `jsonc` 3,
  `json` 3,
  `xml` 3;
- recognized but not bundled,
  because the stated rule is "per `tokei`":
  `diff` 49 (`*.patch`),
  `git-ignore` 10,
  `ini` 4,
  `properties` 4,
  `caddyfile` 3,
  `git-attributes` 2,
  `ghostty` 1,
  `git-config` 1.

The second group is the broader reading of "languages actually present".
It is proposed,
not implemented;
see [Open questions](#open-questions).
Those files open as plain text.

## Intersection with the pinned Helix registry

### Method

The registry is `helix-loader`'s `default_lang_config()`,
which embeds `languages.toml` of the pinned revision
(342 languages,
303 grammars).
The copy in the Cargo checkout inside the `ide-cargo` volume and the clone at
`~/temp/agent/slint-ide-languages.mEr8K9/helix` are byte-identical (`cmp`).

Every measured path was mapped twice:

- statically,
  by a scratch script that reimplements Helix's glob and extension matching;
- through the pinned loader itself,
  by a disposable integration test that called `Loader::language_for_filename`
  and the application's `SyntaxEngine::language_id` on all 10,555 tracked paths.

Both mappings agree on every count.
The disposable test is not committed;
its source and output are in the scratch directory named under [Evidence](#evidence).

### Result per measured language

- TypeScript:
  selected.
  All 5,415 files are Helix `typescript`.
- JavaScript:
  selected.
  All 72 files are Helix `javascript`.
- Rust:
  selected.
  All 663 files are Helix `rust`.
- Kotlin:
  selected.
  All 70 files are Helix `kotlin`.
- Slint:
  selected.
  All 5 files are Helix `slint`.
- QML:
  selected.
  All 8 files are Helix `qml`,
  grammar `qmljs`.
- HCL:
  selected.
  The file is Helix `hcl`.
- SQL:
  selected.
  All 7 files are Helix `sql`.
- Shell:
  selected.
  3 files are Helix `bash` by extension.
  The 2 extensionless `gradlew` wrappers are `bash` by their `#!/bin/sh` shebang.
- C:
  selected.
  The file is Helix `c`.
- C++:
  selected.
  The file is Helix `cpp`.
- Batch:
  selected.
  Both files are Helix `batch`.
- Dockerfile:
  selected.
  All 3 files are Helix `dockerfile`,
  which also recognizes every Containerfile spelling.
- JSON:
  selected.
  370 files are Helix `json` and 198 are `jsonc`,
  both on grammar `json`.
- TOML:
  selected.
  238 files are Helix `toml` and 228 are `miseconfig`,
  both on grammar `toml`.
- YAML:
  selected.
  15 files are Helix `github-action`,
  6 are `yaml`,
  and 3 are `docker-compose`,
  all on grammar `yaml`.
- HTML:
  selected.
  All 152 files are Helix `html`.
- CSS:
  selected.
  All 55 files are Helix `css`.
- XML:
  selected.
  All 137 files are Helix `xml`.
- SVG:
  selected.
  All 7 files are Helix `xml`.
- Markdown:
  selected.
  All 1,466 files are Helix `markdown`.
- MDX:
  selected as Markdown.
  All 36 files are Helix `markdown`,
  because `mdx` is a Markdown file type in the registry.
  Helix has no JSX-aware MDX language,
  and none is added.
- Plain Text:
  not selected.
  The pinned loader recognizes none of these files,
  so all 395 are read as plain text,
  which is what they are.

Every measured language except Plain Text is supported by Helix at the pinned revision.

## Per-grammar record

Each grammar was fetched at the revision the pinned registry names
(`git rev-parse HEAD` in each fetched source equals the pinned value),
built in the bounded container,
and published with its notices under `licenses/<grammar>/`.
Sizes are the published `.so` files.
"Injects" lists the statically named injection targets of the languages on that grammar,
including inherited query files.

### Grammars for measured languages

#### `typescript`

- Helix language:
  `typescript`.
- Source:
  `https://github.com/tree-sitter/tree-sitter-typescript`,
  subpath `typescript`,
  revision `75b3874edb2dc714fb1fd77a32013d0f8699989f`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  1,439,480 bytes.
- Inherits queries:
  `ecma`,
  `_typescript`.
- Injects:
  `bash`,
  `comment`,
  `css`,
  `graphql`,
  `jsdoc`,
  `regex`,
  plus a language named by a template tag.

#### `javascript`

- Helix languages:
  `javascript`,
  and `jsx` for free.
- Source:
  `https://github.com/tree-sitter/tree-sitter-javascript`,
  revision `58404d8cf191d69f2674a8fd507bd5776f46cb11`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  435,672 bytes.
- Inherits queries:
  `ecma`,
  `_javascript`.
- Injects:
  the same targets as `typescript`.

#### `rust`

- Helix languages:
  `rust`,
  and `rust-format-args-macro`,
  an injection-only language for macro token trees.
- Source:
  `https://github.com/tree-sitter/tree-sitter-rust`,
  revision `77a3747266f4d621d0757825e6b11edcbf991ca5`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  1,144,784 bytes.
- Injects:
  `comment`,
  `html`,
  `json`,
  `markdown-rustdoc`,
  `regex`,
  `rust`,
  `rust-format-args-macro`,
  `rust-format-args`,
  `slint`,
  `sql`.

#### `kotlin`

- Helix language:
  `kotlin`.
- Source:
  `https://github.com/fwcd/tree-sitter-kotlin`,
  revision `f66d2908542e93c0204c6c241f794afe4e9cd5d1`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  5,768,128 bytes.
- Injects:
  `comment`,
  `regex`.

#### `slint`

- Helix language:
  `slint`.
- Source:
  `https://github.com/slint-ui/tree-sitter-slint`,
  revision `f94f96ce093ec153f037228ac2fac5f1a3cd9ac6`.
- License:
  MIT for the compiled sources,
  in the REUSE layout.
  See [The Slint grammar notice](#the-slint-grammar-notice).
  `LICENSES/MIT.txt` and `REUSE-headers.txt` shipped.
- Size:
  627,872 bytes.
- Injects:
  `comment`.

#### `qmljs`

- Helix language:
  `qml`.
- Source:
  `https://github.com/yuja/tree-sitter-qmljs`,
  revision `0b2b25bcaa7d4925d5f0dda16f6a99c588a437f1`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  1,368,840 bytes.
- Injects:
  `comment`,
  `javascript`.

#### `hcl`

- Helix languages:
  `hcl`,
  and `tfvars` and `docker-bake` for free.
- Source:
  `https://github.com/tree-sitter-grammars/tree-sitter-hcl`,
  revision `64ad62785d442eb4d45df3a1764962dafd5bc98b`.
- License:
  Apache-2.0,
  `LICENSE` shipped.
  The source has no `NOTICE` file.
- Size:
  111,792 bytes.
- Injects:
  `comment`.

#### `sql`

- Helix language:
  `sql`.
- Source:
  `https://github.com/DerekStride/tree-sitter-sql`,
  revision `851e9cb257ba7c66cc8c14214a31c44d2f1e954e`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  11,098,288 bytes,
  the largest grammar and about a third of the runtime.
- Injects:
  nothing.

#### `bash`

- Helix languages:
  `bash`,
  and `env` and `pkgbuild` for free.
- Source:
  `https://github.com/tree-sitter/tree-sitter-bash`,
  revision `a06c2e4415e9bc0346c6b86d401879ffb44058f7`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  1,364,648 bytes.
- Injects:
  `awk`,
  `bash`,
  `comment`,
  `jq`,
  `regex`.

#### `c`

- Helix language:
  `c`.
- Source:
  `https://github.com/tree-sitter/tree-sitter-c`,
  revision `b780e47fc780ddc8da13afa35a3f4ed5c157823d`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  632,000 bytes.
- Injects:
  `c`,
  `comment`.

#### `cpp`

- Helix language:
  `cpp`.
- Source:
  `https://github.com/tree-sitter/tree-sitter-cpp`,
  revision `8b5b49eb196bec7040441bee33b2c9a4838d6967`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  5,592,040 bytes.
- Inherits queries:
  `c`.
- Injects:
  `c`,
  `comment`,
  `cpp`,
  plus a language named by a raw-string delimiter.

#### `batch`

- Helix language:
  `batch`.
- Source:
  `https://github.com/wharflab/tree-sitter-batch`,
  revision `5fc5f54267ef9a78e7a5319b80e092194252aabf`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  178,264 bytes.
- Injects:
  `comment`,
  `regex`.

#### `dockerfile`

- Helix language:
  `dockerfile`.
- Source:
  `https://github.com/camdencheek/tree-sitter-dockerfile`,
  revision `971acdd908568b4531b0ba28a445bf0bb720aba5`.
- License:
  MIT by its `LICENSE` text and `Cargo.toml`,
  `LICENSE` shipped.
  Its `package.json` and `tree-sitter.json` say ISC instead;
  both are permissive,
  and the shipped notice is the `LICENSE` text.
- Size:
  74,800 bytes.
- Injects:
  `bash`,
  `comment`,
  plus a language named by a heredoc file name.

#### `json`

- Helix languages:
  `json`,
  `jsonc`,
  and `json-ld` for free.
- Source:
  `https://github.com/tree-sitter/tree-sitter-json`,
  revision `001c28d7a29832b06b0e831ec77845553c89b56d`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  20,464 bytes.
- Inherits queries:
  `jsonc` inherits `json`.
- Injects:
  `comment`.

#### `toml`

- Helix languages:
  `toml`,
  `miseconfig`,
  and `jjconfig`,
  `cross-config`,
  and `git-cliff-config` for free.
- Source:
  `https://github.com/tree-sitter-grammars/tree-sitter-toml`,
  revision `64b56832c2cffe41758f28e05c756a3a98d16f41`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  41,472 bytes.
- Inherits queries:
  `miseconfig` inherits `toml`.
- Injects:
  `comment`;
  `miseconfig` also injects `bash` into task `run` strings,
  or a language named by a shebang.

#### `yaml`

- Helix languages:
  `yaml`,
  `github-action`,
  `docker-compose`,
  and `nestedtext`,
  `gitlab-ci`,
  and `woodpecker-ci` for free.
- Source:
  `https://github.com/ikatyang/tree-sitter-yaml`,
  revision `0e36bed171768908f331ff7dff9d956bae016efb`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  206,056 bytes.
- Inherits queries:
  `github-action` and `docker-compose` inherit `yaml`.
- Injects:
  `bash`,
  `comment`;
  `github-action` also injects `javascript`.

#### `html`

- Helix languages:
  `html`,
  and `webc` for free.
- Source:
  `https://github.com/tree-sitter/tree-sitter-html`,
  revision `73a3947324f6efddf9e17c0ea58d454843590cc0`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  37,912 bytes.
- Injects:
  `comment`,
  `css`,
  `javascript`,
  `regex`.

#### `css`

- Helix language:
  `css`.
- Source:
  `https://github.com/tree-sitter/tree-sitter-css`,
  revision `dda5cfc5722c429eaba1c910ca32c2c0c5bb1a3f`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  132,560 bytes.
- Injects:
  `comment`.

#### `xml`

- Helix languages:
  `xml`,
  and `msbuild` for free.
- Source:
  `https://github.com/RenjiSann/tree-sitter-xml`,
  revision `48a7c2b6fb9d515577e115e6788937e837815651`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  73,704 bytes.
- Injects:
  `comment`.

#### `markdown`

- Helix languages:
  `markdown`,
  `markdown-rustdoc` (Rust documentation comments),
  and `quarto` and `rmarkdown` for free.
- Source:
  `https://github.com/tree-sitter-grammars/tree-sitter-markdown`,
  subpath `tree-sitter-markdown`,
  revision `f969cd3ae3f9fbd4e43205431d0ae286014c05b5`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  376,160 bytes.
- Injects:
  `html`,
  `markdown.inline`,
  `toml`,
  `yaml`,
  plus a language named by a fence info string or shebang.
  A fence in a bundled language is painted;
  any other fence stays unpainted,
  which matches "documentation fences do not add languages".

### Companion grammars

The rule and its evidence are under [the injected-grammar rule][injected-rule].
Trigger counts come from `rg` over the measurement root,
each pattern first shown to match a control file.

#### Included: `comment`

- Purpose:
  tags,
  issue references,
  mentions,
  and links inside comments of every selected language.
- Trigger:
  61,448 line comments in 2,514 TypeScript,
  Rust,
  Kotlin,
  TOML,
  and shell files.
- Source:
  `https://github.com/stsewd/tree-sitter-comment`,
  revision `66272d2b6c73fb61157541b69dd0a7ce7b42a5ad`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  20,960 bytes.

#### Included: `jsdoc`

- Purpose:
  tags and types inside `/** */` comments in TypeScript and JavaScript.
  It was already in the initial slice.
- Trigger:
  63,873 documentation-comment openings in 4,838 files.
- Source:
  `https://github.com/tree-sitter/tree-sitter-jsdoc`,
  revision `658d18dcdddb75c760363faa4963427a7c6b52db`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  53,848 bytes.

#### Included: `regex`

- Purpose:
  regular-expression literals in TypeScript and JavaScript,
  and pattern strings in Rust,
  Kotlin,
  shell,
  Batch,
  and HTML.
- Trigger:
  245 ECMAScript regular-expression literals in 70 files,
  and 2 `Regex::new(` calls in Rust.
- Source:
  `https://github.com/tree-sitter/tree-sitter-regex`,
  revision `b2ac15e27fce703d2f37a79ccd94a5c0cbe9720b`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  40,944 bytes.

#### Included: `rust-format-args`

- Purpose:
  placeholders inside Rust formatting macros.
- Trigger:
  5,079 formatting-macro calls in 407 Rust files.
- Source:
  `https://github.com/nik-rev/tree-sitter-rust-format-args`,
  revision `84ffe550e261cf5ea40a0ec31849ba2443bae99f`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  28,592 bytes.

#### Included: `markdown_inline`

- Purpose:
  inline content of every Markdown paragraph (Helix language `markdown.inline`).
  Without it Markdown has block structure only.
- Trigger:
  169,240 inline code spans in 1,455 Markdown and MDX files.
- Source:
  `https://github.com/tree-sitter-grammars/tree-sitter-markdown`,
  subpath `tree-sitter-markdown-inline`,
  revision `f969cd3ae3f9fbd4e43205431d0ae286014c05b5`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  363,416 bytes.

#### Included: `awk`

- Purpose:
  AWK program text passed to `awk` in shell,
  including shell embedded in workflow `run` steps.
- Trigger:
  12 `awk '...'` invocations in 3 files:
  `.github/workflows/cargo-publish.yml`,
  `.github/workflows/forbidden-strings.yml`,
  and `package-paused/webapp-forge/stress/garage-init.sh`.
- Source:
  `https://github.com/Beaglefoot/tree-sitter-awk`,
  revision `34bbdc7cce8e803096f47b625979e34c1be38127`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  758,360 bytes.
- Note:
  this is the weakest inclusion.
  AWK is a language in its own right that the repository only embeds;
  dropping it costs highlighting inside those 12 program strings and nothing else.

#### Retained without measurement: `tsx`

- Reason:
  the initial slice shipped it,
  and this work extends that slice.
  No `.tsx` file is measured.
- Source:
  `https://github.com/tree-sitter/tree-sitter-typescript`,
  subpath `tsx`,
  revision `75b3874edb2dc714fb1fd77a32013d0f8699989f`.
- License:
  MIT,
  `LICENSE` shipped.
- Size:
  1,472,272 bytes.
- Removing it:
  delete the `"tsx"` entry in `GRAMMARS`,
  delete `tsx_component_is_read_as_tsx`,
  and clear stale `tsx.so` files under `target`,
  because the `runtime` task rejects a built grammar that is no longer selected.

#### Excluded: `graphql`

- Injected by:
  TypeScript and JavaScript,
  for `gql` and `graphql` template tags and `#graphql` strings.
- Trigger:
  0 matches in repository source (the control file matches).

#### Excluded: `jq`

- Injected by:
  shell,
  for program text passed to `jq` or `jaq`.
- Trigger:
  0 invocations with a quoted program.
  The single `--jq '...'` in `.github/workflows/forbidden-strings.yml` is a `gh` option,
  not a `jq` command,
  so the injection does not apply.

#### Excluded: `latex`

- Injected by:
  inline Markdown,
  for `$...$` math.
- Trigger:
  0 display-math lines and 0 common LaTeX commands in Markdown and MDX.
  It would also be documentation-embedded content,
  which the contract excludes.

## Licenses

### Summary

- 25 of 27 grammars are MIT with a root `LICENSE` file.
- `hcl` is Apache-2.0 with a root `LICENSE` file and no `NOTICE` file.
- `slint` is MIT in the REUSE layout,
  described under [The Slint grammar notice](#the-slint-grammar-notice).
- Helix's query files are MPL-2.0;
  `Helix-LICENSE` is published beside them,
  as it was for the initial slice.

Each grammar is a separate shared object,
loaded at run time and shipped beside the LGPL-3.0-or-later application with its own notice.
The [FSF license list][fsf-licenses],
fetched on 2026-10-05,
describes the Expat (MIT) license as compatible with the GNU GPL
and the Apache License 2.0 as compatible with GPL version 3 but not version 2.
LGPL version 3 is GPL version 3 plus added permissions,
and this application is version 3 or later,
so neither license raises a combination or side-by-side distribution question.
Nothing in the selection is copyleft,
proprietary,
or unlicensed.
This is a reading of the shipped license texts and that list,
not a legal review.

### Flags

- `hcl`:
  Apache-2.0 rather than MIT.
  It adds a patent grant and notice-preservation terms;
  the shipped `LICENSE` satisfies the latter.
- `dockerfile`:
  inconsistent metadata (MIT in `LICENSE` and `Cargo.toml`,
  ISC in `package.json` and `tree-sitter.json`).
  Both are permissive.
- `slint`:
  two files in the grammar repository carry non-MIT identifiers.
  `README.md` says `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-1.1 OR LicenseRef-Slint-commercial`,
  and `.prettierrc.toml` says `GPL-3.0-only OR LicenseRef-Slint-commercial`.
  Neither file is compiled or shipped.
  The compiled inputs,
  `grammar.js` and `src/scanner.c`,
  carry `SPDX-License-Identifier: MIT`,
  and `Cargo.toml` and `tree-sitter.json` declare MIT.
- `runtime/queries/yaml/injections.scm`:
  part of that Helix query file is nvim-treesitter code under Apache-2.0,
  with its copyright and license notice inline.
  It ships unmodified,
  as it did in the initial slice.
  It is the only query file among the selected languages with an embedded third-party notice;
  11 more exist in the published tree for languages that are not selected
  (`d`,
  `elixir`,
  `elvish`,
  `gn`,
  `pkl`,
  `rpmspec`,
  `sml`,
  `snakemake`).

### The Slint grammar notice

The `runtime` task stops when a grammar source has no root notice file
(`LICENSE`,
`COPYING`,
`NOTICE`,
`AUTHORS`,
and their spellings).
The Slint grammar is the one selected source without one.
It follows the REUSE layout:

- `LICENSES/MIT.txt` holds the license text,
  with the template line `Copyright (c) <year> <copyright holders>`;
- the actual holder,
  `Copyright © SixtyFPS GmbH <info@slint.dev>`,
  appears only in source headers,
  each beside `SPDX-License-Identifier: MIT`.

Copying `LICENSES/` alone would ship a license text without its copyright notice.
Decision taken,
reported here for confirmation:
the task accepts a source without a root notice only when all of these hold,
and otherwise still stops with `Missing grammar license notices for <grammar>`:

- a `LICENSES/` directory exists;
- at least one of `grammar.js`,
  `src/scanner.c`,
  `src/scanner.cc`,
  and `src/parser.c` has a copyright line in its first 8 lines;
- every `SPDX-License-Identifier` in those headers names exactly one license,
  and `LICENSES/<identifier>.txt` exists.

It then publishes `LICENSES/` unchanged plus `REUSE-headers.txt`,
the header lines copied verbatim and prefixed with their file names.
The check was exercised against the fetched Slint source and seven fixtures:
2 accepted,
6 rejected (no `LICENSES/`,
text for another license,
compound expression,
no copyright line,
no identifier,
no headers).

Alternatives,
with the ranking:

- Accept the REUSE layout with header extraction (implemented).
  Ships the real copyright notice;
  adds one small function to the task.
- Accept `LICENSES/` alone.
  Smaller,
  but ships a placeholder copyright line.
- Exclude Slint until decided.
  No notice question,
  but the application's own interface language would be unhighlighted.

Ranking:
extraction > `LICENSES/` alone,
because only the first ships the notice MIT requires;
`LICENSES/` alone > exclusion,
because Slint is a primary language of this repository.

## Language servers

Server configuration comes from the pinned registry's `language-servers` and `[language-server.<name>].command`.
Host presence was measured two ways:

- `command -v <command>` for each pinned command,
  which is the authority here;
- `hx --health <language>` from the installed Helix 25.07.1,
  used only as a second opinion on presence.
  It reads its own installed configuration,
  not the pinned one:
  it does not know `batch`,
  `miseconfig`,
  or `github-action`,
  and its `dockerfile`,
  `docker-compose`,
  and `markdown` entries lack the pinned `docker-language-server` and `rumdl` servers.
  Where it knows a server,
  it agrees with `command -v`.

The four present servers also start (`--version` exits 0).

### Configured and present on this host

- `typescript`,
  `javascript`,
  and `tsx`:
  `typescript-language-server`,
  version 6.0.0.
- `rust`:
  `rust-analyzer`,
  1.100.0-nightly (2026-09-21).
- `slint`:
  `slint-lsp`,
  1.18.1.
- `qml`:
  `qmlls`,
  6.11.2.

### Configured but absent on this host

- `kotlin`:
  `kotlin-language-server`.
- `hcl`:
  `terraform-ls`.
- `bash`:
  `bash-language-server`.
- `c` and `cpp`:
  `clangd`.
- `dockerfile`:
  `docker-langserver`,
  `docker-language-server`.
- `json` and `jsonc`:
  `vscode-json-language-server`.
- `toml` and `miseconfig`:
  `taplo`,
  `tombi`.
- `yaml`:
  `yaml-language-server`,
  `ansible-language-server`.
- `github-action`:
  `actions-languageserver`,
  `yaml-language-server`,
  `zizmor`.
- `docker-compose`:
  `docker-compose-langserver`,
  `yaml-language-server`,
  `docker-language-server`.
- `html`:
  `vscode-html-language-server`,
  `superhtml`.
- `css`:
  `vscode-css-language-server`.
- `markdown`:
  `marksman`,
  `markdown-oxide`,
  `rumdl`.

### No server configured

- `sql`,
  `batch`,
  and `xml`.
  These stay highlight,
  read,
  and search only.

Installing absent servers is not part of this work.
By the capability-aware rule in the scope history,
a language without a running server keeps highlighting and reports semantic features as unavailable.

## Application behavior change

The pinned registry recognizes 342 languages,
far more than the 49 that the 27 bundled grammars serve.
Before this change,
a recognized file without a bundled grammar,
such as `.gitignore`,
a `.patch`,
or a `.py`,
failed with this diagnostic:

```text
Cannot highlight /project/main.py as python. The bundled parser or highlighting rules could not be loaded.
Rebuild matching language assets and restart the application.
```

That remedy is wrong for a language that was never selected.

`SyntaxEngine` now reads the published `manifest.json`:

- recognized,
  and its grammar is listed:
  highlighted;
- recognized,
  and its grammar is not listed:
  plain text,
  with a debug log naming the language and grammar;
- listed,
  but the library or rules fail to load:
  the existing visible failure,
  unchanged;
- manifest missing,
  malformed,
  or holding an entry that is not a `.so` name:
  engine initialization fails with a diagnostic naming the file and the `runtime` task.

`SyntaxEngine::language_id` returns the bundled Helix language of a file.
The later language-server work needs that name to choose a server.

## Build, size, and duration

All builds ran through `mise run //package/ide/app:runtime` in the existing bounds:
2 GiB memory,
2 CPUs,
512 processes,
with network for the fetch step.
Every grammar built within them;
none had to be reported as too large.

### Asset size

Measured with `du --summarize --bytes target/debug/runtime`
(the `release` copy is identical):

- total:
  34,607,713 bytes,
  from 5,655,910 for the initial slice;
- `grammars/`:
  33,463,328 bytes in 27 files;
- `queries/`:
  1,087,532 bytes in 1,193 files (the whole pinned tree,
  unchanged in size);
- `licenses/`:
  39,593 bytes in 28 files;
- `Helix-LICENSE` and `manifest.json`:
  the remainder.

Three grammars are 65% of the total:
`sql` (11,098,288),
`kotlin` (5,768,128),
and `cpp` (5,592,040).

### Duration

Wall-clock time of `mise run //package/ide/app:runtime` on this host,
while other sessions were also building:

- cold,
  with no fetched source or built grammar (27 fetched,
  27 built):
  82.1 s and 73.0 s in two runs;
- warm,
  with 27 sources up to date and 27 grammars already built:
  4.7,
  16.6,
  7.3,
  5.3,
  and 8.4 s in five runs.

The warm spread is larger than any difference worth reading into it.
`build`,
`test`,
`test:native`,
`test:open`,
`mcp`,
and `inspect:native` depend on `runtime`,
so each pays the warm cost.
No duration was recorded for the initial slice,
so there is no before value to compare.

## Verification

All commands ran in the worktree on 2026-10-05,
with the code at commit `b81716464`.

- `mise run //package/ide/app:runtime`:
  passed.
  27 grammars fetched at their pinned revisions,
  built,
  and published with notices.
- `mise run //package/ide/app:test`:
  30 test binaries,
  163 passed,
  0 failed,
  0 ignored.
  43 of them are new:
  26 in `tests/syntax_inventory.rs`,
  8 in `tests/syntax_injection.rs`,
  and 9 in `tests/syntax_provisioning.rs`.
- `mise run //package/ide/app:lint`:
  passed.
  It runs Clippy on all targets with warnings denied,
  the Rust documentation and line-budget linter,
  and the Slint check of three markup files.
- `mise run //package/ide/app:test:native`:
  14 run,
  14 passed.

Every bundled language is exercised through `SyntaxEngine::highlight`,
the entry the reload worker calls,
against the published `target/debug/runtime`.
`every_language_on_a_bundled_grammar_compiles_its_highlighting_rules` additionally loads each of the 27 libraries
and compiles the rules of all 49 registry languages that use them.

### What the Rust linter covers

The repository's Rust linter configuration turns the documentation and line-budget rules off for `**/tests/**`.
A control confirmed both halves:
an undocumented function under `src` produces two `builtin(require-rustdoc)` errors,
and the same file under `tests` produces none.
So the linter result covers `src/syntax.rs` and `src/bin/ide-runtime.rs`,
and Clippy covers the test files.

### Tests observed failing

The plain-text test was written first and failed against the unchanged engine with the diagnostic quoted under
[Application behavior change](#application-behavior-change).

`controls.mjs` then ran the four syntax test binaries in a disposable package copy with a disposable target cache.
All 13 phases matched their predicted failures:

- committed code,
  before and after the controls:
  47 passed.
- The initial five-grammar runtime in place of the full one:
  29 failed and 18 passed.
  The failures are every inventory and injection test except the five that need only initial-slice grammars.
- The manifest decision in `SyntaxEngine::recognize` removed:
  the 2 plain-text tests failed.
- The manifest entry check removed:
  its 1 test failed.
- `sql.so` removed:
  3 failed,
  the SQL language test,
  the shipped-library test,
  and the rule-compilation test.
- The Slint notice directory removed:
  the shipped-library-and-notice test failed.
- Each companion library removed in turn (`awk`,
  `comment`,
  `jsdoc`,
  `markdown_inline`,
  `regex`,
  `rust-format-args`):
  3 failed each time,
  that grammar's injection test plus the two asset tests.

The first control run found one weak test.
The format-arguments test asserted a placeholder name that Rust paints at its binding anyway,
so it passed without the grammar.
It now asserts the format type,
which only that grammar paints,
and the repeated control fails it as predicted.

### Not exercised

- No native GUI session opened a file of a newly bundled language.
  The worker-to-engine path is language independent
  and is covered for Rust by `tests/file_open.rs` and `tests/file_reload.rs`.
- Samples are short synthetic sources,
  not repository files.
- No language server was started beyond `--version`.
- The REUSE notice check lives inline in `mise.toml`.
  It was exercised by a scratch script that evaluates the function text taken from the task,
  not by a committed test.

## Evidence

Scratch directory `~/temp/agent/ide-runtime-languages-TH9vMy` (private,
not committed):

- `worktree.json`:
  raw `tokei` output.
- `summarize.mjs` and `worktree-summary.json`:
  standalone and embedded-only languages.
- `unreported.mjs`:
  tracked files `tokei` does not classify.
- `helix-facts.mjs` and `static-mapping.json`:
  the static mapping.
- `zz_inventory_probe.rs`,
  `probe-input.mjs`,
  and `loader-mapping.json`:
  the mapping through the pinned loader.
- `closure.mjs` and `closure.json`:
  grammar sources,
  inherited queries,
  and injection targets.
- `triggers.mjs` and `triggers.json`:
  injection trigger counts with controls.
- `licenses.mjs` and `licenses.json`:
  notices and declared licenses per fetched source.
- `reuse-guard.mjs`:
  the REUSE notice check against fixtures.
- `servers.mjs` and `servers.json`:
  server configuration and host presence.
- `controls.mjs` and `controls-2.out`:
  the guard and asset removal controls.
  Per-phase logs and `results.json` are in `~/temp/agent/ide-runtime-languages-control-x2sTjR`;
  the first run,
  which exposed the weak test,
  is in `~/temp/agent/ide-runtime-languages-control-iUon9i`.
- `timings.jsonl` and the `*.log` files:
  task durations and captured task output.
- `counts.mjs`:
  per-binary test totals from a captured log.
- `render.mjs`:
  the Markdown render and style check used on this document.
- `initial-slice-runtime/`:
  the five-grammar runtime kept as a control.

## Open questions

### Files `tokei` does not classify

The narrower reading was implemented:
only `tokei`-classified languages are bundled.
The broader reading would add eight grammars for files that are tracked,
recognized by Helix,
and currently plain text.

- Narrow (implemented).
  Pro:
  matches the confirmed wording exactly.
  Con:
  49 patches and 10 ignore files,
  among others,
  stay unhighlighted.
- Broad:
  add `diff`,
  `git-ignore` (grammar `gitignore`),
  `ini`,
  `properties`,
  `caddyfile`,
  `git-attributes` (grammar `gitattributes`),
  `ghostty`,
  and `git-config`.
  Pro:
  covers what the repository actually contains.
  Con:
  extends the rule beyond its measuring tool,
  and each grammar needs the same license and build check.
- Middle:
  add only `diff` and `git-ignore`,
  the two with ten or more files.

Ranking:
broad > middle,
because the user's stated intent was the languages actually used and every one of these is measured in tracked files;
middle > narrow,
because patches are a frequent reading target in `doc/troubleshooting`.
This is a scope decision,
so it was not taken here.

### Retained `tsx` and included `awk`

Both are recorded with their sizes under [Companion grammars](#companion-grammars) so either can be dropped.

### Markdown palette

Markdown's own captures are almost all `markup.*`
(headings,
emphasis,
links,
raw text),
which the application palette does not map.
Markdown files therefore show color mainly in fenced blocks of bundled languages,
escapes,
and bracket punctuation.
Extending the palette belongs to the renderer,
not to runtime provisioning.

### Shell highlighting of non-shell `mise.toml` tasks

Helix's `miseconfig` rules paint every task `run` string as shell unless it starts with a shebang.
This repository's tasks often set `shell = "node ..."` and hold JavaScript,
which is then painted as shell.
That is pinned Helix behavior;
correcting it would be a custom integration.

### Size of `sql`

`sql.so` is 11,098,288 bytes for 7 measured files.
It is selected because the rule is presence,
not a size or popularity threshold.

[injected-rule]: #injected-grammars-are-optional-so-they-follow-the-same-presence-rule
[fsf-licenses]: https://www.gnu.org/licenses/license-list.html
