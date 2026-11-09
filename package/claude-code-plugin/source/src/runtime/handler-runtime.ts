import { text, } from 'node:stream/consumers';

/**
 Sentinel returned by writers for hook outputs that intentionally emit no stdout.
 A unique symbol avoids treating an empty string as an out-of-band signal.
 */
const NO_STDOUT: unique symbol = Symbol('claude-code-plugin-runtime/hook-intentionally-emits-no-stdout-bytes',);

/**
 Serialized writer result, or {@link NO_STDOUT} when the hook protocol should
 receive no stdout bytes.
 */
type WriterOutput = string | typeof NO_STDOUT;

/**
 Parser converts the raw stdin string into the typed event the handler consumes.
 
 Each parser asserts the trusted JSON shape at its own boundary; inputs come
 from Claude Code's hook dispatch system or from JSON files written by sibling
 hooks in this package, both trusted by contract.
 
 @example
 ```ts
 const parser: Parser<PreToolUseInput> = makeHookInputParser<PreToolUseInput>();
 ```
 */
type Parser<TInput,> = (raw: string,) => TInput;

/**
 Reads one JSON file this package wrote itself, as the shape that file declares.
 */
type OwnedFileParser<TFile,> = (raw: string,) => TFile;

/**
 Serializes a handler response as JSON, which is always a string.

 Narrower than {@link Writer} on purpose: a JSON writer never answers with
 {@link NO_STDOUT}, so callers and tests may treat its result as text.
 */
type JsonWriter<TOutput,> = (output: TOutput,) => string;

/**
 Handler is the pure function at the heart of every hook plugin.
 
 Receives a parsed event and returns the response payload.
 Synchronous or async; the runtime awaits the result.
 */
type HookHandler<TInput, TOutput,> = (input: TInput,) => TOutput | Promise<TOutput>;

/**
 Writer serializes the handler output for stdout.
 
 Returns a string written verbatim, or {@link NO_STDOUT} for intentional
 silence. String outputs receive no trailing newline, matching the wire
 convention Claude Code expects.
 */
type Writer<TOutput,> = (output: TOutput,) => WriterOutput;

/**
 Builds the parser every hook handler uses for its own event payload.
 
 One owner for the trusted-JSON rule and its lint suppression: a hook payload
 arrives from the host that dispatched the hook, on stdin, in the shape its
 `hook_event_name` names, so it is asserted rather than re-validated in eighteen
 places. A handler that ever needs validation belongs in this function, not in a
 copy of it.
 
 @returns parser that asserts the raw payload as `TInput`
 
 @example
 ```ts
 const guardrailParser = makeHookInputParser<PreToolUseInput>();
 guardrailParser('{"hook_event_name":"PreToolUse"}');
 ```
 */
function makeHookInputParser<TInput,>(): Parser<TInput> {
  return function parseHookInput(raw: string,): TInput {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted JSON contract from Claude Code hook system
    return JSON.parse(raw,) as TInput;
  };
}

/**
 Builds the parser used for JSON this package wrote itself: spawn-state records,
 PID mappings, and transcript lines.
 
 Same trusted-JSON rule as {@link makeHookInputParser}, different owner: these
 files are produced by our own hook and CLI code, so a parse failure means a
 truncated write, which callers handle by skipping the record.
 
 @returns parser that asserts the raw file text as `TFile`
 
 @example
 ```ts
 const parseSpawnState = makeOwnedFileParser<SpawnState>();
 parseSpawnState('{"spawnId":"abc"}');
 ```
 */
function makeOwnedFileParser<TFile,>(): OwnedFileParser<TFile> {
  return function parseOwnedJson(raw: string,): TFile {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted JSON written by this package's own hooks and CLI
    return JSON.parse(raw,) as TFile;
  };
}

/**
 Builds the writer that serializes a handler response for stdout.
 
 One owner for the wire convention: no trailing newline, because the host parses
 the payload strictly. The same suppression-free `JSON.stringify` call serves
 every handler that answers in JSON.
 
 @returns writer that serializes `TOutput` as JSON
 
 @example
 ```ts
 const guardrailWriter = makeJsonWriter<GuardrailOutput>();
 process.stdout.write(guardrailWriter({}));
 ```
 */
function makeJsonWriter<TOutput,>(): JsonWriter<TOutput> {
  return function writeJson(output: TOutput,): string {
    return JSON.stringify(output,);
  };
}

/**
 Runtime shell for a hook plugin entry script.
 
 Reads stdin to EOF, runs the parser, dispatches to the handler,
 and writes the writer's serialized result to stdout.
 
 Each generated per-plugin entry script consists of importing the handler,
 parser, and writer for that plugin and calling {@link runHookPlugin} once at the
 top level.
 
 @example
 ```ts
 #!/usr/bin/env node
 import { runHookPlugin } from '../runtime/handler-runtime.ts';
 import { guardrailHandler, guardrailParser, guardrailWriter } from '../handler/guardrail.ts';
 await runHookPlugin({
   parser: guardrailParser,
   handler: guardrailHandler,
   writer: guardrailWriter,
 });
 ```
 */
async function runHookPlugin<TInput, TOutput,>(
  {
    parser,
    handler,
    writer,
  }: {
    readonly parser: Parser<TInput>;
    readonly handler: HookHandler<TInput, TOutput>;
    readonly writer: Writer<TOutput>;
  },
): Promise<void> {
  /**
   Console-sink routing for the hook process. The writer's bytes are the host
   protocol on stdout (Codex parses them strictly, so a trailing log line
   fails every event as `invalid ... JSON output`), so diagnostics go to
   stderr. An explicit `MONOCHROMATIC_LOG_CONSOLE` value still wins.
   */
  process.env
    .MONOCHROMATIC_LOG_CONSOLE ??= 'stderr';
  /**
   Full stdin payload from Claude Code, awaited to EOF before parsing.
   */
  const raw = await text(process.stdin,);
  /**
   Parsed hook event passed to the plugin handler below.
   */
  const event = parser(raw,);
  /**
   Handler response; serialized by the writer for stdout.
   */
  const output = await handler(event,);
  /**
   Serialized payload from the writer; the sentinel means the hook intentionally
   emits no stdout bytes.
   */
  const serialized = writer(output,);
  if (serialized !== NO_STDOUT) {
    process.stdout
      .write(serialized,);
  }
}

export type {
  HookHandler,
  JsonWriter,
  OwnedFileParser,
  Parser,
  Writer,
  WriterOutput,
};

export {
  makeHookInputParser,
  makeJsonWriter,
  makeOwnedFileParser,
  NO_STDOUT,
  runHookPlugin,
};
