/**
 * Every import form the rule allows in a package that builds normally.
 *
 * Expected diagnostics: none.
 *
 * @module
 */

// Relative import landing inside the default artifact root.
import { parse, } from '../dist/final/node/index.mjs';
// Type-only import of the same artifact; the rule checks these too, and allows them here.
import type { Parsed, } from '../dist/final/node/index.d.mts';
// The package's own bare name, which resolves through the exports map.
import { parse as parseThroughExports, } from '@monochromatic-dev/test-fixture-case-standard';
// Another workspace package's source subpath, the sanctioned cross-package channel.
import { expect, } from '@monochromatic-dev/test-harness-core/ts';
// Test-only data matching the fixture allowlist.
import { SAMPLE, } from './fixture.data.ts';

expect(parse(SAMPLE,),).toBe(SAMPLE,);
expect(parseThroughExports(SAMPLE,),).toBe(SAMPLE,);

/**
 * Keeps the type-only import referenced.
 */
export type Echoed = Parsed;
