/**
 Container integration test for the encrypted env store reader.
 
 Runs inside an Ubuntu matrix container as root and as a non-root user.
 The test installs the sops and age releases the mounted repo's
 `toolchain.lock.json` pins (sha256 checked before use), creates a synthetic
 age identity and encrypted store, then exercises the reader through its built
 artifact. No real credential is read or written.
 
 @module
 */

import { execFileSync, } from 'node:child_process';
import { createHash, } from 'node:crypto';
import {
  chmod,
  mkdir,
  mkdtempDisposable,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  delimiter,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  readEnvLocalJson,
  type ReadEnvLocalJsonOptions,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Repository root from the mounted container path.
 */
const REPO_ROOT = join(
  import.meta.dirname,
  '..',
  '..',
  '..',
  '..',
);

/**
 Real container home that receives the locked sops and age binaries. The
 fixture keeps its own temp root, but the reader must resolve the tools from
 this home.
 */
const REAL_HOME = process.env.HOME
  ?? homedir();

/**
 Directory the locked tool binaries are installed into; put first on `PATH`.
 */
const TOOL_BIN_DIRECTORY = join(REAL_HOME, '.local', 'bin',);

/**
 Platform row of `toolchain.lock.json` for this container's CPU, chosen the way
 the matrix harness chooses the locked Node archive.
 */
const LOCK_PLATFORM = process.arch === 'arm64' ? 'linux-arm64' : 'linux-x64';

/**
 Synthetic provider values.
 */
const VALUES = {
  EXA_API_KEY: 'matrix-exa-sentinel',
  LINKUP_API_KEY: 'matrix-linkup-sentinel',
} as const;

/**
 One archive row of `toolchain.lock.json`.
 */
type LockedArtifact = {
  /**
   Download address of the pinned release asset.
   */
  readonly url: string;
  /**
   Lowercase hexadecimal SHA-256 of the pinned release asset.
   */
  readonly sha256: string;
};

/**
 Whether a parsed JSON value is a plain object.

 @param value - parsed JSON value

 @returns `true` when `value` is a non-null object

 @example
 ```ts
 isRecord({});
 ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Reads the archive row the lock pins for a tool on this container's platform.

 @param tool - tool name in the lock's `tools` object

 @returns the pinned download address and SHA-256

 @throws Error when the lock has no archive row for the tool on this platform

 @example
 ```ts
 const sops = await readLockedArtifact({ tool: 'sops' });
 ```
 */
async function readLockedArtifact({ tool, }: { readonly tool: string; },): Promise<LockedArtifact> {
  /**
   Parsed lock of the mounted repository.
   */
  const lock: unknown = JSON.parse(await readFile(
    join(REPO_ROOT, 'toolchain.lock.json',),
    'utf8',
  ),);
  if ((!isRecord(lock,)) || (!isRecord(lock.tools,)))
    throw new Error('toolchain.lock.json has no tools object',);
  /**
   The lock's entry for the tool.
   */
  const entry = lock.tools[tool];
  if ((!isRecord(entry,)) || (!isRecord(entry.artifacts,)))
    throw new Error(`toolchain.lock.json has no ${tool} artifacts`,);
  /**
   The lock's archive row for this platform.
   */
  const row = entry.artifacts[LOCK_PLATFORM];
  if ((!isRecord(row,)) || ((typeof row.url) !== 'string') || ((typeof row.sha256) !== 'string'))
    throw new Error(`toolchain.lock.json has no ${LOCK_PLATFORM} artifact for ${tool}`,);
  return { url: row.url, sha256: row.sha256.toLowerCase(), };
}

/**
 Downloads a pinned release asset and writes it only when its SHA-256 matches the pin.

 @param artifact - pinned download address and SHA-256

 @param destination - file written after the digest matches

 @throws Error when the download is refused or its digest differs from the pin

 @example
 ```ts
 await downloadVerified({ artifact, destination: '/root/.local/bin/sops' });
 ```
 */
async function downloadVerified({ artifact, destination, }: {
  readonly artifact: LockedArtifact;
  readonly destination: string;
},): Promise<void> {
  /**
   Response of the release asset request.
   */
  const response = await fetch(artifact.url,);
  if (!response.ok)
    throw new Error(`downloading ${artifact.url} returned HTTP ${response.status}`,);
  /**
   Downloaded bytes, held until the digest is checked.
   */
  const bytes = new Uint8Array(await response.arrayBuffer(),);
  /**
   SHA-256 of the downloaded bytes.
   */
  const digest = createHash('sha256',)
    .update(bytes,)
    .digest('hex',);
  if (digest !== artifact.sha256)
    throw new Error(`${artifact.url} has sha256 ${digest}, not the ${artifact.sha256} toolchain.lock.json pins`,);
  await writeFile(destination, bytes,);
}

/**
 Install the sops and age releases `toolchain.lock.json` pins and put them first on `PATH`.
 */
async function installStoreTools(): Promise<void> {
  await mkdir(TOOL_BIN_DIRECTORY, { recursive: true, },);
  /**
   Where the sops binary is installed.
   */
  const sopsPath = join(TOOL_BIN_DIRECTORY, 'sops',);
  await downloadVerified({
    artifact: await readLockedArtifact({ tool: 'sops', },),
    destination: sopsPath,
  },);
  await chmod(sopsPath, 0o755,);
  /**
   Disposable directory holding the downloaded age archive, removed when this
   function returns.
   */
  await using archiveRoot = await mkdtempDisposable(join(scratchRoot(), 'env-local-json-age-',),);
  /**
   Where the age archive is written before extraction.
   */
  const archivePath = join(archiveRoot.path, 'age.tar.gz',);
  await downloadVerified({
    artifact: await readLockedArtifact({ tool: 'age', },),
    destination: archivePath,
  },);
  execFileSync(
    'tar',
    [
      '--extract',
      '--gzip',
      '--file',
      archivePath,
      '--directory',
      TOOL_BIN_DIRECTORY,
      '--strip-components=1',
      'age/age',
      'age/age-keygen',
    ],
    { stdio: 'inherit', },
  );
  process.env.PATH = `${TOOL_BIN_DIRECTORY}${delimiter}${process.env.PATH ?? ''}`;
}

/**
 Create an age identity and an encrypted store in a disposable directory.
 
 @returns fixture root, identity path, and store path
 */
async function createFixture(): Promise<{
  readonly root: string;
  readonly identityPath: string;
  readonly storePath: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}> {
  /**
   Fixture root, removed when the caller's scope ends.
   */
  const directory = await mkdtempDisposable(join(scratchRoot(), 'env-local-json-matrix-',),);
  const root = directory.path;
  await chmod(root, 0o700,);
  /**
   Local value for identityPath.
   */
  const identityPath = join(root, 'identity.txt',);
  execFileSync('age-keygen', ['--output', identityPath,], { stdio: 'ignore', },);
  await chmod(identityPath, 0o600,);
  /**
   Local value for identityText.
   */
  const identityText = await readFile(identityPath, 'utf8',);
  /**
   Public key extracted from the identity file's comment line.
   */
  const recipient = publicKeyFromIdentity({ identityText, },);
  /**
   Local value for plaintextPath.
   */
  const plaintextPath = join(root, 'plain.json',);
  /**
   Local value for storePath.
   */
  const storePath = join(root, '.env.local.json',);
  await writeFile(plaintextPath, JSON.stringify(VALUES,), 'utf8',);
  execFileSync(
    'sops',
    [
      '--encrypt',
      '--age',
      recipient,
      '--input-type',
      'json',
      '--output-type',
      'json',
      '--output',
      storePath,
      plaintextPath,
    ],
    { stdio: 'ignore', },
  );
  await chmod(storePath, 0o600,);
  return {
    root,
    identityPath,
    storePath,
    [Symbol.asyncDispose]: async function removeFixtureRoot(): Promise<void> {
      await directory.remove();
    },
  };
}

//endregion Fixtures

/**
 Extracts the age public key from an identity file's `public key:` comment.

 @param identityText - full text of the generated identity file

 @returns age recipient string

 @throws Error when the identity file has no public-key comment

 @example
 ```ts
 const recipient = publicKeyFromIdentity({ identityText });
 ```
 */
function publicKeyFromIdentity({ identityText, }: {
  readonly identityText: string;
}): string {
  /**
   Marker preceding the public key in an age identity file.
   */
  const marker = 'public key: ';
  /**
   Offset of the marker in the identity text.
   */
  const markerStart = identityText.indexOf(marker,);
  if (markerStart === (-1))
    throw new Error('synthetic age identity has no public-key comment',);
  /**
   Text after the marker, possibly including later lines.
   */
  const rest = identityText.slice(markerStart + marker.length,);
  /**
   Offset of the newline ending the comment line.
   */
  const lineEnd = rest.indexOf('\n',);
  return (lineEnd === (-1) ? rest : rest.slice(0, lineEnd)).trim();
}

/**
 Reads a store and returns its rejection.

 @param options - reader options for the expected-failure read

 @returns rejection value

 @throws Error when the read unexpectedly resolves

 @example
 ```ts
 const rejection = await captureRejection({ keys: ['A'] });
 ```
 */
async function captureRejection(
  options: Readonly<ReadEnvLocalJsonOptions>,
): Promise<unknown> {
  try {
    await readEnvLocalJson(options,);
  }
  catch (error) {
    return error;
  }
  throw new Error('expected the encrypted store reader to reject',);
}

await installStoreTools();
await using disposableFixture = await createFixture();

/** Fixture root, identity and store; `disposableFixture` owns their removal. */
const fixture = disposableFixture;

await describe({
  name: 'env-local-json container reader',
  children: [
    it({
      name: 'decrypts requested keys with a real sops and age toolchain',
      fn: async () => {
        /**
         Local value for result.
         */
        const result = await readEnvLocalJson({
          keys: ['EXA_API_KEY', 'LINKUP_API_KEY',],
          path: fixture.storePath,
          cwd: fixture.root,
          env: {
            PATH: process.env.PATH ?? '',
            HOME: REAL_HOME,
            SOPS_AGE_KEY_FILE: fixture.identityPath,
          },
          home: REAL_HOME,
        },);

        expect(result.loaded,).toBe(true,);
        expect(result.values.EXA_API_KEY,).toBe(VALUES.EXA_API_KEY,);
        expect(result.values.LINKUP_API_KEY,).toBe(VALUES.LINKUP_API_KEY,);
      },
    },),
    it({
      name: 'rejects a plaintext store without touching the identity',
      fn: async () => {
        /**
         Local value for plaintextPath.
         */
        const plaintextPath = join(fixture.root, 'plaintext-store.json',);
        await writeFile(plaintextPath, JSON.stringify(VALUES,), 'utf8',);

        /**
         Rejection raised by the plaintext store read.
         */
        const caught = await captureRejection({
          keys: ['EXA_API_KEY',],
          path: plaintextPath,
          cwd: fixture.root,
          env: {
            PATH: process.env.PATH ?? '',
            HOME: REAL_HOME,
          },
          home: REAL_HOME,
        },);
        if (!(Error.isError(caught,)))
          throw new Error('plaintext store rejection was not an Error',);
        expect(caught.name,).toBe('EnvLocalJsonPlaintextError',);
        expect(caught.message,).not.toContain(VALUES.EXA_API_KEY,);
        await rm(plaintextPath,);
      },
    },),
  ],
},);
