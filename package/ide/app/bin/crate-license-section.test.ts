// Checks of the crate license section read the crate-licenses bundle check compares the printed listing by.
// What: which part of a printed listing is a section, the name, identifier, crate count and crate list each section
// states, the whole text of the section, and the sections the real listing of the application's own `--licenses`
// output holds.
// Why: the check demands that every crate the executable links has its license text printed in full, so a read that
// missed a section would report a crate as uncovered, and one that invented a section would compare a text that is not
// there.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { crateLicenseSections } from './crate-license-section.ts';

// A section as the application prints one: the rule above it, its heading, its crate list, the line naming where the
// text comes from, and the rule and blank line that close it.
const printedSection = ({ heading, list, origin }: { heading: string; list: string[]; origin: string }): string =>
  '\n' + '='.repeat(78) + '\n' + heading + '\n' + list.join('\n') + '\n' + origin + '\n' + '='.repeat(78) + '\n\n';

test('reads the section of a text one crate uses', () => {
  const listing = printedSection({
    heading: 'Rust crates under MIT License (MIT): 1 crate',
    list: ['Used by: anyhow 1.0.104'],
    origin: 'Text from the crate file anyhow-1.0.104/LICENSE-MIT',
  }) + 'Copyright (c) The anyhow authors\n';
  const sections = crateLicenseSections(listing);
  assert.equal(sections.length, 1);
  assert.deepEqual(sections[0], {
    whole: listing.slice(listing.indexOf('Rust crates under'), listing.indexOf('Copyright')),
    name: 'MIT License',
    id: 'MIT',
    crateCount: 1,
    usedBy: 'Used by: anyhow 1.0.104\n',
  });
});

test('reads every section of a listing that holds several, in the order printed', () => {
  const two = printedSection({
    heading: 'Rust crates under MIT License (MIT): 2 crates',
    list: ['Used by: anyhow 1.0.104,', '         bitflags 2.9.4'],
    origin: 'Text from the crate file anyhow-1.0.104/LICENSE-MIT',
  }) + 'Permission is hereby granted.\n' + printedSection({
    heading: 'Rust crates under Mozilla Public License 2.0 (MPL-2.0): 1 crate',
    list: ['Used by: option-ext 0.2.0'],
    origin: 'Text: the standard text of this license (no crate file was recognized)',
  }) + 'Mozilla Public License Version 2.0\n';
  const sections = crateLicenseSections(two);
  assert.deepEqual(sections.map(section => [section.name, section.id, section.crateCount]), [
    ['MIT License', 'MIT', 2],
    ['Mozilla Public License 2.0', 'MPL-2.0', 1],
  ]);
  assert.equal(sections[0]?.usedBy, 'Used by: anyhow 1.0.104,\n         bitflags 2.9.4\n');
  assert.equal(sections[1]?.usedBy, 'Used by: option-ext 0.2.0\n');
});

test('reads a name that holds parentheses by taking the last one before the identifier', () => {
  const listing = printedSection({
    heading: 'Rust crates under Apache License 2.0 (Apache-2.0) (MIT): 1 crate',
    list: ['Used by: x 1.0.0'],
    origin: 'Text from the crate file x-1.0.0/LICENSE',
  });
  const sections = crateLicenseSections(listing);
  assert.equal(sections[0]?.name, 'Apache License 2.0 (Apache-2.0)');
  assert.equal(sections[0]?.id, 'MIT');
});

test('reads no section from text that holds none', () => {
  assert.deepEqual(crateLicenseSections(''), []);
  assert.deepEqual(crateLicenseSections('Mozilla Public License Version 2.0\n'), []);
  assert.deepEqual(crateLicenseSections('Used by: anyhow 1.0.104\n'), []);
});

test('reads no section from a heading whose crate list or origin line is missing', () => {
  const heading = 'Rust crates under MIT License (MIT): 1 crate\n';
  assert.deepEqual(crateLicenseSections('\n' + heading + 'Text from the crate file x/LICENSE\n' + '='.repeat(78) + '\n\n'), []);
  assert.deepEqual(crateLicenseSections('\n' + heading + 'Used by: x 1.0.0\n' + '='.repeat(78) + '\n\n'), []);
});

test('reads no section from one whose rule is not the length the application prints', () => {
  const body = '\nRust crates under MIT License (MIT): 1 crate\nUsed by: x 1.0.0\nText from the crate file x/LICENSE\n';
  assert.deepEqual(crateLicenseSections(body + '='.repeat(77) + '\n\n'), []);
  assert.deepEqual(crateLicenseSections(body + '='.repeat(79) + '\n\n'), []);
  assert.equal(crateLicenseSections(body + '='.repeat(78) + '\n\n').length, 1);
});

test('reads no section from a heading whose count or identifier is missing', () => {
  const tail = '\nUsed by: x 1.0.0\nText from the crate file x/LICENSE\n' + '='.repeat(78) + '\n\n';
  for (const heading of [
    'Rust crates under MIT License (MIT): crates',
    'Rust crates under MIT License (): 1 crate',
    'Rust crates under MIT License MIT: 1 crate',
    'Rust crates under MIT License (MIT): 1 nouns',
    'rust crates under MIT License (MIT): 1 crate',
  ]) {
    assert.deepEqual(crateLicenseSections('\n' + heading + tail), [], heading);
  }
});

test('reads a section wherever its heading stands in the listing', () => {
  const section = printedSection({
    heading: 'Rust crates under MIT License (MIT): 1 crate',
    list: ['Used by: x 1.0.0'],
    origin: 'Text from the crate file x/LICENSE',
  });
  // A heading at the very start of the output, and one whose line begins after a carriage return.
  assert.equal(crateLicenseSections(section.slice(1)).length, 1);
  assert.equal(crateLicenseSections('\u2028' + section.slice(1)).length, 1);
});
