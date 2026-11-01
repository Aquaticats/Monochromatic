import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { replacePackageVersion } from './manifest-edit.ts';
import { readPackageVersion } from './package-version.ts';

/**
 Crate manifest with look-alike version lines before and after the one that the release edit changes.
 */
const crateManifest = [
  '[package]',
  'name = "monochromatic-git-policy-cli"',
  'rust-version = "0.4.2"',
  'version = "0.4.2" # published',
  'edition = "2024"',
  '',
  '[package.metadata.binstall]',
  'version = "0.4.2"',
  '',
  '[dependencies]',
  'serde = { version = "0.4.2" }',
  '',
].join('\n');

/**
 Version that the manifests of the read tests hold in their package table, and a different one in look-alike places.
 */
const heldVersion = '3.2.1';
const otherVersion = `${heldVersion}-other`;

await describe({
  name: '',
  children: [
    describe({
      name: readPackageVersion.name,
      children: [
        it({
          name: 'reads the version line of the package table, not a look-alike before or after it',
          fn: async () => {
            const text = [
              '[package]',
              `rust-version = "${otherVersion}"`,
              `version = "${heldVersion}" # published`,
              '[package.metadata.binstall]',
              `version = "${otherVersion}"`,
              '[dependencies]',
              `serde = { version = "${otherVersion}" }`,
            ].join('\n');
            expect(readPackageVersion(text)).toEqual({ found: true, value: heldVersion });
          },
        }),
        it({
          name: 'reads a manifest with carriage returns',
          fn: async () => {
            expect(readPackageVersion(`[package]\r\nversion = "${heldVersion}"\r\n`)).toEqual({
              found: true,
              value: heldVersion,
            });
          },
        }),
        ...[
          '',
          '[package]\nname = "x"\n',
          `[package]\nversion.workspace = true\n[workspace.package]\nversion = "${heldVersion}"\n`,
          `[dependencies]\nserde = { version = "${heldVersion}" }\n`,
          'not toml at all',
        ].map(function missingCase(text) {
          return it({
            name: `reports no version for ${JSON.stringify(text)}`,
            fn: async () => {
              const result = readPackageVersion(text);
              expect(result.found).toBe(false);
              expect(result.found ? '' : result.reason).toContain('[package] table');
            },
          });
        }),
        it({
          name: 'finds the same line that the release edit replaces',
          fn: async () => {
            const result = readPackageVersion(crateManifest);
            expect(result.found).toBe(true);
            expect(replacePackageVersion({ text: crateManifest, from: result.found ? result.value : '', to: heldVersion }).changed)
              .toBe(true);
          },
        }),
      ],
    }),
  ],
});
