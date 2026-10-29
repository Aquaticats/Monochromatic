/**
 Tests for the compositor capability guard: parsing the `--version` line, comparing, and the refusal message.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  compositorVersionProblem,
  FIRST_VERSION_WITH_RUNNER_CONTROLS,
  isAtLeast,
  parseCompositorVersion,
} from './compositor-version.ts';

await describe({
  name: parseCompositorVersion.name,
  children: [
    it({
      name: 'reads the numbers from the version line',
      fn: async ({ expect, }) => {
        expect(parseCompositorVersion('monochromatic-nested-wayland-session 0.4.0\n',),).toEqual([0, 4, 0,],);
        expect(parseCompositorVersion('x 10.20.30',),).toEqual([10, 20, 30,],);
      },
    },),
    it({
      name: 'ignores a pre-release or build suffix, so a local build of a newer compositor is read',
      fn: async ({ expect, }) => {
        expect(parseCompositorVersion('nested 0.5.0-dev',),).toEqual([0, 5, 0,],);
        expect(parseCompositorVersion('nested 0.5.0-alpha.1',),).toEqual([0, 5, 0,],);
        expect(parseCompositorVersion('nested 1.0.0+local',),).toEqual([1, 0, 0,],);
      },
    },),
    it({
      name: 'finds no version in text without one',
      fn: async ({ expect, }) => {
        expect(parseCompositorVersion('',),).toEqual([],);
        expect(parseCompositorVersion('usage: nested',),).toEqual([],);
        expect(parseCompositorVersion('x 1.2',),).toEqual([],);
        expect(parseCompositorVersion('x 1.2.z',),).toEqual([],);
        expect(parseCompositorVersion('x 1..2',),).toEqual([],);
        expect(parseCompositorVersion('x 1.2.3.4',),).toEqual([],);
        expect(parseCompositorVersion('x -dev',),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: isAtLeast.name,
  children: [
    it({
      name: 'compares major, then minor, then patch',
      fn: async ({ expect, }) => {
        const minimum = [0, 4, 0,] as const;
        expect(isAtLeast({ version: [0, 4, 0,], minimum, },),).toBe(true,);
        expect(isAtLeast({ version: [0, 4, 7,], minimum, },),).toBe(true,);
        expect(isAtLeast({ version: [0, 5, 0,], minimum, },),).toBe(true,);
        expect(isAtLeast({ version: [1, 0, 0,], minimum, },),).toBe(true,);
        expect(isAtLeast({ version: [0, 3, 9,], minimum, },),).toBe(false,);
        expect(isAtLeast({ version: [0, 0, 99,], minimum, },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: compositorVersionProblem.name,
  children: [
    it({
      name: 'accepts the first version with the controls and every newer one, with no message',
      fn: async ({ expect, }) => {
        expect(
          compositorVersionProblem({ output: `nested ${FIRST_VERSION_WITH_RUNNER_CONTROLS}`, path: '/c', },),
        ).toBe('',);
        expect(compositorVersionProblem({ output: 'nested 0.4.1', path: '/c', },),).toBe('',);
        expect(compositorVersionProblem({ output: 'nested 0.5.0', path: '/c', },),).toBe('',);
        expect(compositorVersionProblem({ output: 'nested 1.0.0', path: '/c', },),).toBe('',);
        expect(compositorVersionProblem({ output: 'nested 0.5.0-dev', path: '/c', },),).toBe('',);
      },
    },),
    it({
      name: 'refuses an older compositor, naming its version, what is needed and both ways out',
      fn: async ({ expect, }) => {
        const message = compositorVersionProblem({ output: 'nested 0.3.0', path: '/old/nested', },);
        expect(message,).toContain('/old/nested is 0.3.0',);
        expect(message,).toContain('compositor 0.4.0 or newer',);
        expect(message,).toContain('mise upgrade cargo:monochromatic-nested-wayland-session',);
        expect(message,).toContain('E2E_COMPOSITOR',);
        expect(message,).not.toContain('PINNED',);
      },
    },),
    it({
      name: 'refuses output with no version, with the same ways out',
      fn: async ({ expect, }) => {
        const message = compositorVersionProblem({ output: 'oops', path: '/c', },);
        expect(message,).toContain('printed no version ("oops")',);
        expect(message,).toContain('mise upgrade cargo:monochromatic-nested-wayland-session',);
        expect(message,).toContain('E2E_COMPOSITOR',);
      },
    },),
  ],
},);
