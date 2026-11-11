import {
  chmod,
  mkdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
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
  SELF_SHIM_MARKERS,
} from '../dist/final/node/index.mjs';
import {
  captureResolutionError,
  createTempDirectory,
  EXECUTABLE_MODE,
} from './disposable-executable-fixture.ts';

//region Native executable fixtures

/**
 Marker-like payload proving native executables bypass text self-shim inspection.
 Every marker the resolver recognises is carried in the payload,
 so no fixture spells a marker of its own.
 */
const MARKER_PAYLOAD = Buffer.from([...SELF_SHIM_MARKERS,].join(' ',),);

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
 Creates the installed wrapper layout: native executable under the given program name and a `git` link to it.
 
 @param root - Disposable directory that receives the layout.
 
 @param programName - File name of the wrapper executable: the current name or the earlier one.
 
 @returns Directory holding the `git` link.
 
 @example
 ```ts
 const wrapperBin = await createWrapperLink(tempDirectory.path, 'monochromatic-git-policy-cli');
 ```
 */
async function createWrapperLink(root: string, programName: string,): Promise<string> {
  /**
   Directory standing in for the mise install `bin` directory.
   */
  const installBin = join(root, 'install-bin',);
  /**
   Directory standing in for `.cache/monochromatic-git-policy-cli-bin`.
   */
  const linkBin = join(root, 'link-bin',);
  await mkdir(installBin,);
  await mkdir(linkBin,);
  /**
   Native-format wrapper executable.
   */
  const wrapper = join(installBin, programName,);
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

//endregion Native executable fixtures

await describe({
  name: resolveRealGit.name,
  children: NATIVE_HEADER_CASES.map(function mapNativeHeaderCase(nativeHeaderCase,) {
    return it({
      name: `accepts ${nativeHeaderCase.name} header before marker-like payload`,
      fn: async function acceptsNativeHeader(): Promise<void> {
        await using tempDirectory = await createTempDirectory({ prefix: 'git-executable-native-', },);
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
      name: 'skips a git link that resolves to monochromatic-git-policy-cli and selects the next native Git',
      fn: async function skipsWrapperLink(): Promise<void> {
        await using tempDirectory = await createTempDirectory({ prefix: 'git-executable-native-', },);
        /**
         PATH directory holding the wrapper link.
         */
        const linkBin = await createWrapperLink(tempDirectory.path, 'monochromatic-git-policy-cli',);
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
      name: 'skips a git link that resolves to the earlier name cli-git-native and selects the next native Git',
      fn: async function skipsEarlierWrapperLink(): Promise<void> {
        await using tempDirectory = await createTempDirectory({ prefix: 'git-executable-native-', },);
        /**
         PATH directory holding the wrapper link.
         */
        const linkBin = await createWrapperLink(tempDirectory.path, 'cli-git-native',);
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
        await using tempDirectory = await createTempDirectory({ prefix: 'git-executable-native-', },);
        /**
         PATH directory holding only the wrapper link.
         */
        const linkBin = await createWrapperLink(tempDirectory.path, 'monochromatic-git-policy-cli',);
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
