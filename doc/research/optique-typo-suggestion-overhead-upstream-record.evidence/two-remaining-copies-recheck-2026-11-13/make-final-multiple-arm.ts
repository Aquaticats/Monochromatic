// Makes the arm `multiple-final`: the agent's diffs 01 and 02 (the multiple() state fix) with two changes of this
// session's own, and writes the whole as one diff against the trace-patched base.
//
// The two changes:
// - a parser-level test that replaces the last item of a state above the threshold. The agent's parser-level tests
//   only append, so a view that fails to drop the replaced item was caught by a unit test alone;
// - the header of array-view.ts said a view is built with "no closures", while each caller passes a reader function
//   made per view; the sentence now says which closures are meant.
//
// Run: node make-final-multiple-arm.ts > keep/make-final-multiple-arm.jsonl

import { spawnSync, } from 'node:child_process';
import { cpSync, readFileSync, rmSync, writeFileSync, } from 'node:fs';

const HOME = process.env['HOME'];

if (HOME === undefined) throw new Error('HOME is not set',);

const HERE = import.meta.dirname;
const CLONE = `${HOME}/temp/agent/upstream-prototype.O1iPjcqr/optique`;
const DIFFS = `${HOME}/temp/agent/upstream-prototype.3lsGY9v7/keep/diffs`;
const MAIN = 'd26f1a244d4434f9dbfeb0317f6db32533cb8c32';
const TRACE = '/var/mnt/encrypted/repository/Monochromatic/doc/troubleshooting/optique-input-trace-copies-its-map-for-each-recorded-token.patch';
const SOURCE = 'packages/core/src';
const ARM = `${HERE}/arms/multiple-final`;

function replaceOnce({ file, find, replace, }: { file: string; find: string; replace: string; },): void {
  const path = `${ARM}/${SOURCE}/${file}`;
  const text = readFileSync(path, 'utf8',);

  if (text.split(find,).length !== 2) throw new Error(`${file} does not hold the text to replace exactly once`,);

  writeFileSync(path, text.replace(find, replace,),);
}

rmSync(ARM, { recursive: true, force: true, },);
cpSync(`${HERE}/arms/multiple`, ARM, { recursive: true, },);

replaceOnce({
  file: 'internal/array-view.ts',
  find: [
    ' * building a view allocates one empty array and one proxy and no closures: a',
    ' * view is made once per accepted token, and a per-view handler would cost more',
    ' * than the copies it removes on ordinary command lines.',
  ].join('\n',),
  replace: [
    ' * building a view allocates one empty array and one proxy and no trap',
    ' * closures; the reader its caller passes is the only function made per view.',
    ' * A view is made once per accepted token, and a per-view handler would cost',
    ' * more than the copies it removes on ordinary command lines.',
  ].join('\n',),
},);

// A third change, found by the recheck: Object.freeze() on a view threw only after it had made the view's empty
// target array non-extensible, and from then on Object.keys() of that view threw as well, because a proxy over a
// non-extensible target may not report keys the target lacks. The view now refuses to be made non-extensible, so a
// freeze attempt still throws and the view stays readable.
replaceOnce({
  file: 'internal/array-view.ts',
  find: [
    '  deleteProperty(target, property) {',
    '    if (indexOfProperty(property, elementsOf(target).length) !== undefined) {',
    '      refuseWrite();',
    '    }',
    '    return Reflect.deleteProperty(target, property);',
    '  },',
  ].join('\n',),
  replace: [
    '  deleteProperty(target, property) {',
    '    if (indexOfProperty(property, elementsOf(target).length) !== undefined) {',
    '      refuseWrite();',
    '    }',
    '    return Reflect.deleteProperty(target, property);',
    '  },',
    '  preventExtensions() {',
    '    // Freezing or sealing a view first makes its empty target array',
    '    // non-extensible, and a proxy over a non-extensible target may not report',
    '    // the index keys that target lacks, so every later `Object.keys()` of the',
    '    // view would throw.  Refusing here makes the freeze itself throw and',
    '    // leaves the view readable.',
    '    return false;',
    '  },',
  ].join('\n',),
},);
replaceOnce({
  file: 'internal/array-view.ts',
  find: ' * @throws {TypeError} When an element or `length` is written.',
  replace: [' * @throws {TypeError} When an element or `length` is written, or when the view', ' *         is frozen, sealed or made non-extensible.',].join('\n',),
},);
replaceOnce({
  file: 'internal/array-view.test.ts',
  find: '  test("two views of one source do not see each other\'s window", () => {',
  replace: [
    '  test("refuses to be frozen and stays readable afterwards", () => {',
    '    const view = virtualArrayOf(elementsOf(ITEMS));',
    '',
    '    assert.throws(() => Object.freeze(view), TypeError);',
    '    assert.throws(() => Object.seal(view), TypeError);',
    '    assert.throws(() => Object.preventExtensions(view), TypeError);',
    '    // A view whose target was made non-extensible on the way to that error',
    '    // can no longer list its keys.',
    '    assert.ok(Object.isExtensible(view));',
    '    assert.deepEqual(Object.keys(view), ["0", "1", "2", "3"]);',
    '    assert.deepEqual(view, ITEMS);',
    '  });',
    '',
    '  test("two views of one source do not see each other\'s window", () => {',
  ].join('\n',),
},);

// A fourth change, of the same kind: the view refused a write only to an index it holds. A write past its end went
// through to the empty target array, so `view[view.length] = x` raised nothing and read back afterwards as an
// element the view does not have, and a failed `push()` left the same behind. Every index is now refused, in
// assignments and in Object.defineProperty().
replaceOnce({
  file: 'internal/array-view.ts',
  find: [
    '  set(target, property, value) {',
    '    if (',
    '      property === "length" ||',
    '      indexOfProperty(property, elementsOf(target).length) !== undefined',
    '    ) {',
    '      refuseWrite();',
    '    }',
    '    return Reflect.set(target, property, value);',
    '  },',
  ].join('\n',),
  replace: [
    '  set(target, property, value) {',
    '    // An index past the end is refused as well: written through to the empty',
    '    // target it would read back as an element the view does not have.',
    '    if (',
    '      property === "length" ||',
    '      indexOfProperty(property, Number.POSITIVE_INFINITY) !== undefined',
    '    ) {',
    '      refuseWrite();',
    '    }',
    '    return Reflect.set(target, property, value);',
    '  },',
    '  defineProperty(target, property, descriptor) {',
    '    if (',
    '      property === "length" ||',
    '      indexOfProperty(property, Number.POSITIVE_INFINITY) !== undefined',
    '    ) {',
    '      refuseWrite();',
    '    }',
    '    return Reflect.defineProperty(target, property, descriptor);',
    '  },',
  ].join('\n',),
},);
replaceOnce({
  file: 'internal/array-view.test.ts',
  find: '  test("refuses to be frozen and stays readable afterwards", () => {',
  replace: [
    '  test("refuses writes past its end and keeps no trace of them", () => {',
    '    const writable = virtualArrayOf(elementsOf(ITEMS)) as string[];',
    '    const pastEnd = String(ITEMS.length);',
    '',
    '    assert.throws(() => writable.push("z"), TypeError);',
    '    assert.throws(() => writable.unshift("z"), TypeError);',
    '    assert.throws(() => {',
    '      writable[ITEMS.length] = "z";',
    '    }, TypeError);',
    '    assert.throws(() => {',
    '      writable[ITEMS.length + 3] = "z";',
    '    }, TypeError);',
    '    assert.throws(',
    '      () => Object.defineProperty(writable, "0", { value: "z" }),',
    '      TypeError,',
    '    );',
    '    assert.throws(',
    '      () => Object.defineProperty(writable, pastEnd, { value: "z" }),',
    '      TypeError,',
    '    );',
    '    // A write that reached the target array would show up as a key and as an',
    '    // element past the end.',
    '    assert.equal(writable[ITEMS.length], undefined);',
    '    assert.ok(!(ITEMS.length in writable));',
    '    assert.deepEqual(Object.keys(writable), ["0", "1", "2", "3"]);',
    '    assert.deepEqual(writable, ITEMS);',
    '  });',
    '',
    '  test("refuses to be frozen and stays readable afterwards", () => {',
  ].join('\n',),
},);

// A fifth change, found by the independent review of 2026-11-13: the view reported its `length` descriptor from the
// empty target, value 0, while reading `length` gave the presented count. A copy made with
// Object.defineProperties([], Object.getOwnPropertyDescriptors(view)), which the library does to array states in
// annotation-state.ts, came back empty. The descriptor now carries the presented length.
replaceOnce({
  file: 'internal/array-view.ts',
  find: [
    '    // `length` is a non-configurable own property of the target array and the',
    '    // proxy may not hide it, so it is reported from the target.  Index',
    '    // descriptors come from the elements, which is what makes deep equality',
    '    // and property enumeration see the presented elements.',
    '    if (property === "length") {',
    '      return Reflect.getOwnPropertyDescriptor(target, property);',
    '    }',
    '    const elements = elementsOf(target);',
  ].join('\n',),
  replace: [
    '    // `length` is a non-configurable own property of the target array and the',
    '    // proxy may not hide it, so its descriptor comes from the target, with the',
    '    // presented length as its value: the target\'s own value is 0, and a copy',
    '    // made from the view\'s descriptors would otherwise be cut back to nothing.',
    '    // The target\'s `length` is writable, so a proxy may report another value.',
    '    // Index descriptors come from the elements, which is what makes deep',
    '    // equality and property enumeration see the presented elements.',
    '    const elements = elementsOf(target);',
    '    if (property === "length") {',
    '      const descriptor = Reflect.getOwnPropertyDescriptor(target, property);',
    '      return descriptor === undefined',
    '        ? undefined',
    '        : { ...descriptor, value: elements.length };',
    '    }',
  ].join('\n',),
},);
replaceOnce({
  file: 'internal/array-view.test.ts',
  find: '  test("refuses writes past its end and keeps no trace of them", () => {',
  replace: [
    '  test("gives a copy made from its descriptors its length and elements", () => {',
    '    const view = virtualArrayOf(elementsOf(ITEMS));',
    '',
    '    assert.equal(',
    '      Object.getOwnPropertyDescriptor(view, "length")?.value,',
    '      ITEMS.length,',
    '    );',
    '    // A `length` descriptor taken from the empty target would cut this copy',
    '    // back to no elements.',
    '    const copy: string[] = [];',
    '    for (const key of Reflect.ownKeys(view)) {',
    '      const descriptor = Reflect.getOwnPropertyDescriptor(view, key);',
    '      if (descriptor !== undefined) {',
    '        Object.defineProperty(copy, key, descriptor);',
    '      }',
    '    }',
    '    assert.equal(copy.length, ITEMS.length);',
    '    assert.deepEqual(copy, ITEMS);',
    '  });',
    '',
    '  test("refuses writes past its end and keeps no trace of them", () => {',
  ].join('\n',),
},);

// The agent's module headers and one error message spoke of the input buffer as a second user of the view. The
// kept patch holds the state fix only, so they now speak of the state alone.
replaceOnce({
  file: 'internal/array-view.ts',
  find: [
    ' * Two values the parser hands around grow or shrink by one slot per accepted',
    ' * token: the state array of `multiple()` and the context\'s input buffer.',
    ' * Both are typed `readonly T[]` and are read by indexing, spreading and',
    ' * `Array.prototype` methods, so a persistent structure cannot replace them',
    ' * without a type or reader change.  This module keeps the array type and the',
  ].join('\n',),
  replace: [
    ' * The state array of `multiple()` grows by one slot per accepted item.  It is',
    ' * typed `readonly T[]` and is read by indexing, spreading and',
    ' * `Array.prototype` methods, so a persistent structure cannot replace it',
    ' * without a type or reader change.  This module keeps the array type and the',
  ].join('\n',),
},);
replaceOnce({
  file: 'internal/array-view.ts',
  find: [
    ' * @throws {TypeError} Always, because the elements belong to a parser state or',
    ' *         an input buffer that later versions share.',
    ' */',
    'function refuseWrite(): never {',
    '  throw new TypeError(',
    '    "An array view of parser state or input cannot be modified.",',
    '  );',
    '}',
  ].join('\n',),
  replace: [
    ' * @throws {TypeError} Always, because the elements belong to a parser state',
    ' *         that later versions share.',
    ' */',
    'function refuseWrite(): never {',
    '  throw new TypeError("An array view of parser state cannot be modified.");',
    '}',
  ].join('\n',),
},);
replaceOnce({
  file: 'internal/array-view.ts',
  find: [' * `length` throw, because its elements belong to a parser state or an input', ' * buffer that later versions share.',].join('\n',),
  replace: [' * `length` throw, because its elements belong to a parser state that later', ' * versions share.',].join('\n',),
},);
replaceOnce({
  file: 'internal/persistent-vector.ts',
  find: [
    ' * `multiple()` rebuilds its state array for every item it accepts, and the',
    ' * input buffer drops one token per accepted token, so a parse of N positional',
    ' * arguments copies 1 + 2 + ... + N slots when both are plain arrays.  This',
  ].join('\n',),
  replace: [
    ' * `multiple()` rebuilds its state array for every item it accepts, so a parse',
    ' * of N positional arguments copies 1 + 2 + ... + N slots when that state is a',
    ' * plain array.  This',
  ].join('\n',),
},);

// The agent's comment on the threshold cited two measurement scripts of its own by name; upstream has neither.
replaceOnce({
  file: 'internal/persistent-vector.ts',
  find: [' * 2,048, and 26.5 ms with the array copy alone (*crossover-sweep.ts*,', ' * *view-costs.ts*).',].join('\n',),
  replace: ' * 2,048, and 26.5 ms with the array copy alone.',
},);

const NEXT_TEST ='  it("should extend matched command items with optional arguments", () => {';
const NEW_TEST = `  it("should replace the last item of a long state and keep the state before", () => {
    // Each item takes two options, so the call that reads \`-b\` replaces the
    // item the call that read \`-a\` left half filled instead of adding one.
    const parser = multiple(object({
      a: option("-a", string()),
      b: option("-b", string()),
    }));
    const pairs = Array.from(
      { length: stateThreshold + 5 },
      (_, index) => ({ a: \`a\${index}\`, b: \`b\${index}\` }),
    );
    let state = parser.initialState;
    for (const pair of pairs) {
      for (const buffer of [["-a", pair.a], ["-b", pair.b]]) {
        const result = parser.parse({
          buffer,
          state,
          optionsTerminated: false,
          usage: parser.usage,
        });
        assert.ok(result.success);
        if (!result.success) return;
        state = result.next.state;
      }
    }
    // The state is past the size at which it starts sharing its items.
    assert.equal(state.length, pairs.length);

    const halfFilled = parser.parse({
      buffer: ["-a", "half"],
      state,
      optionsTerminated: false,
      usage: parser.usage,
    });
    assert.ok(halfFilled.success);
    if (!halfFilled.success) return;
    assert.equal(halfFilled.next.state.length, pairs.length + 1);

    const filledWithX = parser.parse({
      buffer: ["-b", "x"],
      state: halfFilled.next.state,
      optionsTerminated: false,
      usage: parser.usage,
    });
    const filledWithY = parser.parse({
      buffer: ["-b", "y"],
      state: halfFilled.next.state,
      optionsTerminated: false,
      usage: parser.usage,
    });
    assert.ok(filledWithX.success);
    assert.ok(filledWithY.success);
    if (!filledWithX.success || !filledWithY.success) return;

    // Filling the half-filled item replaces it, so the state keeps its length;
    // a state that keeps the replaced item as well is one item longer here.
    assert.equal(filledWithX.next.state.length, pairs.length + 1);
    assert.equal(filledWithY.next.state.length, pairs.length + 1);
    const withX = parser.complete(filledWithX.next.state);
    const withY = parser.complete(filledWithY.next.state);
    assert.ok(withX.success);
    assert.ok(withY.success);
    if (withX.success && withY.success) {
      assert.deepEqual(withX.value, [...pairs, { a: "half", b: "x" }]);
      assert.deepEqual(withY.value, [...pairs, { a: "half", b: "y" }]);
    }

    // The half-filled state both were taken from still lacks its second
    // option, and the state before it still completes to its own items.
    assert.ok(!parser.complete(halfFilled.next.state).success);
    const before = parser.complete(state);
    assert.ok(before.success);
    if (before.success) assert.deepEqual(before.value, pairs);
  });

`;

replaceOnce({ file: 'modifiers.test.ts', find: NEXT_TEST, replace: `${NEW_TEST}${NEXT_TEST}`, },);

// region The whole as one diff against the trace-patched base

function git(args: string[], cwd: string,): string {
  const child = spawnSync('git', args, { cwd, encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, },);

  if (child.status !== 0) throw new Error(`git ${args.join(' ',)} failed: ${child.stderr.slice(0, 600,)}`,);

  return child.stdout;
}

const worktree = `${HERE}/worktrees/multiple-final`;

// A removed worktree can leave its folder behind with a copy of ignored state in it (a `node_modules` folder was
// found there once), and `git worktree add` refuses a folder that exists.
rmSync(worktree, { recursive: true, force: true, },);
git(['worktree', 'prune',], CLONE,);
git(['worktree', 'add', '--detach', worktree, MAIN,], CLONE,);
git(['apply', '--index', TRACE,], worktree,);

const baseTree = git(['write-tree',], worktree,).trim();

git(['apply', '--index', `${DIFFS}/01-array-view.patch`,], worktree,);
git(['apply', '--index', `${DIFFS}/02-multiple-state.patch`,], worktree,);

const agentTree = git(['write-tree',], worktree,).trim();

for (const file of ['internal/array-view.ts', 'internal/array-view.test.ts', 'internal/persistent-vector.ts', 'modifiers.test.ts',]) {
  cpSync(`${ARM}/${SOURCE}/${file}`, `${worktree}/${SOURCE}/${file}`,);
  git(['add', '--', `${SOURCE}/${file}`,], worktree,);
}

const finalTree = git(['write-tree',], worktree,).trim();
const patch = git(['diff', '--cached', '--binary', baseTree,], worktree,);
const ownChanges = git(['diff', '--stat', agentTree, finalTree,], worktree,);

const PATCH = `${HERE}/keep/multiple-state-fix-with-recheck-changes.patch`;

writeFileSync(PATCH, patch,);
git(['worktree', 'remove', '--force', worktree,], CLONE,);

// Whether the diff applies where a reader would apply it: to upstream's main as it is, and to main with the trace
// patch. Each check is made in a fresh worktree, and the second one's resulting tree must be the tree made here.
function applies(name: string, first: string[],): Record<string, unknown> {
  const fresh = `${HERE}/worktrees/apply-check-${name}`;

  rmSync(fresh, { recursive: true, force: true, },);
  git(['worktree', 'prune',], CLONE,);
  git(['worktree', 'add', '--detach', fresh, MAIN,], CLONE,);

  for (const earlier of first) git(['apply', '--index', earlier,], fresh,);

  const check = spawnSync('git', ['apply', '--index', '--verbose', PATCH,], { cwd: fresh, encoding: 'utf8', },);
  const tree = check.status === 0 ? git(['write-tree',], fresh,).trim() : null;

  git(['worktree', 'remove', '--force', fresh,], CLONE,);

  return { on: name, applies: check.status === 0, tree, detail: `${check.stdout}${check.stderr}`.trim().split('\n',).slice(-8,), };
}

const onMain = applies('main', [],);
const onMainWithTrace = applies('main-with-trace-patch', [TRACE,],);

console.log(JSON.stringify({ baseTree, agentTree, finalTree, patchBytes: patch.length, patchLines: patch.split('\n',).length, ownChanges: ownChanges.trim().split('\n',), },),);
// Two diagnostic twins of the final arm: one whose states are always views, for the checks of what a caller can
// notice, and one whose states are views from three items on, the smallest threshold at which the patch's own
// tests keep their meaning, for upstream's suite.
for (const [twin, threshold,] of [['multiple-final-all-views', 0,], ['multiple-final-views-from-two', 2,],] as const) {
  const folder = `${HERE}/arms/${twin}`;

  rmSync(folder, { recursive: true, force: true, },);
  cpSync(ARM, folder, { recursive: true, },);

  const vectorPath = `${folder}/${SOURCE}/internal/persistent-vector.ts`;
  const vectorText = readFileSync(vectorPath, 'utf8',);

  if (vectorText.split('export const VIEW_THRESHOLD = 1024;',).length !== 2) throw new Error('the threshold line is not there once',);

  writeFileSync(vectorPath, vectorText.replace('export const VIEW_THRESHOLD = 1024;', `export const VIEW_THRESHOLD = ${threshold};`,),);
}

console.log(JSON.stringify(onMain,),);
console.log(JSON.stringify({ ...onMainWithTrace, sameTreeAsMadeHere: onMainWithTrace.tree === finalTree, },),);

// endregion The whole as one diff against the trace-patched base
