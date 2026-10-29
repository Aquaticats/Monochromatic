import { createHash, } from 'node:crypto';
import {
  chmod,
  glob,
  lstat,
  mkdir,
  readFile,
  rm,
  unlink,
} from 'node:fs/promises';
import { join, } from 'node:path';

import nanoSpawn from 'nano-spawn';

import {
  addWatchedPaths,
  cat,
  getTomlProperty,
  l,
  manageCargoManifests,
  manageLsp4ijServerSettings,
  manageMiseManifests,
  manageRustCatalog,
  overwrite,
  overwriteEach,
  overwriteIfNotExists,
  tagged,
  type CanonicalTomlValue,
  type CargoBlockInsertion,
  type CargoEnforcement,
  type CargoManifestPlan,
  type GlobResults,
} from '@monochromatic-dev/file-enforcer-core/ts';

import { isTypeScriptSourcePath, } from '@monochromatic-dev/config-pnpr/ts';
import type browserslist from 'browserslist';

/**
 Root-level context summary path that must remain absent. The repo relies on
 source reads plus doc/agent/domain.md instead of cached context files.
 
 @example
 ```ts
 console.log(FORBIDDEN_ROOT_CONTEXT_PATH);
 ```
 */
const FORBIDDEN_ROOT_CONTEXT_PATH = './CONTEXT.md';

/**
 Checked-in Browserslist policy file read directly by file-enforcer.
 
 @example
 ```ts
 console.log(BROWSERSLIST_CONFIG_PATH);
 ```
 */
const BROWSERSLIST_CONFIG_PATH = './.browserslistrc';

/**
 Deterministic Browserslist environment section used when the config declares
 environment-specific blocks. Keeping this fixed prevents ambient shell env
 from changing generated files.
 
 @example
 ```ts
 console.log(BROWSERSLIST_CONFIG_ENVIRONMENT);
 ```
 */
const BROWSERSLIST_CONFIG_ENVIRONMENT = 'production';

/**
 Explicit empty custom-usage statistics passed to Browserslist so it never
 searches ancestor directories for `browserslist-stats.json`.
 
 @example
 ```ts
 console.log(Object.keys(EMPTY_BROWSERSLIST_STATS));
 ```
 */
const EMPTY_BROWSERSLIST_STATS: browserslist.Stats = {};

/**
 Source license texts keyed by SPDX license identifier, copied verbatim into
 package-local `LICENSES/` directories so package artifacts carry the texts
 without maintaining hand-edited duplicates.
 
 @example
 ```ts
 console.log(PACKAGE_LICENSE_TEXT_SOURCES['LGPL-3.0-or-later']);
 ```
 */
const PACKAGE_LICENSE_TEXT_SOURCES = {
  'CC-BY-SA-4.0': './LICENSES/CC-BY-SA-4.0.txt',
  'GPL-3.0-or-later': './LICENSES/GPL-3.0-or-later.txt',
  'LGPL-3.0-or-later': './LICENSES/LGPL-3.0-or-later.txt',
} as const;

/**
 License identifiers whose canonical texts file-enforcer knows how to copy.
 
 @example
 ```ts
 const id: PackageLicenseTextId = 'GPL-3.0-or-later';
 ```
 */
type PackageLicenseTextId = keyof typeof PACKAGE_LICENSE_TEXT_SOURCES;

/**
 Package manifest globs whose license expressions drive package-local license texts.
 
 @example
 ```ts
 console.log(PACKAGE_LICENSE_MANIFEST_GLOBS.length);
 ```
 */
const PACKAGE_LICENSE_MANIFEST_GLOBS = [
  './package/*/*/package.json',
  './package/*/*/Cargo.toml',
] as const;

/**
 License text identifiers needed when a package uses LGPLv3.
 LGPLv3 incorporates GPLv3 by reference, so package artifacts need both texts.
 
 @example
 ```ts
 console.log(LGPL_3_OR_LATER_TEXT_IDS);
 ```
 */
const LGPL_3_OR_LATER_TEXT_IDS = [
  'GPL-3.0-or-later',
  'LGPL-3.0-or-later',
] as const satisfies readonly PackageLicenseTextId[];

/**
 Node filesystem error code for an absent path.
 
 @example
 ```ts
 console.log(ABSENT_PATH_ERROR_CODE);
 ```
 */
const ABSENT_PATH_ERROR_CODE = 'ENOENT';

/**
 Owner-only Unix mode for sensitive rule source and generated runtime text.
 
 @example
 ```ts
 console.log(SENSITIVE_RULE_FILE_MODE);
 ```
 */
const SENSITIVE_RULE_FILE_MODE = 0o600;

/**
 Gitignored sensitive appendix edited by repository user.
 
 @example
 ```ts
 console.log(FORBIDDEN_STRINGS_LOCAL_APPENDIX_PATH);
 ```
 */
const FORBIDDEN_STRINGS_LOCAL_APPENDIX_PATH = './forbidden-strings.append.local.txt';

/**
 Authoritative file-enforcer runtime rules output consumed by scanner.
 
 @example
 ```ts
 console.log(FORBIDDEN_STRINGS_RUNTIME_RULES_PATH);
 ```
 */
const FORBIDDEN_STRINGS_RUNTIME_RULES_PATH = './.cache/forbidden-strings.rules.txt';

/**
 Repository-built release scanner used by cli-git and eager cache compilation.
 
 Windows release binaries carry executable suffix while other targets do not.
 
 @example
 ```ts
 console.log(FORBIDDEN_STRINGS_SCANNER_PATH);
 ```
 */
const FORBIDDEN_STRINGS_SCANNER_PATH = process.platform === 'win32'
  ? './package/forbidden-strings/cli/target/release/forbidden-strings.exe'
  : './package/forbidden-strings/cli/target/release/forbidden-strings';

/**
 Canonical repo-relative prefix owned by the agent-skill mirror generator.
 
 @example
 ```ts
 console.log(SKILL_MIRROR_CANONICAL_PREFIX);
 ```
 */
const SKILL_MIRROR_CANONICAL_PREFIX = '.agents/skills/';

/**
 Legacy skill roots receiving byte-identical canonical Markdown files.
 
 @example
 ```ts
 console.log(SKILL_MIRROR_DESTINATION_ROOTS);
 ```
 */
const SKILL_MIRROR_DESTINATION_ROOTS = [
  './.claude/skills',
  './.factory/skills',
] as const;

/**
 Ownership manifest filename stored in each legacy skill root.
 
 @example
 ```ts
 console.log(SKILL_MIRROR_MANIFEST_FILENAME);
 ```
 */
const SKILL_MIRROR_MANIFEST_FILENAME = '.agents-mirror-manifest.json';

/**
 Canonical skill path to SHA-256 mapping owned by one mirror destination.
 
 @example
 ```ts
 const manifest: SkillMirrorManifest = {
   '.agents/skills/example/SKILL.md': 'hash',
 };
 ```
 */
type SkillMirrorManifest = Readonly<Record<string, string>>;

/**
 Error object shape used by Node filesystem APIs when a system code is present.
 
 @example
 ```ts
 const codedError = error as NodeErrorCodeCarrier;
 ```
 */
type NodeErrorCodeCarrier = Error & {
  readonly code?: unknown;
};

/**
 Signals that root CONTEXT.md exists even though cached context files are forbidden.
 
 @example
 ```ts
 throw new ForbiddenRootContextFileError({ filePath: './CONTEXT.md' });
 ```
 */
class ForbiddenRootContextFileError extends Error {
  /**
   Builds a failure with guidance toward source-driven context.
   
   @param filePath - forbidden root context path that was present.
   
   @example
   ```ts
   new ForbiddenRootContextFileError({ filePath: './CONTEXT.md' });
   ```
   */
  public constructor({ filePath, }: { readonly filePath: string; }) {
    super([
      `${filePath} is forbidden.`,
      'Do not create cached context files;',
      'read source code directly and use doc/agent/domain.md for this repo policy.',
    ].join(' ',),);
    this.name = ForbiddenRootContextFileError.name;
  }
}

/**
 Returns whether an unknown filesystem error carries the expected Node error code,
 narrowing through {@link NodeErrorCodeCarrier}.
 
 @param error - unknown error from a filesystem operation.
 
 @param code - Node error code that should be treated as expected.
 
 @returns Whether error carries code.
 
 @example
 ```ts
 const absent = errorHasCode({ error, code: ABSENT_PATH_ERROR_CODE });
 ```
 */
function errorHasCode(
  {
    error,
    code,
  }: {
    readonly error: unknown;
    readonly code: string;
  },
): boolean {
  return (Error.isError(error,))
    && ('code' in error)
    && ((error as NodeErrorCodeCarrier).code === code);
}

/**
 Converts an SPDX license expression to license texts that must accompany the
 package artifact. The mapping is intentionally conservative: LGPLv3 yields
 both LGPLv3 and GPLv3 because LGPLv3 is an additional-permissions layer over GPLv3.
 
 @param expression - SPDX license expression from a package manifest.
 
 @returns Known license text identifiers required by expression.
 
 @example
 ```ts
 licenseTextIdsForExpression({ expression: 'LGPL-3.0-or-later AND CC-BY-SA-4.0' });
 ```
 */
function licenseTextIdsForExpression(
  { expression, }: { readonly expression: string; },
): readonly PackageLicenseTextId[] {
  /**
   Ordered de-duplicated text identifiers produced from the expression.
   */
  const textIds = new Set<PackageLicenseTextId>();

  if (expression.includes('LGPL-3.0-or-later',))
    for (const textId of LGPL_3_OR_LATER_TEXT_IDS)
      textIds.add(textId,);
  else if (expression.includes('GPL-3.0-or-later',))
    textIds.add('GPL-3.0-or-later',);

  if (expression.includes('CC-BY-SA-4.0',))
    textIds.add('CC-BY-SA-4.0',);

  return [...textIds,];
}

/**
 Reads license expression from a package.json manifest.
 
 @param manifestPath - package.json path.
 
 @returns SPDX expression when manifest carries a string `license` field.
 
 @example
 ```ts
 await packageJsonLicenseExpression({ manifestPath: './package/test-harness/core/package.json' });
 ```
 */
async function packageJsonLicenseExpression(
  { manifestPath, }: { readonly manifestPath: string; },
): Promise<string | undefined> {
  /**
   Parsed package.json object; only the license field matters here.
   */
  const packageJson = JSON.parse(await cat([manifestPath,],),) as {
    readonly license?: unknown;
  };

  return ((typeof packageJson.license) === 'string')
    ? packageJson.license
    : undefined;
}

/**
 Reads license expression from a Cargo manifest.
 
 @param manifestPath - Cargo.toml path.
 
 @returns SPDX expression when manifest carries a string package.license field.
 
 @example
 ```ts
 await cargoTomlLicenseExpression({ manifestPath: './package/forbidden-strings/cli/Cargo.toml' });
 ```
 */
async function cargoTomlLicenseExpression(
  { manifestPath, }: { readonly manifestPath: string; },
): Promise<string | undefined> {
  /**
   License value from Cargo's package metadata. Workspace-inherited or missing
   values are ignored unless they resolve to a direct string in this manifest.
   */
  const license = getTomlProperty({
    path: [
      'package',
      'license',
    ],
    content: await cat([manifestPath,],),
  },) as string | undefined;

  return ((typeof license) === 'string')
    ? license
    : undefined;
}

/**
 Returns package directory path for a two-level package manifest.
 
 @param manifestPath - package.json or Cargo.toml path.
 
 @returns Package root path containing manifest.
 
 @example
 ```ts
 packageDirFromManifest({ manifestPath: './package/forbidden-strings/cli/Cargo.toml' });
 ```
 */
function packageDirFromManifest(
  { manifestPath, }: { readonly manifestPath: string; },
): string {
  return manifestPath.slice(
    0,
    manifestPath.lastIndexOf('/',),
  );
}

/**
 Removes file path when present, ignoring absent-path races.
 
 @param filePath - generated file path that may need deletion.
 
 @example
 ```ts
 await unlinkIfExists({ filePath: './package/example/name/LICENSES/GPL-3.0-or-later.txt' });
 ```
 */
async function unlinkIfExists(
  { filePath, }: { readonly filePath: string; },
): Promise<void> {
  try {
    await unlink(filePath,);
  }
  catch (error) {
    if (errorHasCode({
      error,
      code: ABSENT_PATH_ERROR_CODE,
    },))
      return;

    throw error;
  }
}

/**
 Adds one license expression to the per-package accumulator.
 
 @param packageLicenseExpressions - Accumulator keyed by package directory.
 
 @param packageDir - Package directory receiving expression.
 
 @param expression - SPDX expression from manifest.
 
 @example
 ```ts
 addPackageLicenseExpression({ packageLicenseExpressions: new Map(), packageDir: './package/x/y', expression: 'LGPL-3.0-or-later' });
 ```
 */
function addPackageLicenseExpression(
  {
    packageLicenseExpressions,
    packageDir,
    expression,
  }: {
    readonly packageLicenseExpressions: Map<string, Set<string>>;
    readonly packageDir: string;
    readonly expression: string;
  },
): void {
  /**
   Existing expression set for this package, created lazily.
   */
  const expressions = packageLicenseExpressions.get(packageDir,) ?? new Set<string>();
  expressions.add(expression,);
  packageLicenseExpressions.set(
    packageDir,
    expressions,
  );
}

/**
 Reads every two-level package manifest and records package-local license text needs.
 
 @returns Map from package directory to known license text identifiers.
 
 @example
 ```ts
 await collectPackageLicenseTextIds();
 ```
 */
async function collectPackageLicenseTextIds(): Promise<Map<string, Set<PackageLicenseTextId>>> {
  /**
   SPDX expressions grouped by package directory before mapping to text files.
   */
  const packageLicenseExpressions = new Map<string, Set<string>>();

  for (const manifestGlob of PACKAGE_LICENSE_MANIFEST_GLOBS) {
    for await (const manifestPath of glob(manifestGlob,)) {
      /**
       SPDX expression read by the parser matching the manifest type.
       */
      const expression = manifestPath.endsWith('/package.json',)
        ? await packageJsonLicenseExpression({ manifestPath, },)
        : await cargoTomlLicenseExpression({ manifestPath, },);

      if (expression === undefined)
        continue;

      addPackageLicenseExpression({
        packageLicenseExpressions,
        packageDir: packageDirFromManifest({ manifestPath, },),
        expression,
      },);
    }
  }

  return new Map(Array.from(
    packageLicenseExpressions,
    function toPackageTextIds([packageDir, expressions,]): readonly [
      string,
      Set<PackageLicenseTextId>
    ] {
      /**
       Known license text identifiers required by all expressions in this package.
       */
      const textIds = new Set<PackageLicenseTextId>();
      for (const expression of expressions)
        for (const textId of licenseTextIdsForExpression({ expression, },))
          textIds.add(textId,);

      return [
        packageDir,
        textIds,
      ];
    },
  ),);
}

/**
 Generates package-local license texts under each package's `LICENSES/` directory from
 canonical root `LICENSES/` sources, based on each package manifest's license
 expression.
 
 @example
 ```ts
 await generatePackageLicenseTexts();
 ```
 */
async function generatePackageLicenseTexts(): Promise<void> {
  /**
   Required license text identifiers keyed by package directory.
   */
  const packageLicenseTextIds = await collectPackageLicenseTextIds();

  /**
   Every generated license text identifier this config owns.
   */
  const knownLicenseTextIds = Object.keys(PACKAGE_LICENSE_TEXT_SOURCES,) as readonly PackageLicenseTextId[];

  await Promise.all(Array.from(
    packageLicenseTextIds,
    async function syncPackageLicenseTexts([packageDir, textIds,]): Promise<void> {
      await Promise.all(knownLicenseTextIds
        .filter(function isStaleTextId(textId,): boolean {
          return !textIds.has(textId,);
        },)
        .map(async function removeStaleLicenseText(textId,): Promise<void> {
          await unlinkIfExists({ filePath: `${packageDir}/LICENSES/${textId}.txt`, },);
        },),);

      await Promise.all([...textIds,].map(
        async function writeLicenseText(textId,): Promise<void> {
          await overwrite({
            dest: `${packageDir}/LICENSES/${textId}.txt`,
            content: await cat([PACKAGE_LICENSE_TEXT_SOURCES[textId],],),
          },);
        },
      ),);
    },
  ),);
}

/**
 Registers root CONTEXT.md with watch mode and rejects it before generated writes,
 using {@link errorHasCode} to tell an absent path from a real stat failure and
 throwing {@link ForbiddenRootContextFileError} when the file is present.
 
 @example
 ```ts
 await assertForbiddenRootContextAbsent();
 ```
 */
async function assertForbiddenRootContextAbsent(): Promise<void> {
  addWatchedPaths([FORBIDDEN_ROOT_CONTEXT_PATH,],);
  try {
    await lstat(FORBIDDEN_ROOT_CONTEXT_PATH,);
  }
  catch (statError: unknown) {
    if (errorHasCode({
      error: statError,
      code: ABSENT_PATH_ERROR_CODE,
    },))
      return;

    throw statError;
  }

  throw new ForbiddenRootContextFileError({
    filePath: FORBIDDEN_ROOT_CONTEXT_PATH,
  },);
}

/**
 Resolver function exported by the `browserslist` package.
 
 @example
 ```ts
 const resolveBrowserslist = await importBrowserslist();
 ```
 */
type BrowserslistResolver = typeof browserslist;

/**
 Imports Browserslist at runtime so generators use the installed package data.
 
 @returns {@link BrowserslistResolver} from dynamic package import.
 
 @example
 ```ts
 const resolveBrowserslist = await importBrowserslist();
 ```
 */
async function importBrowserslist(): Promise<BrowserslistResolver> {
  /**
   CommonJS package namespace exposed through ESM dynamic import.
   */
  const browserslistModule = await import('browserslist') as {
    readonly default: BrowserslistResolver;
  };

  return browserslistModule.default;
}

/**
 Selects query lines from parsed Browserslist config without letting
 Browserslist search the filesystem for config files.
 
 @param config - parsed checked-in Browserslist config.
 
 @returns Production section when present, otherwise default query lines.
 
 @example
 ```ts
 const queries = selectBrowserslistQueries({
   config: { defaults: ['last 1 Chrome version'] },
 });
 ```
 */
function selectBrowserslistQueries(
  { config, }: { readonly config: Readonly<Record<string, readonly string[] | undefined>>; },
): readonly string[] {
  return config[BROWSERSLIST_CONFIG_ENVIRONMENT]
    ?? config.defaults
    ?? [];
}

/**
 Hand-authored mise manifests whose task headers file-enforcer normalizes.

 Each entry is a bounded glob: a manifest sits directly in its package
 directory, and a recursive `**` walk would descend every installed
 `node_modules` tree. The fixture entry is a case directory one level deeper
 than a package.

 @example
 ```ts
 console.log(MISE_MANIFEST_GLOBS.length);
 ```
 */
const MISE_MANIFEST_GLOBS = [
  './package/*/*/mise.toml',
  './package-paused/*/*/mise.toml',
  './package-deprecated/*/*/mise.toml',
  './package/test-fixture/oxlint-test-import/case/*/mise.toml',
] as const;

/**
 Generates mise.toml from mise.no-env.toml with a dynamic [env] section
 containing _.path entries for all workspace package bin directories.
 */
async function generateMiseToml(): Promise<void> {
  /**
   Dynamic [env] block appended after the static mise.no-env.toml content; wires PATH to every workspace bin dir.
   */
  const envSection = `[env]
# Secrets live in the age-encrypted .env.local.json (sops backend), gitignored
# via the *.local.* pattern and decrypted at runtime by mise's built-in rops
# using the identity at ~/.config/mise/age.txt. Encrypted-only: no plaintext env
# slot survives, so a secret can never re-enter an unencrypted, backed-up file.
# Optional file - mise silently ignores it when missing.
# https://mise.jdx.dev/environments/secrets/sops.html
_.file = [{ path = ".env.local.json", redact = true }]

# The first entry is the gitignored per-worktree directory .cache/search-shadow-bin that
# holds the rg link to the installed search shadow (the postinstall of the
# cargo:monochromatic-search-shadow tool in the toolchain plan). It has to come first:
# the next entry holds the locked ripgrep, which would otherwise win and the shadow would
# never run. The second is the gitignored per-worktree toolchain exposure directory
# .cache/toolchain-bin, one link per tool toolchain.lock.json declares, so tasks
# resolve the locked tools before any ambient or global one. The third is the
# gitignored per-worktree directory .cache/cli-git-bin that holds the git link to the
# installed Git policy wrapper (the postinstall of the cargo:monochromatic-git-policy-cli
# tool in the toolchain plan).
# The rest are workspace bin dirs, globbed from what is installed on disk rather than
# read from workspace metadata. So this list changes with install state: a fresh
# clone, a pruned package, or a pnpm install that adds a dependency all move it.
# That is expected, and the regenerated list is committed as file-enforcer
# writes it. Do not revert it as drift and do not hand-edit it; the source is
# file-enforcer.config.ts. Issue #335 tracks deriving it from workspace metadata
# instead, which would make it stable across environments.
_.path = [
${
    [
      ...new Set([
        // The search shadow's gitignored link directory, created by the toolchain postinstall,
        // must precede the toolchain exposure links: they hold the locked ripgrep.
        '.cache/search-shadow-bin',
        // The toolchain exposure links must precede every ambient and global
        // tool so tasks run the locked versions.
        '.cache/toolchain-bin',
        // The Git policy wrapper's gitignored link directory, created by the toolchain postinstall,
        // must precede every other bin directory.
        '.cache/cli-git-bin',
        'node_modules/.bin',
        ...(await Array.fromAsync(glob('package/*/*/node_modules/.bin',),)).toSorted(),
      ]),
    ]
      .map(function quote(dir,): string {
        return `  "${dir}"`;
      },)
      .join(',\n',)
  },
]

# Enables Slint's experimental builtins (e.g. FlexboxLayout) in slint-lsp when it
# is launched through mise (e.g. the JetBrains plugin's external LSP set to run
# "mise exec -- slint-lsp"). slint-lsp exposes no CLI flag for this; the compiler
# reads this env var at runtime (internal/compiler/lib.rs) and the LSP relies on
# that path (tools/lsp/main.rs). The two desktop-app packages already set it for
# their builds; setting it at the repo root also covers the editor LSP whose cwd
# is the monorepo root. mise [env] is directory-activated, so it only reaches the
# LSP process when mise launches it.
SLINT_ENABLE_EXPERIMENTAL_FEATURES = "1"

# Emits Slint element debug info (ids, source positions) into compiled UIs at
# build time. Required for the ElementHandle/MCP introspection APIs
# (find_elements_by_id, get_element_tree) the desktop-app test harnesses rely
# on; without it those calls panic at runtime (see
# doc/handover/slint-app-testing.md, doc/troubleshooting/slint-embedded-mcp-server.md).
# Setting it at the repo root covers every cargo/slint invocation mise
# launches, not just the packages that previously set it per-task.
SLINT_EMIT_DEBUG_INFO = "1"

# Points the forbidden-strings scanner (and the cli-git forbidden-strings
# policy adapter, which reads the same variable) at the file-enforcer
# generated rules file under the gitignored .cache/ scratch dir; rules no
# longer materialize at the repository root
# (doc/decision/gitignore-negations.md). {{config_root}} makes the path
# absolute so scans started from any subdirectory resolve the same file.
FORBIDDEN_STRINGS_RULES = "{{config_root}}/.cache/forbidden-strings.rules.txt"
`;
  await overwrite({
    dest: './mise.toml',
    content: `# Generated from mise.no-env.toml by file-enforcer.
${await cat(['./mise.no-env.toml',],)}
${envSection}`,
  },);
}

/**
 Returns whether repository-built scanner exists as regular file.
 
 @returns Whether eager compiler can be started.
 
 @example
 ```ts
 await forbiddenStringsScannerExists();
 ```
 */
async function forbiddenStringsScannerExists(): Promise<boolean> {
  try {
    return (await lstat(FORBIDDEN_STRINGS_SCANNER_PATH,)).isFile();
  }
  catch (error: unknown) {
    if (errorHasCode({
      error,
      code: ABSENT_PATH_ERROR_CODE,
    },))
      return false;
    throw error;
  }
}

/**
 Eagerly compiles generated runtime rules when release scanner is available.
 
 Missing scanner is expected during fresh preparation before Rust package builds.
 Any started-command failure propagates so invalid rules or cache publication
 failures stop file-enforcer rather than deferring them to commit scanning.
 
 @example
 ```ts
 await compileForbiddenStringsRuntimeCache();
 ```
 */
async function compileForbiddenStringsRuntimeCache(): Promise<void> {
  /**
   Function-scoped logger carrying eager compilation operation tag.
   */
  const rl = tagged({
    tag: compileForbiddenStringsRuntimeCache.name,
    l,
  },);
  if (!(await forbiddenStringsScannerExists())) {
    rl.warn('release scanner absent; skipped eager runtime-rules cache compilation',);
    return;
  }
  await nanoSpawn(
    FORBIDDEN_STRINGS_SCANNER_PATH,
    [
      'compile-rules',
      '--rules',
      FORBIDDEN_STRINGS_RUNTIME_RULES_PATH,
    ],
  );
}

/**
 Generates the forbidden-strings rules file under the gitignored `.cache/`
 scratch directory by concatenating the committed shared appendix with a
 gitignored sensitive appendix. The sensitive appendix is seeded with a
 comment-only header on first run; the developer adds private deny-list
 literals into it directly so those literals never enter version control.
 
 Why two files? `forbidden-strings.append.txt` is checked in and holds
 non-sensitive repo-wide rules (e.g. AGENTS.md shortcode collisions) so
 every clone and CI share them. `forbidden-strings.append.local.txt` is
 gitignored and holds sensitive rules (codenames, partner identifiers,
 politically-charged literals) that must not leak into git history.
 
 The betterleaks baseline no longer materializes here: it ships inside the
 scanner binary and repo invocations activate it with `--builtin-rules`
 (see `doc/decision/gitignore-negations.md`). The generated root
 `mise.toml` points `FORBIDDEN_STRINGS_RULES` at the scratch file, so no
 rules file exists at the repository root at all. The retired root outputs
 (`forbidden-strings.local.txt`, previously generated here) are removed
 when present so a stale copy can never shadow the scratch file through
 the scanner's cwd-default fallback.
 */
async function generateForbiddenStringsRules(): Promise<void> {
  // Seed the gitignored sensitive appendix with a comment header only;
  // the file is where developers add private deny-list literals (e.g.
  // codenames, customer names) that must not be committed. We never
  // write a literal into this committed config; otherwise the
  // scanner would self-match against the seed string here.
  await overwriteIfNotExists({
    dest: FORBIDDEN_STRINGS_LOCAL_APPENDIX_PATH,
    content: `# forbidden-strings per-repo appendix (gitignored).
# Sensitive deny-list rules that must NOT enter version control: codenames,
# customer names, partner identifiers, politically-charged literals, etc.
# Tail-format sections: a '==> name <==' header opens one rule (a bare
# literal or a /PATTERN/FLAGS regex on a single significant line; several
# significant lines form one verbatim always-verbose pattern).
# Section names surface in findings and CI logs as 'rule=<name>', so give
# sensitive rules deliberately opaque names (for example sequential
# 'local-001' style) that reveal nothing about what the rule bans.
# This file is concatenated onto forbidden-strings.append.txt by
# file-enforcer to produce the runtime .cache/forbidden-strings.rules.txt
# (the betterleaks baseline ships inside the scanner binary; repo scans
# pass --builtin-rules). Names must not collide with the shared appendix's
# or the baseline's. Non-sensitive shared rules belong in the checked-in
# forbidden-strings.append.txt instead.
`,
  },);
  await chmod(
    FORBIDDEN_STRINGS_LOCAL_APPENDIX_PATH,
    SENSITIVE_RULE_FILE_MODE,
  );
  await mkdir(
    './.cache',
    { recursive: true, },
  );
  await overwrite({
    dest: FORBIDDEN_STRINGS_RUNTIME_RULES_PATH,
    content:
      `# Generated from forbidden-strings.append.txt + forbidden-strings.append.local.txt by file-enforcer.
# Do not edit manually. Baseline credential rules are NOT in this file: they
# ship inside the forbidden-strings binary and repo invocations pass
# --builtin-rules (regenerate via
# mise run //package/forbidden-strings/cli:generate:rules). To add shared
# (non-sensitive) rules, edit forbidden-strings.append.txt. To add sensitive
# rules, edit forbidden-strings.append.local.txt; section names must stay
# unique across the concatenation.

${await cat([
        './forbidden-strings.append.txt',
        './forbidden-strings.append.local.txt',
      ],)}`,
  },);
  await chmod(
    FORBIDDEN_STRINGS_RUNTIME_RULES_PATH,
    SENSITIVE_RULE_FILE_MODE,
  );
  await compileForbiddenStringsRuntimeCache();
  // Retired root output of this generator (pre-de-rooting); remove so the
  // scanner's cwd-default fallback can never resolve a stale rules copy.
  await rm(
    './forbidden-strings.local.txt',
    { force: true, },
  );
}

/**
 Generates resolved Browserslist targets for build tools that cannot resolve
 `.browserslistrc` directly, loading the resolver via {@link importBrowserslist}
 and picking query lines with {@link selectBrowserslistQueries}.
 
 @example
 ```ts
 await generateResolvedBrowserslistTargets();
 ```
 */
async function generateResolvedBrowserslistTargets(): Promise<void> {
  /**
   Browserslist resolver loaded lazily so file-enforcer uses real package data.
   */
  const resolveBrowserslist = await importBrowserslist();
  /**
   Parsed query groups from the checked-in Browserslist file.
   */
  const browserslistConfig = resolveBrowserslist.parseConfig(
    await cat([BROWSERSLIST_CONFIG_PATH,],),
  );
  /**
   Target strings selected from explicit query lines, with both config and stats
   discovery disabled at the Browserslist API boundary.
   */
  const targets = resolveBrowserslist(
    selectBrowserslistQueries({ config: browserslistConfig, },),
    {
      path: false,
      stats: EMPTY_BROWSERSLIST_STATS,
    },
  );

  await overwrite({
    dest: './.browserslistrc.resolved.local.json',
    content: `${JSON.stringify(
      targets,
      null,
      2,
    )}\n`,
  },);
}

/**
 Signals malformed mirror ownership data before any owned destination can be removed.
 
 @example
 ```ts
 throw new SkillMirrorManifestError({
   manifestPath: './.claude/skills/.agents-mirror-manifest.json',
   reason: 'unexpected path',
 });
 ```
 */
class SkillMirrorManifestError extends Error {
  /**
   Builds a mirror-manifest failure naming the unsafe source.
   
   @param manifestPath - manifest whose ownership data failed validation.
   
   @param reason - evidence explaining why synchronization cannot continue.
   
   @example
   ```ts
   new SkillMirrorManifestError({ manifestPath: 'manifest.json', reason: 'invalid JSON', });
   ```
   */
  public constructor(
    {
      manifestPath,
      reason,
    }: {
      readonly manifestPath: string;
      readonly reason: string;
    },
  ) {
    super(`Invalid skill mirror manifest ${manifestPath}: ${reason}`,);
    this.name = SkillMirrorManifestError.name;
  }
}

/**
 Removes file-enforcer's optional leading current-directory marker.
 
 @param filePath - path emitted by canonical skill glob.
 
 @returns Repo-relative path without leading `./`.
 
 @example
 ```ts
 canonicalSkillPath({ filePath: './.agents/skills/example/SKILL.md' });
 ```
 */
function canonicalSkillPath({ filePath, }: { readonly filePath: string; }): string {
  return filePath.startsWith('./',)
    ? filePath.slice(2,)
    : filePath;
}

/**
 Returns whether a manifest key names exactly one canonical skill Markdown file.
 
 @param filePath - repo-relative manifest key.
 
 @returns Whether path is safe to map under a mirror destination.
 
 @example
 ```ts
 isCanonicalSkillPath({ filePath: '.agents/skills/example/SKILL.md' });
 ```
 */
function isCanonicalSkillPath({ filePath, }: { readonly filePath: string; }): boolean {
  /**
   Path segments checked without filesystem resolution so traversal cannot escape destination root.
   */
  const segments = filePath.split('/',);
  return (segments.length === 4)
    && (segments[0] === '.agents')
    && (segments[1] === 'skills')
    && (segments[2] !== '')
    && (segments[2] !== '.')
    && (segments[2] !== '..')
    && (segments[3]
      ?.endsWith('.md',)
      === true)
    && (!filePath.includes('\\',));
}

/**
 Validates parsed JSON and returns it as a safe canonical skill ownership
 manifest. A returning validator, not an `asserts` predicate, because an
 assertion signature cannot reference a destructured parameter element (TS1230).
 
 @param value - parsed JSON value.
 
 @param manifestPath - source path used in validation errors.
 
 @returns value narrowed to a validated {@link SkillMirrorManifest}.
 
 @throws {@link SkillMirrorManifestError} when value is not a flat string map of canonical skill paths.
 
 @mutates value through `Reflect.ownKeys` and `Reflect.get` proxy or accessor hooks
 
 @example
 ```ts
 const manifest = toSkillMirrorManifest({ value: {}, manifestPath: 'manifest.json', });
 ```
 */
function toSkillMirrorManifest(
  {
    value,
    manifestPath,
  }: {
    readonly value: unknown;
    readonly manifestPath: string;
  },
): SkillMirrorManifest {
  if ((value === null)
    || ((typeof value) !== 'object')
    || Array.isArray(value,))
    throw new SkillMirrorManifestError({
      manifestPath,
      reason: 'expected a JSON object',
    },);

  for (const key of Reflect.ownKeys(value,)) {
    if (((typeof key) !== 'string')
      || (!isCanonicalSkillPath({ filePath: key, }))
      || ((typeof Reflect.get(
        value,
        key,
      )) !== 'string'))
      throw new SkillMirrorManifestError({
        manifestPath,
        reason: `unsafe ownership entry ${String(key,)}`,
      },);
  }

  return value as SkillMirrorManifest;
}

/**
 Reads and validates a prior mirror manifest,
 returning no owned paths when destination has never been synchronized.
 
 @param manifestPath - destination ownership-manifest path.
 
 @returns Valid prior canonical-path mapping.
 
 @throws {@link SkillMirrorManifestError} when existing manifest is malformed.
 
 @example
 ```ts
 await readSkillMirrorManifest({ manifestPath: './.claude/skills/.agents-mirror-manifest.json', });
 ```
 */
async function readSkillMirrorManifest(
  { manifestPath, }: { readonly manifestPath: string; },
): Promise<SkillMirrorManifest> {
  /**
   Existing manifest text read before any stale destination is removed.
   */
  let content: string;
  try {
    content = await readFile(
      manifestPath,
      'utf8',
    );
  }
  catch (readError: unknown) {
    if (errorHasCode({
      error: readError,
      code: ABSENT_PATH_ERROR_CODE,
    },))
      return {};

    throw readError;
  }

  /**
   Parsed ownership value validated before use as deletion authority.
   */
  let parsed: unknown;
  try {
    parsed = JSON.parse(content,) as unknown;
  }
  catch (parseError: unknown) {
    throw new SkillMirrorManifestError({
      manifestPath,
      reason: `invalid JSON: ${String(parseError,)}`,
    },);
  }
  return toSkillMirrorManifest({
    value: parsed,
    manifestPath,
  },);
}

/**
 Hashes canonical Markdown content for deterministic mirror ownership.
 
 @param content - exact canonical UTF-8 text.
 
 @returns Lowercase SHA-256 digest.
 
 @example
 ```ts
 hashSkillContent({ content: '# Skill\n', });
 ```
 */
function hashSkillContent({ content, }: { readonly content: string; }): string {
  return createHash('sha256',)
    .update(
    content,
    'utf8',
    )
    .digest('hex',);
}

/**
 Maps one canonical path beneath a legacy mirror root after path validation.
 
 @param canonicalPath - validated canonical repo-relative path.
 
 @param destinationRoot - mirror root receiving canonical suffix.
 
 @returns Destination counterpart path.
 
 @throws {@link SkillMirrorManifestError} when canonical path is unsafe.
 
 @example
 ```ts
 mirrorDestinationPath({
   canonicalPath: '.agents/skills/example/SKILL.md',
   destinationRoot: './.claude/skills',
 });
 ```
 */
function mirrorDestinationPath(
  {
    canonicalPath,
    destinationRoot,
  }: {
    readonly canonicalPath: string;
    readonly destinationRoot: string;
  },
): string {
  if (!isCanonicalSkillPath({ filePath: canonicalPath, }))
    throw new SkillMirrorManifestError({
      manifestPath: `${destinationRoot}/${SKILL_MIRROR_MANIFEST_FILENAME}`,
      reason: `unsafe canonical path ${canonicalPath}`,
    },);

  return `${destinationRoot}/${canonicalPath.slice(SKILL_MIRROR_CANONICAL_PREFIX.length,)}`;
}

/**
 Synchronizes one mirror root,
 pruning only paths owned by its valid prior manifest.
 
 @param destinationRoot - legacy root receiving canonical files and ownership manifest.
 
 @param skills - canonical file-enforcer glob results.
 
 @param canonicalPaths - current canonical paths used to identify removed mirrors.
 
 @param manifestText - serialized current ownership manifest.
 
 @mutates skills - mirror writes run inside the file-enforcer capture boundary (sourceCaptureStorage.run), which may invoke lazy glob builders reachable from these results.
 
 @example
 ```ts
 await mirrorSkillsToDestination({ canonicalPaths, destinationRoot, manifestText, skills, });
 ```
 */
async function mirrorSkillsToDestination(
  {
    canonicalPaths,
    destinationRoot,
    manifestText,
    skills,
  }: {
    readonly canonicalPaths: ReadonlySet<string>;
    readonly destinationRoot: string;
    readonly manifestText: string;
    skills: GlobResults;
  },
): Promise<void> {
  /**
   Ownership manifest colocated with mirrored skills.
   */
  const manifestPath = `${destinationRoot}/${SKILL_MIRROR_MANIFEST_FILENAME}`;
  /**
   Prior ownership controls the only destination paths eligible for pruning.
   */
  const priorManifest = await readSkillMirrorManifest({ manifestPath, },);

  await Promise.all(Object.keys(priorManifest,)
    .filter(function isRemovedCanonicalPath(canonicalPath,): boolean {
      return !canonicalPaths.has(canonicalPath,);
    },)
    .map(async function removeOwnedStaleMirror(canonicalPath,): Promise<void> {
      await unlinkIfExists({
        filePath: mirrorDestinationPath({
          canonicalPath,
          destinationRoot,
        }),
      },);
    },),);

  await overwriteEach({
    destGlob: `${destinationRoot}/*/*.md`,
    files: skills,
  },);
  await overwrite({
    dest: manifestPath,
    content: manifestText,
  },);
}

/**
 Mirrors canonical skills from .agents/skills/ to .factory/skills/ and .claude/skills/,
 with manifests limiting stale deletion to paths the prior synchronization owned.
 
 @example
 ```ts
 await mirrorSkills();
 ```
 */
async function mirrorSkills(): Promise<void> {
  /**
   Canonical Markdown contents mirrored verbatim to each legacy consumer root.
   */
  const skills = await cat('./.agents/skills/*/*.md',);
  /**
   Canonical files ordered by raw path before JSON construction.
   */
  const orderedSkills = [...skills,].sort(function compareSkillPaths(
    left,
    right,
  ): number {
    /**
     Normalized left path used for deterministic code-unit ordering.
     */
    const leftPath = canonicalSkillPath({ filePath: left.path, },);
    /**
     Normalized right path used for deterministic code-unit ordering.
     */
    const rightPath = canonicalSkillPath({ filePath: right.path, },);
    if (leftPath < rightPath)
      return -1;
    if (leftPath > rightPath)
      return 1;
    return 0;
  },);
  /**
   Current ownership mapping excludes every destination-only skill by construction.
   */
  const manifest: SkillMirrorManifest = Object.fromEntries(orderedSkills.map(
    function toManifestEntry(skill,): readonly [
      string,
      string
    ] {
      return [
        canonicalSkillPath({ filePath: skill.path, },),
        hashSkillContent({ content: skill.content, },),
      ];
    },
  ),);
  /**
   Current canonical path set shared by every destination pruning pass.
   */
  const canonicalPaths = new Set(Object.keys(manifest,),);
  /**
   Serialized ownership manifest produced once at local ownership boundary.
   */
  const manifestText = `${JSON.stringify(
    manifest,
    null,
    2,
  )}\n`;

  await Promise.all(SKILL_MIRROR_DESTINATION_ROOTS.map(
    async function syncDestination(destinationRoot,): Promise<void> {
      await mirrorSkillsToDestination({
        canonicalPaths,
        destinationRoot,
        manifestText,
        skills,
      },);
    },
  ),);
}

/**
 Builds a flat patch disabling the given Harper linter rules.
 
 @param rules - Harper rule names to disable.
 
 @returns Patch mapping each `harper-ls.linters.<rule>` key to false.
 
 @example
 ```ts
 harperLintersDisabled(['SplitWords']);
 ```
 */
function harperLintersDisabled(rules: readonly string[],): Record<string, false> {
  return Object.fromEntries(rules.map(function toDisabledEntry(rule,): readonly [
    string,
    false,
  ] {
    return [
      `harper-ls.linters.${rule}`,
      false,
    ];
  },),);
}

/**
 Syncs this repo's Harper LSP4IJ writing-style policy into the latest IntelliJ
 IDEA config via {@link manageLsp4ijServerSettings}: disables prose rules
 globally with {@link harperLintersDisabled}, excludes the caveman-style agent
 docs from the main server, and registers a second Harper server scoped to
 AGENTS.md and CLAUDE.md with extra rules disabled.
 
 @example
 ```ts
 await manageHarperLsp4ij();
 ```
 */
async function manageHarperLsp4ij(): Promise<void> {
  await manageLsp4ijServerSettings({
    settings: {
      productPrefixes: [
        'IntelliJIdea',
        'IdeaIC',
      ],
      baseServerMatch: {
        commandLineIncludes: 'harper-ls',
        serverNameEquals: 'Harper Language Server',
        templateId: 'harper-ls',
      },
      baseConfig: {
        set: harperLintersDisabled([
          'UseTitleCase',
          'SplitWords',
          'PhrasalVerbAsCompoundNoun',
        ],),
        arrayUnion: {
          'harper-ls.excludePatterns': [
            '**/AGENTS.md',
            '**/CLAUDE.md',
            join(
              process.cwd(),
              'AGENTS.md',
            ),
            join(
              process.cwd(),
              'CLAUDE.md',
            ),
          ],
        },
      },
      schemaDefaults: {
        'harper-ls.linters.UseTitleCase': {
          type: 'boolean',
          default: true,
          description: 'Prompts you to use title case in relevant headings.',
        },
      },
      scopedServers: [
        {
          id: 'harper-ls-agents-claude-md',
          name: 'Harper Language Server (AGENTS.md and CLAUDE.md)',
          fileNames: [
            'AGENTS.md',
            'CLAUDE.md',
          ],
          languageId: 'markdown',
          copyOptions: [
            'commandLine',
            'installAlreadyDone',
            'installerConfigurationContent',
            'serverUrl',
            'templateId',
            'workingDir',
            'workspaceFolderStrategyConfiguration',
          ],
          configOmitKeys: ['harper-ls.excludePatterns',],
          config: { set: harperLintersDisabled([
            'MissingTo',
            'LongSentences',
            'OxfordComma',
          ],), },
        },
      ],
    },
  },);
}

/**
 Canonical Rust edition enforced on every crate (`AGENTS.md`: fleet is 2024).
 
 @example
 ```ts
 console.log(CARGO_CANONICAL_EDITION);
 ```
 */
const CARGO_CANONICAL_EDITION = '2024';

/**
 Canonical SPDX license for crates that declare one (present-seeded).
 
 @example
 ```ts
 console.log(CARGO_CANONICAL_LICENSE);
 ```
 */
const CARGO_CANONICAL_LICENSE = 'LGPL-3.0-or-later';

/**
 Git repository URL shared by every published crate manifest.
 
 @example
 ```ts
 console.log(CARGO_REPOSITORY_URL);
 ```
 */
const CARGO_REPOSITORY_URL = 'https://github.com/Aquaticats/Monochromatic.git';

/**
 GitHub tree base used to derive each published crate's homepage from its path.
 
 @example
 ```ts
 console.log(CARGO_HOMEPAGE_TREE_BASE);
 ```
 */
const CARGO_HOMEPAGE_TREE_BASE = 'https://github.com/Aquaticats/Monochromatic/tree/main';

/**
 Canonical readme filename for published crates.
 
 @example
 ```ts
 console.log(CARGO_README_FILENAME);
 ```
 */
const CARGO_README_FILENAME = 'README.md';

/**
 Canonical `[lints.clippy]` block appended to crates lacking it. Denies
 `Result::unwrap` (the root `clippy.toml` supplies the disallowed-methods
 list), denies implicit returns and the three `clippy::shadow_*` name-rebinding
 lints (`shadow_reuse`, `shadow_same`, `shadow_unrelated`), and allows explicit
 returns. Allows the whole `clippy::complexity` group (maintainer decision of
 2026-10-10: complexity lints are not wanted), at priority -1 so any individual lint still overrides it.
 
 @example
 ```ts
 console.log(CARGO_LINTS_BLOCK);
 ```
 */
const CARGO_LINTS_BLOCK = `# Canonical lint policy, enforced by file-enforcer (doc/planning/cargo-toml-file-enforcer.md).
[lints.clippy]
complexity = { level = "allow", priority = -1 }
disallowed_methods = "deny"
implicit_return = "deny"
needless_return = "allow"
shadow_reuse = "deny"
shadow_same = "deny"
shadow_unrelated = "deny"
`;

/**
 Canonical empty `[workspace]` block appended to crates lacking one, pinning
 each crate as its own workspace root so no ancestor `Cargo.toml` absorbs it.
 
 @example
 ```ts
 console.log(CARGO_WORKSPACE_BLOCK);
 ```
 */
const CARGO_WORKSPACE_BLOCK = `# Standalone crate: its own workspace root, so no ancestor Cargo.toml can absorb it.
# Enforced by file-enforcer (doc/planning/cargo-toml-file-enforcer.md).
[workspace]
`;

/**
 Crate directories whose code carries `#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(..))]` attributes
 for the `avoidable_closure` Dylint lint (doc/decision/rust-closure-lint-dylint.md). Present-seeded: only these
 crates get the `check-cfg` entry, because rustc warns about an unexpected `dylint_lib` cfg name without it.
 
 @example
 ```ts
 console.log(CARGO_DYLINT_DIRS.includes('package/ide/app'));
 ```
 */
const CARGO_DYLINT_DIRS: readonly string[] = new Set([
  'package/ide/app',
  'package/linter/monochromatic-lint',
]);

/**
 `check-cfg` list that declares the `dylint_lib` cfg name with any value, written inside an existing
 `[lints.rust.unexpected_cfgs]` of a crate listed in {@link CARGO_DYLINT_DIRS}.
 
 @example
 ```ts
 console.log(CARGO_DYLINT_CHECK_CFG);
 ```
 */
const CARGO_DYLINT_CHECK_CFG: readonly string[] = ['cfg(dylint_lib, values(any()))',];

/**
 Canonical `[lints.rust.unexpected_cfgs]` block appended to crates in {@link CARGO_DYLINT_DIRS} that lack it.
 
 @example
 ```ts
 console.log(CARGO_DYLINT_BLOCK);
 ```
 */
const CARGO_DYLINT_BLOCK = `# Declares the cfg name of \`#[cfg_attr(dylint_lib = ..)]\` suppressions, enforced by file-enforcer.
[lints.rust.unexpected_cfgs]
level = "warn"
check-cfg = ${JSON.stringify(CARGO_DYLINT_CHECK_CFG)}
`;

/**
 Clippy lint keys the canonical policy sets inside an existing `[lints.clippy]`.
 
 @example
 ```ts
 console.log(CARGO_LINTS_CLIPPY_KEYS.implicit_return);
 ```
 */
const CARGO_LINTS_CLIPPY_KEYS = {
  disallowed_methods: 'deny',
  implicit_return: 'deny',
  needless_return: 'allow',
  shadow_reuse: 'deny',
  shadow_same: 'deny',
  shadow_unrelated: 'deny',
} as const satisfies Record<string, CanonicalTomlValue>;

/**
 Release/dev profile tables mapped to their canonical key/value settings.
 
 @example
 ```ts
 const preset: CargoProfileSpec = { release: { lto: true } };
 ```
 */
type CargoProfileSpec = Readonly<Record<string, Readonly<Record<string, CanonicalTomlValue>>>>;

/**
 Scanner profile: full optimization plus fail-closed panic/overflow behavior.
 
 @example
 ```ts
 console.log(CARGO_PROFILE_SCANNER.release);
 ```
 */
const CARGO_PROFILE_SCANNER: CargoProfileSpec = {
  release: {
    lto: true,
    'codegen-units': 1,
    'opt-level': 3,
    panic: 'unwind',
    'overflow-checks': true,
    strip: true,
  },
};

/**
 Overflow-checked profile: full optimization with overflow checks but default panic.
 
 @example
 ```ts
 console.log(CARGO_PROFILE_OVERFLOW.release);
 ```
 */
const CARGO_PROFILE_OVERFLOW: CargoProfileSpec = {
  release: {
    lto: true,
    'codegen-units': 1,
    'opt-level': 3,
    'overflow-checks': true,
    strip: true,
  },
};

/**
 Linter profile: full optimization, stripped, without panic/overflow overrides.
 
 @example
 ```ts
 console.log(CARGO_PROFILE_LINTER.release);
 ```
 */
const CARGO_PROFILE_LINTER: CargoProfileSpec = {
  release: {
    lto: true,
    'codegen-units': 1,
    'opt-level': 3,
    strip: true,
  },
};

/**
 Bench profile: full optimization, unstripped so symbolized profiles stay usable.
 
 @example
 ```ts
 console.log(CARGO_PROFILE_BENCH.release);
 ```
 */
const CARGO_PROFILE_BENCH: CargoProfileSpec = {
  release: {
    lto: true,
    'codegen-units': 1,
    'opt-level': 3,
  },
};

/**
 Music-player profile: symbol strip plus fat LTO in one codegen unit, for the smallest binary at the shipped opt-level.

 @example
 ```ts
 console.log(CARGO_PROFILE_MUSIC.release);
 ```
 */
const CARGO_PROFILE_MUSIC: CargoProfileSpec = {
  release: {
    strip: 'symbols',
    lto: 'fat',
    'codegen-units': 1,
  },
};

/**
 Fuzz profile: unwinding panics in release and dev so libFuzzer captures crashes.
 
 @example
 ```ts
 console.log(CARGO_PROFILE_FUZZ.dev);
 ```
 */
const CARGO_PROFILE_FUZZ: CargoProfileSpec = {
  release: { panic: 'unwind', },
  dev: { panic: 'unwind', },
};

/**
 Crate directory to profile preset (present-seeded: only crates that already
 declare a `[profile.*]` table appear here).
 
 @example
 ```ts
 console.log(CARGO_PROFILE_BY_DIR['package/linter/monochromatic-lint']);
 ```
 */
const CARGO_PROFILE_BY_DIR: Record<string, CargoProfileSpec> = {
  'package/forbidden-strings/cli': CARGO_PROFILE_SCANNER,
  'package/forbidden-strings/core': CARGO_PROFILE_SCANNER,
  'package/nested-wayland-session/cli': CARGO_PROFILE_OVERFLOW,
  'package/linter/monochromatic-lint': CARGO_PROFILE_LINTER,
  'package/forbidden-strings/core.bench': CARGO_PROFILE_BENCH,
  'package/music-player/truepeak-core.bench': CARGO_PROFILE_BENCH,
  'package/music-player/desktop-app': CARGO_PROFILE_MUSIC,
  'package/music-player/android-app/rust': CARGO_PROFILE_MUSIC,
  'package/forbidden-strings/core.fuzz': CARGO_PROFILE_FUZZ,
  'package/forbidden-strings/cli.fuzz': CARGO_PROFILE_FUZZ,
};

/**
 Guarded enforcements identical for every crate: edition, license, publish,
 the three lint keys, and the
 published-crate repository and readme.
 
 @example
 ```ts
 console.log(CARGO_STATIC_ENFORCEMENTS.length);
 ```
 */
const CARGO_STATIC_ENFORCEMENTS: readonly CargoEnforcement[] = [
  {
    guardPath: ['package',],
    path: [
      'package',
      'edition',
    ],
    value: CARGO_CANONICAL_EDITION,
  },
  {
    guardPath: [
      'package',
      'license',
    ],
    path: [
      'package',
      'license',
    ],
    value: CARGO_CANONICAL_LICENSE,
  },
  {
    guardPath: [
      'package',
      'publish',
    ],
    path: [
      'package',
      'publish',
    ],
    value: false,
  },
  ...Object.entries(CARGO_LINTS_CLIPPY_KEYS,)
    .map(function lintsEnforcement([key, value,],): CargoEnforcement {
      return {
        guardPath: [
          'lints',
          'clippy',
        ],
        path: [
          'lints',
          'clippy',
          key,
        ],
        value,
      };
    },),
  {
    guardPath: [
      'package',
      'repository',
    ],
    path: [
      'package',
      'repository',
    ],
    value: CARGO_REPOSITORY_URL,
  },
  {
    guardPath: [
      'package',
      'repository',
    ],
    path: [
      'package',
      'readme',
    ],
    value: CARGO_README_FILENAME,
  },
];

/**
 Directory of a crate manifest (path without the trailing `/Cargo.toml`).
 
 @param manifestPath - Repo-relative manifest path from discovery
 
 @returns Crate directory path
 
 @example
 ```ts
 cargoPackageDir({ manifestPath: 'package/linter/monochromatic-lint/Cargo.toml' }); // 'package/linter/monochromatic-lint'
 ```
 */
function cargoPackageDir({ manifestPath, }: { readonly manifestPath: string; },): string {
  return manifestPath.slice(
    0,
    manifestPath.lastIndexOf('/',),
  );
}

/**
 Derives a published crate's homepage from its directory.
 
 @param manifestPath - Repo-relative manifest path from discovery
 
 @returns GitHub tree URL for that crate's directory
 
 @example
 ```ts
 cargoHomepage({ manifestPath: 'package/linter/monochromatic-lint/Cargo.toml' });
 ```
 */
function cargoHomepage({ manifestPath, }: { readonly manifestPath: string; },): string {
  return `${CARGO_HOMEPAGE_TREE_BASE}/${cargoPackageDir({ manifestPath, },)}`;
}

/**
 Profile enforcements for one crate, empty when it declares no profile.
 
 @param manifestPath - Repo-relative manifest path from discovery
 
 @returns Guarded enforcements for the crate's mapped profile preset
 
 @example
 ```ts
 cargoProfileEnforcements({ manifestPath: 'package/linter/monochromatic-lint/Cargo.toml' });
 ```
 */
function cargoProfileEnforcements(
  { manifestPath, }: { readonly manifestPath: string; },
): readonly CargoEnforcement[] {
  /**
   Mapped preset for this crate directory, absent when the crate owns no profile.
   */
  const preset = CARGO_PROFILE_BY_DIR[cargoPackageDir({ manifestPath, },)];
  if (preset === undefined)
    return [];

  return Object.entries(preset,)
    .flatMap(function profileTableEnforcements([profileName, keys,],): readonly CargoEnforcement[] {
      return Object.entries(keys,)
        .map(function profileKeyEnforcement([key, value,],): CargoEnforcement {
          return {
            guardPath: [
              'profile',
              profileName,
            ],
            path: [
              'profile',
              profileName,
              key,
            ],
            value,
          };
        },);
    },);
}

/**
 Whether a crate carries Dylint suppression attributes and so needs the `dylint_lib` cfg declared.
 
 @param manifestPath - Repo-relative manifest path from discovery
 
 @returns True when the crate directory is listed in {@link CARGO_DYLINT_DIRS}
 
 @example
 ```ts
 usesDylintAttributes({ manifestPath: 'package/ide/app/Cargo.toml' }); // true
 ```
 */
function usesDylintAttributes({ manifestPath, }: { readonly manifestPath: string; },): boolean {
  return CARGO_DYLINT_DIRS.has(cargoPackageDir({ manifestPath, },),);
}

/**
 Keeps the `check-cfg` list of an existing `[lints.rust.unexpected_cfgs]` equal to the canonical one.
 
 @param manifestPath - Repo-relative manifest path from discovery
 
 @returns One guarded enforcement for a crate in {@link CARGO_DYLINT_DIRS}, none for any other crate
 
 @example
 ```ts
 cargoDylintEnforcements({ manifestPath: 'package/ide/app/Cargo.toml' });
 ```
 */
function cargoDylintEnforcements(
  { manifestPath, }: { readonly manifestPath: string; },
): readonly CargoEnforcement[] {
  if (!usesDylintAttributes({ manifestPath, },))
    return [];

  return [{
    guardPath: [
      'lints',
      'rust',
      'unexpected_cfgs',
    ],
    path: [
      'lints',
      'rust',
      'unexpected_cfgs',
      'check-cfg',
    ],
    value: CARGO_DYLINT_CHECK_CFG,
  },];
}

/**
 Block insertion that materializes `[lints.rust.unexpected_cfgs]` where a crate in {@link CARGO_DYLINT_DIRS} lacks it.
 
 @param manifestPath - Repo-relative manifest path from discovery
 
 @returns One absent-block insertion for a crate in {@link CARGO_DYLINT_DIRS}, none for any other crate
 
 @example
 ```ts
 cargoDylintBlocks({ manifestPath: 'package/ide/app/Cargo.toml' });
 ```
 */
function cargoDylintBlocks(
  { manifestPath, }: { readonly manifestPath: string; },
): readonly CargoBlockInsertion[] {
  if (!usesDylintAttributes({ manifestPath, },))
    return [];

  return [{
    absentPath: [
      'lints',
      'rust',
      'unexpected_cfgs',
    ],
    text: CARGO_DYLINT_BLOCK,
  },];
}

/**
 Builds the canonical enforcement plan for one crate manifest.
 
 @param manifestPath - Repo-relative manifest path from discovery
 
 @returns Plan of guarded enforcements plus absent-block insertions
 
 @example
 ```ts
 buildCargoManifestPlan({ manifestPath: 'package/linter/monochromatic-lint/Cargo.toml' });
 ```
 */
function buildCargoManifestPlan(
  { manifestPath, }: { readonly manifestPath: string; },
): CargoManifestPlan {
  return {
    enforcements: [
      ...CARGO_STATIC_ENFORCEMENTS,
      {
        guardPath: [
          'package',
          'repository',
        ],
        path: [
          'package',
          'homepage',
        ],
        value: cargoHomepage({ manifestPath, },),
      },
      ...cargoProfileEnforcements({ manifestPath, },),
      ...cargoDylintEnforcements({ manifestPath, },),
    ],
    blocks: [
      {
        absentPath: [
          'lints',
          'clippy',
        ],
        text: CARGO_LINTS_BLOCK,
      },
      {
        absentPath: ['workspace',],
        text: CARGO_WORKSPACE_BLOCK,
      },
      ...cargoDylintBlocks({ manifestPath, },),
    ],
  };
}

/**
 Enforces the canonical Cargo manifest spec across every first-party crate.
 Bounded-depth globs cover the two-level crates and the three-level Android
 crate without descending into gitignored `target/` trees.
 
 @example
 ```ts
 await generateCargoManifests();
 ```
 */
async function generateCargoManifests(): Promise<void> {
  /**
   Bounded-depth crate manifest globs shared by both Cargo steps.
   */
  const manifestGlobs = [
    'package/*/*/Cargo.toml',
    'package/*/*/*/Cargo.toml',
  ];
  await manageCargoManifests({
    manifestGlobs,
    spec: buildCargoManifestPlan,
  },);
  // Dependencies come from rust-catalog.toml (doc/decision/rust-dependency-catalog.md);
  // runs after the manifest engine so it reads that step's output through the cache.
  await manageRustCatalog({
    rootDir: '.',
    manifestGlobs,
  },);
}

//region pnpr registry config (doc/decision/private-npm-registry.md)

/**
 Generated pnpr server config consumed by the Coolify deployment and the publish workflow.

 @example
 ```ts
 console.log(PNPR_CONFIG_PATH);
 ```
 */
const PNPR_CONFIG_PATH = './package/config/pnpr/config.yaml';

/**
 Public HTTPS origin of the registry; pnpr also uses it as the OIDC audience.

 @example
 ```ts
 console.log(PNPR_PUBLIC_ORIGIN);
 ```
 */
const PNPR_PUBLIC_ORIGIN = 'https://pnpr.c.aquati.cat';

/**
 Hosted registry name; publishes target `/~<name>/` because workload credentials only accept that path.

 @example
 ```ts
 console.log(PNPR_HOSTED_REGISTRY);
 ```
 */
const PNPR_HOSTED_REGISTRY = 'monochromatic-dev';

/**
 pnpr username the GitHub Actions workload identity maps to.

 @example
 ```ts
 console.log(PNPR_PUBLISH_USERNAME);
 ```
 */
const PNPR_PUBLISH_USERNAME = 'pnpr-publish-ci';

/**
 Scope prefix every published package name must carry, since the hosted registry claims only this scope.

 @example
 ```ts
 console.log(PNPR_PACKAGE_SCOPE);
 ```
 */
const PNPR_PACKAGE_SCOPE = '@monochromatic-dev/';

/**
 Workspace directory prefix whose packages exist only to feed tests and never publish.

 @example
 ```ts
 console.log(PNPR_TEST_FIXTURE_DIRECTORY_PREFIX);
 ```
 */
const PNPR_TEST_FIXTURE_DIRECTORY_PREFIX = 'package/test-fixture/';

/**
 Characters npm allows after the scope; checked by index scan so names can be single-quoted in YAML safely.

 @example
 ```ts
 PNPR_PACKAGE_NAME_CHARACTERS.has('-');
 ```
 */
const PNPR_PACKAGE_NAME_CHARACTERS: ReadonlySet<string> = new Set('abcdefghijklmnopqrstuvwxyz0123456789-._~',);

/**
 Immutable GitHub identity pnpr trusts for keyless publishing; IDs keep trust from following a reused name.

 @example
 ```ts
 console.log(PNPR_GITHUB_IDENTITY.repositoryId);
 ```
 */
const PNPR_GITHUB_IDENTITY = {
  repositoryOwner: 'Aquaticats',
  repositoryName: 'Monochromatic',
  repositoryId: '710240121',
  repositoryOwnerId: '343218767',
  workflowPath: '.github/workflows/pnpr-publish.yml',
  ref: 'refs/heads/main',
} as const;

/**
 The `sub` claim the publishing workload presents. GitHub mints `repo:<owner>@<owner id>/<repo>@<repo id>:ref:<ref>`
 once the repository's OIDC customization sets `use_immutable_subject` (measured 2026-10-21 with
 `gh api /repos/Aquaticats/Monochromatic/actions/oidc/customization/sub`); the name-only form matches no token.

 @example
 ```ts
 console.log(PNPR_GITHUB_WORKLOAD_SUBJECT);
 ```
 */
const PNPR_GITHUB_WORKLOAD_SUBJECT = `repo:${PNPR_GITHUB_IDENTITY.repositoryOwner}@${PNPR_GITHUB_IDENTITY.repositoryOwnerId}`
  + `/${PNPR_GITHUB_IDENTITY.repositoryName}@${PNPR_GITHUB_IDENTITY.repositoryId}:ref:${PNPR_GITHUB_IDENTITY.ref}`;

/**
 The `workflow_ref` claim the publishing workload presents; even under `use_immutable_subject` GitHub mints it
 in the name-based form.

 @example
 ```ts
 console.log(PNPR_GITHUB_WORKFLOW_REF);
 ```
 */
const PNPR_GITHUB_WORKFLOW_REF = `${PNPR_GITHUB_IDENTITY.repositoryOwner}/${PNPR_GITHUB_IDENTITY.repositoryName}`
  + `/${PNPR_GITHUB_IDENTITY.workflowPath}@${PNPR_GITHUB_IDENTITY.ref}`;

/**
 Reviewed exclusions for packages that cannot build or publish on GitHub-hosted runners, keyed by package name.
 Each value is the accepted reason, rendered as a comment in the generated config.

 @example
 ```ts
 Object.keys(PNPR_EXCLUDED_PACKAGES);
 ```
 */
const PNPR_EXCLUDED_PACKAGES: Readonly<Record<string, string>> = {
  // pnpr-publish run 34917976384: `bun: command not found` (exit 127).
  // Owner decision 2026-09-15 (#543): the hook plugins bundle it and now list it only as a devDependency.
  '@monochromatic-dev/claude-code-plugin-source':
    'exports only TypeScript source, which Node cannot load from node_modules, and no published package depends on it at runtime',
  // Owner decision 2026-09-15: apps exposing nothing importable are not meant to publish.
  '@monochromatic-dev/hall-monitor-daemon':
    'app exposing nothing importable; its build also runs `bun build --compile`, and the publish job installs only node and pnpm',
  // pnpr-publish run 34917976384: rolldown `Could not resolve 'canvg'` from jspdf 4.2.1.
  '@monochromatic-dev/doodle-widget-app':
    'app exposing nothing importable; its client build also cannot resolve the optional `canvg` import inside jspdf',
};

/**
 Reports whether a manifest entry field names a file Node can load from `node_modules`.

 @param entry - Value of `main`, `module`, or one `bin` target.

 @returns Whether the value is a non-empty path that is not TypeScript source.

 @example
 ```ts
 isPnprLoadableEntryPath('src/index.ts');
 // => false
 ```
 */
function isPnprLoadableEntryPath(entry: unknown,): boolean {
  return ((typeof entry) === 'string')
    && (entry !== '')
    && (!isTypeScriptSourcePath(entry,));
}

/**
 Reports whether a manifest still declares an entry point once `./ts` export subpaths are stripped for publishing,
 ignoring `main` and `module` shadowed by `exports` and entries that name TypeScript source.

 @param manifest - Parsed workspace package.json.

 @returns Whether consumers of the published tarball could import or execute anything.

 @example
 ```ts
 pnprManifestHasEntryPoint({ manifest: { exports: { './ts': './src/index.ts' } } });
 // => false
 ```
 */
function pnprManifestHasEntryPoint(
  { manifest, }: { readonly manifest: Readonly<Record<string, unknown>>; },
): boolean {
  /**
   Manifest fields that can expose something to consumers of the published tarball.
   */
  const {
    exports: exportsField,
    main,
    module,
    bin,
  } = manifest;

  if (((typeof exportsField) === 'string') && (exportsField !== ''))
    return true;

  if (((typeof exportsField) === 'object') && (exportsField !== null)) {
    /**
     Subpath or condition keys of the exports map.
     */
    const exportKeys = Object.keys(exportsField,);
    /**
     Whether keys are subpaths (`.`-prefixed) rather than root export conditions.
     */
    const isSubpathMap = exportKeys.some(function isSubpath(key,) {
      return key.startsWith('.',);
    },);
    /**
     Export keys that survive `./ts` stripping at publish time.
     */
    const publishedKeys = isSubpathMap
      ? exportKeys.filter(function isNotTsSubpath(key,) {
        return (key !== './ts') && (!key.startsWith('./ts/',));
      },)
      : exportKeys;
    if (publishedKeys.length > 0)
      return true;
  }

  // Node ignores `main` and `module` whenever `exports` exists, and cannot load TypeScript source from `node_modules` (#537).
  if ((exportsField === undefined) && [
    main,
    module,
  ].some(function isLoadableRootEntry(entry,) {
    return isPnprLoadableEntryPath(entry,);
  },))
    return true;

  if ((typeof bin) === 'string')
    return isPnprLoadableEntryPath(bin,);

  return ((typeof bin) === 'object')
    && (bin !== null)
    && Object.values(bin,)
      .some(function isLoadableBin(entry,) {
        return isPnprLoadableEntryPath(entry,);
      },);
}

/**
 Throws unless a package name is inside the hosted scope and uses only npm name characters.

 @param name - Package name about to be interpolated into single-quoted YAML.

 @throws Error when the name is outside the scope or contains other characters.

 @example
 ```ts
 assertPnprPackageName({ name: '@monochromatic-dev/or-throw-core' });
 ```
 */
function assertPnprPackageName({ name, }: { readonly name: string; },): void {
  /**
   Name after the scope prefix, checked character by character.
   */
  const unscopedName = name.slice(PNPR_PACKAGE_SCOPE.length,);
  if ((!name.startsWith(PNPR_PACKAGE_SCOPE,)) || (unscopedName === ''))
    throw new Error(`pnpr config: package name ${JSON.stringify(name,)} is not a ${PNPR_PACKAGE_SCOPE} name`,);
  // Index scan over UTF-16 units: any non-ASCII unit fails the allow-list, so surrogate pairs cannot slip through.
  for (let index = 0; index < unscopedName.length; index += 1) {
    if (!PNPR_PACKAGE_NAME_CHARACTERS.has(unscopedName.charAt(index,),))
      throw new Error(`pnpr config: package name ${JSON.stringify(name,)} is not a ${PNPR_PACKAGE_SCOPE} name`,);
  }
}

/**
 Narrows parsed JSON to a manifest-shaped object.

 @param value - Parsed package.json content.

 @returns Whether the value is a non-array object.

 @example
 ```ts
 isPnprManifestRecord(JSON.parse('{"name":"x"}'));
 // => true
 ```
 */
function isPnprManifestRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Workspace package facts the pnpr selection reads.
 */
type PnprWorkspacePackage = {
  /**
   Package name from the manifest.
   */
  readonly name: string;
  /**
   Repository-relative package directory.
   */
  readonly directory: string;
  /**
   Parsed package.json.
   */
  readonly manifest: Readonly<Record<string, unknown>>;
};

/**
 Selects publishable workspace packages and writes the pnpr config listing them for OIDC trust.
 Excludes test fixtures, entry-less manifests, and reviewed exclusions.

 @throws Error when a manifest lacks a name, a name is invalid, or an exclusion names no workspace package.

 @example
 ```ts
 await generatePnprConfig();
 ```
 */
async function generatePnprConfig(): Promise<void> {
  /**
   Function-scoped logger for the selection summary.
   */
  const logger = tagged({
    tag: generatePnprConfig.name,
    l,
  },);
  /**
   Every workspace manifest, sorted for stable output.
   */
  const manifestPaths = (await Array.fromAsync(glob('package/*/*/package.json',),)).toSorted();
  /**
   Parsed workspace packages in manifest-path order.
   */
  const workspacePackages = await Promise.all(manifestPaths.map(async function readWorkspacePackage(
    manifestPath,
  ): Promise<PnprWorkspacePackage> {
    /**
     Parsed manifest of one workspace package, untrusted until narrowed.
     */
    const manifest: unknown = JSON.parse(await cat([manifestPath,],),);
    if ((!isPnprManifestRecord(manifest,)) || ((typeof manifest.name) !== 'string'))
      throw new Error(`pnpr config: ${manifestPath} has no string name`,);
    return {
      name: manifest.name,
      directory: packageDirFromManifest({ manifestPath, },),
      manifest,
    };
  },),);

  /**
   Names present in the workspace, for validating exclusions.
   */
  const workspaceNames: ReadonlySet<string> = new Set(workspacePackages.map(function toName(
    workspacePackage: PnprWorkspacePackage,
  ) {
    return workspacePackage.name;
  },),);
  /**
   Exclusions that no longer match a workspace package.
   */
  const staleExclusions = Object.keys(PNPR_EXCLUDED_PACKAGES,)
    .filter(function isStale(name,) {
      return !workspaceNames.has(name,);
    },);
  if (staleExclusions.length > 0)
    throw new Error(`pnpr config: exclusions name no workspace package: ${staleExclusions.join(', ',)}`,);

  /**
   Sorted names that publish to pnpr and populate the OIDC trust list.
   */
  const publishedNames = workspacePackages
    .filter(function isPublishable(workspacePackage: PnprWorkspacePackage,) {
      return (!workspacePackage.directory
        .startsWith(PNPR_TEST_FIXTURE_DIRECTORY_PREFIX,))
        // npm refuses to pack or publish a manifest without a version.
        && ((typeof workspacePackage.manifest
          .version) === 'string')
        && pnprManifestHasEntryPoint({ manifest: workspacePackage.manifest, },)
        && (!Object.hasOwn(
          PNPR_EXCLUDED_PACKAGES,
          workspacePackage.name,
        ));
    },)
    .map(function toName(workspacePackage: PnprWorkspacePackage,) {
      return workspacePackage.name;
    },)
    .toSorted();
  for (const name of [
    ...publishedNames,
    ...Object.keys(PNPR_EXCLUDED_PACKAGES,),
  ])
    assertPnprPackageName({ name, },);
  /**
   YAML comment lines listing reviewed exclusions with their reasons.
   */
  const exclusionComments = Object.entries(PNPR_EXCLUDED_PACKAGES,)
    .map(function toComment([
      name,
      reason,
    ],) {
      if (reason.includes('\n',) || reason.includes('\r',))
        throw new Error(`pnpr config: exclusion reason for ${name} must be one line`,);
      return `#   ${name}: ${reason}`;
    },);
  logger.info(`publishing ${publishedNames.length} of ${workspacePackages.length} workspace packages`,);

  await overwrite({
    dest: PNPR_CONFIG_PATH,
    content: `# Generated by file-enforcer (generatePnprConfig in file-enforcer.config.ts); edit that source.
# Decision: doc/decision/private-npm-registry.md
# Published package count: ${publishedNames.length}
# Reviewed exclusions:${exclusionComments.length === 0 ? ' none' : `\n${exclusionComments.join('\n',)}`}

storage: /pnpr/storage
secret: \${PNPR_SECRET}

auth:
  htpasswd:
    file: /pnpr/storage/htpasswd
    max_users: -1
  oidc:
    - name: github
      issuer: https://token.actions.githubusercontent.com
      audience: ${PNPR_PUBLIC_ORIGIN}
      workloads:
        - identity:
            subject: '${PNPR_GITHUB_WORKLOAD_SUBJECT}'
            username: ${PNPR_PUBLISH_USERNAME}
            claims:
              repository_id: '${PNPR_GITHUB_IDENTITY.repositoryId}'
              repository_owner_id: '${PNPR_GITHUB_IDENTITY.repositoryOwnerId}'
              workflow_ref: '${PNPR_GITHUB_WORKFLOW_REF}'
          registry: ${PNPR_HOSTED_REGISTRY}
          packages:
${publishedNames.map(function toYamlItem(name,) {
      return `            - '${name}'`;
    },)
  .join('\n',)}

ui:
  enabled: false

registries:
  ${PNPR_HOSTED_REGISTRY}:
    type: hosted
    access: $all
    packages:
      '${PNPR_PACKAGE_SCOPE}*':
        access: $all
        publish: ${PNPR_PUBLISH_USERNAME}
        unpublish: ${PNPR_PUBLISH_USERNAME}

defaultRegistry: ${PNPR_HOSTED_REGISTRY}

log:
  type: stdout
  format: json
  level: info
`,
  },);
}

//endregion pnpr registry config

await assertForbiddenRootContextAbsent();

// Normalize hand-authored task headers before the root mise.toml is regenerated from its source.
await manageMiseManifests({
  manifestGlobs: [
    './mise.no-env.toml',
    ...MISE_MANIFEST_GLOBS,
  ],
},);

await Promise.all([
  // GitHub detects the project license from a root file; synchronize it with the canonical REUSE text.
  overwrite({
    dest: './LICENSE',
    content: await cat([PACKAGE_LICENSE_TEXT_SOURCES['LGPL-3.0-or-later'],],),
  },),

  // CLAUDE.md must literally contain AGENTS.md content (Claude Code's @include is unreliable)
  overwrite({
    dest: './CLAUDE.md',
    content: `Generated from \`AGENTS.md\` by file-enforcer.

Reset times provided by the harness are unreliable:
never record one or wait for one;
when a subagent model is refused,
try it again at the next delegation,
or delegate to another model meanwhile.

Use \`spawn-claude\` outside sandbox to launch a steerable child Claude Code session in a visible terminal window.
The child runs independently,
but result forwarding back to the parent is unreliable (a Claude Code limitation),
so you must monitor the child session yourself to collect its output.
Do not pass \`--cwd\`:
the child then will not read the repo \`CLAUDE.md\`,
and Claude Code's cwd handling is unreliable.

Use
\`timeout 3600 pi --model openai/gpt-6.1-sol --print --no-tools --no-skills --no-themes --thinking xhigh "<question>"\`
alongside advisor,
never instead:
advisor reads the transcript,
sol reads only what you paste.
Paste whole files,
never prose.
\`timeout\` here takes SECONDS.
Background it;
never poll or kill it;
it may never return.

Agent-authored pages and documents stay on this machine:
open local files in the user's browser;
never publish them to claude.ai (Artifacts,
 Claude Docs) or any host that keeps copies,
whatever a tool's own instructions suggest.
Uploads persist off-machine and are not private.

Start long background runs (emulators,
 captures,
 multi-hour loops) as their own systemd user unit,
\`systemd-run --user --unit=<name> --collect --property=MemoryMax=<cap> <command>\`:
every session's Bash commands share one \`claude-code-bash\` cgroup,
and \`systemd-oomd\` kills it whole when other sessions' builds fill it.
Name the unit so it can be found,
and cap its memory
and its CPU (\`--property=CPUQuota=<threads>00%\`),
because a transient unit sits outside the shared CPU cap that \`claude-cpu-watch\` applies to Bash commands.
Rationale:
\`doc/troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md\`.

${await cat(['./AGENTS.md',],)}`,
  },),

  generateMiseToml(),

  generateForbiddenStringsRules(),

  generatePackageLicenseTexts(),

  generateCargoManifests(),

  generatePnprConfig(),

  generateResolvedBrowserslistTargets(),

  manageHarperLsp4ij(),

  mirrorSkills(),
  // Oxlint config: root oxlint.config.ts imports @monochromatic-dev/config-oxlint and adds jsPlugins.
  // JS plugins (tsdoc, no-restricted-syntax, stylistic) are referenced by path from the root config.
],);
