// The grammar library names a runtime manifest lists, against the names the build script writes there.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { isGrammarLibraryName } from './grammar-library-name.ts';

// The grammar libraries the runtime manifest of an IDE build names.
const shipped = ['rust.so', 'typescript.so', 'c-sharp.so', 'markdown_inline.so', 'embedded-template.so', 'yaml.so'];

test('accepts the library names a shipped manifest lists', () => {
  for (const name of shipped) assert.equal(isGrammarLibraryName(name), true, name);
});

test('accepts a name of letters, digits, underscores and hyphens before the suffix', () => {
  for (const name of ['a.so', 'A.so', '0.so', '_.so', '-.so', '___.so', 'x-y_z-9.so']) assert.equal(isGrammarLibraryName(name), true, name);
});

test('rejects a name that does not end in the shared-object suffix', () => {
  for (const name of ['rust.SO', 'rust.so.dll', 'rust', 'rust.so ', ' rust.so']) assert.equal(isGrammarLibraryName(name), false, JSON.stringify(name));
});

test('rejects a name whose stem is empty or holds a character the manifest never lists', () => {
  for (const name of ['.so', '', 'so', 'rust.so.so', 'rust x.so', 'rust/x.so', 'r\u00e9.so', 'rust.so\n']) assert.equal(isGrammarLibraryName(name), false, JSON.stringify(name));
});
