/**
 CachyOS base image selection by host CPU level.

 Per the owner's 2026-10-29 container base distro decision
 (`doc/decision/container-base-distro.md`), a container this repo builds starts from the
 most optimized CachyOS image the host CPU can run: `cachyos-v4`, else `cachyos-v3`, else
 plain `cachyos`. CachyOS publishes no v2 image. Hosts that are not x86-64 start from
 Alpine, but only for a caller that has an Alpine variant of its image; every other caller
 gets a {@link NoAlpineVariantError} instead of a silently wrong build.

 The CPU level comes from glibc's dynamic loader, the method the CachyOS `cachyos-repo.sh`
 installer uses (`ld-linux-x86-64.so.2 --help` and a match on `x86-64-v4 (supported, searched)`).
 */

import { execFile, } from 'node:child_process';
import { promisify, } from 'node:util';
import {
  logger as defaultLogger,
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 The generic `promisify(execFile)` form trips `strict-void-return`;
 see `doc/troubleshooting/oxlint-promisify-void-return.md`.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 `execFile` with a promise result, which never goes through a shell.
 */
const execFileAsync = promisifyExecFile(execFile,);

//region Constants

/**
 CPU level of a CachyOS image: `v4` and `v3` name the x86-64 microarchitecture levels, `plain` the baseline.
 */
export type CachyosLevel = 'v4' | 'v3' | 'plain';

/**
 Which base image a resolution picked.
 - `cachyos-v4`, `cachyos-v3`, `cachyos`: the CachyOS image for that CPU level
 - `alpine`: the fallback for hosts that are not x86-64
 */
export type BaseImageKind = 'cachyos-v4' | 'cachyos-v3' | 'cachyos' | 'alpine';

/**
 Result of choosing a base image.
 */
export type BaseImageResolution = {
  /**
   Fully qualified, unpinned image reference.
   */
  readonly image: string;
  /**
   Which image family the reference belongs to.
   */
  readonly kind: BaseImageKind;
  /**
   Why this image was chosen, including any fallback cause.
   */
  readonly reason: string;
};

/**
 CachyOS image per CPU level, all unpinned (the owner requires `latest`).
 */
export const CACHYOS_IMAGE_BY_LEVEL: Readonly<Record<CachyosLevel, string>> = {
  v4: 'docker.io/cachyos/cachyos-v4:latest',
  v3: 'docker.io/cachyos/cachyos-v3:latest',
  plain: 'docker.io/cachyos/cachyos:latest',
} as const;

/**
 Image kind per CPU level.
 */
const KIND_BY_LEVEL: Readonly<Record<CachyosLevel, BaseImageKind>> = {
  v4: 'cachyos-v4',
  v3: 'cachyos-v3',
  plain: 'cachyos',
} as const;

/**
 Alpine image used on hosts that are not x86-64, unpinned.
 */
export const ALPINE_IMAGE = 'docker.io/library/alpine:latest';

/**
 Build argument name Containerfiles read the CachyOS base image from.
 */
export const CACHYOS_IMAGE_BUILD_ARGUMENT_NAME = 'CACHYOS_IMAGE';

/**
 Loader locations, in order of preference. `/lib64` is the path on Fedora, Ubuntu, Debian and Arch
 (where it links to `/usr/lib`); `/lib` is the path the CachyOS installer script reads.
 */
const LOADER_PATHS: readonly string[] = [
  '/lib64/ld-linux-x86-64.so.2',
  '/lib/ld-linux-x86-64.so.2',
] as const;

/**
 Wall-clock cap for one loader run, in milliseconds. `--help` returns at once.
 */
const LOADER_TIMEOUT_MS = 10_000;

/**
 Output cap for one loader run, in bytes. `--help` prints about a kilobyte.
 */
const LOADER_MAX_BUFFER_BYTES = 1_048_576;

/**
 Suffix the loader prints after a level it supports and searches for libraries.
 */
const SUPPORTED_SUFFIX = ' (supported, searched)';

/**
 Prefix of every level line the loader prints.
 */
const LEVEL_LINE_PREFIX = 'x86-64-v';

//endregion Constants

//region Errors

/**
 A pacman-only image was requested on a host with no CachyOS image.

 Names the image so the person reading the failure knows which build to change.
 */
export class NoAlpineVariantError extends Error {
  /**
   Name of the image that has no Alpine variant.
   */
  readonly #imageName: string;

  /**
   Architecture of the host that has no CachyOS image.
   */
  readonly #arch: string;

  /**
   @param imageName - What was being built (a Containerfile or image name), for the message

   @param arch - `process.arch` value of the host
   */
  constructor({
    imageName,
    arch,
  }: {
    readonly imageName: string;
    readonly arch: string;
  },) {
    super(
      `Cannot build "${imageName}" on a "${arch}" host: CachyOS publishes images for x86-64 only, `
        + `and "${imageName}" has no Alpine variant to fall back to. `
        + `Build it on an x86-64 host, or write an Alpine variant of "${imageName}" `
        + `and resolve its base image with alpineVariant set to true.`,
    );
    this.name = 'NoAlpineVariantError';
    this.#imageName = imageName;
    this.#arch = arch;
  }

  /**
   Name of the image that has no Alpine variant.

   @returns the image name given at construction
   */
  get imageName(): string {
    return this.#imageName;
  }

  /**
   Architecture of the host that has no CachyOS image.

   @returns the architecture given at construction
   */
  get arch(): string {
    return this.#arch;
  }
}

/**
 No glibc loader at a known path produced a level report (a non-glibc host such as Alpine, a missing file, or a failed run).
 */
export class LoaderUnavailableError extends Error {
  /**
   @param detail - What was tried and why each attempt failed
   */
  constructor(detail: string,) {
    super(`No glibc loader produced an x86-64 level report: ${detail}`,);
    this.name = 'LoaderUnavailableError';
  }
}

//endregion Errors

//region CPU level detection

/**
 Reads the x86-64 level report of the host's glibc dynamic loader.

 @param l - Logger; defaults to the shared logger

 @returns the loader's `--help` output

 @throws LoaderUnavailableError when no loader at a known path ran successfully

 @example
 ```ts
 const report = await readLoaderOutput();
 ```
 */
export async function readLoaderOutput({ l: parentLogger = defaultLogger, }: {
  readonly l?: Logger;
} = {},): Promise<string> {
  /**
   Logger tagged with this function.
   */
  const l: Logger = tagged({
    tag: readLoaderOutput.name,
    l: parentLogger,
  },);
  /**
   Why each loader path failed, for the thrown error.
   */
  const failures: string[] = [];

  for (const loaderPath of LOADER_PATHS) {
    try {
      /* oxlint-disable no-await-in-loop -- the second path is only tried after the first fails */
      /**
       Loader run; rejects when the file is missing, exits non-zero or times out.
       */
      const run = await execFileAsync(
        loaderPath,
        ['--help',],
        {
          encoding: 'utf8',
          timeout: LOADER_TIMEOUT_MS,
          maxBuffer: LOADER_MAX_BUFFER_BYTES,
        },
      );
      /* oxlint-enable no-await-in-loop */
      l.debug(`read the level report of ${loaderPath}`,);
      return run.stdout;
    }
    catch (error) {
      /**
       Failure text for the log line and the thrown error.
       */
      const failure = `${loaderPath}: ${String(error,)}`;
      l.debug(`loader did not run: ${failure}`,);
      failures.push(failure,);
    }
  }

  throw new LoaderUnavailableError(failures.join('; ',),);
}

/**
 Result of reading the loader's level report.
 */
type LevelDetection = {
  /**
   Highest CachyOS level the report supports.
   */
  readonly level: CachyosLevel;
  /**
   Why that level was chosen.
   */
  readonly reason: string;
};

/**
 Picks the highest CachyOS image level the loader reports as supported.

 Matches the CachyOS installer: a level counts when its line ends in `(supported, searched)`.
 `x86-64-v4` wins over `x86-64-v3`; a report with level lines but neither, or with no
 level lines at all, picks `plain`, with a reason that tells the two apart.

 @param loaderOutput - Text printed by `ld-linux-x86-64.so.2 --help`

 @returns the level and the reason it was chosen

 @example
 ```ts
 detectCachyosLevel('  x86-64-v3 (supported, searched)').level; // 'v3'
 ```
 */
export function detectCachyosLevel(loaderOutput: string,): LevelDetection {
  /**
   Trimmed lines naming an `x86-64-vN` level, supported or not.
   */
  const levelLines = loaderOutput
    .split('\n',)
    .map(function trimLine(line,) {
      return line.trim();
    },)
    .filter(function isLevelLine(line,) {
      return line.startsWith(LEVEL_LINE_PREFIX,);
    },);
  /**
   Level names the loader reports as supported and searched, such as `x86-64-v4`.
   */
  const supported: ReadonlySet<string> = new Set(
    levelLines
      .filter(function isSupported(line,) {
        return line.endsWith(SUPPORTED_SUFFIX,);
      },)
      .map(function levelName(line,) {
        return line.slice(
          0,
          line.length - SUPPORTED_SUFFIX.length,
        );
      },),
  );

  if (supported.has('x86-64-v4',)) {
    return {
      level: 'v4',
      reason: 'the loader reports x86-64-v4 (supported, searched)',
    };
  }

  if (supported.has('x86-64-v3',)) {
    return {
      level: 'v3',
      reason: 'the loader reports x86-64-v3 (supported, searched) and not x86-64-v4',
    };
  }

  if (levelLines.length > 0) {
    return {
      level: 'plain',
      reason: 'the loader lists x86-64 levels but reports neither v4 nor v3 as supported and searched',
    };
  }

  return {
    level: 'plain',
    reason: 'the loader printed no x86-64 level lines',
  };
}

//endregion CPU level detection

//region Base image resolution

/**
 Runs the loader reader and turns its one expected failure into data.

 @param readLoader - Loader report reader

 @param l - Logger tagged by the caller

 @returns the report text, or the reason no report exists in `missing`

 @throws Error any failure other than {@link LoaderUnavailableError}
 */
async function readLoaderReport({
  readLoader,
  l,
}: {
  readonly readLoader: () => Promise<string>;
  readonly l: Logger;
},): Promise<{
  readonly output: string;
  readonly missing: string;
}> {
  try {
    return {
      output: await readLoader(),
      missing: '',
    };
  }
  catch (error) {
    // A missing loader falls back to the plain image; anything else is a bug and propagates.
    if (!(error instanceof LoaderUnavailableError))
      throw error;
    l.warn(`loader unavailable: ${error.message}`,);
    return {
      output: '',
      missing: error.message,
    };
  }
}

/**
 Options shared by every base image function.
 */
export type ResolveBaseImageOptions = {
  /**
   What is being built, named in the error when there is no usable base. A Containerfile path or image tag.
   */
  readonly imageName: string;
  /**
   Whether `imageName` has an Alpine variant. When `false`, a host that is not x86-64 throws
   {@link NoAlpineVariantError}; when `true`, it resolves to {@link ALPINE_IMAGE} and the
   caller is responsible for building its Alpine variant.
   */
  readonly alpineVariant: boolean;
  /**
   Host architecture, in `process.arch` spelling. Defaults to the running process's.
   */
  readonly arch?: string;
  /**
   Reader of the loader's `--help` output; it throws {@link LoaderUnavailableError} when the loader is missing.
   Defaults to {@link readLoaderOutput}.
   */
  readonly readLoader?: () => Promise<string>;
  /**
   Logger. Defaults to the shared logger.
   */
  readonly l?: Logger;
};

/**
 Chooses the base image for a container build on this host.

 On x86-64: the CachyOS image for the highest CPU level the glibc loader reports, falling
 back to plain `cachyos` (and logging why) when the loader is missing or reports no level.
 On any other architecture: Alpine when the caller has an Alpine variant of its image,
 otherwise a thrown {@link NoAlpineVariantError}.

 @param imageName - What is built, named in the error when there is no usable base

 @param alpineVariant - Whether the image has an Alpine variant

 @param arch - Host architecture; defaults to `process.arch`

 @param readLoader - Loader report reader; defaults to {@link readLoaderOutput}

 @param l - Logger; defaults to the shared logger

 @returns the chosen image, its family and the reason

 @throws NoAlpineVariantError when the host is not x86-64 and the image has no Alpine variant

 @example
 ```ts
 await resolveBaseImage({ imageName: 'localhost/monochromatic/terminal', alpineVariant: false, });
 // \{ image: 'docker.io/cachyos/cachyos-v4:latest', kind: 'cachyos-v4', reason: '...' \}
 ```
 */
export async function resolveBaseImage({
  imageName,
  alpineVariant,
  arch = process.arch,
  readLoader = readLoaderOutput,
  l: parentLogger = defaultLogger,
}: ResolveBaseImageOptions,): Promise<BaseImageResolution> {
  /**
   Logger tagged with this function.
   */
  const l: Logger = tagged({
    tag: resolveBaseImage.name,
    l: parentLogger,
  },);

  l.debug(`resolving the base image of "${imageName}" for arch "${arch}", alpineVariant ${String(alpineVariant,)}`,);

  if (arch !== 'x64') {
    if (!alpineVariant) {
      l.error(`"${imageName}" has no Alpine variant and arch "${arch}" has no CachyOS image`,);
      throw new NoAlpineVariantError({
        imageName,
        arch,
      },);
    }

    /**
     Reason recorded for the Alpine fallback.
     */
    const alpineReason = `arch "${arch}" is not x64 and CachyOS publishes x86-64 images only`;
    l.info(`${ALPINE_IMAGE}: ${alpineReason}`,);
    return {
      image: ALPINE_IMAGE,
      kind: 'alpine',
      reason: alpineReason,
    };
  }

  /**
   Loader report; empty when no loader ran.
   */
  const report = await readLoaderReport({
    readLoader,
    l,
  },);

  if (report.missing !== '') {
    /**
     Reason recorded for the plain fallback when no loader ran.
     */
    const missingReason = `no glibc loader ran, so the CPU level is unknown (${report.missing})`;
    l.warn(`${CACHYOS_IMAGE_BY_LEVEL.plain}: ${missingReason}`,);
    return {
      image: CACHYOS_IMAGE_BY_LEVEL.plain,
      kind: KIND_BY_LEVEL.plain,
      reason: missingReason,
    };
  }

  /**
   Highest supported level and why.
   */
  const {
    level,
    reason,
  } = detectCachyosLevel(report.output,);
  /**
   Image for that level.
   */
  const image = CACHYOS_IMAGE_BY_LEVEL[level];

  if (level === 'plain')
    l.warn(`${image}: ${reason}`,);
  else
    l.info(`${image}: ${reason}`,);

  return {
    image,
    kind: KIND_BY_LEVEL[level],
    reason,
  };
}

/**
 Builds the `podman build` arguments that hand the resolved base image to a Containerfile.

 The Containerfile declares `ARG CACHYOS_IMAGE` before its `FROM`.

 @param options - What is built, whether it has an Alpine variant, and injectable host inputs

 @returns `['--build-arg', 'CACHYOS_IMAGE=<image>']`

 @throws NoAlpineVariantError when the host is not x86-64 and the image has no Alpine variant

 @example
 ```ts
 await cachyosBuildArguments({ imageName: 'terminal', alpineVariant: false, });
 // ['--build-arg', 'CACHYOS_IMAGE=docker.io/cachyos/cachyos-v4:latest']
 ```
 */
export async function cachyosBuildArguments(options: ResolveBaseImageOptions,): Promise<readonly string[]> {
  /**
   Image chosen for this host.
   */
  const { image, } = await resolveBaseImage(options,);
  return [
    '--build-arg',
    `${CACHYOS_IMAGE_BUILD_ARGUMENT_NAME}=${image}`,
  ];
}

//endregion Base image resolution
