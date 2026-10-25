/**
 Tests for the decision whether an entry's give-up of a provider is set aside: the give-ups hold while another writing
 provider reads wet, and are set aside when none does, so that giving a provider up never leaves every provider dry.

 The providers given up, the time each refusal still holds a provider out and the reading are numbers and sets the case
 moves by hand, and the logger is one that keeps its lines, so no case waits and nothing is asked.

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
  writingProviderFloor,
} from '../dist/final/node/index.mjs';
import { dryExcept, } from './dry-except.test-fixture.ts';

/**
 The time a provider no refusal has held is still held out: none.

 @returns Zero milliseconds

 @example
 ```ts
 const remaining = providerRecord({ of: heldForNothing, },);
 ```
 */
function heldForNothing(): number {
  return 0;
}

/**
 A floor source whose one floor needs Hyper: translators are served by Hyper alone among the providers the case reads.

 @param dry - per provider, whether it cannot be asked in the view under question

 @returns One clause while Hyper cannot be asked, none while it can

 @example
 ```ts
 const clauses = floorNeedsHyper({ dry: dryExcept({ wet: [], },), },);
 // => ['translators 0 of 8 reachable, floor 2']
 ```
 */
function floorNeedsHyper({ dry, }: { readonly dry: ProviderRecord<boolean>; },): readonly string[] {
  return dry.hyper ? ['translators 0 of 8 reachable, floor 2',] : [];
}

/**
 A decision over sets and times the case moves, and the lines it writes.

 @param writing - the providers whose wet reading keeps the run going (default: every provider)

 @param unmetFloors - the floor source the decision asks, none where the case asks for the writing roster's rule

 @returns The decision, the providers given up, the milliseconds each provider is still held out, the entry's words and
 the lines written

 @example
 ```ts
 const { aside, given, } = decisionOver({},);
 ```
 */
function decisionOver(
  {
    writing = EVERY_PROVIDER_COUNTS,
    unmetFloors,
  }: {
    readonly writing?: ProviderRecord<boolean>;
    readonly unmetFloors?: (args: { readonly dry: ProviderRecord<boolean>; },) => readonly string[];
  },
): {
  readonly aside: ReturnType<typeof createGiveUpAside>;
  readonly given: Set<ProviderName>;
  readonly remaining: Record<ProviderName, number>;
  readonly entry: { words: string; };
  readonly lines: string[];
} {
  /**
   The providers the entry has given up.
   */
  const given = new Set<ProviderName>();
  /**
   How much longer a refusal holds each provider out, in milliseconds.
   */
  const remaining: Record<ProviderName, number> = providerRecord({ of: heldForNothing, },);
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
    remaining,
    entry,
    lines,
    aside: createGiveUpAside({
      unmetFloors: unmetFloors ?? writingProviderFloor({ writing, },),
      given,
      left: function left({ provider, },): number {
        return remaining[provider];
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
        const { aside, given, remaining, } = decisionOver({
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

        remaining.neuralwatt = 60_000;
        given.add('hyper',);
        expect(aside.isAside(),).toBe(true,);

        remaining.neuralwatt = 0;
        expect(aside.isAside(),).toBe(false,);
      },
    },),

    it({
      name: 'SAYS WHEN A GIVE-UP IS SET ASIDE, once, with the entry and how each other writing provider reads, and '
        + 'says when it is back in force',
      fn: async () => {
        const { aside, given, remaining, lines, } = decisionOver({
          writing: providerRecord({
            of: function writesNeuralwattSyntheticAndHyper(provider,): boolean {
              return (provider === 'neuralwatt') || (provider === 'synthetic') || (provider === 'hyper');
            },
          },),
        },);
        given.add('synthetic',);
        remaining.hyper = 90_000;
        aside.noteReading({ dry: dryExcept({ wet: ['hyper',], },), },);
        aside.noteReading({ dry: dryExcept({ wet: ['hyper',], },), },);
        aside.reconsider();
        expect(lines,).toEqual([
          'warn [reconsider] synthetic: entry CatEntry1 sets its give-up aside and waits out the cooldowns of synthetic as '
            + 'before, because the providers left (neuralwatt dry, bedrock dry, deepseek dry, xiaomi dry, anthropic dry, '
            + 'hyper held for 90000ms, openrouter dry) cannot meet every bench floor without synthetic: no writing '
            + 'provider can be asked; reading synthetic dry would cost the entry what waiting would have saved',
        ],);

        lines.length = 0;
        remaining.hyper = 0;
        aside.reconsider();
        expect(lines,).toEqual([
          'info [reconsider] synthetic: entry CatEntry1 puts its give-up back in force, because the providers left '
            + '(hyper) meet every bench floor again, so synthetic stays unreachable until the entry ends',
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

    it({
      name: 'SETS A GIVE-UP ASIDE WHILE A BENCH FLOOR STAYS UNMET, though a writing provider reads wet, and keeps it in '
        + 'force once the providers left meet every floor',
      fn: async () => {
        const { aside, given, remaining, } = decisionOver({ unmetFloors: floorNeedsHyper, },);
        given.add('synthetic',);
        aside.noteReading({ dry: dryExcept({ wet: ['deepseek',], },), },);
        expect(aside.isAside(),).toBe(true,);

        aside.noteReading({
          dry: dryExcept({
            wet: [
              'deepseek',
              'hyper',
            ],
          },),
        },);
        expect(aside.isAside(),).toBe(false,);

        // THE FLOOR IS LOST AGAIN WHEN THE PROVIDER THAT MET IT IS HELD, and found again when its hold ends.
        remaining.hyper = 60_000;
        expect(aside.isAside(),).toBe(true,);
        remaining.hyper = 0;
        expect(aside.isAside(),).toBe(false,);

        // AND WHEN IT IS GIVEN UP TOO: a provider given up counts as unavailable, whatever the reading says.
        given.add('hyper',);
        expect(aside.isAside(),).toBe(true,);
      },
    },),

    it({
      name: 'ASKS THE FLOOR SOURCE ABOUT THE PROVIDERS LEFT ALONE: wet only where the reading is wet, the provider is not '
        + 'held and the entry has not given it up',
      fn: async () => {
        /**
         The views the floor source was asked about.
         */
        const views: ProviderRecord<boolean>[] = [];
        const { aside, given, remaining, } = decisionOver({
          unmetFloors: function unmetFloors({ dry, },): readonly string[] {
            views.push(dry,);
            return [];
          },
        },);
        given.add('synthetic',);
        remaining.hyper = 5_000;
        aside.noteReading({
          dry: dryExcept({
            wet: [
              'neuralwatt',
              'synthetic',
              'hyper',
              'xiaomi',
            ],
          },),
        },);
        views.length = 0;
        expect(aside.isAside(),).toBe(false,);
        expect(views,).toEqual([
          dryExcept({
            wet: [
              'neuralwatt',
              'xiaomi',
            ],
          },),
        ],);
      },
    },),

    it({
      name: 'LETS THE FLOOR SOURCE REPLACE THE WRITING ROSTER: a wet provider outside the roster is counted where the '
        + 'floors say it carries a bench',
      fn: async () => {
        const { aside, given, } = decisionOver({
          writing: providerRecord({
            of: function writesExceptXiaomi(provider,): boolean {
              return provider !== 'xiaomi';
            },
          },),
          unmetFloors: function xiaomiCarriesTheBench({ dry, },): readonly string[] {
            return dry.xiaomi ? ['checkers 0 of 3 reachable, quorum 2',] : [];
          },
        },);
        given.add('synthetic',);
        aside.noteReading({ dry: dryExcept({ wet: ['xiaomi',], },), },);
        expect(aside.isAside(),).toBe(false,);
      },
    },),

    it({
      name: 'NAMES THE UNMET FLOORS in the line that sets a give-up aside, with how each other provider reads, and says '
        + 'when the providers left meet every floor again',
      fn: async () => {
        const { aside, given, remaining, lines, } = decisionOver({ unmetFloors: floorNeedsHyper, },);
        given.add('synthetic',);
        remaining.hyper = 90_000;
        aside.noteReading({ dry: dryExcept({ wet: ['deepseek',], },), },);
        aside.noteReading({ dry: dryExcept({ wet: ['deepseek',], },), },);
        expect(lines,).toEqual([
          'warn [reconsider] synthetic: entry CatEntry1 sets its give-up aside and waits out the cooldowns of synthetic as '
            + 'before, because the providers left (neuralwatt dry, bedrock dry, deepseek wet, xiaomi dry, anthropic dry, '
            + 'hyper held for 90000ms, openrouter dry) cannot meet every bench floor without synthetic: translators 0 of 8 '
            + 'reachable, floor 2; reading synthetic dry would cost the entry what waiting would have saved',
        ],);

        lines.length = 0;
        remaining.hyper = 0;
        aside.noteReading({
          dry: dryExcept({
            wet: [
              'deepseek',
              'hyper',
            ],
          },),
        },);
        expect(lines,).toEqual([
          'info [reconsider] synthetic: entry CatEntry1 puts its give-up back in force, because the providers left '
            + '(deepseek, hyper) meet every bench floor again, so synthetic stays unreachable until the entry ends',
        ],);
      },
    },),
  ],
},);
