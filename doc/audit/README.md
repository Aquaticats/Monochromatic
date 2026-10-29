# Dependency Security Report

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/shim/ungap-structured-clone` is now `package/pnpm-override/shim-ungap-structured-clone`;
> `package/dev-script/watch-restart` is now `package/watch-restart/cli`.

## Socket CLI scan (2026-04-05)

Scanned with Socket CLI v1.1.78 via `socket scan create --report`.
Scan ID:
 `5a68fe36-d693-4717-bf65-7784d11a7b20`.
Total packages:
 629 (626 libraries,
 3 frameworks).

### Critical findings

- **Malware:
  ** None detected
- **Known CVEs:
  ** None detected (CVE/CVSS/EPSS columns empty for all 41 alerts)
- **Typosquatting:
  ** None detected
- **Abandoned:
  ** None detected
- **Manifest confusion:
  ** None detected
- **Install scripts:
  ** None detected
- **Unlicensed:
  ** None detected
- **Obfuscated:
  ** None detected
- **High-entropy strings:
  ** None detected
- **Unstable ownership:
  ** None detected

### Dependency chains for flagged packages

**`picomatch@2.3.2`:
 transitive via stylelint (dev-only):
**

```text
stylelint -> micromatch -> picomatch@2.3.2
stylelint -> globby -> fast-glob -> micromatch
```

Socket flags this as `potentialVulnerability` (medium behavioral risk) for ReDoS
via user-supplied glob patterns with unconstrained regex complexity.
This is a behavioral risk indicator,
 not a CVE.
Two historical CVEs (CVE-2026-33671,
 CVE-2026-33672) are **fixed in 2.3.2**.

Stylelint maintainers are actively discussing migration to `tinyglobby` or `fs.glob` (Node 22+)
but have blocked it on a major release to avoid breaking the `globbyOptions` API.
Two community PRs (stylelint/stylelint#8988,
 stylelint/stylelint#9118) were closed without merge.
The discussion continues in stylelint/stylelint#8051 and stylelint/stylelint#8929.

`picomatch@4.0.4` (used by tsdown,
 tinyglobby,
 rolldown,
 fdir) is a separate major version
and was never affected by these CVEs.

**`openai@6.33.0`:
 direct dep of `package/dev-script/inference-canary`:
**
Socket flags this as `potentialVulnerability` (medium behavioral risk).
The `bin/cli` file defines a `migrate` subcommand that calls `spawnSync` to download and execute
an unsigned tarball from GitHub (`stainless-api/migrate-ts`) via `npx -y`.
No checksum,
 signature,
 or integrity verification.
Risk:
 a compromised GitHub release or MITM could achieve arbitrary code execution.
Mitigation:
 we never invoke `openai migrate`;
 the risk is limited to the binary existing
in `node_modules/.bin`.
 Dev-only dependency.

### Informational findings

#### Potential vulnerabilities (2 packages): behavioral risk, not CVEs

- `picomatch@2.3.2`:
   ReDoS via unconstrained regex from user-supplied globs;
   transitive via stylelint;
   dev-only
- `openai@6.33.0`:
   unsigned tarball download in `migrate` subcommand;
   direct dep;
   dev-only

#### Unpopular packages (4)

- `lezer-toml@1.0.0`:
   direct dep of `package/desktop-daemon/editord`;
   niche Lezer grammar for TOML
- `@mitata/counters@0.0.8`:
   direct dep of `package/test-fixture/file-enforcer-perf`;
   companion to mitata benchmarking
- `@tursodatabase/database-darwin-arm64@0.5.3`:
   transitive platform binary for turso
- `@tursodatabase/database-win32-x64-msvc@0.5.3`:
   transitive platform binary for turso

#### Unmaintained (>3 years without updates, 33 packages)

- argparse,
   json-schema-traverse,
   fast-deep-equal,
   inherits,
   resolve-from,
   is-extglob,
   shebang-command,
   normalize-path,
   safe-buffer,
   to-regex-range,
   string_decoder,
   util-deprecate,
   extend,
   kind-of,
   require-from-string,
   merge2,
   run-parallel,
   queue-microtask,
   is-number,
   text-hex,
   lodash.
  truncate,
   astral-regex,
   cssesc,
   is-plain-object,
   global-modules,
   globjoin,
   svg-tags,
   tree-kill,
   enabled,
   fn.
  name,
   kuler,
   one-time,
   tiny-inflate

All 33 are transitive dependencies.
Count reduced from 54 in the previous scan after removing remark-cli,
 gray-matter,
 and @npmcli/* transitive trees.

**Note on behavioral categories:
**
The previous vlt-based scan reported behavioral indicators (dynamic code execution,
 shell access,
network access,
 new collaborator publishing,
 trivial packages,
 copyleft licenses).
The Socket CLI alert export only includes policy-threshold alerts (maintenance,
 quality,
 supplyChainRisk).
Behavioral indicators are visible on the Socket dashboard but not in the CSV export.

---

## Dependency Source Code Audit

## Excluded from Auditing

Dependencies excluded from full source code audit,
 with rationale.

### Transitive / Unavoidable

- **postcss**:
   Used by stylelint and CSS build pipeline
- **tinyglobby**:
   Used by tsdown and other build tooling

### Trusted / Pre-Vetted

- **dprint**:
   Trusted native formatter;
   dev-only
- **neovim**:
   Trusted MCP integration;
   dev-only
- **@anthropic-ai/sdk**:
   Trusted first-party AI vendor SDK
- **openai**:
   Trusted first-party AI vendor SDK
- **zod**:
   Impossible to avoid.

### Planned for Removal

- **astro**:
   Will be replaced with a simpler SSG
- **@astrojs/mdx**:
   Will be removed with astro
- **`@logtape/*`**:
   To be replaced by our custom logger

## Audited

### feedsmith

- **Version:
  ** >=3.0.0-next.
  2
- **Date:
  ** 2026-03-01
- **Verdict:
  ** Acceptable
- **Notes:
  ** Single-author (macieklamberski),
   MIT,
   3 runtime deps (fast-xml-parser,
   entities,
   strnum).
  No eval/network/fs/DOM in source.
  XML entity processing disabled for parsing (`processEntities: false`),
   mitigating XXE/billion-laughs.
  No ReDoS-vulnerable regex.
  No `__proto__`/`constructor` key filtering in `traverseAndNormalize`,
   but downstream parsers build new objects with hardcoded keys so injected keys are discarded.
  Unbounded recursion in namespace normalization is a theoretical DoS vector on deeply nested XML.
   Mitigate with a response size limit on fetch.
  Generator path uses `processEntities: true` but we only use parsers.
  `parseDate` is a no-op stub (passes strings through);
   our code validates dates via Zod.
  Prerelease semver range means trusting a sole maintainer's `next` branch;
   pin if stability matters.
  Overall:
   clean,
   well-tested (3,400+ tests),
   well-typed codebase with no significant security issues for parse-only use on untrusted feeds.

### stylelint

- **Version:
  ** 17.4.0
- **Date:
  ** 2026-03-03
- **Verdict:
  ** Acceptable
- **Notes:
  ** The codebase is a straightforward CSS linter with:
  - No network activity
  - No shell execution
  - No dynamic code evaluation
  - Proper pinned CI dependencies
  - Benign install scripts
  - Well-scoped file system access (only writes user-specified output files)

### elysia

- **Version:
  ** 1.4.7
- **Date:
  ** 2026-03-03
- **Verdict:
  ** Acceptable
- **Notes:
  ** Elysia's source code is straightforward framework code with no "funny business.
  " It has solid security hygiene (prototype
  pollution guards,
   timing-safe crypto,
   strict mode),
   legitimate use of dynamic code generation for performance,
   and zero data
  collection or exfiltration mechanisms.

### tsdown

- **Version:
  ** 0.18.1
- **Date:
  ** 2026-03-01
- **Verdict:
  ** Acceptable
- **Notes:
  ** Rolldown wrapper that does what it says on the tin.

### lfi

- **Notes:
  ** The lfi package (v4.1.2) should be approved for use.
   A thorough audit of both the GitHub source and the published npm tarball
  reveals no malicious code,
   no install hooks,
   no network calls,
   no filesystem access,
   no environment variable reads,
   and zero
  runtime dependencies.
   The only two flagged patterns (an eval('') for CSP feature detection and a new Function() for function
  composition optimization) are well-known,
   legitimate techniques with safe fallbacks.
   The published tarball matches the expected
  build output from the repository.
   The library is small (~21KB),
   purely functional,
   side-effect-free,
   and does exactly what it
  claims:
   lazy iteration over sync,
   async,
   and concurrent iterables.

### happy-rusty AND happy-opfs

- **Notes:
  ** Both happy-rusty and happy-opfs are clean,
   well-engineered TypeScript libraries with no security red
  flags.
   happy-rusty has zero runtime dependencies and provides Rust-style error handling primitives with strict typing and
  immutable objects.
   happy-opfs builds on this foundation to offer a browser-compatible file system API over OPFS with minimal,
  reputable dependencies.
   Both libraries demonstrate above-average code quality:
   strict TypeScript configurations,
   comprehensive
  test suites,
   thorough JSDoc documentation,
   consistent input validation,
   and no use of eval,
   dynamic code execution,
   install
  scripts,
   or obfuscated code.
   No data exfiltration,
   telemetry,
   or hidden functionality was detected.
   The only minor concern is the
  single-author bus factor across the @happy-ts ecosystem,
   which is common for focused open-source utilities and does not
  constitute a blocking risk.

### smol-toml

- **Notes:
  ** well-engineered,
   zero-runtime-dependency TOML parser/serializer with ~1,300 lines of TypeScript source.
   The audit
  found no suspicious code:
   no network calls,
   telemetry,
   eval,
   environment variable access,
   or postinstall scripts.
   Security is
  handled proactively:
   prototype pollution is mitigated via Object.
  defineProperty,
   all regexes are anchored and ReDoS-safe,
   and
  recursive depth is capped at 1000 to prevent stack overflows.
   CI/CD follows best practices with SHA-pinned GitHub Actions and npm
  provenance publishing.
   The test suite covers parsing,
   serialization,
   error reporting,
   and DoS protection.
   One minor concern:
  skipVoid uses recursion per comment line,
   which could theoretically overflow on pathologically large inputs,
   but this is
  low-severity.
   With ~7M weekly npm downloads and BSD-3-Clause licensing,
   this library is production-ready and safe to adopt.

### superjson

- **Notes:
  ** well-engineered,
   minimal TypeScript library for serializing rich JavaScript types (Date,
  Map,
   Set,
   BigInt,
   etc.) over JSON.
   The codebase is lean (~1,530 lines),
   strictly typed,
   and carries only one runtime dependency.
  It has no install hooks,
   no telemetry,
   no network calls,
   and no obfuscated code;
   the supply chain risk is negligible.
   Security
  is actively addressed:
   prototype pollution is explicitly blocked and tested from both serialize and deserialize vectors.
   Test
  coverage is strong at a roughly 1:1 test-to-code ratio with edge-case and performance regression tests.
   The library is actively
  maintained with recent commits and regular releases.
   The only minor concern is a theoretical ReDoS vector via RegExp
  deserialization,
   which requires attacker control of the trusted payload,
   low risk in typical use.
   This dependency is safe to
  adopt.

### happy-dom

- **Notes:
  ** Uses of eval() and child_process are inherent to
  its purpose as a browser DOM simulator and mirror patterns found in the industry-standard jsdom.
   The project is 100% TypeScript
  with ~300 test files,
   ESLint/Prettier enforcement,
   and CI that explicitly disables install scripts.

## Audit Queue

Remaining dependencies to audit,
 ordered by priority.

### Tier 1: High Risk (server frameworks, untrusted input processing)

### Tier 2: Medium Risk (build pipeline, data transformation)

- [ ] rehype-* / remark-* / unified (content pipeline)

### Tier 3: Lower Risk (utilities, dev-only)

- [ ] type-fest
- [ ] ts-pattern
- [ ] decircular
- [ ] safe-stringify
- [ ] dot-prop
- [ ] nano-spawn
- [ ] execa
- [ ] @optique/core
- [ ] @optique/run
- [ ] preact
- [ ] preact-render-to-string
- ~~watcher~~:
   rejected 2026-05 for `package/dev-script/watch-restart/`;
   chokidar adopted instead (atomic-save + `awaitWriteFinish` stability window,
   larger production track record,
   active maintenance whereas watcher's last commit was 2024-07)
- [ ] @csstools/css-tokenizer
- [ ] string-dedent
- [ ] serialize-error
- [x] chokidar;
       evaluated 2026-05;
       adopted by `package/dev-script/watch-restart/` (one transitive dep `readdirp`,
       `atomic: true` for rename+create,
       `awaitWriteFinish` for chunked writes,
       cross-platform recursion via FSEvents/inotify/ReadDirectoryChangesW)
- [x] ignore (kaelzhang/node-ignore):
       evaluated 2026-05;
       adopted by `package/dev-script/watch-restart/src/filters/gitignore.ts`
       for `--gitignore` / `--ignore-file` parsing (zero runtime deps,
       mirrors git's own semantics including negation `!`,
       anchored leading `/`,
       directory-only trailing `/`,
       and `**`;
       cheaper than reimplementing the spec)
- [ ] the-new-css-reset
- [ ] TODS
- [ ] opentype.
      js
- [ ] browserslist
- [ ] minimatch
- [ ] glob
- [ ] @total-typescript/ts-reset
- [x] @ungap/structured-clone;
       evaluated 2026-05-17;
       replaced via workspace shim `package/shim/ungap-structured-clone/`
       that re-exports `globalThis.structuredClone` (native on Node 17+,
       Bun,
       and every browser at the Firefox ESR 140 baseline).
       Reached transitively from `package/ssg/aquati.cat/` through `mdast-util-to-hast`
       (4 default-export call sites in `lib/state.js`,
       `lib/footer.js`) and `rehype-autolink-headings` (1 call site in `lib/index.js`);
       both use `structuredClone(value)` with one argument,
       so re-exporting native is API-equivalent.
       Substitution wired in `pnpm-workspace.yaml` `overrides:`,
       clearing the upstream `CWE-502 - Update to 1.3.1 or higher` deprecation.
       Catalog pin and root `devDependencies` entry removed;
       no workspace package imported the upstream directly.
       See `package/shim/ungap-structured-clone/README.md`.
- ~~remark-lint-*~~:
   removed;
   replaced by markdownlint-cli2 + dprint-plugin-markdown
- [ ] istanbul-lib-report (dev-only)
- ~~@shikijs/transformers~~:
   removed;
   replaced by CSS Custom Highlight API with Lezer parsers
- [ ] remark-github-blockquote-alert
- [ ] remark-sectionize
- [ ] rehype-slug-custom-id
- [ ] rehype-autolink-headings
- [ ] rehype-preset-minify
- [ ] rehype-parse
- [ ] rehype-stringify
- [ ] to-vfile
- [ ] vue + vue-tsc (dev-only,
       type-checking)
