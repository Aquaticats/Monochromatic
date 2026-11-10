// The safety-sweep declaration of a policy source, against the shipped source's own text and the shapes a
// replacement has to leave alone.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { safetySweepDeclaration } from './refresh-policy-sweep.ts';

// The line src/refresh_policy.rs carries at the time of writing, with the lines around it.
const shipped = [
  '/// every directory,',
  '///  so each directory is reread once per pass instead of once per interval.',
  'pub const SAFETY_SWEEP: Duration = Duration::from_secs(1);',
  '',
  '/// A notified item is not reread sooner than this after its previous read started.',
].join('\n');

test('reads the declaration and its seconds from the shipped source', () => {
  const declaration = safetySweepDeclaration(shipped);
  assert.deepEqual(declaration, { text: 'pub const SAFETY_SWEEP: Duration = Duration::from_secs(1);', seconds: 1 });
});

test('reads a longer interval as one whole number', () => {
  assert.equal(safetySweepDeclaration('pub const SAFETY_SWEEP: Duration = Duration::from_secs(120);')?.seconds, 120);
});

test('reports the declaration text a replacement rewrites, and only that text', () => {
  const declaration = safetySweepDeclaration(shipped);
  assert.equal(declaration?.text, 'pub const SAFETY_SWEEP: Duration = Duration::from_secs(1);');
  const rewritten = shipped.replace(declaration?.text ?? '', 'REWRITTEN');
  assert.equal(rewritten.includes('pub const SAFETY_SWEEP'), false, rewritten);
  assert.equal(rewritten.includes('/// every directory,'), true, rewritten);
});

test('reads nothing from a source that declares no sweep interval', () => {
  assert.equal(safetySweepDeclaration(''), undefined);
  assert.equal(safetySweepDeclaration('pub const SWEEP: Duration = Duration::from_secs(1);'), undefined);
});

test('reads nothing from a declaration without seconds', () => {
  assert.equal(safetySweepDeclaration('pub const SAFETY_SWEEP: Duration = Duration::from_secs();'), undefined);
});

test('reads the next complete declaration when an earlier one stops before its closing text', () => {
  const source = [
    'pub const SAFETY_SWEEP: Duration = Duration::from_secs(10x);',
    'pub const SAFETY_SWEEP: Duration = Duration::from_secs(2);',
  ].join('\n');
  assert.deepEqual(safetySweepDeclaration(source), { text: 'pub const SAFETY_SWEEP: Duration = Duration::from_secs(2);', seconds: 2 });
});

test('keeps a comment that spells the declaration out of the answer', () => {
  const source = '/// const SAFETY_SWEEP_MS = 1_000;\npub const SAFETY_SWEEP: Duration = Duration::from_secs(3);';
  assert.equal(safetySweepDeclaration(source)?.seconds, 3);
});
