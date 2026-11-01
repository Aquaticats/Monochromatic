/**
 Adversarial name classification keeps retention sweeps inside the logger-owned families. @module
 */
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  _loggerOwnedDay as loggerOwnedDay,
  _NOT_LOGGER_OWNED as NOT_LOGGER_OWNED,
} from '@monochromatic-dev/logger-core/node';

//region Owned names carry the day key that ages them

/**
 Owned names paired with the day key each must classify to.
 */
const OWNED_CASES: readonly (readonly [name: string, day: string])[] = [
  // Day-directory names classify by their exact day key.
  ['2026-10-22', '2026-10-22',],
  // Shape-only validation: a calendar-impossible day key stays owned.
  ['2026-99-99', '2026-99-99',],
  ['2026-02-30T12-00-00.000Z.x.log.jsonl', '2026-02-30',],
  ['2026-10-22T14-00-01.123Z.550e8400-e29b-41d4-a716-446655440000.log.jsonl', '2026-10-22',],
  ['2026-10-22T14-00-01.123Z.550e8400-e29b-41d4-a716-446655440000.probe.tmp', '2026-10-22',],
  ['2026-12-31T23-59-59.999Z.x.log.jsonl', '2026-12-31',],
  ['2026-01-01T00-00-00.000Z.x.probe.tmp', '2026-01-01',],
];

/**
 Newline embedded literally so adversarial names stay byte-exact without escapes.
 */
const NEWLINE_IN_DAY_KEY = `2026-10-2
2T14-00-01.x.log.jsonl`;

/**
 Newline embedded literally inside the timestamp body of an otherwise owned name.
 */
const NEWLINE_IN_TIMESTAMP_BODY = `2026-10-22T14-00
-01.x.log.jsonl`;

/**
 Foreign names a sweep must never classify as logger-owned.
 */
const FOREIGN_CASES: readonly string[] = [
  // Empty and short names.
  '',
  '2',
  '2026',
  '2026-10-2',
  // Segment-width and separator shapes that only look like a day key.
  '2026--22',
  '2026--22-01',
  '2026-1-22T14-00-01.x.log.jsonl',
  '2026-1-22T....log.jsonl',
  '2026.10.22',
  '2026-10-22x.log.jsonl',
  '2026-10-2234.log.jsonl',
  '2026-10-22 ',
  // Traversal shapes, both as pure prefixes and with owned suffixes attached.
  '../../2026-10-22',
  '../../2026-10-22T14-00-01.x.log.jsonl',
  '2026-10-22/../..',
  '2026-10-22/../../x.log.jsonl',
  // Newlines inside the day key.
  NEWLINE_IN_DAY_KEY,
  `2026-10-22
`,
  // Unicode digits that only resemble ASCII digits.
  '２０２６-10-22',
  '2026-10-２２',
  // Deliberate neighbors and the sweep's own stamp.
  'translation-repair-runs-2026-10-22',
  'translation-repair-runs-2026-10-22T14-00-01.x.log.jsonl',
  '.logger-retention-sweep',
  // Suffix families outside the owned log and probe names.
  '2026-10-22T14-00-01.x.tmp',
  '2026-10-22T14-00-01.x.probe.tmp.bak',
  '2026-10-22T14-00-01.x.log.jsonl.bak',
  '2026-10-22T14-00-01.x.LOG.JSONL',
  '2026-10-22T14-00-01.x.log.jsonl ',
  // Command separators with no owned suffix.
  '2026-10-22;rm -rf',
];

//endregion Owned names carry the day key that ages them

await describe({
  name: '',
  children: [
    describe({
      name: loggerOwnedDay.name,
      children: [
        //region Owned families

        ...OWNED_CASES.map(function ownedCase([name, day,]) {
          return it({
            name: `classifies ${JSON.stringify(name,)} as owned with day key ${day}`,
            fn: async () => {
              expect(loggerOwnedDay(name,),).toBe(day,);
            },
          },);
        },),

        //endregion Owned families

        //region Foreign names stay out of every sweep

        ...FOREIGN_CASES.map(function foreignCase(name,) {
          return it({
            name: `classifies ${JSON.stringify(name,)} as not logger owned`,
            fn: async () => {
              expect(loggerOwnedDay(name,),).toBe(NOT_LOGGER_OWNED,);
            },
          },);
        },),

        //endregion Foreign names stay out of every sweep

        //region Classification is shape-only

        it({
          name: 'classifies by name shape only, ignoring calendar validity and the timestamp body',
          fn: async () => {
            // The day key needs only ASCII digits in YYYY-MM-DD widths, and
            // the body between the T separator and the family suffix is never
            // read. Destination directories are owner-only (0o700), so this
            // deliberate narrowness keeps the sweep one linear pass.
            expect(loggerOwnedDay('2026-99-99',),).toBe('2026-99-99',);
            expect(loggerOwnedDay(NEWLINE_IN_TIMESTAMP_BODY,),).toBe('2026-10-22',);
            expect(loggerOwnedDay('2026-10-22T14;00;00;x.log.jsonl',),).toBe('2026-10-22',);
          },
        },),

        it({
          name: 'returns the unique foreign sentinel rather than a nullish value',
          fn: async () => {
            expect(typeof NOT_LOGGER_OWNED,).toBe('symbol',);
            expect(loggerOwnedDay('notes.txt',),).toBe(NOT_LOGGER_OWNED,);
          },
        },),

        //endregion Classification is shape-only
      ],
    },),
  ],
},);
