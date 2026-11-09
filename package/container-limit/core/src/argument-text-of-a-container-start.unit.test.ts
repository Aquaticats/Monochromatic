import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  argumentTextNear,
  arrayContentsStart,
  spreadAtStart,
  startVerbOf,
} from './argument-text-of-a-container-start.ts';

/**
 Characters of a fixture, which every reader takes as a list.
 */
function charactersOf(text: string,): readonly string[] {
  return text.split('',);
}

await describe({
  name: '',
  children: [
    describe({
      name: argumentTextNear.name,
      children: [
        it({
          name: 'reads the contents of an argument array',
          fn: async () => {
            const characters = charactersOf("'podman', ['run', '--rm']",);
            expect(argumentTextNear({
              characters,
              from: 8,
              to: characters.length,
            },),).toEqual({
              text: "'run', '--rm'",
              inArrayLiteral: true,
            },);
          },
        },),
        it({
          name: 'reads the array a call wraps',
          fn: async () => {
            const characters = charactersOf("podman(['run', '--rm'])",);
            expect(argumentTextNear({
              characters,
              from: 6,
              to: characters.length,
            },),).toEqual({
              text: "'run', '--rm'",
              inArrayLiteral: true,
            },);
          },
        },),
        it({
          name: 'reads the array a named property holds',
          fn: async () => {
            const characters = charactersOf("'podman', args: ['run', '--rm'],",);
            expect(argumentTextNear({
              characters,
              from: 8,
              to: characters.length,
            },),).toEqual({
              text: "'run', '--rm'",
              inArrayLiteral: true,
            },);
          },
        },),
        it({
          name: 'reads a shell command line, joined across its continuations',
          fn: async () => {
            const characters = charactersOf('podman run --rm \\\n  --init\nnext',);
            expect(argumentTextNear({
              characters,
              from: 6,
              to: characters.length,
            },),).toEqual({
              text: 'run --rm \\\n  --init',
              inArrayLiteral: false,
            },);
          },
        },),
        it({
          name: 'reads nothing when the program name ends the text',
          fn: async () => {
            const characters = charactersOf("'podman'",);
            expect(argumentTextNear({
              characters,
              from: 8,
              to: characters.length,
            },),).toEqual({
              text: '',
              inArrayLiteral: false,
            },);
          },
        },),
      ],
    },),
    describe({
      name: startVerbOf.name,
      children: [
        it({
          name: 'reads each of the three verbs',
          fn: async () => {
            expect(startVerbOf("'run'",).verb,).toBe('run',);
            expect(startVerbOf("'create'",).verb,).toBe('create',);
            expect(startVerbOf('build --memory=2g',).verb,).toBe('build',);
          },
        },),
        it({
          name: 'reads no verb from a variable or a non-start verb',
          fn: async () => {
            expect(startVerbOf("'image', 'exists'",).verb,).toBeUndefined();
            expect(startVerbOf('args',).verb,).toBeUndefined();
          },
        },),
      ],
    },),
    describe({
      name: arrayContentsStart.name,
      children: [
        it({
          name: 'finds the array an opening quote sits in',
          fn: async () => {
            const characters = charactersOf("['podman', 'build']",);
            expect(arrayContentsStart({
              characters,
              index: 1,
            },).startsAt,).toBe(1,);
          },
        },),
        it({
          name: 'finds nothing when the mention follows something other than a bracket',
          fn: async () => {
            const characters = charactersOf("spawn('podman', args)",);
            expect(arrayContentsStart({
              characters,
              index: 6,
            },).startsAt,).toBeUndefined();
          },
        },),
      ],
    },),
    describe({
      name: spreadAtStart.name,
      children: [
        it({
          name: 'reads the name and the call a spread starts with',
          fn: async () => {
            expect(spreadAtStart("\n  ...containerRunArguments({ image })",),).toEqual({
              name: 'containerRunArguments',
              isCall: true,
            },);
          },
        },),
        it({
          name: 'reads a spread of a bare name',
          fn: async () => {
            expect(spreadAtStart("...pre, 'sign'",),).toEqual({
              name: 'pre',
              isCall: false,
            },);
          },
        },),
        it({
          name: 'reads nothing from a text that starts with a word',
          fn: async () => {
            expect(spreadAtStart("'run', ...pre",),).toEqual({ isCall: false, },);
          },
        },),
        it({
          name: 'reads nothing from a spread with no name',
          fn: async () => {
            expect(spreadAtStart("'...', 'run'",),).toEqual({ isCall: false, },);
          },
        },),
      ],
    },),
  ],
},);
