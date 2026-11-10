/**
 Checks that the two shapes the root `mise.toml` shares between task bodies are written once, in its `[vars]` section.

 The root task file already keeps its shared task-body scripts in `[vars]` (`fanout`, `fanout_packages`, `run_test_files`,
 `parse_usage_args`, `dispatch_workspace_node`, `run_playwright`). Two more shapes were hand-copied into the bodies that
 needed them: the predicate that skips deprecated and paused trees, and the raised stdout cap for reading the task list.
 Either copy can drift from the other with nothing to notice, so the checks below hold both to one definition and to an
 interpolation at every site that needs it.

 The counts are relations rather than numbers: the predicate has exactly as many readers as the file has test-tree
 walkers, and the cap has exactly as many readers as the file has task-list reads. The unit-test exemption list is
 counted the same way, because a second copy of it, or a `test:unit` template that stopped reading it, would leave an
 exemption declared and never honoured.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { readFile, } from 'node:fs/promises';

//region The file and its shapes

/**
 The root task file every package task inherits from.
 */
const taskFile = await readFile(new URL('../../../../mise.toml', import.meta.url,), 'utf8',);

/**
 Name of the `[vars]` table that owns the shared task bodies.
 */
const varsHeader = '[vars]';

/**
 The shared snippet that answers whether a test file sits in a deprecated or paused tree.
 */
const inactivitySnippet = '{{vars.is_inactive_test}}';

/**
 The shared snippet that declares the raised stdout cap for `mise tasks --json`.
 */
const bufferSnippet = '{{vars.mise_tasks_json_max_buffer}}';

/**
 The declaration each snippet must have exactly one of.
 */
const inactivityDeclaration = 'const isInactiveTest = ';

/**
 The declaration of the stdout cap.
 */
const bufferDeclaration = 'const MISE_TASKS_JSON_MAX_BUFFER = ';

/**
 The shared snippet that declares the packages whose trees hold no unit test file.
 */
const exemptionSnippet = '{{vars.unit_test_exempt_packages}}';

/**
 The declaration of the unit-test exemption list.
 */
const exemptionDeclaration = 'const unitTestExemptPackages = new Map([';

/**
 Text that starts every task-list read whose output needs the raised cap.
 */
const taskListRead = "execFileSync('mise', ['tasks'";

/**
 Text that starts every walk of the test tree, all of which consult the inactivity predicate.
 */
const testTreeWalk = "for await (const file of glob('**/*.unit.test.ts'";

//endregion

//region Reading the file

/**
 How many times a fixed text occurs.

 @param text - text to search

 @param needle - text to count

 @returns the count of non-overlapping occurrences

 @example
 ```ts
 countOf('aa', 'a') === 2;
 ```
 */
function countOf(text: string, needle: string,): number {
  return text.split(needle,).length - 1;
}

/**
 The text of the `[vars]` table: from its header to the next top-level table.

 @param text - contents of the root task file

 @returns the vars section, or an empty text when the file has none
 */
function varsSection(text: string,): string {
  const start = text.indexOf(`${varsHeader}\n`,);
  if (start === (-1)) return '';
  const rest = text.slice(start + varsHeader.length,);
  const end = rest.indexOf('\n[',);
  return end === (-1) ? rest : rest.slice(0, end,);
}

/**
 The vars section of the root task file.
 */
const sharedSnippets = varsSection(taskFile,);

//endregion

await describe({
  name: 'root mise.toml shared task-body shapes',
  children: [
    it({
      name: 'finds the sections it counts, so a renamed table cannot make the checks below vacuous',
      fn: async function findsItsSubjects() {
        expect(sharedSnippets,).not.toBe('',);
        expect(countOf(taskFile, testTreeWalk,),).toBeGreaterThan(0,);
        expect(countOf(taskFile, taskListRead,),).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'declares the deprecated-and-paused predicate once, in the vars section, and reads it from every test-tree walk',
      fn: async function onePredicate() {
        expect(countOf(sharedSnippets, inactivityDeclaration,),).toBe(1,);
        expect(countOf(taskFile, inactivityDeclaration,),).toBe(1,);
        expect(countOf(taskFile, inactivitySnippet,),).toBe(countOf(taskFile, testTreeWalk,),);
      },
    },),
    it({
      name: 'declares the unit-test exemption list once, in the vars section, and reads it from the template',
      fn: async function oneExemptionList() {
        /** A second list, or a template that stopped reading this one, would leave an exemption unread and silent. */
        expect(countOf(sharedSnippets, exemptionDeclaration,),).toBe(1,);
        expect(countOf(taskFile, exemptionDeclaration,),).toBe(1,);
        expect(countOf(taskFile, exemptionSnippet,),).toBe(1,);
      },
    },),
    it({
      name: 'declares the raised task-list stdout cap once, in the vars section, and reads it at every task-list read',
      fn: async function oneBufferCap() {
        expect(countOf(sharedSnippets, bufferDeclaration,),).toBe(1,);
        expect(countOf(taskFile, bufferDeclaration,),).toBe(1,);
        expect(countOf(taskFile, bufferSnippet,),).toBe(countOf(taskFile, taskListRead,),);
      },
    },),
  ],
},);
