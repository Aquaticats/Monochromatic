/**
 Command-line core of the `cachyos-image` bin: prints the CachyOS base image for this host so shell
 callers can hand it to `podman build`.

 Usage: `cachyos-image --image-name <name> [--alpine-variant] [--build-argument]`
 */

import { parseArgs, } from 'node:util';
import {
  CACHYOS_IMAGE_BUILD_ARGUMENT_NAME,
  NoAlpineVariantError,
  resolveBaseImage,
  type ResolveBaseImageOptions,
} from './cachyos-image.ts';

/**
 Usage text shown for malformed invocations.
 */
export const CACHYOS_IMAGE_USAGE: string = `usage: cachyos-image --image-name <name> [--alpine-variant] [--build-argument]

Prints the unpinned base image for a container build on this host:
docker.io/cachyos/cachyos-v4:latest, else -v3, else the plain image on x86-64 (by the glibc loader's
CPU level report), and Alpine on any other architecture when --alpine-variant says the image has an Alpine variant.
Without --alpine-variant, a host that is not x86-64 fails and names <name>.
--build-argument prints ${CACHYOS_IMAGE_BUILD_ARGUMENT_NAME}=<image> instead of the bare image.`;

/**
 Outcome of one CLI run, returned instead of written so tests can read it.
 */
export type CachyosImageCliResult = {
  /**
   Text for stdout, empty on failure.
   */
  readonly stdout: string;
  /**
   Text for stderr, empty on success.
   */
  readonly stderr: string;
  /**
   Process exit code: 0 on success, 2 for usage errors, 1 when no base image exists for the host.
   */
  readonly exitCode: 0 | 1 | 2;
};

/**
 Parsed command line.
 */
type ParsedCliArguments = {
  /**
   Value of `--image-name`; empty when missing.
   */
  readonly imageName: string;
  /**
   Whether `--alpine-variant` was given.
   */
  readonly alpineVariant: boolean;
  /**
   Whether `--build-argument` was given.
   */
  readonly buildArgument: boolean;
};

/**
 Parses the command line.

 @param args - Command-line arguments after the program name

 @returns the parsed arguments

 @throws TypeError when an option is unknown or lacks its value
 */
function parseCliArguments(args: readonly string[],): ParsedCliArguments {
  /**
   Option values from `node:util` `parseArgs`, which throws on unknown options.
   */
  const { values, } = parseArgs({
    args: [...args,],
    options: {
      'image-name': { type: 'string', },
      'alpine-variant': { type: 'boolean', },
      'build-argument': { type: 'boolean', },
    },
    strict: true,
  },);

  return {
    imageName: values['image-name'] ?? '',
    alpineVariant: values['alpine-variant'] === true,
    buildArgument: values['build-argument'] === true,
  };
}

/**
 Runs the `cachyos-image` command.

 @param args - Command-line arguments after the program name

 @param resolveOptions - Injectable host inputs (`arch`, `readLoader`, `l`) forwarded to `resolveBaseImage`

 @returns stdout, stderr and the exit code

 @example
 ```ts
 const result = await runCachyosImageCli({ args: ['--image-name', 'terminal',], });
 // result.stdout: 'docker.io/cachyos/cachyos-v4:latest\n'
 ```
 */
export async function runCachyosImageCli({
  args,
  resolveOptions = {},
}: {
  readonly args: readonly string[];
  readonly resolveOptions?: Omit<ResolveBaseImageOptions, 'alpineVariant' | 'imageName'>;
},): Promise<CachyosImageCliResult> {
  try {
    /**
     Command line as options.
     */
    const parsed = parseCliArguments(args,);

    if (parsed.imageName === '') {
      return {
        stdout: '',
        stderr: `--image-name is required\n${CACHYOS_IMAGE_USAGE}\n`,
        exitCode: 2,
      };
    }

    /**
     Image chosen for this host.
     */
    const { image, } = await resolveBaseImage({
      ...resolveOptions,
      imageName: parsed.imageName,
      alpineVariant: parsed.alpineVariant,
    },);

    return {
      stdout: parsed.buildArgument
        ? `${CACHYOS_IMAGE_BUILD_ARGUMENT_NAME}=${image}\n`
        : `${image}\n`,
      stderr: '',
      exitCode: 0,
    };
  }
  catch (error) {
    if (error instanceof NoAlpineVariantError) {
      return {
        stdout: '',
        stderr: `${error.message}\n`,
        exitCode: 1,
      };
    }

    // parseArgs reports unknown options and missing option values as TypeError.
    if (error instanceof TypeError) {
      return {
        stdout: '',
        stderr: `${error.message}\n${CACHYOS_IMAGE_USAGE}\n`,
        exitCode: 2,
      };
    }

    throw error;
  }
}
