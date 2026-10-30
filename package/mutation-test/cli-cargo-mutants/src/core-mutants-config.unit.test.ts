import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 The `exclude_re` entries of the engine's `.cargo/mutants.toml` when the engine's mutation run moved to the shared
 mount-free flow, as they stand in the file. Each is a proven-equivalent mutant with its proof in the comment above it;
 the migration carries them over unchanged, and none may be dropped. A new proof adds an entry here and in the file.
 */
const carriedOverEntries: readonly string[] = [
  "  \"replace ByteSet::empty -> Self with Default::default\",",
  String.raw`  "replace \\| with \\^ in accept_bit",`,
  String.raw`  "replace \\| with \\^ in CountSet::copy_advanced_from",`,
  "  \"replace < with <= in CountSet::clear_above\",",
  "  \"replace == with != in byte_signature\",",
  "  \"replace > with >= in Dfa::validate\",",
  "  \"replace RegexSet::candidates_only_is_match -> bool with false\",",
  String.raw`  "delete match arm b'\\|' \\| b'&' in parse_atom",`,
  "  \"replace Cursor<'a>::eof -> bool with true\",",
  "  \"replace >= with < in Engine::is_match_batch\",",
  String.raw`  "replace && with \\|\\| in Dfa::is_match_batch_sheng",`,
  String.raw`  "replace \\+ with \\* in Regex::is_match_batch_bucketed",`,
  "  \"replace < with > in Regex::is_match_batch_bucketed\",",
  String.raw`  "replace \\+ with \\* in RegexSet::is_match_batch_concat",`,
  "  \"replace - with / in RegexSet::is_match_batch_concat\","
];

/**
 Path of the engine's cargo-mutants configuration.
 */
const configPath = resolve(
  import.meta.dirname,
  '..',
  '..',
  '..',
  'forbidden-strings',
  'core',
  '.cargo',
  'mutants.toml',
);

await describe({
  name: '',
  children: [
    describe({
      name: 'the engine\'s cargo-mutants configuration',
      children: [
        it({
          name: 'still holds every equivalent-mutant exclusion carried over by the migration',
          fn: async () => {
            const lines = (await readFile(
              configPath,
              'utf8',
            )).split('\n');
            expect(lines.filter(function isEntry(line) {
              return carriedOverEntries.includes(line);
            })).toEqual(carriedOverEntries);
          },
        }),
        it({
          name: 'holds fifteen entries, so a dropped proof cannot hide behind an added one',
          fn: async () => {
            const lines = (await readFile(
              configPath,
              'utf8',
            )).split('\n');
            expect(lines.filter(function isEntry(line) {
              return line.startsWith('  "') && line.endsWith('",');
            })).toHaveLength(carriedOverEntries.length);
          },
        }),
        it({
          name: 'is baked into the snapshot image',
          fn: async () => {
            const builder = await readFile(
              resolve(
                configPath,
                '..',
                '..',
                'bin',
                'build-test-image.ts',
              ),
              'utf8',
            );
            expect(builder).toContain("'.cargo',");
          },
        }),
      ],
    }),
  ],
});
