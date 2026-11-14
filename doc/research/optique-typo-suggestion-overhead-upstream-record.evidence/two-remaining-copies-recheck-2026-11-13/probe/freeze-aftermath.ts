// What a failed Object.freeze() leaves behind on a state that is a view. Object.freeze() on a view throws, and the
// question is whether the view still works afterwards: freezing first makes the target array non-extensible, and a
// proxy over a non-extensible target may no longer report keys the target lacks.
//
// Runs on Deno, Node and Bun from a source checkout, with no import map:
//   node freeze-aftermath.ts <checkout folder>

const checkout = (globalThis as { process?: { argv?: string[]; }; }).process?.argv?.[2];

if (checkout === undefined) throw new Error('name the checkout folder',);

const source = `${checkout}/packages/core/src`;
const { multiple, } = await import(`${source}/modifiers.ts`);
const { argument, } = await import(`${source}/primitives.ts`);
const { string, } = await import(`${source}/valueparser.ts`);

const parser = multiple(argument(string(),),);
const first = parser.parse({ buffer: ['a', 'b', 'c', 'd',], state: parser.initialState, optionsTerminated: false, usage: parser.usage, },);
const second = parser.parse(first.next,);

function attempt(run: () => unknown,): unknown {
  try {
    return { returned: run(), };
  }
  catch (error) {
    return { threw: `${(error as Error).name}: ${(error as Error).message}`.slice(0, 160,), };
  }
}

const facts: Record<string, unknown> = {};

for (const name of ['state', 'buffer',] as const) {
  const value = second.next[name] as unknown[];

  facts[name] = {
    before: {
      keys: attempt(() => Object.keys(value,).length),
      spread: attempt(() => [...value,].length),
      json: attempt(() => JSON.stringify(value,).length),
      isExtensible: attempt(() => Object.isExtensible(value,)),
    },
    freeze: attempt(() => Object.isFrozen(Object.freeze(value,),)),
    after: {
      isExtensible: attempt(() => Object.isExtensible(value,)),
      length: attempt(() => value.length),
      index: attempt(() => JSON.stringify(value[0],)),
      keys: attempt(() => Object.keys(value,).length),
      spread: attempt(() => [...value,].length),
      slice: attempt(() => value.slice(0,).length),
      json: attempt(() => JSON.stringify(value,).length),
      hasZero: attempt(() => 0 in value),
      complete: name === 'state' ? attempt(() => JSON.stringify(parser.complete(value,),)) : undefined,
      parseOn: name === 'state'
        ? attempt(() => parser.parse({ buffer: ['z',], state: value, optionsTerminated: false, usage: parser.usage, },).next.state.length)
        : attempt(() => parser.parse({ buffer: value, state: parser.initialState, optionsTerminated: false, usage: parser.usage, },).next.buffer.length),
    },
  };
}

console.log(JSON.stringify(facts,),);
