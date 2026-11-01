/**
 toml-test decoder adapter: TOML on stdin, tagged JSON on stdout.
 
 Satisfies the upstream runner's decoder contract: valid TOML prints its
 tagged JSON encoding and exits zero; invalid input exits non-zero. The TOML
 version is selected by the first CLI argument (`1.0` or `1.1`) so one binary
 serves both `-toml` runs. Imports the built package so the shipped artifact
 is exercised across the consumer boundary.
 
 @module
 */

import {
  parseTOML,
  parseTomlEdit,
} from '../index.ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { AST, } from 'toml-eslint-parser';

import {
  failAdapter,
  readStdin,
} from './adapter-runtime.ts';
import { documentToTagged, } from './decode-to-tagged.ts';
import type {
  ConformanceVersion,
  TaggedTree,
} from './tagged-types.ts';

/**
 Resolve the TOML version selector from the first CLI argument.
 
 Returns a spreadable fragment rather than a bare value so an absent argument
 omits `tomlVersion` entirely (the option is exact-optional and rejects an
 explicit `undefined`).
 
 @returns `{ tomlVersion }` when `1.0` or `1.1` is supplied, otherwise `{}`.
 
 @example
 ```ts
 versionOption(); // { tomlVersion: '1.1' } when invoked as `node decode.ts 1.1`
 ```
 */
function versionOption(): { readonly tomlVersion?: ConformanceVersion; } {
  /**
   Raw first positional argument, if any.
   */
  const raw = process.argv
    .at(2,);
  if ((raw === '1.0') || (raw === '1.1'))
    return { tomlVersion: raw, };
  return {};
}

/**
 Strictly decode stdin bytes as UTF-8.
 
 A fatal decoder rejects the malformed-byte and surrogate corpus cases that a
 lossy decode would silently replace, which is correct decoder behavior since
 TOML mandates UTF-8 input.
 
 @param bytes - Buffered stdin bytes.
 
 @returns Decoded source on success, or a failure with a diagnostic.
 
 @example
 ```ts
 decodeUtf8({ bytes, }); // { ok: true, source: 'a = 1\n' }
 ```
 */
function decodeUtf8(
  { bytes, }: { readonly bytes: Buffer; },
): {
  readonly ok: true;
  readonly source: string
} | {
  readonly ok: false;
  readonly message: string
} {
  try {
    return {
      ok: true,
      source: new TextDecoder(
        'utf-8',
        { fatal: true, },
      )
        .decode(bytes,),
    };
  }
  catch (caught: unknown) {
    return {
      ok: false,
      message: `invalid UTF-8: ${String(caught,)}`,
    };
  }
}

/**
 Parse `source` via {@link parseTomlEdit} and project the result to a
 tagged tree via {@link documentToTagged}.
 
 @param source - Decoded TOML source.
 
 @returns Tagged tree on success, or a failure with the parser diagnostic.
 
 @example
 ```ts
 parseToTree({ source: 'a = 1', }); // { ok: true, tree: { a: { type: 'integer', value: '1' } } }
 ```
 */
function parseToTree(
  { source, }: { readonly source: string; },
): {
  readonly ok: true;
  readonly tree: TaggedTree
} | {
  readonly ok: false;
  readonly message: string
} {
  try {
    // Validate through the package (rejects invalid TOML incl. a bare CR, and
    // applies the version rules) before walking the AST for the tagged tree.
    parseTomlEdit({
      source,
      ...versionOption(),
    },);
    /**
     Root parser result crossing from foreign AST ownership into projection.
     */
    const program: ForeignBorrowed<AST.TOMLProgram> = parseTOML(
      source,
      versionOption(),
    );
    return {
      ok: true,
      tree: documentToTagged({ program, },),
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
 Run the decoder adapter end to end.
 
 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   Strict UTF-8 decode of stdin.
   */
  const decoded = decodeUtf8({ bytes: await readStdin(), },);
  if (!decoded.ok) {
    failAdapter({ message: decoded.message, },);
    return;
  }
  /**
   Parse and projection result.
   */
  const result = parseToTree({ source: decoded.source, },);
  if (!result.ok) {
    failAdapter({ message: result.message, },);
    return;
  }
  process.stdout
    .write(JSON.stringify(result.tree,),);
}

await main();
