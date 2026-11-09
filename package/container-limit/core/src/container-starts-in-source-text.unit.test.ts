import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CONTAINER_PROGRAMS,
  CONTAINER_START_VERBS,
} from './container-start-vocabulary.ts';
import { containerStartInvocations, } from './container-starts-in-source-text.ts';

/**
 The starts of a TypeScript fixture.
 */
function startsOf(text: string,): readonly string[] {
  return containerStartInvocations({
    text,
    language: 'typescript',
  },).map((start,) => `${start.program} ${start.verb ?? 'arguments'}`,);
}

await describe({
  name: containerStartInvocations.name,
  children: [
    it({
      name: 'reads the argument array after a program name',
      fn: async () => {
        expect(startsOf("execFileSync('podman', ['run', '--rm']);",),).toEqual(['podman run',],);
      },
    },),
    it({
      name: 'reads a program name that is the array literal first element',
      fn: async () => {
        expect(startsOf("return [\n  'podman',\n  'build',\n  '--tag',\n];",),).toEqual(['podman build',],);
      },
    },),
    it({
      name: 'reads a start whose arguments an array the file defines holds',
      fn: async () => {
        expect(startsOf(
          "const pre = ['run', '--rm']);\nexecFileSync('podman', [...pre, 'sign']);",
        ),).toEqual(['podman arguments',],);
      },
    },),
    it({
      name: 'reads a start whose arguments a call in an imported module builds',
      fn: async () => {
        expect(containerStartInvocations({
          text: "spawn('podman', [...containerRunArguments({ image })]);",
          language: 'typescript',
          importedDefinitions: new Map([['containerRunArguments', 'return []',],]),
        },).length,).toBe(1,);
      },
    },),
    it({
      name: 'reads no start from a program called with a variable nobody defines',
      fn: async () => {
        expect(startsOf("spawn('podman', options.args);",),).toEqual([],);
      },
    },),
    it({
      name: 'reads no start from a mention whose verb does not start the arguments',
      fn: async () => {
        expect(startsOf("execFileSync('podman', ['image', 'exists', reference]);",),).toEqual([],);
      },
    },),
    it({
      name: 'reads a shell command line and joins its continuations',
      fn: async () => {
        expect(startsOf('const command = `host=$(podman run --rm \\\n  --init image)`;',),).toEqual(
          ['podman run',],
        );
      },
    },),
    it({
      name: 'reads no start from prose that names a program, or from a comment',
      fn: async () => {
        expect(startsOf("const note = 'the period (podman build has no --cpus)';\n// podman run --rm",),).toEqual(
          [],
        );
      },
    },),
    it({
      name: 'reads a program inside a backtick-quoted phrase as prose',
      fn: async () => {
        expect(startsOf("const note = '`podman build` has no --cpus';",),).toEqual([],);
      },
    },),
    it({
      name: 'reports the line the program name sits on',
      fn: async () => {
        expect(containerStartInvocations({
          text: "a\n\nexecFileSync('podman', ['run']);",
          language: 'typescript',
        },)[0]?.line,).toBe(3,);
      },
    },),
    it({
      name: 'reads both programs the vocabulary names',
      fn: async () => {
        for (const program of CONTAINER_PROGRAMS)
          expect(startsOf(`execFileSync('${program}', ['run']);`,),).toEqual([`${program} run`,],);
      },
    },),
    it({
      name: 'reads each verb the vocabulary names',
      fn: async () => {
        for (const verb of CONTAINER_START_VERBS)
          expect(startsOf(`execFileSync('podman', ['${verb}']);`,),).toEqual([`podman ${verb}`,],);
      },
    },),
  ],
},);
