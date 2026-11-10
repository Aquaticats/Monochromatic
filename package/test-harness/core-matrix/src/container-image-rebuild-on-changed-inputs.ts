/**
 Rebuilds a local container image when the inputs it was built from have changed.

 Rationale: the image tasks of `package/terminal/app`, `package/music-player/desktop-app` and `package/ide/app` built an
 image only when none existed, so an image kept the Zig and `cargo-about` it was built with while
 `toolchain.lock.json` moved on. Each image now carries a label holding a digest of its build arguments (and the
 identities of the local images it starts from), and a task builds again whenever the label differs from the digest of the
 arguments the task would pass now.
 */

import {
  execFile,
  spawn,
} from 'node:child_process';
import { createHash, } from 'node:crypto';
import { once, } from 'node:events';
import { promisify, } from 'node:util';

import {
  imageBuildLimitArguments,
  type ImageBuildLimits,
} from '@monochromatic-dev/container-limit-core/ts';
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

/**
 Name of the image label that holds the digest of the inputs the image was built from.
 */
export const IMAGE_BUILD_INPUTS_LABEL_NAME = 'dev.monochromatic.image-build-inputs-sha256';

/**
 Bound of an image build whose caller gave none: 4 GiB and 4 CPUs, the bound `package/terminal/app` has put on the build
 of its Ghostty and Slint image.
 Adopted for the CachyOS images built here, not measured.
 `podman build` has no process or disk-weight option, so the bound is memory and the CFS quota only.
 */
export const DEFAULT_IMAGE_BUILD_LIMITS: ImageBuildLimits = {
  memory: '4g',
  cpus: 4,
};

/**
 Whether a build flag sets the memory bound.

 @param flag - one bounding flag

 @returns true for `--memory` in either spelling
 */
function setsMemory(flag: string,): boolean {
  return flag.startsWith('--memory',);
}

/**
 Flags that bound one image build.

 @param boundingFlags - the flags the caller gave

 @returns the caller's flags when they bound the memory themselves, else the default bound ahead of them

 @example
 ```ts
 const flags = boundingFlagsOf([]);
 ```
 */
export function boundingFlagsOf(boundingFlags: readonly string[],): readonly string[] {
  return boundingFlags.some(setsMemory,)
    ? boundingFlags
    : [
      ...imageBuildLimitArguments(DEFAULT_IMAGE_BUILD_LIMITS,),
      ...boundingFlags,
    ];
}

/**
 The container engine operations this module needs, so a test can stand in for `podman`.
 */
export type ContainerEngine = {
  /**
   Whether a local image with this name exists.
   */
  readonly hasImage: (image: string,) => Promise<boolean>;
  /**
   Identity of a local image, which changes whenever the image is built again.
   */
  readonly idOf: (image: string,) => Promise<string>;
  /**
   Value of one label of a local image, the empty string when the image carries no such label.
   */
  readonly labelOf: (labelQuery: ImageLabelQuery,) => Promise<string>;
  /**
   Runs `build` with these arguments, showing the engine's output, and rejects when it fails.
   */
  readonly build: (buildCommandArguments: readonly string[],) => Promise<void>;
};

/**
 Which label of which local image to read.
 */
export type ImageLabelQuery = {
  /**
   Name of the local image.
   */
  readonly image: string;
  /**
   Name of the label.
   */
  readonly name: string;
};

/**
 What one image build is made from.
 */
export type ContainerImageInputs = {
  /**
   Name the image is tagged with.
   */
  readonly image: string;
  /**
   Containerfile to build, relative to `contextDirectory`.
   */
  readonly containerfile: string;
  /**
   Build context directory, `.` for the caller's working directory.
   */
  readonly contextDirectory: string;
  /**
   The `--build-arg NAME=value` pairs that change what the build installs, which the digest covers.
   */
  readonly buildArguments: readonly string[];
  /**
   Flags that bound the build (CPU, memory) but do not change what it installs, which the digest ignores.
   */
  readonly boundingFlags: readonly string[];
  /**
   Local images the Containerfile starts from, whose identities the digest covers.
   */
  readonly baseImages: readonly string[];
};

/**
 Options of the functions that work on {@link ContainerImageInputs}.
 */
export type ContainerImageOptions = {
  /**
   What the image is built from.
   */
  readonly inputs: ContainerImageInputs;
  /**
   The container engine; defaults to `podman` through {@link podmanEngineWith}.
   */
  readonly engine?: ContainerEngine;
  /**
   Logger; defaults to the shared logger.
   */
  readonly l?: Logger;
};

/**
 Runs `podman` and captures its output.

 @param podmanArguments - arguments after `podman`

 @param l - logger that records a failed run

 @returns standard output with the final newline removed

 @throws Error when `podman` cannot start or exits non-zero
 */
async function podmanOutput({
  podmanArguments,
  l,
}: {
  readonly podmanArguments: readonly string[];
  readonly l: Logger;
},): Promise<string> {
  try {
    /**
     The finished process.
     */
    const finished = await execFileAsync(
      'podman',
      podmanArguments,
      { encoding: 'utf8', },
    );
    return finished.stdout
      .trimEnd();
  }
  catch (error) {
    l.debug(`podman ${podmanArguments.join(' ',)} failed: ${String(error,)}`,);
    throw error;
  }
}

/**
 Runs `podman` and tells whether it exited zero.

 @param podmanArguments - arguments after `podman`

 @param l - logger that records a failed run

 @returns `true` when `podman` exited zero
 */
async function podmanSucceeds({
  podmanArguments,
  l,
}: {
  readonly podmanArguments: readonly string[];
  readonly l: Logger;
},): Promise<boolean> {
  try {
    await podmanOutput({
      podmanArguments,
      l,
    },);
    return true;
  }
  catch (error) {
    l.debug(`podman ${podmanArguments.join(' ',)} did not succeed: ${String(error,)}`,);
    return false;
  }
}

/**
 Runs `podman build`, showing its output on the caller's terminal.

 @param buildCommandArguments - arguments after `podman build`

 @throws Error when the build exits non-zero or is killed
 */
async function buildWithPodman(buildCommandArguments: readonly string[],): Promise<void> {
  /**
   The build, with its output shown on the caller's terminal.
   */
  const child = spawn(
    'podman',
    [
      'build',
      ...buildCommandArguments,
    ],
    { stdio: 'inherit', },
  );
  await once(
    child,
    'close',
  );
  if (child.exitCode !== 0) {
    throw new Error(`podman build ${buildCommandArguments.join(' ',)} ended with ${
      child.signalCode === null ? `exit ${String(child.exitCode,)}` : `signal ${child.signalCode}`
    }`,);
  }
}

/**
 `podman` as the container engine.

 @param l - logger that records failed `podman` runs; defaults to the shared logger

 @returns the engine

 @example
 ```ts
 const engine = podmanEngineWith();
 ```
 */
export function podmanEngineWith({ l: parentLogger = defaultLogger, }: { readonly l?: Logger; } = {},): ContainerEngine {
  /**
   Logger tagged with this function.
   */
  const l: Logger = tagged({
    tag: podmanEngineWith.name,
    l: parentLogger,
  },);
  return {
    hasImage: function hasImage(image: string,): Promise<boolean> {
      return podmanSucceeds({
        podmanArguments: [
          'image',
          'exists',
          image,
        ],
        l,
      },);
    },
    idOf: function idOf(image: string,): Promise<string> {
      return podmanOutput({
        podmanArguments: [
          'image',
          'inspect',
          '--format',
          '{{.Id}}',
          image,
        ],
        l,
      },);
    },
    labelOf: async function labelOf({
      image,
      name,
    }: ImageLabelQuery,): Promise<string> {
      try {
        return await podmanOutput({
          podmanArguments: [
            'image',
            'inspect',
            '--format',
            `{{index .Config.Labels "${name}"}}`,
            image,
          ],
          l,
        },);
      }
      catch (error) {
        l.debug(`could not read label ${name} of ${image}: ${String(error,)}`,);
        return '';
      }
    },
    build: buildWithPodman,
  };
}

/**
 Marks a local base image that does not exist, so a missing base changes the digest.
 */
const MISSING_BASE_IMAGE_ID = 'missing';

/**
 Digests what an image is built from.

 @param inputs - what the image is built from

 @param engine - container engine that reads the base images

 @returns lowercase hexadecimal SHA-256 of the build arguments and the identities of the base images

 @example
 ```ts
 await inputsDigestOf({ inputs, engine: podmanEngineWith(), },);
 // '3581863e3cbcfd0fce0efc682ce7e9e93da96668a977791d356a26d2a7e9b17a'
 ```
 */
export async function inputsDigestOf({
  inputs,
  engine,
}: {
  readonly inputs: ContainerImageInputs;
  readonly engine: ContainerEngine;
},): Promise<string> {
  /**
   Identity of each base image, in the order the inputs list them.
   */
  const baseImageIds = await Promise.all(inputs.baseImages
    .map(async function idOfBase(baseImage: string,): Promise<string> {
      return await engine.hasImage(baseImage,) ? await engine.idOf(baseImage,) : MISSING_BASE_IMAGE_ID;
    },),);
  return createHash('sha256',)
    .update(JSON.stringify({
      buildArguments: inputs.buildArguments,
      baseImageIds,
    },),)
    .digest('hex',);
}

/**
 Builds the image unconditionally and labels it with the digest of its inputs.

 @param inputs - what the image is built from

 @param l - logger; defaults to the shared logger

 @param engine - container engine; defaults to `podman`

 @throws Error when the engine's build fails

 @example
 ```ts
 await buildContainerImage({ inputs, },);
 ```
 */
export async function buildContainerImage({
  inputs,
  l: parentLogger = defaultLogger,
  engine = podmanEngineWith({ l: parentLogger, },),
}: ContainerImageOptions,): Promise<void> {
  /**
   Logger tagged with this function.
   */
  const l: Logger = tagged({
    tag: buildContainerImage.name,
    l: parentLogger,
  },);
  /**
   Digest the image is labelled with.
   */
  const digest = await inputsDigestOf({
    inputs,
    engine,
  },);
  l.info(`building ${inputs.image} with inputs digest ${digest}`,);
  await engine.build([
    ...boundingFlagsOf(inputs.boundingFlags,),
    ...inputs.buildArguments,
    '--label',
    `${IMAGE_BUILD_INPUTS_LABEL_NAME}=${digest}`,
    '--tag',
    inputs.image,
    '--file',
    inputs.containerfile,
    inputs.contextDirectory,
  ],);
}

/**
 Tells whether the local image was built from the inputs a build would use now.

 @param inputs - what the image is built from

 @param l - logger; defaults to the shared logger

 @param engine - container engine; defaults to `podman`

 @returns `true` when the image exists and its label equals the digest of the current inputs

 @example
 ```ts
 await isContainerImageCurrent({ inputs, },);
 // false when the image is missing, unlabelled, or built from other inputs
 ```
 */
export async function isContainerImageCurrent({
  inputs,
  l: parentLogger = defaultLogger,
  engine = podmanEngineWith({ l: parentLogger, },),
}: ContainerImageOptions,): Promise<boolean> {
  /**
   Logger tagged with this function.
   */
  const l: Logger = tagged({
    tag: isContainerImageCurrent.name,
    l: parentLogger,
  },);
  if (!await engine.hasImage(inputs.image,)) {
    l.info(`${inputs.image} does not exist`,);
    return false;
  }

  /**
   Digest the image carries, empty when it was built before images were labelled.
   */
  const labelled = await engine.labelOf({
    image: inputs.image,
    name: IMAGE_BUILD_INPUTS_LABEL_NAME,
  },);
  /**
   Digest of the inputs a build would use now.
   */
  const current = await inputsDigestOf({
    inputs,
    engine,
  },);
  if (labelled !== current) {
    l.info(`${inputs.image} carries inputs digest "${labelled}", the current inputs digest is ${current}`,);
  }
  return labelled === current;
}

/**
 Builds the image unless it is already current.

 @param inputs - what the image is built from

 @param l - logger; defaults to the shared logger

 @param engine - container engine; defaults to `podman`

 @returns whether a build ran

 @throws Error when the engine's build fails

 @example
 ```ts
 await ensureContainerImage({ inputs, },);
 // true when the image was missing or stale and a build ran
 ```
 */
export async function ensureContainerImage({
  inputs,
  l: parentLogger = defaultLogger,
  engine = podmanEngineWith({ l: parentLogger, },),
}: ContainerImageOptions,): Promise<boolean> {
  if (await isContainerImageCurrent({
    inputs,
    l: parentLogger,
    engine,
  },)) {
    return false;
  }
  await buildContainerImage({
    inputs,
    l: parentLogger,
    engine,
  },);
  return true;
}
