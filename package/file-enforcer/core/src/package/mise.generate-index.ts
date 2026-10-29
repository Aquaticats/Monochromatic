/**
 Generates `data/packages.generated.ts` from Repology data.
 
 Uses a podman container running repology-updater to fetch package metadata
 for our target repos, then extracts per-manager package names via SQL.
 Filters out packages installable via the toolchain plan
 (`src/toolchain/plan.ts`) or via language-ecosystem backends.
 
 First run builds the container image and initializes the database (~15 min).
 Subsequent runs are incremental; only changed repo data is re-fetched.
 
 @example
 ```bash
 node package/file-enforcer/core/src/package/mise.generate-index.ts
 ```
 */

import { cachyosBuildArguments, } from '@monochromatic-dev/test-harness-core-matrix/ts';
import spawn from 'nano-spawn';
import { writeFile, } from 'node:fs/promises';
import { resolve, } from 'node:path';
import { toolPlanInstallableNames, } from '../toolchain/plan.ts';

/**
 Container image name for the repology-updater environment.
 */
const IMAGE_NAME = 'repology-updater';

/**
 Podman volume names for persistent state across incremental runs.
 */
const STATE_VOLUME = 'repology-state';
/**
 Podman volume for parsed repository data.
 */
const PARSED_VOLUME = 'repology-parsed';
/**
 Podman volume for PostgreSQL data directory.
 */
const PG_VOLUME = 'repology-pgdata';

/**
 Package directory containing the generator/ subdirectory.
 */
const PKG_DIR = resolve(
  import.meta.dirname,
  '..',
  '..',
);

/**
 Output path for the generated TypeScript file.
 */
const OUTPUT_PATH = resolve(
  PKG_DIR,
  'src',
  'data',
  'packages.generated.ts',
);

/**
 Generator context directory containing Containerfile, SQL, config.
 */
const GENERATOR_DIR = resolve(
  PKG_DIR,
  'generator',
);

/**
 Managers we support, keyed by the name used in our `PackageManager` type.
 Values are not used here but kept for documentation.
 */
const SUPPORTED_MANAGERS = new Set([
  'apt',
  'dnf',
  'pacman',
  'apk',
  'zypper',
  'brew',
  'choco',
  'scoop',
],);

//region Container operations

/**
 Builds the container image, passing the GitHub token as a build secret
 for authenticated git clones. Falls back to unauthenticated clones
 when no token is available.
 */
async function ensureImage(): Promise<void> {
  console.log('[generate-index] building container image...',);
  /**
   GitHub token from either env var; empty string when neither is set.
   */
  const token = process.env
    .MISE_GITHUB_TOKEN
    ?? process
      .env
      .GITHUB_TOKEN
    ?? '';
  /**
   `--secret` args for podman; empty when no token, so unauthenticated clones run instead.
   */
  const secretArgs = token !== ''
    ? [
      '--secret',
      `id=github_token,env=GITHUB_TOKEN`,
    ]
    : [];
  /**
   Subprocess env; injects `GITHUB_TOKEN` only when a token was found, so podman picks it up via `--secret`.
   */
  const env = token !== ''
    ? {
      ...process.env,
      GITHUB_TOKEN: token,
    }
    : process.env;
  /**
   `--build-arg CACHYOS_IMAGE=<image>` for the CachyOS image this host's CPU level supports (the image has no Alpine variant).
   */
  const baseImageArgs = await cachyosBuildArguments({
    imageName: IMAGE_NAME,
    alpineVariant: false,
  },);
  await spawn(
    'podman',
    [
      'build',
      ...secretArgs,
      ...baseImageArgs,
      '-t',
      IMAGE_NAME,
      '-f',
      resolve(
        GENERATOR_DIR,
        'Containerfile',
      ),
      GENERATOR_DIR,
    ],
    { env, },
  );
  console.log('[generate-index] image ready',);
}

/**
 Ensures the named podman volumes exist for persistent state.
 */
async function ensureVolumes(): Promise<void> {
  await Promise.all(
    [
      STATE_VOLUME,
      PARSED_VOLUME,
      PG_VOLUME,
    ]
      .map(
      async function ensureOneVolume(vol: string,): Promise<void> {
        try {
          await spawn(
            'podman',
            [
              'volume',
              'inspect',
              vol,
            ],
          );
        }
        catch (inspectError: unknown) {
          void inspectError;
          console.log(`[generate-index] creating volume: ${vol}`,);
          await spawn(
            'podman',
            [
              'volume',
              'create',
              vol,
            ],
          );
        }
      },
      ),
  );
}

/**
 Runs the repology-updater container with the given arguments.
 Mounts persistent volumes for incremental update support.
 
 @param args - Arguments to pass to the entrypoint
 
 @returns Captured stdout
 */
async function runContainer(args: readonly string[],): Promise<string> {
  /**
   Spawn result; stdout is returned to the caller, stderr is mirrored to the console for visibility.
   */
  const result = await spawn(
    'podman',
    [
      'run',
      '--rm',
      '-v',
      `${STATE_VOLUME}:/state:Z`,
      '-v',
      `${PARSED_VOLUME}:/parsed:Z`,
      '-v',
      `${PG_VOLUME}:/var/lib/pgsql/data:Z`,
      IMAGE_NAME,
      ...args,
    ],
  );
  if (result.stderr
    !== '')
    console.error(result.stderr,);
  return result.stdout;
}

//endregion Container operations

//region Toolchain plan filter

/**
 Loads the tool names the typed toolchain plan can install
 (`src/toolchain/plan.ts`), replacing the former `mise registry` read:
 after the MPM toolchain migration the plan is the source of truth for which
 tools this repository provisions itself.
 
 @returns Set of toolchain-installable tool names (lowercase)
 */
function loadToolchainPlanNames(): ReadonlySet<string> {
  /**
   Set of toolchain plan names; lowercase for case-insensitive matching downstream.
   */
  const names = new Set<string>();
  for (const name of toolPlanInstallableNames())
    names.add(name.toLowerCase(),);
  console.log(`[generate-index] loaded ${names.size} toolchain plan entries`,);
  return names;
}

/**
 Repology effname prefixes for language-ecosystem packages whose
 upstream registries are also available as mise backends.
 Only includes ecosystems where mise has a native backend:
 
 - `cargo:`: Rust crates (Repology `rust:`)
 - `pipx:`: Python packages (Repology `python:`)
 - `npm:`: Node packages (Repology `node:`)
 - `go:`: Go modules (Repology `go:`)
 - `gem:`: Ruby gems (Repology `ruby:`)
 
 Ecosystems without mise backends (perl, haskell, erlang, ocaml,
 lua, php, r, java, gap, texlive) are NOT filtered; those
 packages genuinely need OS package managers.
 
 @example
 ```
 rust:ripgrep   -> cargo:ripgrep     (mise)
 python:black   -> pipx:black        (mise)
 node:prettier  -> npm:prettier      (mise)
 ```
 */
const MISE_BACKEND_PREFIXES = [
  'go:',
  'node:',
  'python:',
  'ruby:',
  'rust:',
] as const;

/**
 Checks whether a Repology effname belongs to a language ecosystem
 that mise can install via its native backends (cargo, pipx, npm, go, gem).
 
 @param effname - Repology canonical project name
 
 @returns `true` if the package is installable via a mise backend
 */
function isMiseBackendPackage(effname: string,): boolean {
  return MISE_BACKEND_PREFIXES.some(
    function hasPrefix(prefix,): boolean {
      return effname.startsWith(prefix,);
    },
  );
}

//endregion Mise registry

//region Code generation

/**
 Single project entry extracted from Repology.
 */
type RepologyProject = {
  readonly effname: string;
  readonly repos: Readonly<Record<string, string>>;
};

/**
 Manager name paired with that manager's package name, as `Object.entries` yields a project's repos.
 */
type RepoEntry = readonly [
  string,
  string,
];

/**
 Generates the TypeScript source for `packages.generated.ts`
 from filtered Repology project data, building one entry per project
 with {@link buildPCall}.
 
 @param projects - Filtered project entries
 
 @returns TypeScript source code
 */
function generateTypeScript(projects: readonly RepologyProject[],): string {
  /**
   Date portion of the current ISO timestamp; used in the generated file header.
   */
  const [today,] = new Date().toISOString()
    .split('T',);
  /**
   Output buffer: header + entries + closing token, joined with newlines at the end.
   */
  const lines: string[] = [
    '/**',
    ' * Auto-generated from Repology package metadata.',
    ' * Do not edit manually: run the index generator to rebuild.',
    ' *',
    ` * Generated: ${today}`,
    ` * Entries: ${projects.length}`,
    ' */',
    '',
    "import type { PackageEntry, } from '../package/types.ts';",
    "import { p, } from '../package/p.ts';",
    '',
    '/** Auto-generated package entries from Repology, keyed by effname. */',
    'export const generated: readonly PackageEntry[] = [',
  ];

  for (const project of projects) {
    /**
     Generated `p(...)` call string for this project; appended verbatim into the output array.
     */
    const entry = buildPCall(project,);
    lines.push(`  ${entry},`,);
  }

  lines.push(
    '] as const;',
    ''
  );
  return lines.join('\n',);
}

/**
 Builds a `p()` call string for a single project, escaping string literals
 with {@link escapeString}.
 Uses string shorthand when the package name matches effname in all available repos.
 Otherwise uses the object form with `yes` array.
 
 @param project - Repology project with per-manager package names
 
 @returns TypeScript expression string like `p('curl')` or `p({ effname: '...', yes: [...] })`
 */
function buildPCall(project: RepologyProject,): string {
  /**
   Per-manager entries from Repology filtered down to managers we generate code for.
   */
  const managers = Object
    .entries(project.repos,)
    .filter(function isSupported([manager,]: RepoEntry,): boolean {
      return SUPPORTED_MANAGERS.has(manager,);
    },);

  /**
   Check if all managers use the effname as package name
   */
  const allSameName = managers.every(
    function matchesEffname([, pkgname,]: RepoEntry,): boolean {
      return pkgname === project
        .effname;
    },
  );

  /**
   Check if available in ALL supported managers with same name
   */
  if (allSameName && (managers.length
    === SUPPORTED_MANAGERS
      .size))
    return `p('${escapeString(project.effname,)}',)`;

  /**
   Build yes array entries
   */
  const yesEntries = managers.map(
    function formatEntry([manager, pkgname,]: RepoEntry,): string {
      if (pkgname === project
        .effname)
        return `'${manager}'`;
      return `['${manager}', '${escapeString(pkgname,)}',]`;
    },
  );

  return `p({ effname: '${escapeString(project.effname,)}', yes: [${
    yesEntries.join(', ',)
  },], },)`;
}

/**
 Escapes single quotes in a string for safe inclusion in a TypeScript string literal.
 
 @param value - Raw string value
 
 @returns Escaped string safe for single-quoted TypeScript literals
 */
function escapeString(value: string,): string {
  return value
    .replaceAll(
    '\\',
    String.raw`\\`,
    )
    .replaceAll(
    "'",
    String.raw`\'`,
    );
}

//endregion Code generation

//region Main

console.log('[generate-index] starting package index generation',);

/**
 Step 1: Load the toolchain plan names for filtering
 */
const miseTools = loadToolchainPlanNames();

/**
 Step 2: Build container and ensure volumes
 */
await ensureImage();
await ensureVolumes();

/**
 Step 3: Fetch and process repos (auto-inits schema on first run)
 */
console.log('[generate-index] fetching and processing repos...',);
await runContainer([
  '--fetch',
  '--fetch',
  '--parse',
  '--database',
],);

/**
 Step 5: Extract package data via SQL
 */
console.log('[generate-index] extracting package data...',);
/**
 Raw JSON output from the repology-updater extract step.
 */
const rawJson = await runContainer(['--extract',],);
/**
 Unparsed JSON for type-safe narrowing from any.
 */
const rawParsed: unknown = JSON.parse(rawJson.trim(),);
/* oxlint-disable typescript/no-unsafe-type-assertion -- shape validated by upstream SQL output format */
/**
 Parsed Repology project entries with per-manager package names.
 */
const projects = rawParsed as RepologyProject[];
/* oxlint-enable typescript/no-unsafe-type-assertion */
console.log(`[generate-index] extracted ${projects.length} projects from Repology`,);

/**
 Step 6: Filter out mise-installable packages
 */
const filtered = projects.filter(
  function notMiseInstallable(
    project: Pick<RepologyProject, 'effname'>,
  ): boolean {
    if (miseTools.has(project.effname
      .toLowerCase(),))
      return false;
    if (isMiseBackendPackage(project.effname,))
      return false;
    return true;
  },
);
/**
 Number of packages filtered out because mise can install them directly.
 */
const removedCount = projects.length
  - filtered
    .length;
console.log(
  `[generate-index] ${removedCount} packages filtered (mise registry + mise backends)`,
);
console.log(`[generate-index] ${filtered.length} packages remaining`,);

/**
 Step 7: Generate and write TypeScript
 */
const source = generateTypeScript(filtered,);
await writeFile(
  OUTPUT_PATH,
  source,
  'utf8',
);
console.log(`[generate-index] wrote ${OUTPUT_PATH}`,);

//endregion Main
