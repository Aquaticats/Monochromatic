import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { adversarialValues } from './adversarial-fixture.ts';
import { usesOnly } from '@monochromatic-dev/shell-argument-core/ts/character-set.ts';
import { CargoMutantsToolingError } from './error.ts';
import {
  buildImageTag,
  buildLogBase,
  buildUnitName,
  stemOf,
} from './names.ts';

/**
 Characters a valid unit name may hold.
 */
const unitNameAlphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-';

await describe({
  name: '',
  children: [
    describe({
      name: stemOf.name,
      children: [
        it({
          name: 'removes a trailing .rs',
          fn: async () => {
            expect(stemOf('lock.rs')).toBe('lock');
          },
        }),
        it({
          name: 'keeps a name without the suffix',
          fn: async () => {
            expect(stemOf('lock')).toBe('lock');
          },
        }),
        it({
          name: 'keeps a .rs that is not the suffix',
          fn: async () => {
            expect(stemOf('a.rs.bak')).toBe('a.rs.bak');
          },
        }),
      ],
    }),
    describe({
      name: buildUnitName.name,
      children: [
        it({
          name: 'joins prefix, label and stem, turning underscores into hyphens',
          fn: async () => {
            expect(buildUnitName({ prefix: 'cli-git-mutpar', label: 'w5', file: 'push_env.rs' })).toBe('cli-git-mutpar-w5-push-env');
          },
        }),
        ...adversarialValues.map(function adversarialUnit(file) {
          return it({
            name: `gives a valid unit name for ${JSON.stringify(file)}`,
            fn: async () => {
              expect(usesOnly({ text: buildUnitName({ prefix: 'p', label: 'l', file }), allowed: unitNameAlphabet })).toBe(true);
            },
          });
        }),
        it({
          name: 'rejects a name over the systemd length limit',
          fn: async () => {
            expect(() => buildUnitName({ prefix: 'p', label: 'l', file: 'x'.repeat(300) })).toThrow(CargoMutantsToolingError);
          },
        }),
      ],
    }),
    describe({
      name: buildLogBase.name,
      children: [
        it({
          name: 'puts the attempt-less log name in the directory',
          fn: async () => {
            expect(buildLogBase({ directory: '/logs', prefix: 'mutpar', label: 'w5', file: 'lock.rs' })).toBe('/logs/mutpar-w5-lock.log');
          },
        }),
        it({
          name: 'turns a path separator into a hyphen so the log stays in the directory',
          fn: async () => {
            expect(buildLogBase({ directory: '/logs', prefix: 'p', label: 'l', file: '../../etc/x.rs' })).toBe('/logs/p-l-..-..-etc-x.log');
          },
        }),
        ...adversarialValues.map(function adversarialLog(file) {
          return it({
            name: `keeps the log for ${JSON.stringify(file)} directly in the directory`,
            fn: async () => {
              const base = buildLogBase({ directory: '/logs', prefix: 'p', label: 'l', file });
              expect(base.startsWith('/logs/p-l-')).toBe(true);
              expect(base.slice('/logs/'.length)).not.toContain('/');
            },
          });
        }),
      ],
    }),
    describe({
      name: buildImageTag.name,
      children: [
        it({
          name: 'gives the label followed by the index for the default template',
          fn: async () => {
            expect(buildImageTag({ template: '{label}{index}', label: 'w5', index: 2 })).toBe('w52');
          },
        }),
        it({
          name: 'follows another template',
          fn: async () => {
            expect(buildImageTag({ template: 'run-{label}-{index}-x', label: 'a', index: 0 })).toBe('run-a-0-x');
          },
        }),
        it({
          name: 'rejects a template without the index placeholder',
          fn: async () => {
            expect(() => buildImageTag({ template: '{label}', label: 'a', index: 0 })).toThrow(CargoMutantsToolingError);
          },
        }),
        it({
          name: 'rejects a tag that starts with a hyphen',
          fn: async () => {
            expect(() => buildImageTag({ template: '-{index}', label: 'a', index: 0 })).toThrow(CargoMutantsToolingError);
          },
        }),
        it({
          name: 'rejects a tag with a slash',
          fn: async () => {
            expect(() => buildImageTag({ template: 'a/{index}', label: 'a', index: 0 })).toThrow(CargoMutantsToolingError);
          },
        }),
        it({
          name: 'rejects a tag over the length limit',
          fn: async () => {
            expect(() => buildImageTag({ template: `${'x'.repeat(130)}{index}`, label: 'a', index: 0 })).toThrow(CargoMutantsToolingError);
          },
        }),
      ],
    }),
  ],
});
