/**
 Tests for finding the installed compositor: what is made of the answer of `mise where`, and a real lookup.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  COMPOSITOR_EXECUTABLE,
  COMPOSITOR_TOOL,
  compositorPathFromMiseResult,
  installedCompositorPath,
} from './compositor-path.ts';
import { pathExists, } from './files.ts';

await describe({
  name: compositorPathFromMiseResult.name,
  children: [
    it({
      name: 'joins the printed install directory with the executable, whatever version the directory names',
      fn: async ({ expect, }) => {
        expect(compositorPathFromMiseResult({ status: 0, stdout: '/m/installs/nested/0.4.0', stderr: '', },),).toBe(
          `/m/installs/nested/0.4.0/bin/${COMPOSITOR_EXECUTABLE}`,
        );
        expect(compositorPathFromMiseResult({ status: 0, stdout: '/m/installs/nested/9.9.9\n', stderr: 'WARN', },),)
          .toBe(`/m/installs/nested/9.9.9/bin/${COMPOSITOR_EXECUTABLE}`,);
      },
    },),
    it({
      name: 'refuses a failed lookup, naming the install command and the override',
      fn: async ({ expect, }) => {
        expect(() => compositorPathFromMiseResult({ status: 1, stdout: '', stderr: 'not installed', },),).toThrow(
          `mise install ${COMPOSITOR_TOOL}`,
        );
        expect(() => compositorPathFromMiseResult({ status: -1, stdout: '', stderr: 'spawn mise ENOENT', },),).toThrow(
          'E2E_COMPOSITOR',
        );
      },
    },),
    it({
      name: 'refuses an empty answer and an answer that is not one absolute directory',
      fn: async ({ expect, }) => {
        expect(() => compositorPathFromMiseResult({ status: 0, stdout: '', stderr: '', },),).toThrow('E2E_COMPOSITOR',);
        expect(() => compositorPathFromMiseResult({ status: 0, stdout: 'relative/dir', stderr: '', },),).toThrow(
          'not one absolute directory',
        );
        expect(() => compositorPathFromMiseResult({ status: 0, stdout: '/a\n/b', stderr: '', },),).toThrow(
          'not one absolute directory',
        );
      },
    },),
  ],
},);

await describe({
  name: installedCompositorPath.name,
  children: [
    it({
      name: 'asks mise from the given directory and gives the executable of the active install',
      fn: async ({ expect, }) => {
        // Where mise has the compositor installed (and active at the repository), the answer is an absolute path to it.
        const path = await installedCompositorPath({ cwd: import.meta.dirname, },);
        expect(path.endsWith(`/bin/${COMPOSITOR_EXECUTABLE}`,),).toBe(true,);
        expect(await pathExists(path,),).toBe(true,);
      },
    },),
  ],
},);
