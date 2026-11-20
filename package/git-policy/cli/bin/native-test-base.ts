/**
 The base images of the container runners: the newest CachyOS image the host CPU runs, on its `latest` tag, plus what
 each runner needs from the network. The gate's base (`native-test-base.Containerfile`) adds the Rust toolchain, the
 build tools and the tools the tests start (Git, Node, pnpm, strace, cargo-nextest), plus the pinned browserslist data
 tarballs an offline `file-enforcer check` resolves against; the end-to-end base
 (`e2e/concurrent-commits.Containerfile`) adds Node and Git. Every tool is a current pacman package, with no pin (owner
 decisions of 2026-11-04, recorded in both Containerfiles).

 The gate image builds without network from a content-addressed base, but the repository's container base rule
 (`doc/decision/container-base-distro.md`) forbids pinning a base to a version or a digest. This module keeps both:
 each run pulls the `latest` tag once, and the derived base image is named after what that pull returned, the
 Containerfile text and the build arguments. A pull that returns the image already held, an unchanged Containerfile and
 unchanged arguments find the derived base present and build nothing; an upstream update, or a change to the
 Containerfile or the arguments, builds a new derived base with network, and every later step of the gate stays offline
 and cached on that base's ID.
 */
import { createHash, } from 'node:crypto';
import {
  mkdir,
  mkdtempDisposable,
  readFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  buildLimitArguments,
  type ContainerLimits,
} from './container-limits.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

/**
 Repository path of the resolver command that prints the CachyOS base image for this host.
 */
const resolverPath = resolve(
  import.meta.dirname,
  '..',
  '..',
  '..',
  'test-harness',
  'core-matrix',
  'src',
  'bin',
  'cachyos-image.ts',
);

/**
 Name the resolver reports in its error when the host has no CachyOS image, so the failure names this package's image.
 */
const resolverImageName = 'git-policy-cli-native-test-base';

/**
 Containerfile of the gate's base image, beside the package manifest.
 */
const gateContainerfilePath = resolve(
  import.meta.dirname,
  '..',
  'native-test-base.Containerfile',
);

/**
 Repository of the gate's base image; its tag names the CachyOS image, the Containerfile it came from and the arguments
 that build was given.
 */
const gateRepository = 'localhost/git-policy-native-test-base';

/**
 Repository path of the browserslist data lock: the versions and sha512 digests of the pinned npm tarballs the gate's
 base image caches for `file-enforcer`'s `browserslist` rule (`browserslist-data.lock.json` at the repository root).
 */
export const browserslistDataLockPath: string = resolve(
  import.meta.dirname,
  '..',
  '..',
  '..',
  '..',
  'browserslist-data.lock.json',
);

/**
 Name of the build argument that carries the browserslist data lock into the gate base's Containerfile.
 */
const browserslistDataLockArgument = 'BROWSERSLIST_DATA_LOCK';

/**
 Containerfile of the end-to-end suite's base image.
 */
const endToEndContainerfilePath = resolve(
  import.meta.dirname,
  '..',
  'e2e',
  'concurrent-commits.Containerfile',
);

/**
 Repository of the end-to-end suite's base image; its tag names the CachyOS image and the Containerfile it came from.
 */
const endToEndRepository = 'localhost/cli-git-concurrent-e2e';

/**
 Hexadecimal digits of a hash kept in the tag: enough to tell two images or two Containerfiles apart.
 */
const tagHashLength = 12;

/**
 Prefix of an image ID in `podman image inspect` output that is not part of the hash.
 */
const digestPrefix = 'sha256:';

/**
 The CachyOS image a pull left on the host.
 */
export type PulledCachyos = {
  /**
   CachyOS image reference the pull used, such as `docker.io/cachyos/cachyos-v4:latest`.
   */
  readonly cachyosImage: string;
  /**
   Content-addressed ID of the CachyOS image the pull left on the host, which derived bases are built on.
   */
  readonly cachyosImageId: string;
};

/**
 A derived base image and how it was reached.
 */
export type TestBase = {
  /**
   Content-addressed ID of the derived base image, which every stage of the gate image starts from.
   */
  readonly id: string;
  /**
   Tag of the derived base image, which names the CachyOS image and the Containerfile it came from.
   */
  readonly tag: string;
  /**
   Whether this run built the derived base; false when an earlier run already had.
   */
  readonly built: boolean;
} & PulledCachyos;

/**
 Short lowercase hexadecimal hash of a text or an image ID, for use in an image tag.

 @param text - text to hash; an image ID is hashed as its digits

 @returns the first digits of the SHA-256 of the text
 */
function shortHash(text: string): string {
  /**
   Full digest of the text.
   */
  const digest = createHash('sha256')
    .update(text)
    .digest('hex');
  return digest.slice(
    0,
    tagHashLength,
  );
}

/**
 Name the derived base image after its inputs: the pulled CachyOS image, the Containerfile and the arguments the build is
 given.

 @param repository - image repository the tag belongs to

 @param cachyosImageId - ID of the CachyOS image the pull left on the host, with or without the `sha256:` prefix

 @param containerfileText - text of the derived base's Containerfile

 @param buildArgumentsText - text of the arguments the build is given, which carry files the build reads instead of
 copying them; empty when the build takes none, which leaves the tag the one the Containerfile text alone produces

 @returns the image reference; it changes when an input changes and only then

 @example
 ```ts
 const tag = testBaseTag({ repository: 'localhost/x', cachyosImageId: '4872c09919c6...', containerfileText: 'FROM x\n' });
 ```
 */
export function testBaseTag({
  repository,
  cachyosImageId,
  containerfileText,
  buildArgumentsText = '',
}: {
  readonly repository: string;
  readonly cachyosImageId: string;
  readonly containerfileText: string;
  readonly buildArgumentsText?: string;
},): string {
  /**
   Image ID digits, so the same image is named the same with or without the `sha256:` prefix.
   */
  const digits = cachyosImageId.startsWith(digestPrefix)
    ? cachyosImageId.slice(digestPrefix.length)
    : cachyosImageId;
  /**
   Hash of the build's inputs: the Containerfile alone for a build that is given no arguments, so a tag recorded before
   the arguments existed still names the same image, and the arguments added after a newline otherwise.
   */
  const inputsHash = buildArgumentsText === ''
    ? shortHash(containerfileText,)
    : shortHash(`${containerfileText}\n${buildArgumentsText}`,);
  return `${repository}:${digits.slice(
    0,
    tagHashLength,
  )}-${inputsHash}`;
}

/**
 Ask the repository's resolver which CachyOS image this host's CPU runs.

 @returns the image reference on the `latest` tag

 @throws NativeVerificationError when the host is not x86-64, because this base has no Alpine variant, or the resolver prints nothing
 */
async function resolveCachyosImage(): Promise<string> {
  /**
   Result of the resolver; its trail goes to the terminal's error stream and its stdout holds only the image.
   */
  const resolved = await runCommand({
    command: 'node',
    args: [
      resolverPath,
      '--image-name',
      resolverImageName,
    ],
    capture: true,
  });
  /**
   The reference the resolver printed.
   */
  const image = resolved.stdout
    .trim();
  if (image === '')
    throw new NativeVerificationError('The CachyOS image resolver printed no image.');
  return image;
}

/**
 Read the content-addressed ID of an image held on the host.

 @param image - image reference or ID

 @returns the full ID without the `sha256:` prefix as the only member, or an empty list when the host holds no such image
 */
async function localImageIds(image: string): Promise<readonly string[]> {
  /**
   Result of inspecting the image; a missing image is a normal answer here.
   */
  const inspected = await runCommand({
    command: 'podman',
    args: [
      'image',
      'inspect',
      image,
      '--format',
      '{{.Id}}',
    ],
    capture: true,
    allowFailure: true,
  });
  if (inspected.status !== 0)
    return [];
  /**
   The printed ID.
   */
  const id = inspected.stdout
    .trim();
  return [
    id.startsWith(digestPrefix)
      ? id.slice(digestPrefix.length)
      : id,
  ];
}

/**
 Pull the `latest` tag so the host holds the registry's current image, and report which image that is.

 `--policy=newer` asks the registry for the tag's digest and downloads only a different image, so an unchanged image
 costs one small request. A registry that cannot be reached is not a verdict on the image the host already holds: the
 run continues on it and says so, because a gate that cannot start offline would only teach people to stop running it.

 @returns the CachyOS image this host's CPU runs and the ID of the copy the host holds after the pull

 @throws NativeVerificationError when the host is not x86-64, or the pull fails and the host holds no copy of the image

 @example
 ```ts
 const source = await pullCachyosLatest();
 ```
 */
export async function pullCachyosLatest(): Promise<PulledCachyos> {
  /**
   CachyOS image this host's CPU runs.
   */
  const cachyosImage = await resolveCachyosImage();
  /**
   Result of the pull; a failure is judged below against what the host already holds.
   */
  const pulled = await runCommand({
    command: 'podman',
    args: [
      'pull',
      '--quiet',
      '--policy=newer',
      cachyosImage,
    ],
    capture: true,
    allowFailure: true,
  });
  /**
   The ID of the image the host holds, whatever the pull said.
   */
  const [cachyosImageId] = await localImageIds(cachyosImage);
  if (cachyosImageId === undefined)
    throw new NativeVerificationError(`Pulling ${cachyosImage} failed (status ${String(pulled.status ?? 'none')}) and the host holds no copy. ${pulled.stderr}`);
  if (pulled.status !== 0)
    console.warn(`Pulling ${cachyosImage} failed (status ${String(pulled.status ?? 'none')}); continuing with the copy the host holds, which may not be the registry's latest. ${pulled.stderr}`);
  return {
    cachyosImage,
    cachyosImageId,
  };
}

/**
 Build a derived base image on the pulled CachyOS image, with network for the package manager.

 @param tag - reference to give the image

 @param containerfilePath - Containerfile that reads the base through a build argument

 @param cachyosImage - CachyOS image reference passed as the Containerfile's base argument

 @param limits - container limits of the build

 @param buildArguments - `NAME=value` arguments the Containerfile reads, which carry a file the build context lacks

 @throws NativeVerificationError when the build fails
 */
async function buildDerivedImage({
  tag,
  containerfilePath,
  cachyosImage,
  limits,
  buildArguments,
}: {
  readonly tag: string;
  readonly containerfilePath: string;
  readonly cachyosImage: string;
  readonly limits: ContainerLimits;
  readonly buildArguments: readonly string[];
},): Promise<void> {
  // Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
  // whose backing device is unknowable.
  /**
   Parent of the empty build context.
   */
  const scratchBase = join(
    import.meta.dirname,
    '..',
    'node_modules',
    '.monochromatic',
    'git-policy',
  );
  await mkdir(
    scratchBase,
    { recursive: true },
  );
  // The Containerfiles copy nothing, so an empty directory is the whole context.
  /**
   Empty build context, removed when the build ends.
   */
  await using context = await mkdtempDisposable(join(
    scratchBase,
    'monochromatic-git-policy-cli-base-',
  ));
  await runCommand({
    command: 'podman',
    args: [
      'build',
      '--pull=never',
      '--build-arg',
      `CACHYOS_IMAGE=${cachyosImage}`,
      ...buildArguments.flatMap(function buildArgument(argument) {
        return [
          '--build-arg',
          argument,
        ];
      }),
      ...buildLimitArguments(limits),
      '--tag',
      tag,
      '--file',
      containerfilePath,
      context.path,
    ],
  });
}

/**
 Make sure the derived image for a pulled CachyOS image exists, building it only when this host has not yet.

 @param repository - image repository of the derived image

 @param containerfilePath - Containerfile the derived image is built from

 @param source - the CachyOS image the derived image starts from

 @param limits - container limits of a build, if one is needed

 @param buildArguments - `NAME=value` arguments the build is given, which the tag names too

 @returns the derived image and whether this run built it

 @throws NativeVerificationError when the build fails
 */
async function ensureDerivedImage({
  repository,
  containerfilePath,
  source,
  limits,
  buildArguments,
}: {
  readonly repository: string;
  readonly containerfilePath: string;
  readonly source: PulledCachyos;
  readonly limits: ContainerLimits;
  readonly buildArguments: readonly string[];
},): Promise<TestBase> {
  /**
   Reference of the derived image for these inputs.
   */
  const tag = testBaseTag({
    repository,
    cachyosImageId: source.cachyosImageId,
    containerfileText: await readFile(
      containerfilePath,
      'utf8',
    ),
    buildArgumentsText: buildArguments.join('\n'),
  });
  /**
   ID of a derived image an earlier run built, when there is one.
   */
  const [existing] = await localImageIds(tag);
  if (existing !== undefined) {
    return {
      ...source,
      id: existing,
      tag,
      built: false,
    };
  }
  await buildDerivedImage({
    tag,
    containerfilePath,
    cachyosImage: source.cachyosImage,
    limits,
    buildArguments,
  });
  /**
   ID of the image just built.
   */
  const [id] = await localImageIds(tag);
  if (id === undefined)
    throw new NativeVerificationError(`The build of ${tag} finished but the host holds no such image.`);
  return {
    ...source,
    id,
    tag,
    built: true,
  };
}

/**
 Make sure the gate's base image for the registry's current CachyOS image exists, and return its ID.

 @param limits - container limits of a build, if one is needed

 @param source - the pulled CachyOS image; pulled now when absent

 @returns the gate's base image and whether this run built it

 @throws NativeVerificationError when the host is not x86-64, the pull fails with no local copy, or the build fails

 @example
 ```ts
 const base = await ensureTestBase({ limits: containerLimits });
 console.log(base.id);
 ```
 */
export async function ensureTestBase({
  limits,
  source,
}: {
  readonly limits: ContainerLimits;
  readonly source?: PulledCachyos;
},): Promise<TestBase> {
  /**
   Text of the browserslist data lock, which the build reads through an argument: the Containerfile copies nothing from
   its build context.
   */
  const browserslistDataLockText = await readFile(
    browserslistDataLockPath,
    'utf8',
  );
  return ensureDerivedImage({
    repository: gateRepository,
    containerfilePath: gateContainerfilePath,
    source: source ?? await pullCachyosLatest(),
    limits,
    // The lock's text is a build argument because the Containerfile copies nothing from its build context, and the tag
    // names it too: a changed lock builds a new base instead of leaving a cache the new lock no longer matches.
    buildArguments: [`${browserslistDataLockArgument}=${browserslistDataLockText}`],
  });
}

/**
 Make sure the end-to-end suite's base image for the registry's current CachyOS image exists, and return its ID.

 @param limits - container limits of a build, if one is needed

 @param source - the pulled CachyOS image; pulled now when absent

 @returns the end-to-end base image and whether this run built it

 @throws NativeVerificationError when the host is not x86-64, the pull fails with no local copy, or the build fails

 @example
 ```ts
 const base = await ensureEndToEndBase({ limits: containerLimits });
 console.log(base.tag);
 ```
 */
export async function ensureEndToEndBase({
  limits,
  source,
}: {
  readonly limits: ContainerLimits;
  readonly source?: PulledCachyos;
},): Promise<TestBase> {
  return ensureDerivedImage({
    repository: endToEndRepository,
    containerfilePath: endToEndContainerfilePath,
    source: source ?? await pullCachyosLatest(),
    limits,
    buildArguments: [],
  });
}
