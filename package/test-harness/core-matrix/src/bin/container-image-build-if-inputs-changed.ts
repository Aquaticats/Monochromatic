#!/usr/bin/env node
/**
 `container-image-build-if-inputs-changed`: builds a local container image when it is missing or was built from other inputs.

 Usage: `container-image-build-if-inputs-changed --image <name> --containerfile <file> [--context <dir>] [--base-image <name>]...
 [--bounding-flag=<flag>]... [--force] -- <podman build argument>...`

 The words after `--` are the `--build-arg NAME=value` pairs, passed to `podman build` as they are and digested together
 with the identity of each `--base-image` (a local image the Containerfile starts from). The image carries the digest as a
 label, so the next run builds again exactly when a build argument or a base image changed. `--bounding-flag=<flag>` passes
 one flag that limits the build (`--bounding-flag=--memory=4g`) without entering the digest. `--context` defaults to `.`.
 `--force` builds even when the image is current, which an explicit `image` task wants. Exit status: 0 when the image is
 current or built, 1 when the build fails, 2 for a malformed command line.
 */

import { parseArgs, } from 'node:util';

import {
  buildContainerImage,
  ensureContainerImage,
  type ContainerImageInputs,
} from '../container-image-rebuild-on-changed-inputs.ts';

/**
 Shortest correct command line, printed with every usage error.
 */
const USAGE = 'usage: container-image-build-if-inputs-changed --image <name> --containerfile <file> [--context <dir>] '
  + '[--base-image <name>]... [--bounding-flag=<flag>]... [--force] -- <podman build argument>...';

/**
 Why the command line is malformed.
 */
class UsageError extends Error {
  /**
   Builds the error.

   @param message - what is wrong with the command line
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'UsageError';
  }
}

/**
 Reads the command line into the inputs of one image build.

 @param args - words after the program name

 @returns the build inputs and whether `--force` was given

 @throws UsageError when an option is unknown, lacks its value, or a required option is missing
 */
function parseCommandLine(args: readonly string[],): {
  readonly inputs: ContainerImageInputs;
  readonly force: boolean
} {
  try {
    /**
     Options and the words after `--`.
     */
    const {
      values,
      positionals,
    } = parseArgs({
      args: [...args,],
      options: {
        'image': { type: 'string', },
        'containerfile': { type: 'string', },
        'context': {
          type: 'string',
          default: '.',
        },
        'base-image': {
          type: 'string',
          multiple: true,
          default: [],
        },
        'bounding-flag': {
          type: 'string',
          multiple: true,
          default: [],
        },
        'force': {
          type: 'boolean',
          default: false,
        },
      },
      allowPositionals: true,
      strict: true,
    },);
    if ((values.image === undefined) || (values.containerfile === undefined)) {
      throw new UsageError('--image and --containerfile are required',);
    }
    return {
      inputs: {
        image: values.image,
        containerfile: values.containerfile,
        contextDirectory: values.context,
        buildArguments: positionals,
        boundingFlags: values['bounding-flag'],
        baseImages: values['base-image'],
      },
      force: values.force,
    };
  }
  catch (error) {
    if (error instanceof UsageError) {
      throw error;
    }
    throw new UsageError(String(error,),);
  }
}

try {
  /**
   What to build and whether to build regardless.
   */
  const {
    inputs,
    force,
  } = parseCommandLine(process.argv
    .slice(2,),);
  try {
    await (force ? buildContainerImage({ inputs, },) : ensureContainerImage({ inputs, },));
  }
  catch (error) {
    process.stderr
      .write(`${String(error,)}\n`,);
    process.exitCode = 1;
  }
}
catch (error) {
  if (!(error instanceof UsageError)) {
    throw error;
  }
  process.stderr
    .write(`${error.message}\n${USAGE}\n`,);
  process.exitCode = 2;
}
