import {
  mkdtemp,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import spawn from 'nano-spawn';

import {
  fieldOf,
  isJsonRecord,
} from './json-shape.ts';
import {
  prepareManifestForPnpr,
  type PnprPublishTarget,
} from './publish-plan.ts';
import {
  REPOSITORY_ROOT,
  type WorkspaceEntry,
} from './registry-read.ts';

import { scratchParent, } from './scratch-parent.ts';
//region Build, pack, token, publish

/**
 Logger root for per-package publish steps.
 */
const moduleLogger = tagged({ tag: 'pnpr-publish-steps', },);

/**
 Reports whether a package's mise file declares a `build` task.

 @param entry - Workspace package.

 @returns Whether `mise run //<dir>:build` exists.

 @example
 ```ts
 await declaresBuildTask({ entry });
 ```
 */
async function declaresBuildTask({ entry, }: { readonly entry: WorkspaceEntry; },): Promise<boolean> {
  try {
    /**
     Package task file text.
     */
    const miseToml = await readFile(
      join(
        REPOSITORY_ROOT,
        entry.directory,
        'mise.toml',
      ),
      'utf8',
    );
    return miseToml.split('\n',)
      .some(function isBuildTask(line,) {
        return (line.trim() === '[tasks.build]') || (line.trim() === '[tasks."build"]');
      },);
  }
  catch (error) {
    moduleLogger.debug(`no readable mise.toml for ${entry.name}: ${String(error,)}`,);
    return false;
  }
}

/**
 Builds a package through its mise `build` task when it declares one.

 @param entry - Workspace package to build.

 @example
 ```ts
 await buildIfDeclared({ entry });
 ```
 */
export async function buildIfDeclared({ entry, }: { readonly entry: WorkspaceEntry; },): Promise<void> {
  if (!(await declaresBuildTask({ entry, },))) {
    moduleLogger.info(`${entry.name}: no build task; packing sources as-is`,);
    return;
  }
  moduleLogger.info(`${entry.name}: mise run //${entry.directory}:build`,);
  await spawn(
    'mise',
    [
      'run',
      `//${entry.directory}:build`,
    ],
    {
      cwd: REPOSITORY_ROOT,
      stdio: 'inherit',
    },
  );
}

/**
 Packs a package with pnpm, then rewrites the manifest inside the tarball for pnpr,
 so the worktree itself is never modified.

 @param entry - Workspace package to pack.

 @returns Path of the rewritten tarball.

 @throws Error when pnpm writes no tarball or the packed manifest is not an object.

 @example
 ```ts
 await packForPnpr({ entry });
 ```
 */
export async function packForPnpr({ entry, }: { readonly entry: WorkspaceEntry; },): Promise<string> {
  /**
   Scratch directory for this package's tarballs.
   */
  const scratch = await mkdtemp(join(
    await scratchParent(),
    'pnpr-pack-',
  ),);
  await spawn(
    'pnpm',
    [
      'pack',
      '--pack-destination',
      scratch,
    ],
    { cwd: join(
      REPOSITORY_ROOT,
      entry.directory,
    ), },
  );
  /**
   Tarball pnpm wrote.
   */
  const packed = (await readdir(scratch,)).find(function isTarball(file,) {
    return file.endsWith('.tgz',);
  },);
  if (packed === undefined)
    throw new Error(`pnpm pack wrote no tarball for ${entry.name}`,);
  /**
   Extraction directory holding the `package/` root.
   */
  const extracted = await mkdtemp(join(
    scratch,
    'extract-',
  ),);
  await spawn(
    'tar',
    [
      '--extract',
      '--gzip',
      '--file',
      join(
        scratch,
        packed,
      ),
      '--directory',
      extracted,
    ],
  );
  /**
   Packed manifest path.
   */
  const manifestPath = join(
    extracted,
    'package',
    'package.json',
  );
  /**
   Manifest as packed, with workspace specifiers already rewritten by pnpm.
   */
  const packedManifest: unknown = JSON.parse(await readFile(
    manifestPath,
    'utf8',
  ),);
  if (!isJsonRecord(packedManifest,))
    throw new Error(`packed manifest for ${entry.name} is not an object`,);
  await writeFile(
    manifestPath,
    `${JSON.stringify(
      prepareManifestForPnpr(packedManifest,),
      null,
      2,
    )}\n`,
  );
  /**
   Rewritten tarball path.
   */
  const rewritten = join(
    scratch,
    `pnpr-${packed}`,
  );
  await spawn(
    'tar',
    [
      '--create',
      '--gzip',
      '--file',
      rewritten,
      '--directory',
      extracted,
      'package',
    ],
  );
  return rewritten;
}

/**
 Obtains a pnpr workload credential from GitHub Actions, or a local override for testing.

 @param audience - OIDC audience configured in pnpr.

 @returns Registry token.

 @throws Error outside Actions when no override is set, or when GitHub returns no token.

 @example
 ```ts
 await requestPublishToken({ audience: 'https://pnpr.c.aquati.cat' });
 ```
 */
export async function requestPublishToken({ audience, }: { readonly audience: string; },): Promise<string> {
  /**
   Local-test token, never set in CI.
   */
  const override = process.env
    .PNPR_PUBLISH_TOKEN;
  if ((override !== undefined) && (override !== ''))
    return override;
  /**
   GitHub's ID token request endpoint and bearer, present only with `id-token: write`.
   */
  const {
    ACTIONS_ID_TOKEN_REQUEST_URL: requestUrl,
    ACTIONS_ID_TOKEN_REQUEST_TOKEN: requestToken,
  } = process.env;
  if ((requestUrl === undefined) || (requestToken === undefined))
    throw new Error('no GitHub OIDC request environment; the job needs permissions id-token: write',);
  /**
   GitHub ID token response.
   */
  const response = await fetch(
    `${requestUrl}&audience=${encodeURIComponent(audience,)}`,
    {
      headers: { authorization: `bearer ${requestToken}`, },
    },
  );
  /**
   Token value from GitHub, untrusted until narrowed.
   */
  const value = fieldOf({
    value: await response.json(),
    key: 'value',
  },);
  if ((!response.ok) || ((typeof value) !== 'string'))
    throw new Error(`GitHub ID token request failed with HTTP ${response.status}`,);
  // Mask before any later log line could echo the credential.
  console.log(`::add-mask::${value}`,);
  return `pnpr_workload_${value}`;
}

/**
 Publishes a tarball to the hosted registry path the workload credential allows.

 @param tarball - Rewritten tarball path.

 @param target - Registry target from config.

 @param origin - Registry origin to publish to.

 @param tag - Dist-tag for this version.

 @param token - Registry credential.

 @example
 ```ts
 await publishTarball({ tarball, target, origin, tag: 'latest', token });
 ```
 */
export async function publishTarball(
  {
    tarball,
    target,
    origin,
    tag,
    token,
  }: {
    readonly tarball: string;
    readonly target: PnprPublishTarget;
    readonly origin: string;
    readonly tag: string;
    readonly token: string;
  },
): Promise<void> {
  /**
   Named hosted registry URL under the ecosystem prefix the 0.1.0-alpha.16 base mounts npm at.
   */
  const registry = new URL(`${origin}/npm/~${target.registryName}/`,);
  /**
   Per-run npm userconfig referencing the token through the environment, so it never reaches disk.
   */
  const userconfig = join(
    await mkdtemp(join(
      await scratchParent(),
      'pnpr-npmrc-',
    ),),
    '.npmrc',
  );
  await writeFile(
    userconfig,
    `//${registry.host}${registry.pathname}:_authToken=\${NODE_AUTH_TOKEN}\n`,
  );
  /**
   Child environment without GitHub's ID token request variables:
   npm's CI OIDC exchange would otherwise post the GitHub ID token to pnpr's absent npm exchange endpoint.
   */
  const {
    ACTIONS_ID_TOKEN_REQUEST_URL: _requestUrl,
    ACTIONS_ID_TOKEN_REQUEST_TOKEN: _requestToken,
    ...inheritedEnvironment
  } = process.env;
  void _requestUrl;
  void _requestToken;
  try {
    /**
     Completed publish with npm's captured output.
     */
    const result = await spawn(
      'npm',
      [
        'publish',
        tarball,
        '--registry',
        registry.href,
        '--tag',
        tag,
        '--provenance=false',
        '--userconfig',
        userconfig,
      ],
      {
        env: {
          ...inheritedEnvironment,
          NODE_AUTH_TOKEN: token,
        },
      // Output is captured rather than inherited so the caller can recognize an `E403` refusal and retry it.
        stdin: 'ignore',
      },
    );
    moduleLogger.info(result.output,);
  }
  catch (error: unknown) {
    if (Error.isError(error,) && ('output' in error))
      moduleLogger.error(String(error.output,),);
    throw error;
  }
}

//endregion Build, pack, token, publish
