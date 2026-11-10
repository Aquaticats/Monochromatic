// Checks of the registry source path reader behind the `crate-licenses` bundle check.
// What: each expected value is what the regular expression the scan replaced returned for the same text. The
// comparison that made the scan's equivalence exhaustive ran beside this file and is quoted in the commit that
// introduced it, because a pattern kept as the oracle would be a site the next regex audit files again.
// Why: the check fails when a crate whose source the executable names has no license text embedded, so a path the scan
// misses reports a missing text that is there, and a path it invents reports one that is not.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { registryCratesNamedIn } from './registry-crate-paths.ts';

/// The path segment every registry crate source sits below.
const registry = '/registry/src/index.crates.io-';

/// The hash folder name cargo unpacks one registry into.
const hash = '6f17d22bba15001f';

test(
  'reads a crate and its version from a registry source path',
  () => {
    assert.deepEqual(
      registryCratesNamedIn(registry + hash + '/serde-1.0.219/src/lib.rs'),
      [
        {
          name: 'serde',
          version: '1.0.219'
        }
      ]
    );
  }
);

test(
  'reads every path a binary names, in order and duplicates included',
  () => {
    assert.deepEqual(
      registryCratesNamedIn([
        registry + hash + '/serde-1.0.219/src/a.rs',
        registry + hash + '/serde-1.0.219/src/b.rs',
        registry + hash + '/syn-2.0.87/src/lib.rs'
      ].join('\u0000')),
      [
        {
          name: 'serde',
          version: '1.0.219'
        },
        {
          name: 'serde',
          version: '1.0.219'
        },
        {
          name: 'syn',
          version: '2.0.87'
        }
      ]
    );
  }
);

test(
  'reads a pre-release and build metadata as part of the version',
  () => {
    assert.deepEqual(
      registryCratesNamedIn(registry + hash + '/leaf-1.0.0-beta.1+build/src/lib.rs'),
      [
        {
          name: 'leaf',
          version: '1.0.0-beta.1+build'
        }
      ]
    );
  }
);

test(
  'takes the shortest name whose release reads',
  () => {
    assert.deepEqual(
      registryCratesNamedIn(registry + hash + '/a-1-2.3.4/src/lib.rs'),
      [
        {
          name: 'a-1',
          version: '2.3.4'
        }
      ]
    );
  }
);

test(
  'keeps a release as written, leading zeros and all',
  () => {
    assert.deepEqual(
      registryCratesNamedIn(registry + hash + '/serde-01.02.3/src/lib.rs'),
      [
        {
          name: 'serde',
          version: '01.02.3'
        }
      ]
    );
  }
);

test(
  'keeps an underscore inside a crate name and out of a release suffix',
  () => {
    assert.deepEqual(
      registryCratesNamedIn(registry + hash + '/serde_derive-1.0.219/src/lib.rs'),
      [
        {
          name: 'serde_derive',
          version: '1.0.219'
        }
      ]
    );
    assert.deepEqual(
      registryCratesNamedIn(registry + hash + '/serde-1.0.219-_/src/lib.rs'),
      []
    );
  }
);

test(
  'reads no crate from a path a release does not close',
  () => {
    assert.deepEqual(
      registryCratesNamedIn(registry + hash + '/serde-1.2.3'),
      []
    );
    assert.deepEqual(
      registryCratesNamedIn(registry + hash + '/serde-1.2/src/lib.rs'),
      []
    );
    assert.deepEqual(
      registryCratesNamedIn(registry + hash + '/serde-1.0.219.4/src/lib.rs'),
      []
    );
  }
);

test(
  'reads no crate from a folder that is not a lowercase hexadecimal hash',
  () => {
    assert.deepEqual(
      registryCratesNamedIn(registry + '6F17D22BBA15001F/serde-1.0.219/src/lib.rs'),
      []
    );
    assert.deepEqual(
      registryCratesNamedIn(registry + '/serde-1.0.219/src/lib.rs'),
      []
    );
  }
);

test(
  'resumes after a path the scan rejects',
  () => {
    assert.deepEqual(
      registryCratesNamedIn(registry + 'zz/' + registry + hash + '/toml-0.8.19/src/lib.rs'),
      [
        {
          name: 'toml',
          version: '0.8.19'
        }
      ]
    );
  }
);

test(
  'reads no crate from text without a registry path',
  () => {
    assert.deepEqual(registryCratesNamedIn('no registry path here'), []);
    assert.deepEqual(registryCratesNamedIn(''), []);
  }
);
