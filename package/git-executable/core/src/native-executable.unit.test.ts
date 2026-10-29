import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
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
  RealGitNotFoundError,
  resolveRealGit,
  type ResolveRealGitOptions,
} from '../dist/final/node/index.mjs';

//region Native executable fixtures

/**
 File mode making native-format fixtures executable by owner.
 */
const EXECUTABLE_MODE = 0o755;

/**
 Marker-like payload proving native executables bypass text self-shim inspection.
 */
const MARKER_PAYLOAD = Buffer.from('@monochromatic-dev/git-policy-cli',);

/**
 Supported native executable signatures.
 */
const NATIVE_HEADER_CASES: readonly {
  readonly name: string;
  readonly hexPrefix: string;
}[] = [
  { name: 'ELF', hexPrefix: '7f454c46', },
  { name: 'PE', hexPrefix: '4d5a', },
  { name: 'big-endian 32-bit Mach-O', hexPrefix: 'feedface', },
  { name: 'big-endian 64-bit Mach-O', hexPrefix: 'feedfacf', },
  { name: 'little-endian 32-bit Mach-O', hexPrefix: 'cefaedfe', },
  { name: 'little-endian 64-bit Mach-O', hexPrefix: 'cffaedfe', },
  { name: 'big-endian universal Mach-O', hexPrefix: 'cafebabe', },
  { name: 'little-endian universal Mach-O', hexPrefix: 'bebafeca', },
  { name: 'big-endian 64-bit universal Mach-O', hexPrefix: 'cafebabf', },
  { name: 'little-endian 64-bit universal Mach-O', hexPrefix: 'bfbafeca', },
];

/**
 Disposable temporary directory used by native-header tests.
 */
type TempDirectory = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates disposable temporary directory for native executable fixture.
 
 @returns Directory removed when async disposal completes.
 
 @example
 ```ts
 await using tempDirectory = await createTempDirectory();
 ```
 */
async function createTempDirectory(): Promise<TempDirectory> {
  /**
   Absolute fixture root unique to current test.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'git-executable-native-',
  ),);

  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Creates the installed wrapper layout: native executable `cli-git-native` and a `git` link to it.
 
 @param root - Disposable directory that receives the layout.
 
 @returns Directory holding the `git` link.
 
 @example
 ```ts
 const wrapperBin = await createWrapperLink(tempDirectory.path);
 ```
 */
async function createWrapperLink(root: string,): Promise<string> {
  /**
   Directory standing in for the mise install `bin` directory.
   */
  const installBin = join(root, 'install-bin',);
  /**
   Directory standing in for `.cache/cli-git-bin`.
   */
  const linkBin = join(root, 'link-bin',);
  await mkdir(installBin,);
  await mkdir(linkBin,);
  /**
   Native-format wrapper executable.
   */
  const wrapper = join(installBin, 'cli-git-native',);
  await writeFile(
    wrapper,
    Buffer.from('7f454c46', 'hex',),
    { mode: EXECUTABLE_MODE, },
  );
  await symlink(
    wrapper,
    join(linkBin, 'git',),
  );
  return linkBin;
}

/**
 Captures the failure of one resolution attempt.
 
 @param options - Resolver options for the attempt.
 
 @returns Thrown value, or undefined when resolution succeeded.
 
 @example
 ```ts
 const caught = await captureResolutionError({ pathEnv: '/nowhere', commonGitPaths: [], });
 ```
 */
async function captureResolutionError(options: ResolveRealGitOptions,): Promise<unknown> {
  try {
    await resolveRealGit(options,);
  }
  catch (error: unknown) {
    return error;
  }
  return undefined;
}

//endregion Native executable fixtures

await describe({
  name: resolveRealGit.name,
  children: NATIVE_HEADER_CASES.map(function mapNativeHeaderCase(nativeHeaderCase,) {
    return it({
      name: `accepts ${nativeHeaderCase.name} header before marker-like payload`,
      fn: async function acceptsNativeHeader(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         PATH directory containing native-format candidate.
         */
        const nativeBin = join(tempDirectory.path, 'native-bin',);
        await mkdir(nativeBin,);
        /**
         Native-format candidate containing wrapper marker after recognized header.
         */
        const nativeGit = join(nativeBin, 'git',);
        await writeFile(
          nativeGit,
          Buffer.concat([
            Buffer.from(nativeHeaderCase.hexPrefix, 'hex',),
            MARKER_PAYLOAD,
          ],),
          { mode: EXECUTABLE_MODE, },
        );
        await chmod(
          nativeGit,
          EXECUTABLE_MODE,
        );

        expect(await resolveRealGit({
          pathEnv: nativeBin,
          commonGitPaths: [nativeGit,],
        },),).toBe(nativeGit,);
      },
    },);
  },),
},);

await describe({
  name: `${resolveRealGit.name} with the native wrapper link`,
  children: [
    it({
      name: 'skips a git link that resolves to cli-git-native and selects the next native Git',
      fn: async function skipsWrapperLink(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         PATH directory holding the wrapper link.
         */
        const linkBin = await createWrapperLink(tempDirectory.path,);
        /**
         PATH directory holding a native Git that is not the wrapper.
         */
        const realBin = join(tempDirectory.path, 'real-bin',);
        await mkdir(realBin,);
        /**
         Native-format real Git candidate.
         */
        const realGit = join(realBin, 'git',);
        await writeFile(
          realGit,
          Buffer.from('7f454c46', 'hex',),
          { mode: EXECUTABLE_MODE, },
        );

        /**
         PATH with the wrapper link ahead of the real Git.
         */
        const pathEnv = [linkBin, realBin,].join(delimiter,);

        expect(await resolveRealGit({
          pathEnv,
          commonGitPaths: [],
        },),).toBe(realGit,);
      },
    },),
    it({
      name: 'reports no real Git when the wrapper link is the only candidate',
      fn: async function rejectsOnlyWrapperLink(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         PATH directory holding only the wrapper link.
         */
        const linkBin = await createWrapperLink(tempDirectory.path,);
        /**
         Failure recorded for a PATH that exposes only the wrapper.
         */
        const caught = await captureResolutionError({
          pathEnv: linkBin,
          commonGitPaths: [],
        },);

        expect(caught,).toBeInstanceOf(RealGitNotFoundError,);
      },
    },),
  ],
},);
