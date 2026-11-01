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
 const parser: Parser<PreToolUseInput> = function jsonParser(raw): PreToolUseInput {
   // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted JSON contract from Claude Code hook system
   return JSON.parse(raw) as PreToolUseInput;
 };
 ```
 */
type Parser<TInput,> = (raw: string,) => TInput;

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
  Parser,
  Writer,
  WriterOutput,
};

export {
  NO_STDOUT,
  runHookPlugin,
};
