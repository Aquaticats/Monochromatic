/**
 Tests for the decision whether an entry's give-up of a provider is set aside: the give-ups hold while another writing
 provider reads wet, and are set aside when none does, so that giving a provider up never leaves every provider dry.

 The providers given up, the holds and the reading are numbers and sets the case moves by hand, and the logger is one
 that keeps its lines, so no case waits and nothing is asked.

 Fixtures are cat-themed invention; no meter is read and no provider is asked.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  type ProviderName,
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  createGiveUpAside,
  EVERY_PROVIDER_COUNTS,
} from '../dist/final/node/index.mjs';

/**
 A reading in which only the named providers are not dry.

 @param wet - providers the reading found wet

 @returns Dryness per provider

 @example
 ```ts
 const dry = dryExcept({ wet: ['neuralwatt',], },);
 ```
 */
function dryExcept({ wet, }: { readonly wet: readonly ProviderName[]; },): ProviderRecord<boolean> {
  return providerRecord({
    of: function isDry(provider,): boolean {
      return !wet.includes(provider,);
    },
  },);
}

/**
 The hold of a provider no refusal has held: none.

 @returns Zero milliseconds

 @example
 ```ts
 const holds = providerRecord({ of: holdNothing, },);
 ```
 */
function holdNothing(): number {
  return 0;
}

/**
 A decision over sets and holds the case moves, and the lines it writes.

 @param writing - the providers whose wet reading keeps the run going (default: every provider)

 @returns The decision, the providers given up, the holds in milliseconds, the entry's words and the lines written

 @example
 ```ts
 const { aside, given, } = decisionOver({},);
 ```
 */
function decisionOver(
  { writing = EVERY_PROVIDER_COUNTS, }: { readonly writing?: ProviderRecord<boolean>; },
): {
  readonly aside: ReturnType<typeof createGiveUpAside>;
  readonly given: Set<ProviderName>;
  readonly holds: Record<ProviderName, number>;
  readonly entry: { words: string; };
  readonly lines: string[];
} {
  /**
   The providers the entry has given up.
   */
  const given = new Set<ProviderName>();
  /**
   How much longer each refusal holds a provider out.
   */
  const holds: Record<ProviderName, number> = providerRecord({ of: holdNothing, },);
  /**
   The entry, worded for the lines.
   */
  const entry = { words: 'entry CatEntry1', };
  /**
   Lines the decision writes.
   */
  const lines: string[] = [];
  return {
    given,
    holds,
    entry,
    lines,
    aside: createGiveUpAside({
      writing,
      given,
      left: function left({ provider, },): number {
        return holds[provider];
      },
      entryWords: function entryWords(): string {
        return entry.words;
      },
      l: levelCapturingLogger({ lines, },),
    },),
  };
}

await describe({
  name: createGiveUpAside.name,
  concurrency: 1,
  children: [
    it({
      name: 'SETS NOTHING ASIDE WHILE NO PROVIDER IS GIVEN UP, whatever the reading, and says nothing',
      fn: async () => {
        const { aside, lines, } = decisionOver({},);
        expect(aside.isAside(),).toBe(false,);
        aside.noteReading({ dry: dryExcept({ wet: [], },), },);
        expect(aside.isAside(),).toBe(false,);
        expect(lines,).toEqual([],);
      },
    },),

    it({
      name: 'SETS A GIVE-UP ASIDE BEFORE ANY READING, when nothing is known of the other providers',
      fn: async () => {
        const { aside, given, } = decisionOver({},);
        given.add('synthetic',);
        expect(aside.isAside(),).toBe(true,);
      },
    },),

    it({
      name: 'KEEPS A GIVE-UP IN FORCE WHILE ANOTHER WRITING PROVIDER READS WET and is not held',
      fn: async () => {
        const { aside, given, lines, } = decisionOver({},);
        given.add('synthetic',);
        aside.noteReading({ dry: dryExcept({ wet: ['neuralwatt',], },), },);
        expect(aside.isAside(),).toBe(false,);
        expect(lines,).toEqual([],);
      },
    },),

    it({
      name: 'COUNTS NO PROVIDER AS AN ALTERNATIVE THAT IS HELD OUT, GIVEN UP, OR NOT IN THE ROSTER',
      fn: async () => {
        const { aside, given, holds, } = decisionOver({
          writing: providerRecord({
            of: function writesExceptXiaomi(provider,): boolean {
              return provider !== 'xiaomi';
            },
          },),
        },);
        given.add('synthetic',);
        aside.noteReading({
          dry: dryExcept({
            wet: [
              'neuralwatt',
              'xiaomi',
              'hyper',
            ],
          },),
        },);
        expect(aside.isAside(),).toBe(false,);

        holds.neuralwatt = 60_000;
        given.add('hyper',);
        expect(aside.isAside(),).toBe(true,);

        holds.neuralwatt = 0;
        expect(aside.isAside(),).toBe(false,);
      },
    },),

    it({
      name: 'SAYS WHEN A GIVE-UP IS SET ASIDE, once, with the entry and how each other writing provider reads, and '
        + 'says when it is back in force',
      fn: async () => {
        const { aside, given, holds, lines, } = decisionOver({
          writing: providerRecord({
            of: function writesNeuralwattSyntheticAndHyper(provider,): boolean {
              return (provider === 'neuralwatt') || (provider === 'synthetic') || (provider === 'hyper');
            },
          },),
        },);
        given.add('synthetic',);
        holds.hyper = 90_000;
        aside.noteReading({ dry: dryExcept({ wet: ['hyper',], },), },);
        aside.noteReading({ dry: dryExcept({ wet: ['hyper',], },), },);
        aside.reconsider();
        expect(lines,).toEqual([
          'warn [reconsider] synthetic: entry CatEntry1 sets its give-up aside and waits out the cooldowns of synthetic as '
            + 'before, because no other writing provider reads wet (neuralwatt dry, hyper held for 90000ms), and reading '
            + 'synthetic dry would end the run for a provider that serves again after its cooldown',
        ],);

        lines.length = 0;
        holds.hyper = 0;
        aside.reconsider();
        expect(lines,).toEqual([
          'info [reconsider] synthetic: entry CatEntry1 puts its give-up back in force, because hyper reads wet again, '
            + 'so synthetic stays unreachable until the entry ends',
        ],);
        aside.reconsider();
        expect(lines.length,).toBe(1,);
      },
    },),

    it({
      name: 'SAYS AGAIN AFTER AN ENTRY BEGINS: forgetting what was logged lets the next set-aside be written, and '
        + 'names every provider set aside in spending order',
      fn: async () => {
        const { aside, given, lines, } = decisionOver({},);
        given.add('hyper',);
        given.add('synthetic',);
        aside.noteReading({ dry: dryExcept({ wet: [], },), },);
        expect(lines.length,).toBe(1,);
        expect(lines[0],).toContain('synthetic, hyper: entry CatEntry1 sets its give-up aside',);

        aside.forget();
        given.clear();
        given.add('hyper',);
        aside.noteReading({ dry: dryExcept({ wet: [], },), },);
        expect(lines.length,).toBe(2,);
        expect(lines[1],).toContain('hyper: entry CatEntry1 sets its give-up aside',);
      },
    },),

    it({
      name: 'NAMES THE RUN WHERE NO ENTRY WAS NAMED',
      fn: async () => {
        const { aside, given, entry, lines, } = decisionOver({},);
        entry.words = 'the run';
        given.add('synthetic',);
        aside.noteReading({ dry: dryExcept({ wet: [], },), },);
        expect(lines[0],).toContain('synthetic: the run sets its give-up aside',);
      },
    },),
  ],
},);
