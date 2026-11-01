import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  FORCE_BACKGROUND_ENV,
  isPlainSleep,
  shouldForceBackground,
} from './force-background.ts';

await describe({
  name: shouldForceBackground.name,
  children: [
    it({
      name: 'names the opt-out environment variable',
      fn: async () => {
        expect(FORCE_BACKGROUND_ENV,).toBe('MONOCHROMATIC_BASH_FORCE_BACKGROUND',);
      },
    },),
    it({
      name: 'forces a main-thread call by default',
      fn: async () => {
        expect(shouldForceBackground({ command: 'ls', env: {}, },),).toBe(true,);
      },
    },),
    it({
      name: 'leaves a subagent call alone',
      fn: async () => {
        expect(shouldForceBackground({ agentId: 'agent-1', command: 'ls', env: {}, },),).toBe(false,);
      },
    },),
    it({
      name: 'leaves a call alone when the opt-out is off',
      fn: async () => {
        expect(
          shouldForceBackground({ command: 'ls', env: { [FORCE_BACKGROUND_ENV]: 'off', }, },),
        ).toBe(false,);
      },
    },),
    it({
      name: 'still forces for any value other than off',
      fn: async () => {
        expect(
          shouldForceBackground({ command: 'ls', env: { [FORCE_BACKGROUND_ENV]: 'on', }, },),
        ).toBe(true,);
        expect(
          shouldForceBackground({ command: 'ls', env: { [FORCE_BACKGROUND_ENV]: '', }, },),
        ).toBe(true,);
      },
    },),
    it({
      name: 'leaves a plain sleep in the foreground',
      fn: async () => {
        expect(shouldForceBackground({ command: 'sleep 30', env: {}, },),).toBe(false,);
      },
    },),
    it({
      name: 'forces a sleep that is chained with other work',
      fn: async () => {
        expect(shouldForceBackground({ command: 'sleep 30 && ls', env: {}, },),).toBe(true,);
      },
    },),
  ],
},);

await describe({
  name: isPlainSleep.name,
  children: [
    it({
      name: 'accepts an integer duration',
      fn: async () => {
        expect(isPlainSleep('sleep 30',),).toBe(true,);
      },
    },),
    it({
      name: 'accepts a fractional duration and surrounding whitespace',
      fn: async () => {
        expect(isPlainSleep('  sleep   0.5\n',),).toBe(true,);
        expect(isPlainSleep('sleep\t2',),).toBe(true,);
      },
    },),
    it({
      name: 'rejects a missing, suffixed, or malformed duration',
      fn: async () => {
        expect(isPlainSleep('sleep',),).toBe(false,);
        expect(isPlainSleep('sleep 1m',),).toBe(false,);
        expect(isPlainSleep('sleep .',),).toBe(false,);
        expect(isPlainSleep('sleep 1.2.3',),).toBe(false,);
        expect(isPlainSleep('sleep -1',),).toBe(false,);
      },
    },),
    it({
      name: 'rejects extra arguments, chaining, redirects, and lookalike words',
      fn: async () => {
        expect(isPlainSleep('sleep 1 2',),).toBe(false,);
        expect(isPlainSleep('sleep 5; ls',),).toBe(false,);
        expect(isPlainSleep('sleep 5 && ls',),).toBe(false,);
        expect(isPlainSleep('sleep 5 > out.txt',),).toBe(false,);
        expect(isPlainSleep('sleeper 5',),).toBe(false,);
        expect(isPlainSleep('sleep5',),).toBe(false,);
      },
    },),
  ],
},);
