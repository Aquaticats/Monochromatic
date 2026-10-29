/**
 Tests for the decision whether an entry's give-up of a provider is set aside. Waiting for a given-up provider buys
 something only when its return would make the entry finishable, so the give-up is set aside when the provider is the
 only writing provider not dry, or when with the provider back no bench floor would be left unmet while without it one
 is, and holds otherwise.

 The providers given up, the time each refusal still holds a provider out and the reading are numbers and sets the case
 moves by hand, and the logger is one that keeps its lines, so no case waits and nothing is asked.

 Fixtures are cat-themed invention; no meter is read and no provider is asked.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
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
 A floor source whose one floor needs Synthetic: its return alone lifts the floor.

 @param dry - per provider, whether it cannot be asked in the view under question

 @returns One clause while Synthetic cannot be asked, none while it can

 @example
 ```ts
 const clauses = floorNeedsSynthetic({ dry: dryExcept({ wet: ['synthetic',], },), },);
 // => []
 ```
 */
function floorNeedsSynthetic({ dry, }: { readonly dry: ProviderRecord<boolean>; },): readonly string[] {
  return dry.synthetic ? ['select 1 of 8 reachable, quorum 4',] : [];
}

/**
 The writing roster of a run that keeps every provider going but Xiaomi, whose one seat is a checker.

 @param provider - provider to check

 @returns Whether the provider keeps the run going while it reads wet

 @example
 ```ts
 const writing = providerRecord({ of: writesExceptXiaomi, },);
 ```
 */
function writesExceptXiaomi(provider: ProviderName,): boolean {
  return provider !== 'xiaomi';
}

/**
 A decision over sets and times the case moves, and the lines it writes.

 @param writing - providers whose wet reading keeps the run going (default: every provider)

 @param unmetFloors - floor source the decision asks, none where the case asks for the writing roster's rule

 @returns Decision, providers given up, milliseconds each provider is still held out, entry's words and
 lines written

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
      writing,
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
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(false,);
        aside.noteReading({ dry: dryExcept({ wet: [], },), },);
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(false,);
        expect(lines,).toEqual([],);
      },
    },),

    it({
      name: 'SETS A GIVE-UP ASIDE BEFORE ANY READING, when nothing is known of the other providers: a writing provider '
        + 'given up is the only one not dry',
      fn: async () => {
        const { aside, given, } = decisionOver({},);
        given.add('synthetic',);
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(true,);
      },
    },),

    it({
      name: 'KEEPS A GIVE-UP IN FORCE WHILE ANOTHER WRITING PROVIDER READS WET and is not held, and says so once',
      fn: async () => {
        const { aside, given, lines, } = decisionOver({},);
        given.add('synthetic',);
        aside.noteReading(
          {
            dry: dryExcept({
              wet: [
                'neuralwatt',
                'synthetic',
              ],
            },),
          },
        );
        aside.noteReading(
          {
            dry: dryExcept({
              wet: [
                'neuralwatt',
                'synthetic',
              ],
            },),
          },
        );
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(false,);
        expect(lines.length,).toBe(1,);
        expect(lines[0],).toContain('warn [reconsider] synthetic: entry CatEntry1 keeps its give-up in force',);
        expect(lines[0],).toContain('meet every bench floor without synthetic',);
      },
    },),

    it({
      name: 'COUNTS NO PROVIDER AS AN ALTERNATIVE THAT IS HELD OUT, GIVEN UP, OR NOT IN THE ROSTER: (a) the provider '
        + 'given up is the only writing provider not dry',
      fn: async () => {
        const { aside, given, remaining, } = decisionOver({ writing: providerRecord({ of: writesExceptXiaomi, },), },);
        given.add('synthetic',);
        aside.noteReading(
          {
            dry: dryExcept({
              wet: [
                'neuralwatt',
                'synthetic',
                'xiaomi',
                'hyper',
              ],
            },),
          },
        );
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(false,);

        remaining.neuralwatt = 60_000;
        given.add('hyper',);
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(true,);
        expect(aside.isAside({ provider: 'hyper', },),).toBe(true,);

        remaining.neuralwatt = 0;
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(false,);
      },
    },),

    it({
      name: 'SAYS WHEN A GIVE-UP IS SET ASIDE BECAUSE THE PROVIDER IS THE ONLY WRITING PROVIDER NOT DRY, once, with the '
        + 'entry and how each other provider reads, and says when it is back in force',
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
        aside.noteReading(
          {
            dry: dryExcept({
              wet: [
                'synthetic',
                'hyper',
              ],
            },),
          },
        );
        aside.noteReading(
          {
            dry: dryExcept({
              wet: [
                'synthetic',
                'hyper',
              ],
            },),
          },
        );
        aside.reconsider();
        expect(lines,).toEqual([
          'warn [reconsider] synthetic: entry CatEntry1 sets its give-up aside and waits out the cooldowns of synthetic as '
            + 'before, because synthetic is the only writing provider not dry (neuralwatt dry, bedrock dry, deepseek dry, '
            + 'xiaomi dry, anthropic dry, hyper held for 90000ms, openrouter dry), so reading it dry would end the run for '
            + 'a provider that serves again after its cooldown; reading synthetic dry would cost the entry what waiting '
            + 'would have saved',
        ],);

        lines.length = 0;
        remaining.hyper = 0;
        aside.reconsider();
        expect(lines,).toEqual([
          'info [reconsider] synthetic: entry CatEntry1 puts its give-up back in force, because the providers left '
            + '(neuralwatt dry, bedrock dry, deepseek dry, xiaomi dry, anthropic dry, hyper wet, openrouter dry) meet '
            + 'every bench floor without synthetic, so its return buys nothing; synthetic stays unreachable until the '
            + 'entry ends',
        ],);
        aside.reconsider();
        expect(lines.length,).toBe(1,);
      },
    },),

    it({
      name: 'SAYS AGAIN AFTER AN ENTRY BEGINS: forgetting what was logged lets the next set-aside be written, with one '
        + 'line for each provider set aside in spending order',
      fn: async () => {
        const { aside, given, lines, } = decisionOver({},);
        given.add('hyper',);
        given.add('synthetic',);
        aside.noteReading(
          {
            dry: dryExcept({
              wet: [
                'synthetic',
                'hyper',
              ],
            },),
          },
        );
        expect(lines.length,).toBe(2,);
        expect(lines[0],).toContain('synthetic: entry CatEntry1 sets its give-up aside',);
        expect(lines[1],).toContain('hyper: entry CatEntry1 sets its give-up aside',);

        aside.forget();
        given.clear();
        given.add('hyper',);
        aside.noteReading({ dry: dryExcept({ wet: ['hyper',], },), },);
        expect(lines.length,).toBe(3,);
        expect(lines[2],).toContain('hyper: entry CatEntry1 sets its give-up aside',);
      },
    },),

    it({
      name: 'NAMES THE RUN WHERE NO ENTRY WAS NAMED',
      fn: async () => {
        const { aside, given, entry, lines, } = decisionOver({},);
        entry.words = 'the run';
        given.add('synthetic',);
        aside.noteReading({ dry: dryExcept({ wet: ['synthetic',], },), },);
        expect(lines[0],).toContain('synthetic: the run sets its give-up aside',);
      },
    },),

    it({
      name: '(b) SETS A GIVE-UP ASIDE WHEN ITS RETURN WOULD LEAVE NO FLOOR UNMET WHERE WITHOUT IT ONE IS, though a writing '
        + 'provider reads wet, and names the floor',
      fn: async () => {
        const { aside, given, lines, } = decisionOver({ unmetFloors: floorNeedsSynthetic, },);
        given.add('synthetic',);
        aside.noteReading(
          {
            dry: dryExcept({
              wet: [
                'deepseek',
                'synthetic',
              ],
            },),
          },
        );
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(true,);
        expect(lines.length,).toBe(1,);
        expect(lines[0],).toContain('warn [reconsider] synthetic: entry CatEntry1 sets its give-up aside',);
        expect(lines[0],).toContain(
          'because with synthetic back no bench floor the entry needs would be left unmet, while without it these stay '
            + 'unmet: select 1 of 8 reachable, quorum 4 (',
        );
        expect(lines[0],).toContain('deepseek wet',);
      },
    },),

    it({
      name: 'KEEPS A GIVE-UP IN FORCE WHEN ITS RETURN WOULD STILL LEAVE A FLOOR UNMET, though a writing provider reads '
        + 'wet: the line names the floors that stay unmet with the provider back',
      fn: async () => {
        const { aside, given, lines, } = decisionOver({ unmetFloors: floorNeedsHyper, },);
        given.add('synthetic',);
        aside.noteReading(
          {
            dry: dryExcept({
              wet: [
                'deepseek',
                'synthetic',
              ],
            },),
          },
        );
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(false,);
        expect(lines,).toEqual([
          'warn [reconsider] synthetic: entry CatEntry1 keeps its give-up in force and stops waiting for synthetic\'s '
            + 'cooldowns, because with synthetic back the providers left (neuralwatt dry, bedrock dry, deepseek wet, '
            + 'xiaomi dry, anthropic dry, hyper dry, openrouter dry) would still leave these bench floors unmet: '
            + 'translators 0 of 8 reachable, floor 2; waiting for synthetic would not make the entry finishable',
        ],);
      },
    },),

    it({
      name: 'KEEPS A GIVE-UP IN FORCE WHEN ITS RETURN LIFTS ONE FLOOR AND LEAVES ANOTHER UNMET, as in XingZ6020, where '
        + 'Synthetic back lifted two benches and left five short',
      fn: async () => {
        const { aside, given, lines, } = decisionOver({
          unmetFloors: function syntheticLiftsOnlyTheSelectBench({ dry, },): readonly string[] {
            return dry.synthetic
              ? [
                'select 1 of 8 reachable, quorum 4',
                'slate 1 of 7 reachable, quorum 4',
              ]
              : ['slate 1 of 7 reachable, quorum 4',];
          },
        },);
        given.add('synthetic',);
        aside.noteReading(
          {
            dry: dryExcept({
              wet: [
                'deepseek',
                'synthetic',
              ],
            },),
          },
        );
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(false,);
        expect(lines.length,).toBe(1,);
        expect(lines[0],).toContain('would still leave these bench floors unmet: slate 1 of 7 reachable, quorum 4;',);
      },
    },),

    it({
      name: 'MOVES BETWEEN ASIDE AND HELD AS THE PROVIDERS LEFT CHANGE: a give-up whose return cures the floor is set '
        + 'aside, put back in force once another provider meets the floor without it, and set aside again when that '
        + 'provider is held',
      fn: async () => {
        const { aside, given, remaining, lines, } = decisionOver({
          unmetFloors: function floorNeedsOneOfThem({ dry, },): readonly string[] {
            return (dry.synthetic && dry.hyper) ? ['select 1 of 8 reachable, quorum 4',] : [];
          },
        },);
        given.add('synthetic',);
        aside.noteReading(
          {
            dry: dryExcept({
              wet: [
                'deepseek',
                'synthetic',
              ],
            },),
          },
        );
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(true,);
        aside.noteReading(
          {
            dry: dryExcept({
              wet: [
                'deepseek',
                'hyper',
                'synthetic',
              ],
            },),
          },
        );
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(false,);
        expect(lines.length,).toBe(2,);
        expect(lines[1],).toContain(
          'info [reconsider] synthetic: entry CatEntry1 puts its give-up back in force, because the providers left',
        );
        expect(lines[1],).toContain('meet every bench floor without synthetic',);

        // Hyper is held: the floor is unmet without Synthetic again, and Synthetic's return cures it.
        remaining.hyper = 60_000;
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(true,);
        aside.reconsider();
        expect(lines.length,).toBe(3,);
        expect(lines[2],).toContain('warn [reconsider] synthetic: entry CatEntry1 sets its give-up aside',);
      },
    },),

    it({
      name: 'DECIDES FOR EACH PROVIDER GIVEN UP: Synthetic\'s return cures the floor and is set aside, Hyper\'s return '
        + 'leaves it unmet and holds, though both are given up beside a wet DeepSeek',
      fn: async () => {
        const { aside, given, } = decisionOver({ unmetFloors: floorNeedsSynthetic, },);
        given.add('synthetic',);
        given.add('hyper',);
        aside.noteReading(
          {
            dry: dryExcept({
              wet: [
                'deepseek',
                'synthetic',
                'hyper',
              ],
            },),
          },
        );
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(true,);
        expect(aside.isAside({ provider: 'hyper', },),).toBe(false,);
      },
    },),

    it({
      name: 'DOES NOT ADD BACK A GIVEN-UP PROVIDER WHOSE OWN METER READS DRY: waiting does not bring it back, so its '
        + 'return cures nothing and the give-up holds',
      fn: async () => {
        const { aside, given, } = decisionOver({ unmetFloors: floorNeedsSynthetic, },);
        given.add('synthetic',);
        // Its meter reads spendable: the return cures the floor.
        aside.noteReading(
          {
            dry: dryExcept({
              wet: [
                'deepseek',
                'synthetic',
              ],
            },),
          },
        );
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(true,);

        // Its meter reads dry: no wait brings it back, so the floor stays unmet either way.
        aside.noteReading({ dry: dryExcept({ wet: ['deepseek',], },), },);
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(false,);
      },
    },),

    it({
      name: 'ASKS THE FLOOR SOURCE TWICE, with the providers left and then with the given-up provider back: wet only '
        + 'where the reading is wet, the provider is not held and the entry has not given it up',
      fn: async () => {
        /**
         The views the floor source was asked about.
         */
        const views: ProviderRecord<boolean>[] = [];
        const { aside, given, remaining, } = decisionOver({
          unmetFloors: function unmetFloors({ dry, },): readonly string[] {
            views.push(dry,);
            return ['select 1 of 8 reachable, quorum 4',];
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
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(false,);
        expect(views,).toEqual([
          dryExcept({
            wet: [
              'neuralwatt',
              'xiaomi',
            ],
          },),
          dryExcept({
            wet: [
              'neuralwatt',
              'synthetic',
              'xiaomi',
            ],
          },),
        ],);
      },
    },),

    it({
      name: '(a) READS THE ROSTER AND NOT THE FLOORS: a wet provider outside the roster is no alternative, though the '
        + 'floors say it carries a bench',
      fn: async () => {
        const { aside, given, lines, } = decisionOver({
          writing: providerRecord({ of: writesExceptXiaomi, },),
          unmetFloors: function xiaomiCarriesTheBench({ dry, },): readonly string[] {
            return dry.xiaomi ? ['checkers 0 of 3 reachable, quorum 2',] : [];
          },
        },);
        given.add('synthetic',);
        aside.noteReading(
          {
            dry: dryExcept({
              wet: [
                'xiaomi',
                'synthetic',
              ],
            },),
          },
        );
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(true,);
        expect(lines[0],).toContain('because synthetic is the only writing provider not dry',);
      },
    },),

    it({
      name: 'READS THE ROSTER IN THE FLOOR OF A CALLER WITH NO BENCHES TOO: Xiaomi given up and wet, outside the roster, '
        + 'is no writing provider, so its return buys the entry nothing and the give-up holds',
      fn: async () => {
        const { aside, given, } = decisionOver({ writing: providerRecord({ of: writesExceptXiaomi, },), },);
        given.add('xiaomi',);
        aside.noteReading({ dry: dryExcept({ wet: ['xiaomi',], },), },);
        expect(aside.isAside({ provider: 'xiaomi', },),).toBe(false,);
      },
    },),

    it({
      name: '(a) NEEDS THE GIVEN-UP PROVIDER TO WRITE AND TO BE NOT DRY: Xiaomi given up writes nothing, and a provider '
        + 'whose own meter reads dry is not the one not dry',
      fn: async () => {
        const { aside, given, } = decisionOver({
          writing: providerRecord({ of: writesExceptXiaomi, },),
          unmetFloors: floorNeedsHyper,
        },);
        given.add('xiaomi',);
        aside.noteReading({ dry: dryExcept({ wet: ['xiaomi',], },), },);
        expect(aside.isAside({ provider: 'xiaomi', },),).toBe(false,);

        given.clear();
        given.add('synthetic',);
        aside.noteReading({ dry: dryExcept({ wet: [], },), },);
        expect(aside.isAside({ provider: 'synthetic', },),).toBe(false,);
      },
    },),
  ],
},);
