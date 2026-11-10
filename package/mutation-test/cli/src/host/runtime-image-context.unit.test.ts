/**
 Checks what the runtime image's `COPY . /baked` takes from the repository root.

 The build folder is the repository root, so the only thing between a local file and an image layer is
 `runtime/containerignore`. `.gitignore` keeps local configuration, secrets and machine-specific files out of git and
 the ignorefile has to keep the same files out of the layer; `probe` in the repository's own words is a
 `podman build` whose image listing shows which paths arrived.

 The patterns are asserted rather than matched, because both files use gitignore syntax and this package's unit tests
 carry no matcher. A pattern that stops matching a real file has to be caught by running a build, which
 `test:integration` does.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { readFile, } from 'node:fs/promises';

/**
 The ignorefile the runtime image build passes to `podman build`.
 */
const ignorefile = await readFile(
  new URL('../../runtime/containerignore', import.meta.url,),
  'utf8',
);

/**
 Patterns that keep a gitignored local or secret file out of the image layer: a plaintext environment file, a local
 environment variant, and the `*.local.*` family that covers the appended deny list, the resolved browser targets and
 the decrypted secret store.
 */
const localFilePatterns: readonly string[] = ['.env', '.env*.local', '*.local.*'];

/**
 Patterns that keep the build output and cache trees out of the layer.
 */
const buildTreePatterns: readonly string[] = ['**/target', '**/node_modules'];

await describe({
  name: 'runtime image build context',
  children: [
    it({
      name: 'reads an ignorefile, so the checks below cannot pass on an empty file',
      fn: async function hasIgnorefile() {
        expect(ignorefile.length,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'keeps every gitignored local or secret file out of the image layer',
      fn: async function excludesLocalFiles() {
        const missing = localFilePatterns.filter(function absent(pattern,) {
          return !ignorefile.includes(`${pattern}\n`,);
        },);
        expect(missing,).toEqual([],);
      },
    },),
    it({
      name: 'keeps the build output and dependency trees out of the image layer',
      fn: async function excludesBuildTrees() {
        const missing = buildTreePatterns.filter(function absent(pattern,) {
          return !ignorefile.includes(`${pattern}\n`,);
        },);
        expect(missing,).toEqual([],);
      },
    },),
    it({
      name: 'still sends the source tree, so the mutation run has code to work on',
      fn: async function keepsSources() {
        const patterns = ignorefile.split('\n',)
          .map(function withoutTrailingSpace(line,) {
            return line.trimEnd();
          },);
        // A line that names the whole tree would take the code away with the secrets.
        expect(patterns,).not.toContain('*',);
        expect(patterns,).not.toContain('package/',);
        expect(patterns,).not.toContain('package',);
      },
    },),
  ],
},);
