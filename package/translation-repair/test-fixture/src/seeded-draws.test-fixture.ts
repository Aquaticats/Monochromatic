//region Seeded draws
// A SMALL RANDOM GENERATOR AND A PICK FROM A LIST, for tests that read generated pages and need the same pages on every run.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 Multiplier of the linear congruential generator (Numerical Recipes).
 */
const GENERATOR_MULTIPLIER = 1_664_525;

/**
 Increment of the same generator.
 */
const GENERATOR_INCREMENT = 1_013_904_223;

/**
 How many values the generator can state, which is two to the power of its width of thirty-two bits.
 */
const GENERATOR_MODULUS = 4_294_967_296;

/**
 How many draws are spent before the first is given out, since the first draws of neighbouring seeds are alike.
 */
const SPENT_DRAWS = 8;

/**
 A small random generator, so the pages are the same on every run.

 @param seed - what the generator starts from

 @returns Function that draws a number from zero up to one

 @example
 ```ts
 const rand = randomOf({ seed: 3, },);
 ```
 */
export function randomOf({ seed, }: { readonly seed: number; },): () => number {
  /**
   The generator's state, mutated only by the draws.
   */
  const state = { a: seed % GENERATOR_MODULUS, };
  /**
   The next draw: a linear congruential step.

   @returns Number from zero up to one
   */
  function draw(): number {
    state.a = ((state.a * GENERATOR_MULTIPLIER) + GENERATOR_INCREMENT) % GENERATOR_MODULUS;
    return state.a / GENERATOR_MODULUS;
  }
  for (let spent = 0; spent < SPENT_DRAWS; spent += 1)
    draw();
  return draw;
}

/**
 Picks one of the options by a draw from the generator.

 @param options - what to pick from

 @param rand - generator

 @returns One option

 @throws {@link Error} when there is no option

 @example
 ```ts
 const eol = pickFrom({ options: ['\n', '\r\n',], rand, },);
 ```
 */
export function pickFrom<Option,>(
  {
    options,
    rand,
  }: {
    readonly options: readonly Option[];
    readonly rand: () => number;
  },
): Option {
  /**
   The option drawn.
   */
  const picked = options[Math.floor(rand() * options.length,)];
  if (picked === undefined)
    throw new Error('no option to pick',);
  return picked;
}

/**
 Draws a whole number from zero up to a bound, which the generator states as a fraction of one.

 @param rand - generator

 @param bound - exclusive upper bound

 @returns Whole number from zero to one less than the bound

 @example
 ```ts
 const at = integerBelow({ rand, bound: 5, },);
 ```
 */
export function integerBelow(
  {
    rand,
    bound,
  }: {
    readonly rand: () => number;
    readonly bound: number;
  },
): number {
  return Math.floor(rand() * bound,);
}

//endregion Seeded draws
