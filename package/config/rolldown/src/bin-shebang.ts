import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import type { Plugin, } from 'rolldown';

//region Types: manifest bin shapes and chunk projection

/**
 `bin` field shape from a package manifest:
 one target shared by every command,
 or a command-to-target map.
 
 @example
 ```ts
 const manifest: PackageManifestBin = { bin: { 'my-cli': './dist/final/node/cli.mjs' } };
 ```
 */
export type PackageManifestBin = {
  readonly bin?: string | Readonly<Record<string, string>>;
};

/**
 Chunk fields the injection step reads,
 project-owned and deeply readonly so foreign chunk shapes borrow safely.
 
 @example
 ```ts
 const chunk: BinTargetChunk = { fileName: 'cli.mjs', isEntry: true };
 ```
 */
type BinTargetChunk = {
  readonly fileName: string;
  readonly isEntry: boolean;
};

/**
 Interpreter line injected at byte 0 of bin-targeted Node chunks.
 
 A chunk without a shebang makes the Unix kernel fall back to `/bin/sh`,
 which parses bundled JavaScript as a shell script and resolves `import`
 to ImageMagick's screenshot command (issue 139).
 */
const NODE_SHEBANG = '#!/usr/bin/env node';

//endregion Types: manifest bin shapes and chunk projection

//region Bin target resolution

/**
 Normalize one manifest bin target or emitted path to forward-slash form.
 
 Manifest authors write `./dist/final/node/cli.mjs` and
 `dist/final/node/cli.mjs` interchangeably,
 so matching must not depend on which form was written.
 
 @param target - Manifest bin target or output-dir-relative emitted path.
 
 @returns Forward-slash path without `./` prefix and trailing slash.
 
 @example
 ```ts
 normalizeBinTarget('./dist/final/node/cli.mjs');
 ```
 */
function normalizeBinTarget(target: string,): string {
  /**
   Target with Windows separators folded to the manifest's forward slashes.
   */
  const slashNormalized = target.replaceAll(
    '\\',
    '/',
  );
  /**
   Slash-normalized target without a leading `./`.
   */
  const unprefixed = slashNormalized.startsWith('./',)
    ? slashNormalized.slice('./'.length,)
    : slashNormalized;
  return unprefixed.endsWith('/',)
    ? unprefixed.slice(
      0,
      -'/'.length,
    )
    : unprefixed;
}

/**
 Collect normalized built-output bin targets from a manifest.
 
 Both manifest `bin` forms are supported because the repository uses both:
 the string form appears where one command wraps the package,
 the map form where a package ships several commands.
 
 @param manifest - Bin-bearing manifest slice as parsed from package.json.
 
 @returns Normalized package-root-relative paths of declared bin outputs.
 
 @example
 ```ts
 binTargetPaths({ manifest: { bin: { 'my-cli': './dist/final/node/cli.mjs' } } });
 ```
 */
export function binTargetPaths({
  manifest,
}: {
  readonly manifest: PackageManifestBin;
},): readonly string[] {
  /**
   Manifest `bin` field in either accepted form.
   */
  const { bin, } = manifest;
  if (bin === undefined)
    return [];
  return (((typeof bin) === 'string')
    ? [bin,]
    : Object.values(bin,)
  )
    .map(normalizeBinTarget,);
}

/**
 Narrow an unknown parsed manifest to its bin-bearing shape.
 
 Validates only what {@link binTargetPaths} reads:
 an optional `bin` that is a string or a string-valued object.
 
 @param value - Parsed package.json content to narrow.
 
 @returns Nothing; success narrows value for the caller.
 
 @throws Error when value is not manifest-shaped or `bin` holds non-strings.
 
 @example
 ```ts
 assertPackageManifestBin(JSON.parse('{}'));
 ```
 */
function assertPackageManifestBin(
  value: unknown,
): asserts value is PackageManifestBin {
  if (((typeof value) !== 'object') || (value === null))
    throw new Error('package.json must parse to an object',);
  /**
   Manifest object exposing only the `bin` field to validate.
   */
  const withBin = value as {
    readonly bin?: unknown;
  };
  /**
   Raw `bin` field before string-or-map narrowing.
   */
  const { bin, } = withBin;
  if (bin === undefined)
    return;
  if ((typeof bin) === 'string')
    return;
  if (((typeof bin) !== 'object') || (bin === null))
    throw new Error('package.json bin must be a string or an object',);
  /**
   Whether every declared bin target is a string.
   */
  const targetsAreStrings = Object.values(bin,)
    .every(function isStringTarget(target: unknown,): boolean {
      return ((typeof target) === 'string');
    },);
  if (!targetsAreStrings)
    throw new Error('package.json bin targets must be strings',);
}

/**
 Read bin targets of the containing package from its manifest.
 
 The manifest is read at build time so matching always sees the `bin`
 entries a published install would see,
 never a stale config-time copy.
 
 @param packageDir - Directory holding the consuming package.json; defaults to the build cwd.
 
 @returns Normalized package-root-relative paths of declared bin outputs.
 
 @throws Error when package.json is absent, malformed, or bin-shaped wrong.
 
 @example
 ```ts
 const targets = await readBinTargetPaths({ packageDir: process.cwd() });
 ```
 */
export async function readBinTargetPaths({
  packageDir = process.cwd(),
}: {
  readonly packageDir?: string;
},): Promise<readonly string[]> {
  /**
   Raw parsed manifest before shape narrowing.
   */
  const parsedManifest: unknown = JSON.parse(
    await readFile(
      join(
        packageDir,
        'package.json',
      ),
      'utf8',
    ),
  );
  assertPackageManifestBin(parsedManifest,);
  return binTargetPaths({ manifest: parsedManifest, },);
}

//endregion Bin target resolution

//region Shebang injection

/**
 Prepend the Node shebang to chunk code that has none.
 
 A chunk already starting with `#!` is a user error outside this contract:
 its interpreter line is left untouched and nothing is validated about it.
 
 @param code - Emitted chunk code as rolldown generated it.
 
 @returns Code whose first line is the Node shebang when it lacked one.
 
 @example
 ```ts
 injectNodeShebang('console.log(1);');
 ```
 */
export function injectNodeShebang(code: string,): string {
  if (code.startsWith('#!',))
    return code;
  return `${NODE_SHEBANG}\n${code}`;
}

//endregion Shebang injection

//region Plugin

/**
 Rolldown plugin injecting Node shebangs into bin-targeted entry chunks.
 
 Reads the containing package's `bin` entries and prepends
 `#!/usr/bin/env node` to each matching emitted entry chunk that lacks a
 shebang, so a built CLI can never fall through to `/bin/sh`.
 Executable bits are set separately by `shebangExecutablePlugin`,
 which reads the injected line after this rewrite.
 
 Chunks no `bin` entry targets are left untouched, and a `bin` entry
 pointing at output this build does not emit is ignored here:
 typo detection is not this plugin's job.
 
 @param outputDir - Output directory of the build, joined with chunk names for matching.
 
 @param packageDir - Directory holding the consuming package.json; defaults to the build cwd.
 
 @returns Rolldown plugin rewriting bin-targeted chunks in `renderChunk`.
 
 @example
 ```ts
 plugins: [binShebangPlugin({ outputDir: 'dist/final/node' })];
 ```
 */
export function binShebangPlugin({
  outputDir,
  packageDir = process.cwd(),
}: {
  readonly outputDir: string;
  readonly packageDir?: string;
},): Plugin {
  return {
    name: 'monochromatic:bin-shebang',

    /**
     Rewrite the chunk when it is one the manifest's `bin` entries target.
     
     Bin targets are resolved per chunk so watch rebuilds pick up
     `bin` edits without restart, and no state outlives the call.
     */
    async renderChunk(
      code: string,
      chunk: BinTargetChunk,
    ) {
      if (!chunk.isEntry)
        return null;
      /**
       Normalized bin targets of the consuming package.
       */
      const binTargets = new Set(await readBinTargetPaths({ packageDir, },),);
      /**
       Emitted path of this chunk in manifest `bin` form.
       */
      const emittedTarget = normalizeBinTarget(`${outputDir}/${chunk.fileName}`,);
      if (!binTargets.has(emittedTarget))
        return null;
      return injectNodeShebang(code,);
    },
  };
}

//endregion Plugin
