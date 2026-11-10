// Checks of the crate list reader behind the `crate-licenses` bundle check.
// What: each expected value is what the regular expression each scan replaced returned for the same text; the
// comparisons that made every scan's equivalence exhaustive ran beside this file and are quoted in the commit that
// introduced it, because a pattern kept as the oracle would be a site the next regex audit files again.
// Why: the check compares a printed listing against the crate list the build embedded, so a misread item reports
// either a missing license text or a crate that has one.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { withoutUsedByPrefix } from './crate-license-listing.ts';

test(
  'removes the label a crate list starts with',
  () => {
    assert.equal(
      withoutUsedByPrefix('Used by: serde 1.0.219'),
      'serde 1.0.219'
    );
  }
);

test(
  'leaves a crate list that carries no label alone',
  () => {
    assert.equal(
      withoutUsedByPrefix('serde 1.0.219'),
      'serde 1.0.219'
    );
  }
);

test(
  'removes the label only at the start of the list',
  () => {
    assert.equal(
      withoutUsedByPrefix('         syn 2.0.87\nUsed by: serde 1.0.219'),
      '         syn 2.0.87\nUsed by: serde 1.0.219'
    );
  }
);

test(
  'removes one label when the text repeats it',
  () => {
    assert.equal(
      withoutUsedByPrefix('Used by: Used by: serde 1.0.219'),
      'Used by: serde 1.0.219'
    );
  }
);

test(
  'keeps a shorter prefix and an empty list as they are',
  () => {
    assert.equal(
      withoutUsedByPrefix('Used by:'),
      'Used by:'
    );
    assert.equal(
      withoutUsedByPrefix(''),
      ''
    );
  }
);
