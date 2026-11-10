// The colour-sequence stripping of the idle-cost and bundle inspections, against the log text the IDE prints.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { withoutSgrSequences } from './sgr-sequences.ts';

const escape = String.fromCharCode(0x1b);

test('drops a colour sequence and keeps the text around it', () => {
  assert.equal(withoutSgrSequences(escape + '[31mred' + escape + '[0m text'), 'red text');
});

test('drops a sequence with several parameters', () => {
  assert.equal(withoutSgrSequences(escape + '[1;38;5;208mbright' + escape + '[m'), 'bright');
});

test('drops adjacent sequences', () => {
  assert.equal(withoutSgrSequences(escape + '[0m' + escape + '[1mkept'), 'kept');
});

test('keeps an escape that is not the start of a colour sequence', () => {
  assert.equal(withoutSgrSequences('a' + escape + 'b'), 'a' + escape + 'b');
  assert.equal(withoutSgrSequences('a' + escape + '[?25hb'), 'a' + escape + '[?25hb');
});

test('keeps a sequence whose parameter holds a character no parameter may hold', () => {
  assert.equal(withoutSgrSequences('a' + escape + '[1!2m b'), 'a' + escape + '[1!2m b');
});

test('keeps a sequence with no final m, and still drops one that follows it', () => {
  assert.equal(withoutSgrSequences(escape + '[0;3unfinished'), escape + '[0;3unfinished');
  assert.equal(withoutSgrSequences(escape + '[0;3unfinished' + escape + '[32mkept'), escape + '[0;3unfinishedkept');
});

test('keeps a sequence the text ends inside', () => {
  assert.equal(withoutSgrSequences('log ' + escape + '[3'), 'log ' + escape + '[3');
  assert.equal(withoutSgrSequences('log ' + escape), 'log ' + escape);
});

test('returns text without escapes unchanged, including an empty one', () => {
  assert.equal(withoutSgrSequences(''), '');
  assert.equal(withoutSgrSequences('watching directory /a/b'), 'watching directory /a/b');
});
