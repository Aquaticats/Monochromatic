// What a write to a state leaves behind when that state is a view: each kind of write is tried on a fresh state,
// and the state is read back afterwards. A write that was refused should leave nothing; one that raised nothing
// should have taken effect the way it does on a plain array.
//
// Runs on Deno, Node and Bun from a source checkout, with no import map:
//   node write-aftermath.ts <checkout folder>

const checkout = (globalThis as { process?: { argv?: string[]; }; }).process?.argv?.[2];

if (checkout === undefined) throw new Error('name the checkout folder',);

const source = `${checkout}/packages/core/src`;
const { multiple, } = await import(`${source}/modifiers.ts`);
const { argument, } = await import(`${source}/primitives.ts`);
const { string, } = await import(`${source}/valueparser.ts`);

const parser = multiple(argument(string(),),);

function freshState(): unknown[] {
  const first = parser.parse({ buffer: ['a', 'b', 'c',], state: parser.initialState, optionsTerminated: false, usage: parser.usage, },);

  return parser.parse(first.next,).next.state;
}

function attempt(run: () => unknown,): unknown {
  try {
    return { returned: run(), };
  }
  catch (error) {
    return { threw: `${(error as Error).name}: ${(error as Error).message}`.slice(0, 120,), };
  }
}

const WRITES: Record<string, (state: unknown[],) => unknown> = {
  push: (state,) => state.push('pushed',),
  unshift: (state,) => state.unshift('unshifted',),
  assignAtLength: (state,) => {
    state[state.length] = 'assigned';

    return 'no error';
  },
  assignPastLength: (state,) => {
    state[state.length + 3] = 'assigned';

    return 'no error';
  },
  assignFirst: (state,) => {
    state[0] = 'assigned';

    return 'no error';
  },
  defineFirst: (state,) => {
    Object.defineProperty(state, '0', { value: 'defined', configurable: true, enumerable: true, writable: true, },);

    return 'no error';
  },
  defineAtLength: (state,) => {
    Object.defineProperty(state, String(state.length,), { value: 'defined', configurable: true, enumerable: true, writable: true, },);

    return 'no error';
  },
  deleteFirst: (state,) => delete state[0],
  splice: (state,) => state.splice(0, 1,).length,
  fill: (state,) => state.fill('filled',).length,
  copyWithin: (state,) => state.copyWithin(0, 1,).length,
  pop: (state,) => typeof state.pop(),
  setSymbol: (state,) => {
    (state as unknown as Record<symbol, unknown>)[Symbol.for('@recheck/write-aftermath',)] = 1;

    return 'no error';
  },
};

const facts: Record<string, unknown> = {};

for (const [name, write,] of Object.entries(WRITES,)) {
  const state = freshState();
  const lengthBefore = state.length;
  const outcome = attempt(() => write(state,));

  facts[name] = {
    outcome,
    after: {
      length: attempt(() => state.length),
      keys: attempt(() => Object.keys(state,).join(',',)),
      elementAtOldLength: attempt(() => JSON.stringify(state[lengthBefore],) ?? 'undefined'),
      oldLengthIn: attempt(() => lengthBefore in state),
      first: attempt(() => JSON.stringify(state[0],)),
      spreadLength: attempt(() => [...state,].length),
      json: attempt(() => JSON.stringify(state,)),
      complete: attempt(() => JSON.stringify(parser.complete(state,),)),
    },
  };
}

console.log(JSON.stringify(facts,),);
