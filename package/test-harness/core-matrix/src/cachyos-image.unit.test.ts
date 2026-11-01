/**
 Tests for CachyOS base image selection by CPU level and architecture.
 Loader output, architecture and logger are injected, so no test depends on the host running them.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ALPINE_IMAGE,
  CACHYOS_IMAGE_BY_LEVEL,
  cachyosBuildArguments,
  detectCachyosLevel,
  LoaderUnavailableError,
  NoAlpineVariantError,
  readLoaderOutput,
  resolveBaseImage,
  runCachyosImageCli,
} from '../dist/final/node/index.mjs';

/**
 Loader report of a CPU with v4, v3 and v2 (this repository's Ryzen 9 7950X host prints exactly these lines).
 */
const LOADER_V4 = `Usage: /lib64/ld-linux-x86-64.so.2 [OPTION]... EXECUTABLE-FILE [ARGS-FOR-PROGRAM...]
This program interpreter self-identifies as: /lib64/ld-linux-x86-64.so.2

Subdirectories of glibc-hwcaps directories, in priority order:
  x86-64-v4 (supported, searched)
  x86-64-v3 (supported, searched)
  x86-64-v2 (supported, searched)
`;

/**
 Loader report of a CPU with v3 and v2 but not v4.
 */
const LOADER_V3 = `Subdirectories of glibc-hwcaps directories, in priority order:
  x86-64-v4
  x86-64-v3 (supported, searched)
  x86-64-v2 (supported, searched)
`;

/**
 Loader report of a CPU with v2 only.
 */
const LOADER_V2 = `Subdirectories of glibc-hwcaps directories, in priority order:
  x86-64-v4
  x86-64-v3
  x86-64-v2 (supported, searched)
`;

/**
 Loader report that names a level the loader supports but does not search.
 */
const LOADER_V4_NOT_SEARCHED = `Subdirectories of glibc-hwcaps directories, in priority order:
  x86-64-v4 (supported, not searched)
  x86-64-v3 (supported, not searched)
`;

/**
 Loader report with no level lines, as an old glibc prints.
 */
const LOADER_NO_LEVELS = `Usage: ld.so [OPTION]... EXECUTABLE-FILE [ARGS-FOR-PROGRAM...]
`;

/**
 Logger that records messages, so tests can check the logged branch and reason.

 @param messages - Array that receives `[level] message` entries

 @returns a logger writing into `messages`
 */
function recordingLogger(messages: string[],) {
  /**
   Builds one level's logging function.

   @param level - Severity label

   @returns function appending to `messages`
   */
  function at(level: string,) {
    return function record(message: string,): void {
      messages.push(`[${level}] ${message}`,);
    };
  }

  return {
    debug: at('debug',),
    error: at('error',),
    fatal: at('fatal',),
    flush: function flush(): Promise<void> {
      return Promise.resolve();
    },
    info: at('info',),
    trace: at('trace',),
    warn: at('warn',),
  };
}

await describe({
  name: 'cachyos-image',
  children: [
    //region detectCachyosLevel

    describe({
      name: detectCachyosLevel.name,
      children: [
        it({
          name: 'v4 wins when the loader reports v4, v3 and v2',
          fn: async () => {
            expect(detectCachyosLevel(LOADER_V4,).level,).toBe('v4',);
          },
        },),

        it({
          name: 'v3 when the loader reports v3 and v2 but not v4',
          fn: async () => {
            expect(detectCachyosLevel(LOADER_V3,).level,).toBe('v3',);
          },
        },),

        it({
          name: 'plain when the loader lists levels but supports only v2',
          fn: async () => {
            const detection = detectCachyosLevel(LOADER_V2,);
            expect(detection.level,).toBe('plain',);
            expect(detection.reason,).toContain('neither v4 nor v3',);
          },
        },),

        it({
          name: 'a level that is supported but not searched does not count',
          fn: async () => {
            expect(detectCachyosLevel(LOADER_V4_NOT_SEARCHED,).level,).toBe('plain',);
          },
        },),

        it({
          name: 'plain when the loader printed no level lines',
          fn: async () => {
            const detection = detectCachyosLevel(LOADER_NO_LEVELS,);
            expect(detection.level,).toBe('plain',);
            expect(detection.reason,).toContain('no x86-64 level lines',);
          },
        },),

        it({
          name: 'plain for empty output',
          fn: async () => {
            expect(detectCachyosLevel('',).level,).toBe('plain',);
          },
        },),
      ],
    },),

    //endregion detectCachyosLevel

    //region resolveBaseImage

    describe({
      name: resolveBaseImage.name,
      children: [
        it({
          name: 'x64 with a v4 loader report resolves to cachyos-v4 and logs the branch',
          fn: async () => {
            const messages: string[] = [];
            const resolution = await resolveBaseImage({
              imageName: 'terminal',
              alpineVariant: false,
              arch: 'x64',
              readLoader: async () => LOADER_V4,
              l: recordingLogger(messages,),
            },);
            expect(resolution.image,).toBe('docker.io/cachyos/cachyos-v4:latest',);
            expect(resolution.kind,).toBe('cachyos-v4',);
            expect(messages.join('\n',),).toContain('docker.io/cachyos/cachyos-v4:latest: the loader reports x86-64-v4',);
          },
        },),

        it({
          name: 'x64 with a v3 loader report resolves to cachyos-v3',
          fn: async () => {
            const resolution = await resolveBaseImage({
              imageName: 'terminal',
              alpineVariant: false,
              arch: 'x64',
              readLoader: async () => LOADER_V3,
              l: recordingLogger([],),
            },);
            expect(resolution.image,).toBe('docker.io/cachyos/cachyos-v3:latest',);
            expect(resolution.kind,).toBe('cachyos-v3',);
          },
        },),

        it({
          name: 'x64 with a v2-only loader report resolves to plain cachyos and warns with the reason',
          fn: async () => {
            const messages: string[] = [];
            const resolution = await resolveBaseImage({
              imageName: 'terminal',
              alpineVariant: false,
              arch: 'x64',
              readLoader: async () => LOADER_V2,
              l: recordingLogger(messages,),
            },);
            expect(resolution.image,).toBe('docker.io/cachyos/cachyos:latest',);
            expect(resolution.kind,).toBe('cachyos',);
            expect(messages.join('\n',),).toContain('[warn]',);
            expect(messages.join('\n',),).toContain('neither v4 nor v3',);
          },
        },),

        it({
          name: 'x64 with a missing loader resolves to plain cachyos and logs why',
          fn: async () => {
            const messages: string[] = [];
            const resolution = await resolveBaseImage({
              imageName: 'terminal',
              alpineVariant: false,
              arch: 'x64',
              readLoader: async () => {
                throw new LoaderUnavailableError('no loader at either path',);
              },
              l: recordingLogger(messages,),
            },);
            expect(resolution.image,).toBe('docker.io/cachyos/cachyos:latest',);
            expect(resolution.reason,).toContain('no glibc loader ran',);
            expect(messages.join('\n',),).toContain('[warn]',);
            expect(messages.join('\n',),).toContain('no glibc loader ran',);
          },
        },),

        it({
          name: 'x64 with a loader that printed no levels resolves to plain cachyos',
          fn: async () => {
            const resolution = await resolveBaseImage({
              imageName: 'terminal',
              alpineVariant: false,
              arch: 'x64',
              readLoader: async () => LOADER_NO_LEVELS,
              l: recordingLogger([],),
            },);
            expect(resolution.image,).toBe('docker.io/cachyos/cachyos:latest',);
            expect(resolution.reason,).toContain('no x86-64 level lines',);
          },
        },),

        it({
          name: 'non-x64 with an Alpine variant resolves to alpine and never reads the loader',
          fn: async () => {
            const messages: string[] = [];
            const resolution = await resolveBaseImage({
              imageName: 'terminal',
              alpineVariant: true,
              arch: 'arm64',
              readLoader: async () => {
                throw new Error('the loader must not be read on arm64',);
              },
              l: recordingLogger(messages,),
            },);
            expect(resolution.image,).toBe(ALPINE_IMAGE,);
            expect(resolution.kind,).toBe('alpine',);
            expect(messages.join('\n',),).toContain('arch "arm64" is not x64',);
          },
        },),

        it({
          name: 'non-x64 without an Alpine variant rejects with NoAlpineVariantError naming the image and logs an error',
          fn: async () => {
            const messages: string[] = [];
            const resolution = resolveBaseImage({
              imageName: 'localhost/monochromatic/ide',
              alpineVariant: false,
              arch: 'arm64',
              readLoader: async () => LOADER_V4,
              l: recordingLogger(messages,),
            },);
            await expect(resolution,).rejects.toBeInstanceOf(NoAlpineVariantError,);
            await expect(resolution,).rejects.toThrow('"localhost/monochromatic/ide"',);
            await expect(resolution,).rejects.toThrow('no Alpine variant',);
            await expect(resolution,).rejects.toHaveProperty('imageName', 'localhost/monochromatic/ide',);
            await expect(resolution,).rejects.toHaveProperty('arch', 'arm64',);
            expect(messages.join('\n',),).toContain('[error]',);
          },
        },),

        it({
          name: 'an unexpected failure from the loader reader propagates instead of falling back',
          fn: async () => {
            await expect(
              resolveBaseImage({
                imageName: 'terminal',
                alpineVariant: false,
                arch: 'x64',
                readLoader: async () => {
                  throw new Error('disk on fire',);
                },
                l: recordingLogger([],),
              },),
            ).rejects.toThrow('disk on fire',);
          },
        },),

        it({
          name: 'defaults probe this host and return a CachyOS image on x64 or reject on other architectures',
          fn: async () => {
            const resolution = resolveBaseImage({
              imageName: 'terminal',
              alpineVariant: false,
              l: recordingLogger([],),
            },);
            if (process.arch === 'x64') {
              const { image, } = await resolution;
              expect(Object.values(CACHYOS_IMAGE_BY_LEVEL,),).toContain(image,);
              return;
            }
            await expect(resolution,).rejects.toThrow('no Alpine variant',);
          },
        },),
      ],
    },),

    //endregion resolveBaseImage

    //region cachyosBuildArguments

    describe({
      name: cachyosBuildArguments.name,
      children: [
        it({
          name: 'returns the --build-arg pair carrying the resolved image',
          fn: async () => {
            expect(
              await cachyosBuildArguments({
                imageName: 'terminal',
                alpineVariant: false,
                arch: 'x64',
                readLoader: async () => LOADER_V3,
                l: recordingLogger([],),
              },),
            ).toEqual(['--build-arg', 'CACHYOS_IMAGE=docker.io/cachyos/cachyos-v3:latest',],);
          },
        },),

        it({
          name: 'rejects with NoAlpineVariantError on non-x64 without an Alpine variant',
          fn: async () => {
            await expect(
              cachyosBuildArguments({
                imageName: 'terminal',
                alpineVariant: false,
                arch: 'riscv64',
                l: recordingLogger([],),
              },),
            ).rejects.toBeInstanceOf(NoAlpineVariantError,);
          },
        },),
      ],
    },),

    //endregion cachyosBuildArguments

    //region readLoaderOutput

    describe({
      name: readLoaderOutput.name,
      children: [
        it({
          name: 'reads a level report on a glibc x64 Linux host and rejects with LoaderUnavailableError elsewhere',
          fn: async () => {
            const output = readLoaderOutput({ l: recordingLogger([],), },);
            if ((process.platform === 'linux') && (process.arch === 'x64'))
              expect(await output,).toContain('glibc-hwcaps',);
            else
              await expect(output,).rejects.toBeInstanceOf(LoaderUnavailableError,);
          },
        },),
      ],
    },),

    //endregion readLoaderOutput

    //region runCachyosImageCli

    describe({
      name: runCachyosImageCli.name,
      children: [
        it({
          name: 'prints the bare image',
          fn: async () => {
            const result = await runCachyosImageCli({
              args: ['--image-name', 'terminal',],
              resolveOptions: {
                arch: 'x64',
                readLoader: async () => LOADER_V4,
                l: recordingLogger([],),
              },
            },);
            expect(result.exitCode,).toBe(0,);
            expect(result.stdout,).toBe('docker.io/cachyos/cachyos-v4:latest\n',);
          },
        },),

        it({
          name: '--build-argument prints CACHYOS_IMAGE=<image>',
          fn: async () => {
            const result = await runCachyosImageCli({
              args: ['--image-name', 'terminal', '--build-argument',],
              resolveOptions: {
                arch: 'x64',
                readLoader: async () => LOADER_V3,
                l: recordingLogger([],),
              },
            },);
            expect(result.stdout,).toBe('CACHYOS_IMAGE=docker.io/cachyos/cachyos-v3:latest\n',);
          },
        },),

        it({
          name: '--alpine-variant prints Alpine on arm64',
          fn: async () => {
            const result = await runCachyosImageCli({
              args: ['--image-name', 'terminal', '--alpine-variant',],
              resolveOptions: {
                arch: 'arm64',
                l: recordingLogger([],),
              },
            },);
            expect(result.exitCode,).toBe(0,);
            expect(result.stdout,).toBe(`${ALPINE_IMAGE}\n`,);
          },
        },),

        it({
          name: 'exits 1 and names the image on arm64 without --alpine-variant',
          fn: async () => {
            const result = await runCachyosImageCli({
              args: ['--image-name', 'terminal',],
              resolveOptions: {
                arch: 'arm64',
                l: recordingLogger([],),
              },
            },);
            expect(result.exitCode,).toBe(1,);
            expect(result.stdout,).toBe('',);
            expect(result.stderr,).toContain('"terminal"',);
          },
        },),

        it({
          name: 'exits 2 with usage when --image-name is missing',
          fn: async () => {
            const result = await runCachyosImageCli({
              args: [],
              resolveOptions: {
                arch: 'x64',
                l: recordingLogger([],),
              },
            },);
            expect(result.exitCode,).toBe(2,);
            expect(result.stderr,).toContain('--image-name is required',);
          },
        },),

        it({
          name: 'exits 2 with usage on an unknown option',
          fn: async () => {
            const result = await runCachyosImageCli({
              args: ['--image-name', 'terminal', '--bogus',],
              resolveOptions: {
                arch: 'x64',
                l: recordingLogger([],),
              },
            },);
            expect(result.exitCode,).toBe(2,);
            expect(result.stderr,).toContain('usage: cachyos-image',);
          },
        },),
      ],
    },),

    //endregion runCachyosImageCli
  ],
},);
