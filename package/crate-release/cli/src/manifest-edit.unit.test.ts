import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CrateReleaseError } from './error.ts';
import {
  replaceDependencyPin,
  replacePackageVersion,
} from './manifest-edit.ts';

/**
 Crate manifest with look-alike version lines before and after the one that must change.
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
 Fuzz manifest whose own version is a placeholder and whose dependency pins the crate exactly.
 */
const fuzzManifest = [
  '[package]',
  'name = "cli-fuzz"',
  'version = "0.0.0"',
  '',
  '[dependencies]',
  'monochromatic-git-policy-cli = { path = "../cli", version = "=0.4.2" }',
  'monochromatic-git-policy-cli-extra = { path = "../extra", version = "=0.4.2" }',
  'monochromatic-jsonc-edit = { path = "../../jsonc-edit/core-rust", version = "=0.4.2" }',
  '',
].join('\n');

/**
 Fuzz manifest that pins its crate with a table of its own, as `package/jsonc-edit/core-rust.fuzz` does.
 */
const jsoncFuzzManifest = [
  '[package]',
  'name = "jsonc-edit-fuzz"',
  'version = "0.0.0"',
  '',
  '[dependencies]',
  'arbitrary = { workspace = true, features = ["derive"] }',
  '',
  '[dependencies.monochromatic-jsonc-edit]',
  'path = "../core-rust"',
  'version = "=0.1.2"',
  '',
  '[lib]',
  'name = "jsonc_edit_fuzz"',
  'path = "src/lib.rs"',
  '',
].join('\n');

/**
 Fuzz manifest whose pin table follows comment lines inside a dependencies block, as
 `package/forbidden-strings/core.fuzz` does, and whose package name starts with the pinned crate's name.
 */
const regexFuzzManifest = [
  '[package]',
  'name = "forbidden-regex-fuzz"',
  'version = "0.0.0"',
  '',
  '[dependencies]',
  'regex = { workspace = true }',
  '',
  '# The engine under test, by path (sibling crate).',
  '[dependencies.forbidden-regex]',
  'path = "../core"',
  'version = "=0.1.1"',
  '',
  '[workspace]',
  'members = ["."]',
  '',
].join('\n');

/**
 Run an edit that must throw and return what it threw.

 @param action - edit under test
 @returns the thrown value
 */
function thrownBy(action: () => unknown): unknown {
  try {
    action();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Replace one line of a manifest text.

 @param text - manifest text
 @param oldLine - line to find
 @param newLine - line to put in its place
 @returns the edited text
 */
function swapLine({
  text,
  oldLine,
  newLine,
}: {
  readonly text: string;
  readonly oldLine: string;
  readonly newLine: string;
}): string {
  return text
    .split('\n')
    .map(function swap(line) {
      return line === oldLine ? newLine : line;
    })
    .join('\n');
}

await describe({
  name: '',
  children: [
    describe({
      name: replacePackageVersion.name,
      children: [
        it({
          name: 'changes only the version line of the package table and keeps its comment',
          fn: async () => {
            const edit = replacePackageVersion({ text: crateManifest, from: '0.4.2', to: '0.4.3' });
            expect(edit.changed).toBe(true);
            expect(edit.replacedLines).toBe(1);
            expect(edit.text).toBe(swapLine({
              text: crateManifest,
              oldLine: 'version = "0.4.2" # published',
              newLine: 'version = "0.4.3" # published',
            }));
          },
        }),
        it({
          name: 'leaves rust-version and version lines of other tables alone',
          fn: async () => {
            const edit = replacePackageVersion({ text: crateManifest, from: '0.4.2', to: '0.4.3' });
            expect(edit.text).toContain('rust-version = "0.4.2"');
            expect(edit.text).toContain('[package.metadata.binstall]\nversion = "0.4.2"');
            expect(edit.text).toContain('serde = { version = "0.4.2" }');
          },
        }),
        it({
          name: 'handles a two-digit patch version without touching its prefix',
          fn: async () => {
            const edit = replacePackageVersion({
              text: swapLine({ text: crateManifest, oldLine: 'version = "0.4.2" # published', newLine: 'version = "0.4.9"' }),
              from: '0.4.9',
              to: '0.4.10',
            });
            expect(edit.text).toContain('\nversion = "0.4.10"\n');
          },
        }),
        it({
          name: 'is a no-op that reports no change when the manifest already holds the target',
          fn: async () => {
            const edit = replacePackageVersion({ text: crateManifest, from: '0.4.1', to: '0.4.2' });
            expect(edit.changed).toBe(false);
            expect(edit.replacedLines).toBe(0);
            expect(edit.text).toBe(crateManifest);
          },
        }),
        it({
          name: 'does not mistake a longer version for the target by prefix',
          fn: async () => {
            const text = swapLine({ text: crateManifest, oldLine: 'version = "0.4.2" # published', newLine: 'version = "0.4.10"' });
            const caught = thrownBy(() => replacePackageVersion({ text, from: '0.4.0', to: '0.4.1' }));
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('"0.4.10"');
          },
        }),
        it({
          name: 'does not mistake a longer version for the current one by prefix',
          fn: async () => {
            const text = swapLine({ text: crateManifest, oldLine: 'version = "0.4.2" # published', newLine: 'version = "0.4.21"' });
            const caught = thrownBy(() => replacePackageVersion({ text, from: '0.4.2', to: '0.4.3' }));
            expect(caught).toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'refuses a version that is neither the current one nor the target and names it',
          fn: async () => {
            const caught = thrownBy(() => replacePackageVersion({ text: crateManifest, from: '0.3.0', to: '0.3.1' }));
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('"0.4.2"');
            expect((caught as Error).message).toContain('"0.3.0"');
          },
        }),
        it({
          name: 'replaces a prerelease version as one whole value',
          fn: async () => {
            const text = swapLine({ text: crateManifest, oldLine: 'version = "0.4.2" # published', newLine: 'version = "1.0.0-rc.1"' });
            const edit = replacePackageVersion({ text, from: '1.0.0-rc.1', to: '1.0.0' });
            expect(edit.text).toContain('\nversion = "1.0.0"\n');
          },
        }),
        it({
          name: 'reads a single-quoted literal string and keeps its quotes',
          fn: async () => {
            const text = swapLine({ text: crateManifest, oldLine: 'version = "0.4.2" # published', newLine: `version = '0.4.2'` });
            expect(replacePackageVersion({ text, from: '0.4.2', to: '0.4.3' }).text).toContain(`\nversion = '0.4.3'\n`);
          },
        }),
        it({
          name: 'keeps carriage returns of a CRLF manifest',
          fn: async () => {
            const text = crateManifest.replaceAll('\n', '\r\n');
            const edit = replacePackageVersion({ text, from: '0.4.2', to: '0.4.3' });
            expect(edit.text).toContain('version = "0.4.3" # published\r\n');
            expect(edit.text.split('\r\n')).toHaveLength(text.split('\r\n').length);
          },
        }),
        it({
          name: 'accepts a package header with a trailing comment',
          fn: async () => {
            const text = crateManifest.replace('[package]', '[package] # the crate');
            expect(replacePackageVersion({ text, from: '0.4.2', to: '0.4.3' }).changed).toBe(true);
          },
        }),
        it({
          name: 'refuses a manifest whose version is inherited from a workspace',
          fn: async () => {
            const text = swapLine({ text: crateManifest, oldLine: 'version = "0.4.2" # published', newLine: 'version.workspace = true' });
            expect(thrownBy(() => replacePackageVersion({ text, from: '0.4.2', to: '0.4.3' }))).toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'refuses a manifest without a package table',
          fn: async () => {
            const text = '[dependencies]\nversion = "0.4.2"\n';
            expect(thrownBy(() => replacePackageVersion({ text, from: '0.4.2', to: '0.4.3' }))).toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'refuses a value followed by text that is not a comment',
          fn: async () => {
            const text = swapLine({ text: crateManifest, oldLine: 'version = "0.4.2" # published', newLine: 'version = "0.4.2" garbage' });
            expect(thrownBy(() => replacePackageVersion({ text, from: '0.4.2', to: '0.4.3' }))).toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'treats a regular-expression character in the version literally',
          fn: async () => {
            const text = swapLine({ text: crateManifest, oldLine: 'version = "0.4.2" # published', newLine: 'version = "0x4x2"' });
            expect(thrownBy(() => replacePackageVersion({ text, from: '0.4.2', to: '0.4.3' }))).toBeInstanceOf(CrateReleaseError);
          },
        }),
      ],
    }),
    describe({
      name: replaceDependencyPin.name,
      children: [
        it({
          name: 'changes only the pin of the named crate',
          fn: async () => {
            const edit = replaceDependencyPin({
              text: fuzzManifest,
              crateName: 'monochromatic-git-policy-cli',
              from: '0.4.2',
              to: '0.4.3',
            });
            expect(edit.changed).toBe(true);
            expect(edit.replacedLines).toBe(1);
            expect(edit.text).toBe(swapLine({
              text: fuzzManifest,
              oldLine: 'monochromatic-git-policy-cli = { path = "../cli", version = "=0.4.2" }',
              newLine: 'monochromatic-git-policy-cli = { path = "../cli", version = "=0.4.3" }',
            }));
          },
        }),
        it({
          name: 'leaves the placeholder package version and the other crates alone',
          fn: async () => {
            const edit = replaceDependencyPin({
              text: fuzzManifest,
              crateName: 'monochromatic-git-policy-cli',
              from: '0.4.2',
              to: '0.4.3',
            });
            expect(edit.text).toContain('version = "0.0.0"');
            expect(edit.text).toContain('monochromatic-git-policy-cli-extra = { path = "../extra", version = "=0.4.2" }');
            expect(edit.text).toContain('monochromatic-jsonc-edit = { path = "../../jsonc-edit/core-rust", version = "=0.4.2" }');
          },
        }),
        it({
          name: 'is a no-op that reports no change when the pin already holds the target',
          fn: async () => {
            const edit = replaceDependencyPin({
              text: fuzzManifest,
              crateName: 'monochromatic-git-policy-cli',
              from: '0.4.1',
              to: '0.4.2',
            });
            expect(edit.changed).toBe(false);
            expect(edit.replacedLines).toBe(0);
            expect(edit.text).toBe(fuzzManifest);
          },
        }),
        it({
          name: 'does not take a pin on a longer version for the target',
          fn: async () => {
            const text = fuzzManifest.replace('"../cli", version = "=0.4.2"', '"../cli", version = "=0.4.10"');
            const caught = thrownBy(() => replaceDependencyPin({
              text,
              crateName: 'monochromatic-git-policy-cli',
              from: '0.4.0',
              to: '0.4.1',
            }));
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('"=0.4.10"');
          },
        }),
        it({
          name: 'refuses a requirement that is not an exact pin',
          fn: async () => {
            const text = fuzzManifest.replace('version = "=0.4.2" }\nmonochromatic-git-policy-cli-extra', 'version = "0.4.2" }\nmonochromatic-git-policy-cli-extra');
            const caught = thrownBy(() => replaceDependencyPin({
              text,
              crateName: 'monochromatic-git-policy-cli',
              from: '0.4.2',
              to: '0.4.3',
            }));
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('"0.4.2"');
          },
        }),
        it({
          name: 'refuses a pin on a different version than the current one',
          fn: async () => {
            const caught = thrownBy(() => replaceDependencyPin({
              text: fuzzManifest,
              crateName: 'monochromatic-git-policy-cli',
              from: '0.3.0',
              to: '0.3.1',
            }));
            expect(caught).toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'refuses a manifest that does not depend on the crate, even when another crate has its prefix',
          fn: async () => {
            const text = fuzzManifest.replace(
              'monochromatic-git-policy-cli = { path = "../cli", version = "=0.4.2" }\n',
              '',
            );
            const caught = thrownBy(() => replaceDependencyPin({
              text,
              crateName: 'monochromatic-git-policy-cli',
              from: '0.4.2',
              to: '0.4.3',
            }));
            expect(caught).toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'finds the version key whatever its position in the inline table',
          fn: async () => {
            const text = 'dep = { version = "=1.0.0", features = ["a"], path = "../d" }\n';
            expect(replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' }).text)
              .toBe('dep = { version = "=1.0.1", features = ["a"], path = "../d" }\n');
          },
        }),
        it({
          name: 'finds a compact inline table without spaces',
          fn: async () => {
            const text = 'dep={path="../d",version="=1.0.0"}\n';
            expect(replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' }).text)
              .toBe('dep={path="../d",version="=1.0.1"}\n');
          },
        }),
        it({
          name: 'does not take a key that merely ends in version for the version key',
          fn: async () => {
            const text = 'dep = { path = "../d", rust-version = "=1.0.0" }\n';
            expect(thrownBy(() => replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' })))
              .toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'replaces the pin in every table that names the crate',
          fn: async () => {
            const text = '[dependencies]\ndep = { version = "=1.0.0" }\n[dev-dependencies]\ndep = { version = "=1.0.0" }\n';
            const edit = replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' });
            expect(edit.replacedLines).toBe(2);
            expect(edit.text).not.toContain('=1.0.0');
          },
        }),
        it({
          name: 'treats a regular-expression character in the crate name literally',
          fn: async () => {
            const text = 'aXb = { version = "=1.0.0" }\n';
            expect(thrownBy(() => replaceDependencyPin({ text, crateName: 'a.b', from: '1.0.0', to: '1.0.1' })))
              .toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'moves a prerelease pin as one whole value',
          fn: async () => {
            const text = 'dep = { version = "=1.0.0-rc.1" }\n';
            expect(replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0-rc.1', to: '1.0.0' }).text)
              .toBe('dep = { version = "=1.0.0" }\n');
          },
        }),
        it({
          name: 'keeps carriage returns of a CRLF manifest',
          fn: async () => {
            const text = fuzzManifest.replaceAll('\n', '\r\n');
            const edit = replaceDependencyPin({
              text,
              crateName: 'monochromatic-git-policy-cli',
              from: '0.4.2',
              to: '0.4.3',
            });
            expect(edit.text.split('\r\n')).toHaveLength(text.split('\r\n').length);
            expect(edit.text).toContain('version = "=0.4.3" }\r\n');
          },
        }),
      ],
    }),
    describe({
      name: `${replaceDependencyPin.name} for a dependency written as its own table`,
      children: [
        it({
          name: 'moves the pin of a [dependencies.<crate>] table and changes no other line (jsonc-edit.fuzz shape)',
          fn: async () => {
            const edit = replaceDependencyPin({
              text: jsoncFuzzManifest,
              crateName: 'monochromatic-jsonc-edit',
              from: '0.1.2',
              to: '0.1.3',
            });
            expect(edit.changed).toBe(true);
            expect(edit.replacedLines).toBe(1);
            expect(edit.text).toBe(swapLine({
              text: jsoncFuzzManifest,
              oldLine: 'version = "=0.1.2"',
              newLine: 'version = "=0.1.3"',
            }));
          },
        }),
        it({
          name: 'moves the pin of a table that sits after comment lines and holds path before version (core.fuzz shape)',
          fn: async () => {
            const edit = replaceDependencyPin({
              text: regexFuzzManifest,
              crateName: 'forbidden-regex',
              from: '0.1.1',
              to: '0.1.2',
            });
            expect(edit.text).toBe(swapLine({
              text: regexFuzzManifest,
              oldLine: 'version = "=0.1.1"',
              newLine: 'version = "=0.1.2"',
            }));
          },
        }),
        it({
          name: 'moves the pin of a table that holds version before path and a comment between its lines (xml-edit.fuzz shape)',
          fn: async () => {
            const text = [
              '[dependencies.monochromatic-xml-edit]',
              '# the crate under test',
              'version = "=0.1.0" # moves with the crate',
              'path = "../core-rust"',
              '',
            ].join('\n');
            expect(replaceDependencyPin({ text, crateName: 'monochromatic-xml-edit', from: '0.1.0', to: '0.1.1' }).text)
              .toBe(text.replace('"=0.1.0"', '"=0.1.1"'));
          },
        }),
        it({
          name: 'accepts a comment after the header and blanks around the key',
          fn: async () => {
            const text = '  [dependencies.dep]   # pinned\n  path = "../d"\n  version   =   "=1.0.0"\n';
            expect(replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' }).text)
              .toBe('  [dependencies.dep]   # pinned\n  path = "../d"\n  version   =   "=1.0.1"\n');
          },
        }),
        it({
          name: 'moves the pin of a dev-dependencies table and of a build-dependencies table',
          fn: async () => {
            const text = '[dev-dependencies.dep]\nversion = "=1.0.0"\n\n[build-dependencies.dep]\nversion = "=1.0.0"\n';
            const edit = replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' });
            expect(edit.replacedLines).toBe(2);
            expect(edit.text).toBe(text.replaceAll('=1.0.0', '=1.0.1'));
          },
        }),
        it({
          name: 'counts the inline pin and the table pin of one manifest as two lines',
          fn: async () => {
            const text = '[dependencies]\ndep = { version = "=1.0.0" }\n\n[build-dependencies.dep]\nversion = "=1.0.0"\n';
            const edit = replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' });
            expect(edit.replacedLines).toBe(2);
            expect(edit.text).not.toContain('=1.0.0');
          },
        }),
        it({
          name: 'is a no-op that reports no change when the table pin already holds the target',
          fn: async () => {
            const edit = replaceDependencyPin({
              text: jsoncFuzzManifest,
              crateName: 'monochromatic-jsonc-edit',
              from: '0.1.1',
              to: '0.1.2',
            });
            expect(edit.changed).toBe(false);
            expect(edit.replacedLines).toBe(0);
            expect(edit.text).toBe(jsoncFuzzManifest);
          },
        }),
        it({
          name: 'leaves the placeholder package version, other tables and a table of another crate with the same pin alone',
          fn: async () => {
            const text = `${jsoncFuzzManifest}\n[dependencies.other]\npath = "../o"\nversion = "=0.1.2"\n`;
            const edit = replaceDependencyPin({ text, crateName: 'monochromatic-jsonc-edit', from: '0.1.2', to: '0.1.3' });
            expect(edit.text).toContain('version = "0.0.0"');
            expect(edit.text).toContain('[dependencies.other]\npath = "../o"\nversion = "=0.1.2"\n');
            expect(edit.replacedLines).toBe(1);
          },
        }),
        it({
          name: 'does not take a table of a crate whose name only starts with this one',
          fn: async () => {
            const text = '[dependencies.dep-extra]\nversion = "=1.0.0"\n\n[dependencies.dep]\nversion = "=1.0.0"\n';
            const edit = replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' });
            expect(edit.text).toBe('[dependencies.dep-extra]\nversion = "=1.0.0"\n\n[dependencies.dep]\nversion = "=1.0.1"\n');
          },
        }),
        it({
          name: 'refuses a manifest whose only table is that of a crate whose name only starts with this one',
          fn: async () => {
            const text = '[dependencies.dep-extra]\nversion = "=1.0.0"\n';
            expect(thrownBy(() => replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' })))
              .toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'does not take the version key of the next table for a table that has none, and refuses it',
          fn: async () => {
            const text = '[dependencies.dep]\npath = "../d"\n\n[dependencies.other]\nversion = "=1.0.0"\n';
            const caught = thrownBy(() => replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' }));
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('[dependencies.dep]');
          },
        }),
        it({
          name: 'refuses a table with a loose requirement and names the value',
          fn: async () => {
            const text = '[dependencies.dep]\npath = "../d"\nversion = "1.0.0"\n';
            const caught = thrownBy(() => replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' }));
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('"1.0.0"');
          },
        }),
        it({
          name: 'does not take a table pin on a longer version for the current or the target one',
          fn: async () => {
            const text = '[dependencies.dep]\nversion = "=0.1.10"\n';
            expect(thrownBy(() => replaceDependencyPin({ text, crateName: 'dep', from: '0.1.1', to: '0.1.2' })))
              .toBeInstanceOf(CrateReleaseError);
            expect(thrownBy(() => replaceDependencyPin({ text, crateName: 'dep', from: '0.1.0', to: '0.1.1' })))
              .toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'refuses a table whose version is inherited or dotted, which has no quoted value to move',
          fn: async () => {
            const text = '[dependencies.dep]\nversion.workspace = true\n';
            expect(thrownBy(() => replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' })))
              .toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'does not take a key that merely ends in version, or one that is a comment, for the version key',
          fn: async () => {
            const text = '[dependencies.dep]\nrust-version = "=1.0.0"\n# version = "=1.0.0"\n';
            expect(thrownBy(() => replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' })))
              .toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'refuses text after the version value that is not a comment',
          fn: async () => {
            const text = '[dependencies.dep]\nversion = "=1.0.0" garbage\n';
            expect(thrownBy(() => replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' })))
              .toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'does not treat a commented header, an array of tables or a workspace table as a dependency table',
          fn: async () => {
            for (const text of [
              '# [dependencies.dep]\nversion = "=1.0.0"\n',
              '[[dependencies.dep]]\nversion = "=1.0.0"\n',
              '[workspace.dependencies.dep]\nversion = "=1.0.0"\n',
              '[dependencies.dep.extra]\nversion = "=1.0.0"\n',
              '[package.metadata.dependencies.dep]\nversion = "=1.0.0"\n',
            ])
              expect(thrownBy(() => replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' })))
                .toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'stops a table at the next header even when a version line follows a nested header',
          fn: async () => {
            const text = '[dependencies.dep]\npath = "../d"\n[dependencies.dep.extra]\nversion = "=1.0.0"\n';
            expect(thrownBy(() => replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' })))
              .toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'treats a regular-expression character in the crate name of a table literally',
          fn: async () => {
            const text = '[dependencies.aXb]\nversion = "=1.0.0"\n';
            expect(thrownBy(() => replaceDependencyPin({ text, crateName: 'a.b', from: '1.0.0', to: '1.0.1' })))
              .toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'reads a single-quoted pin and keeps its quotes',
          fn: async () => {
            const text = '[dependencies.dep]\nversion = \'=1.0.0\'\n';
            expect(replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' }).text)
              .toBe('[dependencies.dep]\nversion = \'=1.0.1\'\n');
          },
        }),
        it({
          name: 'keeps carriage returns of a CRLF manifest',
          fn: async () => {
            const text = jsoncFuzzManifest.replaceAll('\n', '\r\n');
            const edit = replaceDependencyPin({ text, crateName: 'monochromatic-jsonc-edit', from: '0.1.2', to: '0.1.3' });
            expect(edit.text.split('\r\n')).toHaveLength(text.split('\r\n').length);
            expect(edit.text).toContain('version = "=0.1.3"\r\n');
          },
        }),
        it({
          name: 'computes every pin before returning, so a refused table leaves the inline pin of the same manifest unedited',
          fn: async () => {
            const text = '[dependencies]\ndep = { version = "=1.0.0" }\n\n[build-dependencies.dep]\nversion = "=0.9.0"\n';
            expect(thrownBy(() => replaceDependencyPin({ text, crateName: 'dep', from: '1.0.0', to: '1.0.1' })))
              .toBeInstanceOf(CrateReleaseError);
          },
        }),
      ],
    }),
  ],
});
