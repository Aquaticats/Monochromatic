// Checks of the reading that turns the IDE's handled-key records into times.
// What: the record the reading names, held against the lines the application really writes.
// Why: the reading moved module once already and counted nothing afterwards, so the record it names is checked against
//      a line a session logged rather than against the code that writes it.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { KEY_STAMP_RECORD, keyStamps } from './key-stamps.ts';

// One record the application wrote for a key it handled, copied from a session's log.
const handledKey = '2026-11-10T07:36:34.725416Z DEBUG monochromatic_ide::native::caret_keys: source caret key control=false shift=false anchor=61 head=61 affinity=Downstream paged=0.0';

// A record of the same session that accounts for no key.
const otherRecord = '2026-11-10T07:36:34.456773Z DEBUG ide_app::syntax: language runtime manifest read source=embedded (1255 files, key 2cf5c33d2e26dfb4) grammars=27';

test(
  'a handled key is read from the record the application writes for it',
  () => {
    const stamp = Date.parse('2026-11-10T07:36:34.725416Z');
    assert.deepEqual(
      keyStamps([handledKey, otherRecord, handledKey].join('\n')),
      [stamp, stamp],
      'both handled keys are read, and the record that accounts for no key is not'
    );
  }
);

test(
  'the named record is the one a handled key writes, and no other record carries it',
  () => {
    assert.ok(handledKey.includes(KEY_STAMP_RECORD), 'the named record is in the line a handled key writes');
    assert.ok(!otherRecord.includes(KEY_STAMP_RECORD), 'a record that accounts for no key does not carry the name');
  }
);
