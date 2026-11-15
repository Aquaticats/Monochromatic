// Prints what @optique/core does on inputs long enough to pass the two sizes at which the prototype starts to hand
// out views instead of plain arrays (1,024 items of multiple() state, 16,384 tokens left in the buffer), as one JSON
// document, so that two builds can be compared by comparing their output. It knows nothing about views: it reads
// states and buffers the way any caller of the public interface may.
//
// Long values are recorded as a length, their ends and a hash of their JSON text, not in full.
//
// Run: deno run --no-config --import-map <map for one build> long-input-probe.ts

import { object } from "@optique/core/constructs";
import { formatMessage } from "@optique/core/message";
import { multiple, optional } from "@optique/core/modifiers";
import { parse, parseSync } from "@optique/core/parser";
import { argument, option } from "@optique/core/primitives";
import { integer, string } from "@optique/core/valueparser";

type Json = unknown;

// The sizes below are chosen against the prototype's two thresholds, which this file must not import.
const STATE_THRESHOLD = 1024;
const BUFFER_THRESHOLD = 16384;

const record: Record<string, Json> = {};

function hashOf(text: string): string {
  // FNV-1a over UTF-16 code units, 32 bits: enough to tell two long values apart.
  let hash = 0x811c9dc5;

  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }

  return hash.toString(16);
}

function digest(value: unknown): Json {
  const text = JSON.stringify(value);

  if (text === undefined) return { undefinedJson: String(value) };

  return { jsonLength: text.length, hash: hashOf(text), start: text.slice(0, 60), end: text.slice(-60) };
}

function outcome(result: { success: boolean; value?: unknown; error?: unknown }): Json {
  if (result.success) return { success: true, value: digest(result.value) };

  return { success: false, error: formatMessage(result.error as Parameters<typeof formatMessage>[0]) };
}

function attempt(run: () => Json): Json {
  try {
    return run();
  } catch (error) {
    return { threw: String(error) };
  }
}

function arrayFacts(value: unknown): Json {
  if (value == null || typeof value !== "object") return { notAnObject: String(value) };

  const array = value as readonly unknown[];

  return {
    isArray: Array.isArray(array),
    prototypeIsArray: Object.getPrototypeOf(array) === Array.prototype,
    length: array.length,
    ownIndexKeys: Object.keys(array).length,
    first: digest(array[0]),
    last: digest(array.at(-1)),
    pastEnd: array[array.length] === undefined,
    spread: digest([...array]),
    sliceFromOne: digest(array.slice(1)),
    sliceLastThree: digest(array.slice(-3)),
    mapLength: array.map((item) => item).length,
    concat: digest(([] as unknown[]).concat(array)),
    entriesLength: [...array.entries()].length,
    json: digest(array),
    indexOfLast: array.indexOf(array.at(-1)),
    includesFirst: array.includes(array[0]),
    everyDefined: array.every((item) => item !== undefined),
    hasZero: 0 in array,
    hasLength: array.length in array,
    forEachCount: (() => {
      let count = 0;
      array.forEach(() => {
        count += 1;
      });
      return count;
    })(),
  };
}

function tokens(count: number, prefix: string): string[] {
  return Array.from({ length: count }, (_, index) => `${prefix}${index.toString(36)}`);
}

// region Whole parses of long inputs

const positionalCounts = [STATE_THRESHOLD - 1, STATE_THRESHOLD, STATE_THRESHOLD + 1, STATE_THRESHOLD * 2 + 7, BUFFER_THRESHOLD + 5, BUFFER_THRESHOLD + STATE_THRESHOLD + 3];
const wholeParsers = {
  nameAndPaths: object({ name: optional(option("--name", string())), paths: multiple(argument(string())) }),
  valuesAndRest: object({ values: multiple(option("-v", "--value", string())), rest: multiple(argument(string())) }),
  bareMultiple: multiple(argument(string())),
  boundedMultiple: multiple(argument(string()), { min: 1, max: BUFFER_THRESHOLD }),
};

function inputsFor(count: number): Record<string, readonly string[]> {
  const positional = tokens(count, "p");
  const triples = Math.floor(count / 3);
  const interleaved = Array.from({ length: triples }, (_, index) => ["-v", `v${index}`, `p${index}`]).flat();

  return {
    positional,
    nameFirst: ["--name", "n", ...positional],
    nameLast: [...positional, "--name", "n"],
    nameInTheMiddle: [...positional.slice(0, count >> 1), "--name", "n", ...positional.slice(count >> 1)],
    afterSeparator: ["--", ...positional],
    separatorInTheMiddle: [...positional.slice(0, count >> 1), "--", "-x", ...positional.slice(count >> 1)],
    interleaved,
    interleavedThenPositional: [...interleaved.slice(0, 300), ...positional],
    unknownOptionLate: [...positional, "--bogus"],
  };
}

record["whole"] = Object.fromEntries(
  positionalCounts.map((count) => [
    String(count),
    Object.fromEntries(
      Object.entries(inputsFor(count)).map(([inputName, input]) => [
        inputName,
        Object.fromEntries(
          Object.entries(wholeParsers).map(([parserName, parser]) => [
            parserName,
            attempt(() => outcome(parseSync(parser as Parameters<typeof parseSync>[0], input))),
          ]),
        ),
      ]),
    ),
  ]),
);

record["integers"] = attempt(() =>
  outcome(parseSync(multiple(argument(integer())), Array.from({ length: STATE_THRESHOLD + 9 }, (_, index) => String(index))))
);

// endregion Whole parses of long inputs

// region Step by step on a long buffer and a long state

const stepParser = multiple(argument(string()));
type StepContext = Parameters<typeof stepParser.parse>[0];

function contextFacts(context: { buffer: readonly string[]; state: unknown }): Json {
  return {
    buffer: arrayFacts(context.buffer),
    bufferJoinedHash: hashOf(context.buffer.join(" ")),
    state: arrayFacts(context.state),
    completed: attempt(() => outcome(stepParser.complete(context.state as Parameters<typeof stepParser.complete>[0]) as never)),
  };
}

function stepsFrom(start: StepContext, recordAt: ReadonlySet<number>, limit: number): { facts: Json[]; contexts: StepContext[] } {
  const facts: Json[] = [];
  const contexts: StepContext[] = [start];
  let context = start;

  for (let step = 0; step < limit && context.buffer.length > 0; step += 1) {
    const result = stepParser.parse(context) as { success: boolean; next?: StepContext; consumed?: readonly string[] };

    if (!result.success || result.next === undefined) {
      facts.push({ step, failed: outcome(result as never) });
      break;
    }

    if (recordAt.has(step)) {
      facts.push({
        step,
        consumed: arrayFacts(result.consumed),
        consumedJson: JSON.stringify(result.consumed),
        next: contextFacts(result.next),
        inputAfter: contextFacts(context),
      });
    }

    context = result.next;
    contexts.push(context);
  }

  return { facts, contexts };
}

const longTokens = tokens(BUFFER_THRESHOLD + STATE_THRESHOLD + 40, "t");
const longStart = { buffer: longTokens, state: stepParser.initialState, optionsTerminated: false, usage: stepParser.usage } as StepContext;
// Steps on both sides of each size: the first steps (buffer far above its size), the steps where the state passes
// its size, and the steps where the buffer drops below its size.
const bufferCrossing = longTokens.length - BUFFER_THRESHOLD;
const recordAt = new Set([0, 1, 2, STATE_THRESHOLD - 2, STATE_THRESHOLD - 1, STATE_THRESHOLD, STATE_THRESHOLD + 1, bufferCrossing - 2, bufferCrossing - 1, bufferCrossing, bufferCrossing + 1]);
const long = stepsFrom(longStart, recordAt, bufferCrossing + 3);

record["steps"] = long.facts;
record["stepsTaken"] = long.contexts.length - 1;

// Every earlier context still reads as it did when it was made: lengths in full, contents at a sample.
record["earlierContexts"] = {
  lengthsHash: hashOf(long.contexts.map((context) => `${context.buffer.length}:${(context.state as readonly unknown[]).length}`).join(",")),
  sampled: [0, 1, STATE_THRESHOLD, STATE_THRESHOLD + 1, bufferCrossing - 1, bufferCrossing, long.contexts.length - 1].map((index) => {
    const context = long.contexts[index];

    return context === undefined ? { index, missing: true } : { index, facts: contextFacts(context) };
  }),
};

// endregion Step by step on a long buffer and a long state

// region Two extensions of one long earlier context

function branchFrom(earlier: StepContext): Json {
  const before = JSON.stringify(contextFacts(earlier));
  const first = stepParser.parse({ ...earlier, buffer: ["x", ...earlier.buffer.slice(1)] }) as { success: boolean; next: StepContext };
  const second = stepParser.parse({ ...earlier, buffer: ["y", "q"] }) as { success: boolean; next: StepContext };
  const sameBufferTwice = [stepParser.parse(earlier), stepParser.parse(earlier)] as { success: boolean; next: StepContext }[];
  const firstAgain = stepParser.parse(first.next) as { success: boolean; next: StepContext };

  return {
    first: contextFacts(first.next),
    second: contextFacts(second.next),
    sameBufferTwice: sameBufferTwice.map((result) => contextFacts(result.next)),
    firstExtended: contextFacts(firstAgain.next),
    firstAfterAll: contextFacts(first.next),
    earlierUnchanged: before === JSON.stringify(contextFacts(earlier)),
  };
}

record["branchingAppend"] = [STATE_THRESHOLD - 1, STATE_THRESHOLD, STATE_THRESHOLD + 5, bufferCrossing - 1, bufferCrossing + 1].map((index) => {
  const earlier = long.contexts[index];

  return earlier === undefined ? { index, missing: true } : { index, result: attempt(() => branchFrom(earlier)) };
});

// endregion Two extensions of one long earlier context

// region Replacing the last item of a long state

// An item with two required options is left half filled by its first option, so the next call can only extend it:
// the state keeps its length and its last item is replaced.
record["replaceLast"] = attempt(() => {
  const pairParser = multiple(object({ a: option("-a", string()), b: option("-b", string()) }));
  type PairContext = Parameters<typeof pairParser.parse>[0];
  const facts = (context: PairContext): Json => ({
    buffer: arrayFacts(context.buffer),
    state: arrayFacts(context.state),
    completed: attempt(() => outcome(pairParser.complete(context.state) as never)),
  });
  const pairs = STATE_THRESHOLD + 6;
  let context = {
    buffer: Array.from({ length: pairs }, (_, index) => ["-a", `a${index}`, "-b", `b${index}`]).flat(),
    state: pairParser.initialState,
    optionsTerminated: false,
    usage: pairParser.usage,
  } as PairContext;
  const lengths: number[] = [];
  let calls = 0;

  while (context.buffer.length > 0) {
    const result = pairParser.parse(context) as { success: boolean; next?: PairContext };

    if (!result.success || result.next === undefined) return { failedAtCall: calls, outcome: outcome(result as never) };

    context = result.next;
    lengths.push((context.state as readonly unknown[]).length);
    calls += 1;
  }

  const full = context;
  const halfFilled = pairParser.parse({ ...full, buffer: ["-a", "half"] }) as { success: boolean; next: PairContext };
  const halfBefore = JSON.stringify(facts(halfFilled.next));
  const filledX = pairParser.parse({ ...halfFilled.next, buffer: ["-b", "x"] }) as { success: boolean; next: PairContext };
  const filledY = pairParser.parse({ ...halfFilled.next, buffer: ["-b", "y"] }) as { success: boolean; next: PairContext };
  const afterX = pairParser.parse({ ...filledX.next, buffer: ["-a", "next"] }) as { success: boolean; next: PairContext };

  return {
    calls,
    stateLengthsHash: hashOf(lengths.join(",")),
    lastLengths: lengths.slice(-6),
    full: facts(full),
    halfFilled: JSON.parse(halfBefore),
    filledX: facts(filledX.next),
    filledY: facts(filledY.next),
    afterX: facts(afterX.next),
    filledXAfterAll: facts(filledX.next),
    halfFilledUnchanged: halfBefore === JSON.stringify(facts(halfFilled.next)),
    fullAfterAll: facts(full),
  };
});

// endregion Replacing the last item of a long state

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
    const positional = tokens(BUFFER_THRESHOLD + STATE_THRESHOLD + 3, "p");

    for (const input of [positional, ["--name", "n", ...positional], [...positional, "--name", "n"], [...positional, "--bogus"]]) {
      results.push(outcome(await parse(asyncParser as never, input) as never));
    }

    return results;
  } catch (error) {
    return { threw: String(error) };
  }
})();

// endregion Async value parser

console.log(JSON.stringify(record));
