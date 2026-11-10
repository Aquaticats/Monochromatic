// The directory a log line reports having listed, against the lines the IDE's refresh log carries.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { listedDirectoryPath } from './directory-snapshot-path.ts';

// The line the IDE prints for every directory it reads, as the idle-cost measurement collects it.
const listed = (path: string): string => '2026-10-20T10:00:00.123456Z DEBUG ide_app::refresh: read directory snapshot path=' + path + ' folders=12 files=96 entries=108';

test('reads the directory a listing line names', () => {
  assert.equal(listedDirectoryPath(listed('/home/user/temp/agent/ide-idle-cost-x/project/folder-001')), '/home/user/temp/agent/ide-idle-cost-x/project/folder-001');
});

test('reads nothing from a line that names no directory', () => {
  assert.equal(listedDirectoryPath('2026-10-20T10:00:00.123456Z DEBUG ide_app::refresh: started the safety reread'), undefined);
  assert.equal(listedDirectoryPath(''), undefined);
});

test('reads nothing when the field names no path', () => {
  assert.equal(listedDirectoryPath('read directory snapshot path= folders=12'), undefined);
  assert.equal(listedDirectoryPath('read directory snapshot path='), undefined);
});

test('reads the path up to the first character whitespace names', () => {
  const endings = ['\t', '\n', '\r', ' ', '\u00a0', '\u2028', '\u3000', '\ufeff'];
  for (const ending of endings) {
    assert.equal(listedDirectoryPath('read directory snapshot path=/a/b' + ending + 'folders=1'), '/a/b', JSON.stringify(ending));
  }
});

test('reads the path from the next field when an earlier one names none', () => {
  assert.equal(listedDirectoryPath('read directory snapshot path= path=/a/b folders=1'), '/a/b');
});

test('reads a path that ends the line, and one that follows text without a space', () => {
  assert.equal(listedDirectoryPath('read directory snapshot xpath=/a/b'), '/a/b');
  assert.equal(listedDirectoryPath('read directory snapshot path=/a/b'), '/a/b');
});
