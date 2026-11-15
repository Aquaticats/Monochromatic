// Prints what @optique/core does, as one JSON document, for a fixed set of parsers and inputs, so that two builds of
// the library can be compared by comparing their output. It is written against the public entry points only and
// knows nothing about how a build stores a multiple() state or a token buffer.
//
// What it records:
// - whole parses: the value or the error text of each parser on each input;
// - step by step: after each parse() call of multiple(argument()), what the next state and the next buffer look
//   like to code that reads them as plain arrays (Array.isArray, length, index, spread, slice, JSON);
// - branching: one earlier context extended twice with different tokens, checking that neither result shows the
//   other's item and that the earlier context reads as it did;
// - the same through an async value parser.
//
// Run: deno run --no-config --import-map <map for one build> behaviour-probe.ts

import { object } from "@optique/core/constructs";
import { formatMessage } from "@optique/core/message";
import { multiple, optional } from "@optique/core/modifiers";
import { parse, parseSync } from "@optique/core/parser";
import { argument, option } from "@optique/core/primitives";
import { integer, string } from "@optique/core/valueparser";

type Json = unknown;

const record: Record<string, Json> = {};

function arrayFacts(value: unknown): Json {
  if (value == null || typeof value !== "object") return { notAnObject: String(value) };

  const array = value as readonly unknown[];

  return {
    isArray: Array.isArray(array),
    prototypeIsArray: Object.getPrototypeOf(array) === Array.prototype,
    length: array.length,
    ownIndexKeys: Object.keys(array).length,
    spreadLength: [...array].length,
    sliceFromOneLength: array.slice(1).length,
    mapLength: array.map((item) => item).length,
    concatLength: ([] as unknown[]).concat(array).length,
    entriesLength: [...array.entries()].length,
    hasZero: 0 in array,
    hasLength: array.length in array,
  };
}

function outcome(result: { success: boolean; value?: unknown; error?: unknown }): Json {
  if (result.success) return { success: true, value: result.value };

  return { success: false, error: formatMessage(result.error as Parameters<typeof formatMessage>[0]) };
}

function attempt(run: () => Json): Json {
  try {
    return run();
  } catch (error) {
    return { threw: String(error) };
  }
}

// region Whole parses

const manyTokens = Array.from({ length: 300 }, (_, index) => `token-${index}`);
const inputs: readonly (readonly string[])[] = [
  [],
  ["a"],
  ["a", "b", "c"],
  ["--name", "x", "a", "b"],
  ["a", "--name", "x", "b"],
  ["--", "--name", "x"],
  ["a", "--", "-b", "c"],
  ["1", "2", "3"],
  ["1", "2", "3", "4"],
  ["1", "x"],
  ["-v", "1", "a", "-v", "2", "b"],
  ["--value=3", "z"],
  ["--bogus"],
  ["a", "--bogus"],
  ["a", "a", "a"],
  manyTokens,
  ["--name", "n", ...manyTokens],
  ["--", ...manyTokens],
];
const wholeParsers = {
  nameAndPaths: object({ name: optional(option("--name", string())), paths: multiple(argument(string())) }),
  oneToThreeIntegers: multiple(argument(integer()), { min: 1, max: 3 }),
  valuesAndRest: object({ values: multiple(option("-v", "--value", string())), rest: multiple(argument(string())) }),
  bareMultiple: multiple(argument(string())),
  twoLists: object({ numbers: multiple(option("-n", integer())), words: multiple(argument(string()), { min: 1 }) }),
};

record["whole"] = Object.fromEntries(
  Object.entries(wholeParsers).map(([name, parser]) => [
    name,
    inputs.map((input) => attempt(() => outcome(parseSync(parser as Parameters<typeof parseSync>[0], input)))),
  ]),
);

// endregion Whole parses

// region Step by step

const stepParser = multiple(argument(string()));

function completed(parser: typeof stepParser, state: unknown): Json {
  return attempt(() => outcome(parser.complete(state as Parameters<typeof parser.complete>[0]) as never));
}

function contextFacts(parser: typeof stepParser, context: { buffer: readonly string[]; state: unknown }): Json {
  return {
    buffer: arrayFacts(context.buffer),
    bufferJson: JSON.stringify(context.buffer),
    bufferFirst: context.buffer[0] ?? null,
    bufferLast: context.buffer.at(-1) ?? null,
    bufferJoined: context.buffer.join(" "),
    bufferIncludesB: context.buffer.includes("b"),
    state: arrayFacts(context.state),
    completed: completed(parser, context.state),
  };
}

const stepTokens = ["a", "b", "--", "-c", "d", "e"];
const steps: Json[] = [];
let stepContext: Parameters<typeof stepParser.parse>[0] = {
  buffer: stepTokens,
  state: stepParser.initialState,
  optionsTerminated: false,
  usage: stepParser.usage,
} as Parameters<typeof stepParser.parse>[0];
const contextsByStep: (typeof stepContext)[] = [stepContext];

for (let step = 0; step < stepTokens.length + 1 && stepContext.buffer.length > 0; step += 1) {
  const result = stepParser.parse(stepContext) as {
    success: boolean;
    next?: typeof stepContext;
    consumed?: readonly string[] | number;
    error?: unknown;
  };

  if (!result.success || result.next === undefined) {
    steps.push({ step, failed: outcome(result as never), consumed: result.consumed });
    break;
  }

  steps.push({
    step,
    consumed: arrayFacts(result.consumed),
    consumedJson: JSON.stringify(result.consumed),
    optionsTerminated: result.next.optionsTerminated,
    next: contextFacts(stepParser, result.next),
    inputContextAfter: contextFacts(stepParser, stepContext),
  });
  stepContext = result.next;
  contextsByStep.push(stepContext);
}

record["steps"] = steps;

// endregion Step by step

// region Branching from one earlier context

function branch(parser: typeof stepParser, earlier: typeof stepContext): Json {
  const before = JSON.stringify(contextFacts(parser, earlier));
  const first = parser.parse({ ...earlier, buffer: ["x", "p"] }) as { success: boolean; next: typeof stepContext };
  const second = parser.parse({ ...earlier, buffer: ["y", "q"] }) as { success: boolean; next: typeof stepContext };
  const firstAgain = parser.parse(first.next) as { success: boolean; next: typeof stepContext };
  const secondAgain = parser.parse(second.next) as { success: boolean; next: typeof stepContext };
  const sameTwice = [parser.parse(earlier), parser.parse(earlier)] as { success: boolean; next?: typeof stepContext }[];
  const after = JSON.stringify(contextFacts(parser, earlier));

  return {
    earlierUnchanged: before === after,
    earlier: JSON.parse(after),
    first: contextFacts(parser, first.next),
    second: contextFacts(parser, second.next),
    firstExtended: contextFacts(parser, firstAgain.next),
    secondExtended: contextFacts(parser, secondAgain.next),
    firstAfterAll: contextFacts(parser, first.next),
    secondAfterAll: contextFacts(parser, second.next),
    sameTwice: sameTwice.map((result) => result.next === undefined ? outcome(result as never) : contextFacts(parser, result.next)),
  };
}

record["branchingAppend"] = contextsByStep.slice(0, 4).map((earlier) => attempt(() => branch(stepParser, earlier)));

// An item that takes several tokens, so that a later parse() call replaces the last item instead of appending one.
const pairParser = multiple(object({ a: option("-a", string()), b: optional(option("-b", string())) }));

record["branchingReplaceLast"] = attempt(() => {
  type PairContext = Parameters<typeof pairParser.parse>[0];
  const start = {
    buffer: ["-a", "1", "-b", "2", "-a", "3"],
    state: pairParser.initialState,
    optionsTerminated: false,
    usage: pairParser.usage,
  } as PairContext;
  const afterA = pairParser.parse(start) as { success: boolean; next: PairContext };
  const facts = (context: PairContext): Json => ({
    buffer: JSON.stringify(context.buffer),
    state: arrayFacts(context.state),
    completed: attempt(() => outcome(pairParser.complete(context.state) as never)),
  });
  const before = JSON.stringify(facts(afterA.next));
  const withX = pairParser.parse({ ...afterA.next, buffer: ["-b", "x", "-a", "9"] }) as { success: boolean; next: PairContext };
  const withY = pairParser.parse({ ...afterA.next, buffer: ["-b", "y"] }) as { success: boolean; next: PairContext };
  const withXThenFresh = pairParser.parse(withX.next) as { success: boolean; next: PairContext };

  // An item whose second option is required is left half filled, so the next call can only extend it: the state
  // keeps its length and its last item is replaced. (With an optional second option the library opens a new item.)
  const bothParser = multiple(object({ a: option("-a", string()), b: option("-b", string()) }));
  const bothFacts = (context: PairContext): Json => ({
    buffer: JSON.stringify(context.buffer),
    state: arrayFacts(context.state),
    completed: attempt(() => outcome(bothParser.complete(context.state as never) as never)),
  });
  const firstItem = bothParser.parse({
    buffer: ["-a", "0", "-b", "0"],
    state: bothParser.initialState,
    optionsTerminated: false,
    usage: bothParser.usage,
  } as never) as { success: boolean; next: PairContext };
  const halfFilled = bothParser.parse({ ...firstItem.next, buffer: ["-a", "1"] } as never) as { success: boolean; next: PairContext };
  const halfBefore = JSON.stringify(bothFacts(halfFilled.next));
  const filledX = bothParser.parse({ ...halfFilled.next, buffer: ["-b", "x"] } as never) as { success: boolean; next: PairContext };
  const filledY = bothParser.parse({ ...halfFilled.next, buffer: ["-b", "y"] } as never) as { success: boolean; next: PairContext };

  return {
    halfFilled: JSON.parse(halfBefore),
    filledX: bothFacts(filledX.next),
    filledY: bothFacts(filledY.next),
    filledXAfterAll: bothFacts(filledX.next),
    halfFilledUnchanged: halfBefore === JSON.stringify(bothFacts(halfFilled.next)),
    afterA: JSON.parse(before),
    withX: facts(withX.next),
    withY: facts(withY.next),
    withXThenFresh: facts(withXThenFresh.next),
    withXAfterAll: facts(withX.next),
    afterAUnchanged: before === JSON.stringify(facts(afterA.next)),
    whole: outcome(parseSync(pairParser, ["-a", "1", "-b", "2", "-a", "3", "-a", "4", "-b", "5"]) as never),
  };
});

// endregion Branching from one earlier context

// region Async value parser

const base = string();
const asyncString = { ...base, mode: "async" as const, parse: (input: string) => Promise.resolve(base.parse(input)) };

record["async"] = await (async (): Promise<Json> => {
  try {
    const asyncParser = object({
      name: optional(option("--name", asyncString as never)),
      paths: multiple(argument(asyncString as never)),
    });
    const results: Json[] = [];

    for (const input of [["a", "b"], ["--name", "n", "a", "--", "-b"], manyTokens, ["--bogus"]]) {
      results.push(outcome(await parse(asyncParser as never, input) as never));
    }

    return results;
  } catch (error) {
    return { threw: String(error) };
  }
})();

// endregion Async value parser

console.log(JSON.stringify(record, (_key, value) => typeof value === "bigint" ? `${value}n` : value));
