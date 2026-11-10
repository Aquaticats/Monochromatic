// Checks of the state directory name the confinement check derives from a project path.
// What: the shortened name the application puts below its private state root for one project, which the check
// recomputes to find the directory a confined server wrote to.
// Why: the check looks for the scratch directory a session left inside that path, so a name derived differently makes
// it read another project's state or none, and a wrong name silently reports no leftover.
import assert from 'node:assert/strict';
import { test } from 'node:test';

import { projectStateDirectoryName } from './project-state-directory-name.ts';

test(
  'keeps the name of a project that is already a directory name',
  () => {
    assert.equal(projectStateDirectoryName('/home/user/temp/agent/ide-language-confinement-abc/ts-project'), 'ts-project');
    assert.equal(projectStateDirectoryName('ts-project'), 'ts-project');
    assert.equal(projectStateDirectoryName('a.b-c_d'), 'a.b-c_d');
    assert.equal(projectStateDirectoryName('rust-project'), 'rust-project');
  }
);

test(
  'replaces every character a directory name cannot hold',
  () => {
    assert.equal(projectStateDirectoryName('/agent/my project'), 'my_project');
    assert.equal(projectStateDirectoryName('/agent/a b c'), 'a_b_c');
    assert.equal(projectStateDirectoryName('/agent/tab\tname'), 'tab_name');
    assert.equal(projectStateDirectoryName('/agent/line\nname'), 'line_name');
    assert.equal(projectStateDirectoryName('/agent/slash\\name'), 'slash_name');
  }
);

test(
  'replaces one character with one underscore, not one run with one underscore',
  () => {
    assert.equal(projectStateDirectoryName('/agent/a  b'), 'a__b');
    assert.equal(projectStateDirectoryName('/agent/a   b'), 'a___b');
  }
);

test(
  'replaces the characters outside ASCII with one underscore each',
  () => {
    assert.equal(projectStateDirectoryName('/agent/\u732b'), '_');
    assert.equal(projectStateDirectoryName('/agent/\u65e5\u672c\u8a9e'), '___');
    assert.equal(projectStateDirectoryName('/agent/cafe\u0301'), 'cafe_');
    // A character outside the basic plane is two code units, which the pattern replaced one by one.
    assert.equal(projectStateDirectoryName('/agent/\ud83d\ude00'), '__');
  }
);

test(
  'shortens the name to the length the application keeps',
  () => {
    assert.equal(projectStateDirectoryName('/agent/' + 'a'.repeat(40)), 'a'.repeat(40));
    assert.equal(projectStateDirectoryName('/agent/' + 'a'.repeat(41)), 'a'.repeat(40));
    assert.equal(projectStateDirectoryName('/agent/' + 'a b '.repeat(20)), 'a_b_a_b_a_b_a_b_a_b_a_b_a_b_a_b_a_b_a_b_');
  }
);

test(
  'keeps the dots and dashes of a name that starts with them',
  () => {
    assert.equal(projectStateDirectoryName('/agent/.hidden'), '.hidden');
    assert.equal(projectStateDirectoryName('/agent/-dash'), '-dash');
  }
);

test(
  'reads the name of a path with no component',
  () => {
    // `basename('/')` is the empty name, which the pattern replaced nothing in.
    assert.equal(projectStateDirectoryName('/'), '');
    assert.equal(projectStateDirectoryName('/agent/'), 'agent');
  }
);
