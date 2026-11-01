/**
 Getting the Playwright container image for `mise run prepare:playwright`.

 CI publishes `playwright.Dockerfile` as `ghcr.io/aquaticats/monochromatic-playwright:<tag>`, where `<tag>` is the
 first 16 hex digits of the SHA-256 of the file. This module computes the same tag, pulls that image, gives it the
 local names the Playwright tasks run (`monochromatic-playwright`, and `monochromatic-playwright:<tag>`), and builds the
 file locally only when the pull fails. A tag names one content, so a local `monochromatic-playwright:<tag>` is
 current exactly when the file has not changed since.

 Every effect goes through the injected `podman` function, so tests run the decisions without podman.

 @module
 */

import { createHash, } from 'node:crypto';

//region Names: the registry image, the local names and the tag

/**
 Registry repository CI pushes the image to. The workflow `.github/workflows/playwright-image.yml` names the same one.

 @example
 ```ts
 const reference = `${PLAYWRIGHT_REGISTRY_IMAGE}:${tag}`;
 ```
 */
export const PLAYWRIGHT_REGISTRY_IMAGE = 'ghcr.io/aquaticats/monochromatic-playwright';

/**
 Local image name the Playwright tasks run (`mise run test:browser` and `test:e2e`).

 @example
 ```ts
 await podman(['image', 'exists', PLAYWRIGHT_LOCAL_IMAGE,],);
 ```
 */
export const PLAYWRIGHT_LOCAL_IMAGE = 'monochromatic-playwright';

/**
 Number of leading hex digits of the SHA-256 that make the tag.

 @example
 ```ts
 const tag = digest.slice(0, PLAYWRIGHT_TAG_LENGTH,);
 ```
 */
export const PLAYWRIGHT_TAG_LENGTH = 16;

/**
 The tag of a `playwright.Dockerfile`: the first 16 hex digits of the SHA-256 of its bytes.

 @param dockerfile - Exact bytes of the file, so that the tag equals `sha256sum playwright.Dockerfile | cut --characters=1-16`

 @returns Lowercase hex tag

 @example
 ```ts
 const tag = playwrightImageTag(await readFile('playwright.Dockerfile',),);
 ```
 */
export function playwrightImageTag(dockerfile: Uint8Array,): string {
  return createHash('sha256',)
    .update(dockerfile,)
    .digest('hex',)
    .slice(
    0,
    PLAYWRIGHT_TAG_LENGTH,
    );
}

//endregion Names: the registry image, the local names and the tag

//region Types

/**
 What one podman invocation did.

 @example
 ```ts
 const result: PodmanResult = { exitCode: 0, output: '', };
 ```
 */
export type PodmanResult = {
  /**
   Exit status of podman, 127 when podman could not be started
   */
  readonly exitCode: number;
  /**
   Standard output and standard error together, as the person would have seen them
   */
  readonly output: string;
};

/**
 Runs `podman` with arguments and reports what happened. Never rejects for a non-zero exit.

 @example
 ```ts
 const podman: Podman = async args => ({ exitCode: 0, output: args.join(' ',), });
 ```
 */
export type Podman = (args: readonly string[],) => Promise<PodmanResult>;

/**
 How the image became available.

 `present` means a local image of this tag already existed, `pulled` that it came from the registry, `built` that the
 pull failed and `playwright.Dockerfile` was built locally.

 @example
 ```ts
 const how: PlaywrightImageSource = 'pulled';
 ```
 */
export type PlaywrightImageSource = 'present' | 'pulled' | 'built';

/**
 Inputs of {@link ensurePlaywrightImage}.

 @example
 ```ts
 const options: EnsureOptions = { dockerfile, dockerfilePath: 'playwright.Dockerfile', podman, log: console.log, };
 ```
 */
export type EnsureOptions = {
  /**
   Exact bytes of `playwright.Dockerfile`
   */
  readonly dockerfile: Uint8Array;
  /**
   Path of the file as given to `podman build --file`, relative to the build context `.`
   */
  readonly dockerfilePath: string;
  /**
   Runs podman
   */
  readonly podman: Podman;
  /**
   Prints one line of progress or explanation
   */
  readonly log: (line: string,) => void;
};

//endregion Types

//region Failure

/**
 The image could neither be found, pulled nor built.

 @example
 ```ts
 throw new PlaywrightImageError('podman build failed: ...',);
 ```
 */
export class PlaywrightImageError extends Error {
  /**
   Names the failure for logs and tests
   */
  override readonly name = 'PlaywrightImageError';
}

//endregion Failure

//region Decision: present, pulled or built

/**
 How many trailing lines of podman's output explain a failure that has no `Error:` line.
 */
const REASON_LINES = 3;

/**
 Prefix of the line podman prints last when a command fails.
 */
const ERROR_LINE_PREFIX = 'Error:';

/**
 Why a podman command failed, on one line.

 Podman ends a failed command with an `Error:` line, after progress lines and retry warnings that would bury it, so
 the last such line is the reason; output without one falls back to its last few lines.

 @param output - Combined standard output and standard error

 @returns A short reason, or a note that podman printed nothing

 @example
 ```ts
 reasonFrom('Trying to pull x...\nError: initializing source: unauthorized',); // 'Error: initializing source: unauthorized'
 ```
 */
function reasonFrom(output: string,): string {
  /**
   Non-empty lines of the output, trimmed.
   */
  const lines = output.split('\n',)
    .map(function trimLine(line,) {
      return line.trim();
    },)
    .filter(function isNotEmpty(line,) {
      return line !== '';
    },);
  if (lines.length === 0)
    return 'podman printed nothing';
  /**
   The last line that starts with `Error:`, when there is one.
   */
  const errorLine = lines.findLast(function startsWithError(line,) {
    return line.startsWith(ERROR_LINE_PREFIX,);
  },);
  if (errorLine !== undefined)
    return errorLine;
  return lines.slice(-REASON_LINES,)
    .join(' / ',);
}

/**
 Runs a podman command that must succeed.

 @param podman - Runs podman

 @param args - Arguments after `podman`

 @returns The result of the successful command

 @throws PlaywrightImageError when podman exits non-zero, with the command and the reason

 @example
 ```ts
 await podmanOrThrow({ podman, args: ['tag', 'a', 'b',], },);
 ```
 */
async function podmanOrThrow(
  {
    podman,
    args,
  }: {
    readonly podman: Podman;
    readonly args: readonly string[]
  },
): Promise<PodmanResult> {
  /**
   What podman did.
   */
  const result = await podman(args,);
  if (result.exitCode !== 0) {
    throw new PlaywrightImageError(
      `podman ${args.join(' ',)} exited ${String(result.exitCode,)}: ${reasonFrom(result.output,)}`,
    );
  }
  return result;
}

/**
 Makes `monochromatic-playwright` and `monochromatic-playwright:<tag>` name the image of `playwright.Dockerfile`.

 A local image of this tag is kept as it is; otherwise the registry image of this tag is pulled and given the local
 names; and when the pull fails the reason is printed and the file is built locally with the same names.

 @param dockerfile - Exact bytes of `playwright.Dockerfile`, which decide the tag

 @param dockerfilePath - Path of the file for `podman build --file`

 @param podman - Runs podman

 @param log - Prints one line of progress or explanation

 @returns How the image became available

 @throws PlaywrightImageError when a tag cannot be set, or when the pull failed and the local build failed too

 @example
 ```ts
 const source = await ensurePlaywrightImage({ dockerfile, dockerfilePath: 'playwright.Dockerfile', podman, log: console.log, },);
 ```
 */
export async function ensurePlaywrightImage(
  {
    dockerfile,
    dockerfilePath,
    podman,
    log,
  }: EnsureOptions,
): Promise<PlaywrightImageSource> {
  /**
   Tag of this content.
   */
  const tag = playwrightImageTag(dockerfile,);
  /**
   Local name that records which content the image was made from.
   */
  const tagged = `${PLAYWRIGHT_LOCAL_IMAGE}:${tag}`;
  /**
   Registry reference of this content.
   */
  const reference = `${PLAYWRIGHT_REGISTRY_IMAGE}:${tag}`;

  if ((await podman([
    'image',
    'exists',
    tagged,
  ],)).exitCode === 0) {
    await podmanOrThrow({
      podman,
      args: [
        'tag',
        tagged,
        PLAYWRIGHT_LOCAL_IMAGE,
      ],
    },);
    log(`using ${tagged} (already local)`,);
    return 'present';
  }

  /**
   The pull; a failure is explained and answered with a local build, not thrown.
   */
  const pull = await podman([
    'pull',
    reference,
  ],);
  if (pull.exitCode === 0) {
    await podmanOrThrow({
      podman,
      args: [
        'tag',
        reference,
        tagged,
      ],
    },);
    await podmanOrThrow({
      podman,
      args: [
        'tag',
        reference,
        PLAYWRIGHT_LOCAL_IMAGE,
      ],
    },);
    log(`pulled ${reference}`,);
    return 'pulled';
  }

  log(`could not pull ${reference}: ${reasonFrom(pull.output,)}; building ${dockerfilePath} locally`,);
  /**
   The local build, with the same two names.
   */
  const build = await podman([
    'build',
    '--tag',
    tagged,
    '--tag',
    PLAYWRIGHT_LOCAL_IMAGE,
    '--file',
    dockerfilePath,
    '.',
  ],);
  if (build.exitCode !== 0) {
    throw new PlaywrightImageError(
      `${reference} could not be pulled (${reasonFrom(pull.output,)}) and building ${dockerfilePath} locally failed `
        + `(podman build exited ${String(build.exitCode,)}):\n${build.output}`,
    );
  }
  log(`built ${tagged}`,);
  return 'built';
}

//endregion Decision: present, pulled or built
