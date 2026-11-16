// Prints how the state, the buffer and the consumed tokens that multiple(argument()) hands back look to a caller
// that does more than read them as arrays: printing, cloning, freezing checks and in-place edits. Run on a build
// with plain arrays and on one whose state and buffer are views, the two outputs show what a caller could notice.
//
// Runs on Deno, Node and Bun from a source checkout, with no import map:
//   node view-facts.ts <checkout folder>
//   deno run --allow-read view-facts.ts <checkout folder>
//   bun view-facts.ts <checkout folder>

import { inspect, isDeepStrictEqual, types, } from 'node:util';

const checkout = (globalThis as { process?: { argv?: string[]; }; }).process?.argv?.[2];

if (checkout === undefined) throw new Error('name the checkout folder',);

const source = `${checkout}/packages/core/src`;
const { multiple, } = await import(`${source}/modifiers.ts`);
const { argument, } = await import(`${source}/primitives.ts`);
const { string, } = await import(`${source}/valueparser.ts`);

const parser = multiple(argument(string(),),);

function parsed(): { state: unknown[]; buffer: string[]; consumed: string[]; } {
  const first = parser.parse({ buffer: ['a', 'b', 'c', 'd',], state: parser.initialState, optionsTerminated: false, usage: parser.usage, },);
  const second = parser.parse(first.next,);

  return { state: second.next.state, buffer: second.next.buffer, consumed: second.consumed, };
}

function attempt(run: () => unknown,): unknown {
  try {
    return { returned: run(), };
  }
  catch (error) {
    return { threw: `${(error as Error).name}: ${(error as Error).message}`, };
  }
}

const runtime = (globalThis as { Deno?: { version: { deno: string; }; }; Bun?: { version: string; }; }).Deno?.version.deno !== undefined
  ? `deno ${(globalThis as { Deno?: { version: { deno: string; }; }; }).Deno?.version.deno}`
  : (globalThis as { Bun?: { version: string; }; }).Bun?.version !== undefined
  ? `bun ${(globalThis as { Bun?: { version: string; }; }).Bun?.version}`
  : `node ${(globalThis as { process?: { versions?: { node?: string; }; }; }).process?.versions?.node}`;
const facts: Record<string, unknown> = { runtime, checkout, };

for (const name of ['state', 'buffer', 'consumed',] as const) {
  const value = parsed()[name];

  facts[name] = {
    isArray: Array.isArray(value,),
    isProxy: types.isProxy(value,),
    length: value.length,
    json: JSON.stringify(value,),
    inspect: inspect(value, { depth: 0, },),
    templateString: attempt(() => `${value}`),
    toStringTag: Object.prototype.toString.call(value,),
    constructorIsArray: value.constructor === Array,
    isFrozen: Object.isFrozen(value,),
    isExtensible: Object.isExtensible(value,),
    deepStrictEqualToItsSpread: isDeepStrictEqual(value, [...value,],),
    arrayFromLength: Array.from(value,).length,
    structuredClone: attempt(() => JSON.stringify(structuredClone(value,),)),
    freeze: attempt(() => Object.isFrozen(Object.freeze(parsed()[name],),)),
    writeElement: attempt(() => {
      const target = parsed()[name] as unknown[];

      target[0] = 'changed';

      return JSON.stringify(target,);
    },),
    push: attempt(() => {
      const target = parsed()[name] as unknown[];

      target.push('pushed',);

      return target.length;
    },),
    sortInPlace: attempt(() => (parsed()[name] as unknown[]).sort().length),
    reverseInPlace: attempt(() => (parsed()[name] as unknown[]).reverse().length),
    setLength: attempt(() => {
      const target = parsed()[name] as unknown[];

      target.length = 0;

      return target.length;
    },),
    toSorted: attempt(() => (parsed()[name] as unknown[]).toSorted().length),
    withElement: attempt(() => JSON.stringify((parsed()[name] as unknown[]).with(0, 'replaced',),).length),
  };
}

console.log(JSON.stringify(facts,),);
