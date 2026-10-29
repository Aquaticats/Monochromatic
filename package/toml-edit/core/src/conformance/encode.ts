/**
 toml-test encoder adapter: tagged JSON on stdin, TOML on stdout.
 
 Satisfies the upstream runner's encoder contract: a representable tagged JSON
 object prints TOML and exits zero; an unrepresentable one exits non-zero. The
 top-level object's entries are applied through {@link tomlSet} over a fresh
 {@link emptyTomlEdit}, so the built package's emission path is what the runner
 reparses and compares.
 
 @module
 */

import {
  emptyTomlEdit,
  tomlSet,
  tomlStringify,
} from '../index.ts';

import {
  failAdapter,
  readStdin,
} from './adapter-runtime.ts';
import { taggedToInput, } from './encode-from-tagged.ts';

/**
 Test whether a parsed JSON value is a table (non-array object).
 
 @param value - Parsed JSON node.
 
 @returns True when `value` is a non-array object.
 
 @example
 ```ts
 isJsonTable({}); // true
 ```
 */
function isJsonTable(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Build TOML text from a tagged top-level table, converting each entry via
 {@link taggedToInput} and serializing the result via {@link tomlStringify}.
 
 @param root - Tagged top-level object.
 
 @returns Serialized TOML for the rebuilt document.
 
 @throws {@link TomlTypeError} when an entry is not representable as TOML, which the
         caller turns into a non-zero exit.
 
 @mutates root - `Object.entries` and recursive tagged conversion can invoke caller-owned proxy and accessor hooks.
 
 @example
 ```ts
 buildToml({ root: { a: { type: 'bool', value: 'true' } }, }); // 'a = true\n'
 ```
 */
function buildToml({ root, }: { readonly root: Record<string, unknown>; },): string {
  return tomlStringify({
    edit: Object.entries(root,)
      .reduce(
        /**
         Applies one tagged entry to immutable edit state.
         
         @param current - Edit state returned by previous reduction step.
         
         @param entry - Tagged key and child pair.
         
         @returns Next immutable edit state.
         
         @mutates entry - Tagged child conversion may invoke proxy and accessor hooks recursively.
         */
      function applyEntry(
        current,
        entry,
      ) {
          /**
           Tagged key and child selected by current entry.
           */
        const [key, child,] = entry;
        return tomlSet({
          edit: current,
          path: [key,],
          value: taggedToInput({ tree: child, },),
        },);
      },
      emptyTomlEdit(),
      ),
  },);
}

/**
 Parse stdin JSON and build TOML, capturing both rejection modes.
 
 @param text - Decoded stdin text.
 
 @returns TOML on success, or a failure carrying the diagnostic.
 
 @example
 ```ts
 buildSafely({ text: '{"a":{"type":"bool","value":"true"}}', });
 ```
 */
function buildSafely(
  { text, }: { readonly text: string; },
): {
  readonly ok: true;
  readonly toml: string
} | {
  readonly ok: false;
  readonly message: string
} {
  try {
    /**
     Parsed encoder input, typed `unknown` until the table guard narrows it.
     */
    const parsed: unknown = JSON.parse(text,);
    if (!isJsonTable(parsed,))
      return {
        ok: false,
        message: 'encoder input must be a JSON object',
      };
    return {
      ok: true,
      toml: buildToml({ root: parsed, },),
    };
  }
  catch (caught: unknown) {
    return {
      ok: false,
      message: String(caught,),
    };
  }
}

/**
 Run the encoder adapter end to end.
 
 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   Lenient UTF-8 view of stdin; malformed JSON is caught by the parse below.
   */
  const text = new TextDecoder().decode(await readStdin(),);
  /**
   Parse-and-build result.
   */
  const result = buildSafely({ text, },);
  if (!result.ok) {
    failAdapter({ message: result.message, },);
    return;
  }
  process.stdout
    .write(result.toml,);
}

await main();
