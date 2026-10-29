/**
 Guards the family of defect where an outcome a reader sees is decided by
 which of several concurrent operations happens to end first.

 `Promise.all`, `Promise.race` and `Promise.any` settle on whichever member
 ends first, so where several members can fail with failures a reader tells
 apart, one input printed different refusals on different runs. The package's
 answer is one helper, `allInInputOrder` (`all-in-input-order.ts`), which
 reports the failure of the first failing member in input order. THIS SCAN
 KEEPS THE FAMILY CLOSED: production code calls `Promise.all`,
 `Promise.allSettled`, `Promise.race` and `Promise.any` only where this file
 lists the call, the helper's own one or a site with the reason it is outside
 the family.

 WHAT THE SCAN READS, in package source (not tests and fixtures): the parsed
 program, so a mention in a comment or a string is none. A call it follows is
 one whose callee is a member of the identifier `Promise` named plainly or by
 a string literal, through any wrapper a callee can wear (optional, non-null,
 parentheses), and that name is the combinator called. Every other way it
 sees to reach a combinator is a finding. `Promise` read other than as such a
 callee, a type, `new Promise` or a member that is no combinator: a
 combinator read but not called, `Promise` bound to another name,
 destructured (in a declaration, an assignment or a parameter's default),
 read by a computed key that is no string literal (a variable, a template, an
 expression), cast, or handed on whole. `globalThis` or `global` read other
 than by a named member. A member named `Promise` read off any value, which
 reaches `Promise` off the global object, a chain of its members or a
 namespace import. A member named `constructor` read off any value, which
 reaches `Promise` from any promise. A member named like a combinator read off
 anything but `Promise`, which may be `Promise` reached another way, save
 `AbortSignal.any`, which settles no promise. And any of those names taken by
 destructuring a value other than `Promise` or the global object, whose own
 read is the finding there. A member named plainly that is none of those
 (`Promise.resolve`) reaches no combinator and passes.

 EACH CALL IS KEYED `path#function`, by the function holding it: its own
 name, the name it is bound to (by a declaration, an assignment, a default or
 an object key), or its class and method (`Cat.groom`); `<module>` outside
 any function. A call in a function with no name, or in one whose name
 another function of the same file carries, is a finding, so two sites of
 one file never share a key unsaid. No site is keyed by its line.

 THE LIST IS THIS FILE'S, and each reason was read against the code it
 excuses. The entries of one site excuse its calls of their combinator one
 each, the first entry the first call in source order. A site holding more
 or fewer calls than it has entries is a finding naming every call it holds,
 and none of its entries is checked then, since none can be paired with the
 call it was read against. Each entry names what its reason rests on, and the
 scan holds that to THE CALL IT EXCUSES. `catch` is every catch clause inside
 the call, at least one, each throwing nothing, calling no function named as
 the package names a helper that throws (`assert`, `require`, `rethrow`,
 `throw`), binding its error to a name rather than taking it apart, and
 reading that name only to hand it to a named reader, whole or as an object
 argument's property. A name must stand in the innermost statement making the
 call, or in code that feeds it a variable, directly or through other such
 code: an earlier statement of each block on the way out to the function, or
 the head of a loop whose body holds the call; for a reason resting on how
 settled results are read, also in a later statement of the call's own block
 reading a variable the call's statement declares. A name written as
 `{ inHandler }` must stand in the handler of a `try` whose block holds the
 call, for a reason resting on what that catch does with a rejection. An
 entry whose call no longer holds what it rests on is a finding. The helper's
 own call is listed like any other, counted and resting on nothing the scan
 reads. A tripwire, not a proof: what a named function does is read by hand
 when the entry is written.

 OUT OF THE SCAN'S REACH: `Promise` reached through text the parser does not
 read as code (`eval`, `Function`, a combinator called inside that text);
 through a computed key that is no string literal, or a reflective read
 (`Reflect.get(p, 'constructor')`), off any value but `Promise` and the
 global object, since every such read in the package would otherwise be a
 finding; through an alias another module exports that is handed on or read
 by such a key (one read by a combinator's name is found); combinators of
 other libraries; concurrency that is no `Promise` combinator
 (`mapOverlapped` throws the lowest position's failure by its own
 construction, `p-limit` only bounds); a pair of promises started by hand and
 awaited in turn, which is searched for by reading; a call in a class field's
 initializer or a static block, which is keyed by the function or module
 around the class; and what a function an anchor names actually does.

 THE FIXTURE CASES COME FIRST, so the package-wide case is read against a
 scan shown able to find each form (ledger M21). Fixtures are cat-themed; the
 package case reads this package's own source.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { wordForCount, } from '@monochromatic-dev/translation-repair-typography';
import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  identifierName,
  isTreeNode,
  nodesUnder,
  parentsOf,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Combinator calls
// Which calls of a `Promise` combinator production code makes, keyed by the
// function they sit in, which reads of `Promise` it cannot follow, and which
// calls the list excuses.

/**
 Combinators that settle by completion rather than by position.
 */
const COMBINATORS: ReadonlySet<string> = new Set([
  'all',
  'allSettled',
  'any',
  'race',
],);

/**
 Members named like a combinator that the package reads off something other
 than `Promise`, each as `object.member`: `AbortSignal.any` combines abort
 signals into one and settles no promise.
 */
const COMBINATOR_NAMES_ELSEWHERE: ReadonlySet<string> = new Set(['AbortSignal.any',],);

/**
 Node kinds that open a function, which holds the calls inside it.
 */
const FUNCTION_KINDS: ReadonlySet<string> = new Set([
  'ArrowFunctionExpression',
  'FunctionDeclaration',
  'FunctionExpression',
],);

/**
 Node kinds that open a class, whose name a method's site carries.
 */
const CLASS_KINDS: ReadonlySet<string> = new Set([
  'ClassDeclaration',
  'ClassExpression',
],);

/**
 Names of the global object, one of whose members is `Promise` itself.
 */
const GLOBAL_OBJECTS: ReadonlySet<string> = new Set([
  'global',
  'globalThis',
],);

/**
 Where an identifier stands in a type, which no call can make: each as the
 kind of the node holding it and the field it is held in.
 */
const TYPE_POSITIONS: ReadonlySet<string> = new Set([
  'TSClassImplements.expression',
  'TSInterfaceHeritage.expression',
  'TSQualifiedName.left',
  'TSTypeQuery.exprName',
  'TSTypeReference.typeName',
],);

/**
 Where an identifier is the name a member or a key carries rather than a read
 of a binding, wherever that name is not computed.
 */
const NAME_POSITIONS: ReadonlySet<string> = new Set([
  'MemberExpression.property',
  'MethodDefinition.key',
  'Property.key',
  'PropertyDefinition.key',
  'TSMethodSignature.key',
  'TSPropertySignature.key',
],);

/**
 Where a statement stands in a list of statements, as the kind of the node
 holding the list and the field holding it; the earlier statements of such a
 list are where a statement feeding a call is looked for.
 */
const STATEMENT_LISTS: ReadonlySet<string> = new Set([
  'BlockStatement.body',
  'Program.body',
  'StaticBlock.body',
  'SwitchCase.consequent',
],);

/**
 Loops whose head can declare what their body reads, each with the fields its
 head is made of.
 */
const LOOP_HEADS: ReadonlyMap<string, readonly string[]> = new Map([
  [
    'ForInStatement',
    [
      'left',
      'right',
    ],
  ],
  [
    'ForOfStatement',
    [
      'left',
      'right',
    ],
  ],
  [
    'ForStatement',
    [
      'init',
      'test',
      'update',
    ],
  ],
],);

/**
 Where a pattern takes apart the value it destructures, with the field of the
 same node holding that value.
 */
const DESTRUCTURED_FROM: ReadonlyMap<string, string> = new Map([
  [
    'AssignmentExpression.left',
    'right',
  ],
  [
    'AssignmentPattern.left',
    'right',
  ],
  [
    'VariableDeclarator.id',
    'init',
  ],
],);

/**
 Functions a quiet catch may hand its error to, each of which turns a caught
 value into text or a record and throws nothing.
 */
const CATCH_READERS: ReadonlySet<string> = new Set([
  'errorName',
  'refusalOf',
  'refusalText',
],);

/**
 How the package names a helper that throws, a caught value or a refusal of
 its own; a catch calling one is no quiet catch.
 */
const THROWING_PREFIXES: readonly string[] = [
  'assert',
  'require',
  'rethrow',
  'throw',
];

/**
 What an entry rests on where its reason is the catches inside the call
 rather than a name; a reserved word, so no identifier can be mistaken for it.
 */
const CATCH_ANCHOR = 'catch';

/**
 Site of a call no function holds.
 */
const MODULE_SITE = '<module>';

/**
 Name a class's methods are keyed under where the class has none.
 */
const NAMELESS_CLASS = '<class>';

/**
 Why a listed call is outside the family.
 */
type Reason =
  | 'cannot-reject'
    | 'catches-own'
    | 'settled-in-input-order'
    | 'shared-abort';

/**
 What a listed reason rests on: a name the call must still be made with,
 `catch` for the catches inside the call, or a name the handler of a `try`
 around the call must hold.

 @example
 ```ts
 const anchor: Anchor = { inHandler: 'throwIfAborted', };
 ```
 */
type Anchor =
  | string
    | {
    /**
     Name the handler of a `try` whose block holds the call must hold, where
     what the reason rests on is what that catch does with a rejection.
     */
      readonly inHandler: string;
    };

/**
 One call the scan excuses.

 @example
 ```ts
 const site: ListedSite = { site: 'nap.ts#napAll', combinator: 'all', reason: 'catches-own', restsOn: ['catch',], note: 'each cat naps alone', };
 ```
 */
type ListedSite = {
  /**
   File, relative to `src`, and the function the call sits in.
   */
  readonly site: string;

  /**
   Combinator called.
   */
  readonly combinator: string;

  /**
   Why no completion order can show: one of the ways a call is outside the family.
   */
  readonly reason: Reason;

  /**
   What the reason rests on, each held to the call; empty for the helper's
   own call alone, which rests on its count.
   */
  readonly restsOn: readonly Anchor[];

  /**
   What in the code makes the reason hold here, read against that code.
   */
  readonly note: string;
};

/**
 One call of a combinator found in a file, keyed.
 */
type FoundCall = {
  /**
   File and function the call sits in.
   */
  readonly site: string;

  /**
   Combinator called.
   */
  readonly combinator: string;

  /**
   Where the call starts, `path:line`.
   */
  readonly place: string;

  /**
   The call itself.
   */
  readonly call: TreeNode;

  /**
   The function the call sits in, or the program at module level.
   */
  readonly holder: TreeNode;

  /**
   Each node of the call's file with the node holding it.
   */
  readonly parents: ReadonlyMap<TreeNode, TreeNode>;
};

/**
 A finding of one file, with the offset that orders it.
 */
type PlacedFinding = {
  /**
   Offset of the node the finding is about.
   */
  readonly at: number;

  /**
   The finding.
   */
  readonly text: string;
};

/**
 What one file holds: its keyed calls of a combinator, and the findings no
 list can excuse.
 */
type FileReading = {
  /**
   Calls of a combinator that could be keyed, in source order.
   */
  readonly calls: readonly FoundCall[];

  /**
   Reads of `Promise` the scan cannot follow, and calls it cannot key.
   */
  readonly findings: readonly PlacedFinding[];
};

/**
 One node met by the walk, with what its findings and its key need.
 */
type Visit = {
  /**
   The node.
   */
  readonly node: TreeNode;

  /**
   The node holding it; the program is its own.
   */
  readonly parent: TreeNode;

  /**
   Field of the parent the node is held in, empty for the program.
   */
  readonly field: string;

  /**
   The function the node sits in, or the program.
   */
  readonly holder: TreeNode;

  /**
   Name of the class the node sits in, which a method's key carries.
   */
  readonly className: string;
};

/**
 The member a member expression reads: named plainly or by a string literal,
 or by a key the scan cannot read.
 */
type MemberKey =
  | {
    readonly kind: 'named';
    readonly name: string;
  }
    | {
      readonly kind: 'computed';
      readonly key: TreeNode;
    };

/**
 Code before a call that an anchor may stand in: a statement, or a loop's head.
 */
type ReachPart = {
  /**
   Names it declares, through which it feeds the code reading them.
   */
  readonly declares: readonly string[];

  /**
   Its nodes: the statement, or each field of the head.
   */
  readonly nodes: readonly TreeNode[];
};

/**
 What an anchor of a listed call is looked for in, gathered on the way out
 from the call to the function holding it.
 */
type CallSurroundings = {
  /**
   The innermost statement making the call, or an arrow's expression body
   where no statement holds it.
   */
  readonly own: TreeNode;

  /**
   Code before the call that could feed it, nearest first: the earlier
   statements of each statement list on the way out, and the head of each
   loop whose body holds the call.
   */
  readonly earlier: readonly ReachPart[];

  /**
   Statements after the call's own in the list holding it, or holding the
   `export` around it, the only ones a variable its statement declares is in
   scope for.
   */
  readonly later: readonly TreeNode[];

  /**
   Handlers of the `try` statements whose block holds the call, nearest first.
   */
  readonly handlers: readonly TreeNode[];
};

/**
 Where a node starts in a file.

 @param file - file holding the node, whose path the place names

 @param node - node a finding is about

 @returns `path:line`, the line one-based

 @example
 ```ts
 const place = placeOf({ file, node, },); // 'cat.ts:3'
 ```
 */
function placeOf({ file, node, }: { readonly file: SourceText; readonly node: TreeNode; },): string {
  /**
   Lines before the node, its own included.
   */
  const lines = file.text
    .slice(
    0,
    node.start,
    )
    .split('\n',)
    .length;
  return `${file.path}:${String(lines,)}`;
}

/**
 The nodes a field holds, a list or one node.

 @param value - field the walk descends into, which holds one node, a list of
 them or a value that is no node, depending on the field

 @returns Nodes in it, so a list and a single child are walked alike;
 none for a field holding no node

 @example
 ```ts
 const statements = treeNodesIn(program.body,);
 ```
 */
function treeNodesIn(value: unknown,): readonly TreeNode[] {
  return (Array.isArray(value,) ? value : [value,]).filter(function isNode(item: unknown,): item is TreeNode {
    return isTreeNode(item,);
  },);
}

/**
 Where a node stands: the kind of the node holding it and the field it is held in.

 @param parent - node holding it, whose kind says what one of its fields means

 @param field - field it is held in, since one kind holds an identifier as a
 name in one field and as a read in another

 @returns `Kind.field`, which the position sets are keyed by

 @example
 ```ts
 const position = positionOf({ parent, field: 'callee', },); // 'NewExpression.callee'
 ```
 */
function positionOf({ parent, field, }: { readonly parent: TreeNode; readonly field: string; },): string {
  return `${parent.type}.${field}`;
}

/**
 The field of its parent a node is held in.

 @param node - node whose position a check needs, where only its parent is known

 @param parent - node holding it

 @returns Field holding it directly or in a list

 @throws {@link Error} where no field of the parent holds the node, which the
 parent map never records

 @example
 ```ts
 const field = fieldOf({ node, parent, },); // 'key' for the key of a shorthand property
 ```
 */
function fieldOf({ node, parent, }: { readonly node: TreeNode; readonly parent: TreeNode; },): string {
  /**
   The field holding it, with its value.
   */
  const holding = Object.entries(parent,).find(function holds([, value,],): boolean {
    return treeNodesIn(value,).includes(node,);
  },);
  if (holding === undefined) {
    throw new Error(`unreachable: no field of a ${parent.type} holds the ${node.type} at offset ${
      String(node.start,)
    }, though the parent map names it the node's parent`,);
  }
  return holding[0];
}

/**
 The name a key node carries, as a class member or an object key names it.

 @param key - key node of a property, a method or a class field

 @param computed - whether the key is written in brackets, where only a
 string literal names it

 @returns Name, `#` before a private one, empty where the key is computed from anything but a string literal

 @example
 ```ts
 const name = keyName({ key: method.key, computed: method.computed === true, },); // 'groom'
 ```
 */
function keyName({ key, computed, }: { readonly key: unknown; readonly computed: boolean; },): string {
  if (!isTreeNode(key,))
    return '';
  /**
   The key's value, a string for a string literal, and its name, a string for
   an identifier or a private name.
   */
  const {
    value,
    name,
  } = key;
  if ((key.type === 'Literal') && ((typeof value) === 'string'))
    return value;
  if (computed)
    return '';
  if ((key.type === 'PrivateIdentifier') && ((typeof name) === 'string'))
    return `#${name}`;
  return identifierName({ node: key, },);
}

/**
 The member a member expression reads.

 @param member - member expression, whose key decides whether the scan can name what it reads

 @returns Member's name, or the key the scan cannot read

 @throws {@link Error} where the expression holds no property node, which the parser never builds

 @example
 ```ts
 const key = memberKeyOf({ member: node, },); // { kind: 'named', name: 'all', } for Promise['all']
 ```
 */
function memberKeyOf({ member, }: { readonly member: TreeNode; },): MemberKey {
  /**
   The member's key.
   */
  const { property, } = member;
  if (!isTreeNode(property,))
    throw new Error(`unreachable: a ${member.type} at offset ${String(member.start,)} holds no property node`,);
  /**
   Name the key carries, empty where it is computed from anything but a string.
   */
  const name = keyName({
    key: property,
    computed: member.computed === true,
  },);
  return (name === '')
    ? {
      kind: 'computed',
      key: property,
    }
    : {
      kind: 'named',
      name,
    };
}

/**
 Name of the function a call calls, as written at the call.

 @param call - call whose callee is named, so a catch's check can tell a named
 reader or a helper that throws from any other function

 @returns Callee's name, or a plainly named member's, empty for any other callee

 @example
 ```ts
 const name = calleeNameOf({ call, },); // 'warn' for l.warn(...)
 ```
 */
function calleeNameOf({ call, }: { readonly call: TreeNode; },): string {
  /**
   The callee, without wrappers.
   */
  const { inner: callee, } = unwrapped({ node: call.callee, },);
  if (!isTreeNode(callee,))
    return '';
  if (callee.type !== 'MemberExpression')
    return identifierName({ node: callee, },);
  /**
   The member called.
   */
  const key = memberKeyOf({ member: callee, },);
  return (key.kind === 'named') ? key.name : '';
}

/**
 Name a class's methods are keyed under.

 @param node - class whose name its methods' sites carry

 @param parent - node holding the class, which binds an unnamed one

 @param field - field of the parent the class is held in, which says whether
 the parent binds it to a name

 @returns Its own name, the name it is bound to, or a stand-in for a class with neither

 @example
 ```ts
 const className = classNameOf({ node, parent, field, },); // 'Cat'
 ```
 */
function classNameOf(
  {
    node,
    parent,
    field,
  }: {
    readonly node: TreeNode;
    readonly parent: TreeNode;
    readonly field: string;
  },
): string {
  /**
   The class's own name.
   */
  const own = identifierName({ node: node.id, },);
  if (own !== '')
    return own;
  /**
   The name it is bound to, where a declaration binds it.
   */
  const bound = (positionOf({
    parent,
    field,
  },) === 'VariableDeclarator.init')
    ? identifierName({ node: parent.id, },)
    : '';
  return (bound === '') ? NAMELESS_CLASS : bound;
}

/**
 Name a function's calls are keyed by.

 @param visit - function node where the walk met it, whose parent binds an
 unnamed function and whose class names a method

 @returns Class and method for a method or a class field, else its own
 name, else the name a declaration, an assignment, a default or an object key
 binds it to; empty where it has none

 @example
 ```ts
 const name = functionNameOf({ visit, },); // 'Cat.groom'
 ```
 */
function functionNameOf({ visit, }: { readonly visit: Visit; },): string {
  /**
   The function, the node holding it, the field it is held in and the class
   around it, which between them carry every name it can be keyed by.
   */
  const {
    node,
    parent,
    field,
    className,
  } = visit;
  /**
   Where the function stands.
   */
  const position = positionOf({
    parent,
    field,
  },);
  if ((position === 'MethodDefinition.value') || (position === 'PropertyDefinition.value')) {
    /**
     Name of the method or the field.
     */
    const member = keyName({
      key: parent.key,
      computed: parent.computed === true,
    },);
    return (member === '') ? '' : `${className}.${member}`;
  }
  /**
   The function's own name.
   */
  const own = identifierName({ node: node.id, },);
  if (own !== '')
    return own;
  if (position === 'VariableDeclarator.init')
    return identifierName({ node: parent.id, },);
  if ((position === 'AssignmentExpression.right') || (position === 'AssignmentPattern.right'))
    return identifierName({ node: parent.left, },);
  if (position === 'Property.value') {
    return keyName({
      key: parent.key,
      computed: parent.computed === true,
    },);
  }
  return '';
}

/**
 The nodes a node holds, each with what its own visit needs.

 @param visit - node visited, which hands its children the function and the
 class they sit in

 @returns One visit per child node

 @example
 ```ts
 pending.push(...childVisits({ visit, },),);
 ```
 */
function childVisits({ visit, }: { readonly visit: Visit; },): readonly Visit[] {
  /**
   The node whose children are visited.
   */
  const { node, } = visit;
  /**
   Function the children sit in.
   */
  const holder = FUNCTION_KINDS.has(node.type,) ? node : visit.holder;
  /**
   Class the children sit in.
   */
  const className = CLASS_KINDS.has(node.type,)
    ? classNameOf({
      node,
      parent: visit.parent,
      field: visit.field,
    },)
    : visit.className;
  return Object.entries(node,).flatMap(function childrenOf([field, value,],): readonly Visit[] {
    return treeNodesIn(value,).map(function visitOf(child,): Visit {
      return {
        node: child,
        parent: node,
        field,
        holder,
        className,
      };
    },);
  },);
}

/**
 The combinator a call calls on `Promise`, recording its callee as called.

 @param call - call whose callee may be a combinator of `Promise`, which is
 then a call the scan follows rather than a read it cannot

 @param called - member expressions met as a callee, which the read of
 `Promise` inside them is then no finding for

 @returns Combinator, empty where the call calls none

 @example
 ```ts
 const combinator = combinatorCalled({ call, called, },); // 'all' for Promise.all(...)
 ```
 */
function combinatorCalled({ call, called, }: { readonly call: TreeNode; readonly called: Set<TreeNode>; },): string {
  /**
   The callee, without wrappers.
   */
  const { inner: callee, } = unwrapped({ node: call.callee, },);
  if ((!isTreeNode(callee,)) || (callee.type !== 'MemberExpression')
    || (identifierName({ node: callee.object, },) !== 'Promise'))
    return '';
  /**
   The member called.
   */
  const key = memberKeyOf({ member: callee, },);
  if ((key.kind !== 'named') || (!COMBINATORS.has(key.name,)))
    return '';
  called.add(callee,);
  return key.name;
}

/**
 Words for a computed key the scan cannot read.

 @param key - key node, whose kind is what a finding names

 @returns `a variable`, `a template` or `an expression`

 @example
 ```ts
 const kind = keyKindOf({ key, },); // 'a template'
 ```
 */
function keyKindOf({ key, }: { readonly key: TreeNode; },): string {
  if (key.type === 'Identifier')
    return 'a variable';
  return (key.type === 'TemplateLiteral') ? 'a template' : 'an expression';
}

/**
 Whether a binding or assignment target takes its value apart.

 @param target - left side of a declarator, an assignment or a default, whose
 shape decides whether `Promise` is bound whole or destructured

 @returns Whether it is an object or array pattern, which the finding then
 names as destructuring

 @example
 ```ts
 const destructures = isPattern({ target: declarator.id, },);
 ```
 */
function isPattern({ target, }: { readonly target: unknown; },): boolean {
  return isTreeNode(target,) && ((target.type === 'ObjectPattern') || (target.type === 'ArrayPattern'));
}

/**
 What a read of `Promise` that is neither a type, a name nor a followed call says.

 @param visit - identifier where the walk met it, whose parent decides the form

 @param called - member expressions met as a callee, whose read is a call the scan follows

 @returns Finding's text after its place, empty where the read reaches no
 combinator or is a call the scan follows

 @example
 ```ts
 const text = promiseReadText({ visit, called, },);
 ```
 */
function promiseReadText({ visit, called, }: { readonly visit: Visit; readonly called: ReadonlySet<TreeNode>; },): string {
  /**
   The node holding the read and the field holding it, which decide its form.
   */
  const {
    parent,
    field,
  } = visit;
  /**
   Where the read stands.
   */
  const position = positionOf({
    parent,
    field,
  },);
  if (position === 'NewExpression.callee')
    return '';
  if (position === 'MemberExpression.object') {
    /**
     The member read off `Promise`.
     */
    const key = memberKeyOf({ member: parent, },);
    if (key.kind === 'computed') {
      return `Promise is read by ${keyKindOf({ key: key.key, },)} as its key, which the scan cannot read; name the `
        + 'member plainly or by a string';
    }
    if ((!COMBINATORS.has(key.name,)) || called.has(parent,))
      return '';
    return `Promise.${key.name} is read other than called, which the scan cannot follow; call it where it is named`;
  }
  /**
   Where the value read goes, for a declaration, an assignment or a default.
   */
  const target = (position === 'VariableDeclarator.init') ? parent.id : parent.left;
  /**
   How a declaration, an assignment or a default takes `Promise`.
   */
  const taking = new Map([
    ['VariableDeclarator.init', 'in a declaration',],
    ['AssignmentExpression.right', 'in an assignment',],
    ['AssignmentPattern.right', 'as a parameter\'s default',],
  ],).get(position,);
  if (taking === undefined) {
    return 'Promise is read other than by a named member, new Promise or a type, which the scan cannot follow; call '
      + 'the combinator on Promise where it is named';
  }
  return isPattern({ target, },)
    ? `Promise is destructured ${taking}, which the scan cannot follow; call the combinator on Promise where it is `
      + 'named'
    : 'Promise is bound to another name, through which the scan cannot follow a combinator; call the combinator on '
      + 'Promise where it is named';
}

/**
 What a read of the global object that is neither a type nor a name says.

 @param visit - identifier where the walk met it, whose parent decides the form

 @param name - global object's name as written, which the finding repeats

 @returns Finding's text after its place, empty for a member named plainly
 or by a string, which the read of that member is judged by

 @example
 ```ts
 const text = globalReadText({ visit, name: 'globalThis', },);
 ```
 */
function globalReadText({ visit, name, }: { readonly visit: Visit; readonly name: string; },): string {
  if ((positionOf({
    parent: visit.parent,
    field: visit.field,
  },) === 'MemberExpression.object') && (memberKeyOf({ member: visit.parent, },).kind === 'named'))
    return '';
  return `${name} is read other than by a named member, which can reach Promise where the scan cannot follow it; name `
    + 'the member plainly';
}

/**
 The finding an identifier makes where it reads `Promise` or the global
 object in a way that could reach a combinator the scan cannot follow.

 @param file - file read, whose path the finding's place names

 @param visit - node where the walk met it, whose parent and field decide
 whether it reads anything at all

 @param called - member expressions met as a callee, so a followed call's
 read of `Promise` is no finding

 @returns Finding, or none

 @example
 ```ts
 findings.push(...unfollowableAt({ file, visit, called, },),);
 ```
 */
function unfollowableAt(
  {
    file,
    visit,
    called,
  }: {
    readonly file: SourceText;
    readonly visit: Visit;
    readonly called: ReadonlySet<TreeNode>;
  },
): readonly PlacedFinding[] {
  /**
   Name the node reads, if it is an identifier.
   */
  const name = identifierName({ node: visit.node, },);
  if ((name !== 'Promise') && (!GLOBAL_OBJECTS.has(name,)))
    return [];
  /**
   Where the identifier stands.
   */
  const position = positionOf({
    parent: visit.parent,
    field: visit.field,
  },);
  if (TYPE_POSITIONS.has(position,) || (NAME_POSITIONS.has(position,) && (visit.parent.computed !== true)))
    return [];
  /**
   What the read says, empty where it reaches nothing the scan must follow.
   */
  const text = (name === 'Promise')
    ? promiseReadText({
      visit,
      called,
    },)
    : globalReadText({
      visit,
      name,
    },);
  return (text === '')
    ? []
    : [{
      at: visit.node.start,
      text: `${placeOf({
        file,
        node: visit.node,
      },)}: ${text}`,
    },];
}

/**
 Whether a member expression reads `Promise` itself off its object, by a
 member named `Promise` or `constructor`, which a read of its own reports.

 @param node - object a member is read off, which may itself be the read a finding already names

 @returns Whether it is such a read, so the member read off it is no second finding

 @example
 ```ts
 const reached = readsPromiseItself({ node: object, },); // true for p.constructor
 ```
 */
function readsPromiseItself({ node, }: { readonly node: unknown; },): boolean {
  if ((!isTreeNode(node,)) || (node.type !== 'MemberExpression'))
    return false;
  /**
   The member the object reads.
   */
  const key = memberKeyOf({ member: node, },);
  return (key.kind === 'named') && ((key.name === 'Promise') || (key.name === 'constructor'));
}

/**
 What a member read by name says where its name can reach a combinator the
 scan cannot follow.

 @param member - member expression, whose name and object decide the form

 @returns Finding's text after its place, empty for a member that reaches
 none or whose object's read is the finding

 @example
 ```ts
 const text = memberReadText({ member: node, },); // 'all is read off P, ...' for P.all
 ```
 */
function memberReadText({ member, }: { readonly member: TreeNode; },): string {
  /**
   The member read.
   */
  const key = memberKeyOf({ member, },);
  if (key.kind === 'computed')
    return '';
  /**
   What it is read off, without wrappers.
   */
  const { inner: object, } = unwrapped({ node: member.object, },);
  /**
   The object's name where it is an identifier, empty for any other expression.
   */
  const objectName = identifierName({ node: object, },);
  /**
   The object in words.
   */
  const off = (objectName === '') ? 'an expression' : objectName;
  if (key.name === 'Promise') {
    return `Promise is read off ${off}, through which the scan cannot follow a combinator; call the combinator on `
      + 'Promise where it is named';
  }
  if (key.name === 'constructor') {
    return 'constructor is read, which reaches Promise from any promise where the scan cannot follow it; call the '
      + 'combinator on Promise where it is named, or read what is wanted another way';
  }
  if ((!COMBINATORS.has(key.name,)) || (objectName === 'Promise')
    || COMBINATOR_NAMES_ELSEWHERE.has(`${objectName}.${key.name}`,) || readsPromiseItself({ node: object, },))
    return '';
  return `${key.name} is read off ${off}, which may be Promise reached where the scan cannot follow it; call the `
    + 'combinator on Promise where it is named';
}

/**
 The finding a member expression makes where its name can reach a combinator
 the scan cannot follow.

 @param file - file read, whose path the finding's place names

 @param visit - node where the walk met it, read only where it is a member expression

 @returns Finding, placed at the member's name, or none

 @example
 ```ts
 findings.push(...memberReadAt({ file, visit, },),);
 ```
 */
function memberReadAt({ file, visit, }: { readonly file: SourceText; readonly visit: Visit; },): readonly PlacedFinding[] {
  /**
   The node, read where it is a member expression.
   */
  const { node, } = visit;
  if ((node.type !== 'MemberExpression') || (!isTreeNode(node.property,)))
    return [];
  /**
   What the read says.
   */
  const text = memberReadText({ member: node, },);
  return (text === '')
    ? []
    : [{
      at: node.property.start,
      text: `${placeOf({
        file,
        node: node.property,
      },)}: ${text}`,
    },];
}

/**
 The findings an object pattern makes where it takes a member by a name that
 can reach a combinator the scan cannot follow.

 @param file - file read, whose path each finding's place names

 @param visit - node where the walk met it, read only where it is an object
 pattern, whose parent holds the value it takes apart

 @returns One finding per such name, placed at its key; none where the value
 is `Promise` or the global object, whose own read is the finding

 @example
 ```ts
 findings.push(...destructuredAt({ file, visit, },),);
 ```
 */
function destructuredAt({ file, visit, }: { readonly file: SourceText; readonly visit: Visit; },): readonly PlacedFinding[] {
  /**
   The pattern, the node holding it and the field holding it.
   */
  const {
    node,
    parent,
    field,
  } = visit;
  if (node.type !== 'ObjectPattern')
    return [];
  /**
   Field of the parent holding the value taken apart, empty where the pattern
   is a parameter or nested, whose value no node holds.
   */
  const from = DESTRUCTURED_FROM.get(positionOf({
    parent,
    field,
  },),) ?? '';
  /**
   That value's name where it is an identifier.
   */
  const fromName = (from === '') ? '' : identifierName({ node: unwrapped({ node: parent[from], },).inner, },);
  if ((fromName === 'Promise') || GLOBAL_OBJECTS.has(fromName,))
    return [];
  return treeNodesIn(node.properties,).flatMap(function reachingName(property,): readonly PlacedFinding[] {
    /**
     Name the property takes.
     */
    const name = keyName({
      key: property.key,
      computed: property.computed === true,
    },);
    if ((name !== 'Promise') && (name !== 'constructor') && (!COMBINATORS.has(name,)))
      return [];
    return [{
      at: property.start,
      text: `${placeOf({
        file,
        node: property,
      },)}: ${name} is destructured off a value, which the scan cannot follow; call the combinator on Promise where `
        + 'it is named',
    },];
  },);
}

/**
 Reads one file's calls of a combinator, each keyed by the function it sits
 in, and the reads of `Promise` and calls the scan cannot follow or key.

 @param file - package source, parsed so a mention in a comment or a string is no read

 @returns Keyed calls, which the list is checked against, and the findings
 no list can excuse

 @example
 ```ts
 const { calls, findings, } = combinatorCallsIn({ file, },);
 ```
 */
function combinatorCallsIn({ file, }: { readonly file: SourceText; },): FileReading {
  /**
   The file's program.
   */
  const { program, } = parseSource({ file, },);
  /**
   Each node's parent, which a catch and an anchor are read through.
   */
  const parents = parentsOf({ program, },);
  /**
   Calls of a combinator, with the function each sits in.
   */
  const found: {
    readonly call: TreeNode;
    readonly combinator: string;
    readonly holder: TreeNode;
  }[] = [];
  /**
   Reads the scan cannot follow, so far.
   */
  const findings: PlacedFinding[] = [];
  /**
   Member expressions met as a call's callee, which are calls, not reads.
   */
  const called = new Set<TreeNode>();
  /**
   Each function's name, empty for one that has none.
   */
  const names = new Map<TreeNode, string>();
  /**
   Nodes still to visit; a parent is always visited before its children, so a
   call marks its callee and a function is named before anything inside it.
   */
  const pending: Visit[] = [{
    node: program,
    parent: program,
    field: '',
    holder: program,
    className: NAMELESS_CLASS,
  },];
  for (let visit = pending.pop(); visit !== undefined; visit = pending.pop()) {
    /**
     The node visited.
     */
    const { node, } = visit;
    if (FUNCTION_KINDS.has(node.type,)) {
      names.set(
        node,
        functionNameOf({ visit, },),
      );
    }
    if (node.type === 'CallExpression') {
      /**
       Combinator the call makes, if it makes one.
       */
      const combinator = combinatorCalled({
        call: node,
        called,
      },);
      if (combinator !== '') {
        found.push({
          call: node,
          combinator,
          holder: visit.holder,
        },);
      }
    }
    findings.push(
      ...unfollowableAt({
        file,
        visit,
        called,
      },),
      ...memberReadAt({
        file,
        visit,
      },),
      ...destructuredAt({
        file,
        visit,
      },),
    );
    pending.push(...childVisits({ visit, },),);
  }
  /**
   How many functions of the file carry each name.
   */
  const sharing = new Map<string, number>();
  for (const name of names.values()) {
    sharing.set(
      name,
      (sharing.get(name,) ?? 0) + 1,
    );
  }
  /**
   Calls keyed by their function, in source order.
   */
  const calls: FoundCall[] = [];
  for (const { call, combinator, holder, } of found.toSorted(function bySource(left, right,): number {
    return left.call.start - right.call.start;
  },)) {
    /**
     Where the call starts.
     */
    const place = placeOf({
      file,
      node: call,
    },);
    /**
     Name of the function the call sits in; every function is named when the
     walk meets it, before the calls inside it.
     */
    const name = (holder === program) ? MODULE_SITE : nonNullishOrThrow(names.get(holder,),);
    /**
     Functions of the file carrying that name; one at least, the holder's own.
     */
    const carriers = (holder === program) ? 1 : nonNullishOrThrow(sharing.get(name,),);
    if (name === '') {
      findings.push({
        at: call.start,
        text: `${place}: Promise.${combinator} sits in a function with no name to key it by; name the function, or `
          + 'bind it to a name',
      },);
    }
    else if (carriers > 1) {
      findings.push({
        at: call.start,
        text: `${place}: Promise.${combinator} sits in ${file.path}#${name}, and ${String(carriers,)} ${
          wordForCount({
            count: carriers,
            one: 'function',
            many: 'functions',
          },)
        } in ${file.path} are named ${name}, so no listed site can tell them apart; give each its own name`,
      },);
    }
    else {
      calls.push({
        site: `${file.path}#${name}`,
        combinator,
        place,
        call,
        holder,
        parents,
      },);
    }
  }
  return {
    calls,
    findings,
  };
}

/**
 Whether a read of a caught error hands it to a named reader: as a direct
 argument of the reader's call, or as the value of a property of an object
 that is one.

 @param node - identifier reading the error inside a catch clause

 @param parents - each node's parent in the node's file, through which the
 call it is handed to is found

 @returns Whether a named reader is what takes it

 @example
 ```ts
 const read = handedToReader({ node, parents, },); // true for refusalText({ error, })
 ```
 */
function handedToReader(
  {
    node,
    parents,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): boolean {
  /**
   The node holding the read; every node in a catch clause has one.
   */
  const parent = nonNullishOrThrow(parents.get(node,),);
  /**
   What stands among a call's arguments: the read itself, or the object
   literal whose property's value it is.
   */
  const argument = ((parent.type === 'Property') && (parent.value === node))
    ? nonNullishOrThrow(parents.get(parent,),)
    : node;
  /**
   The node holding that argument.
   */
  const call = (argument === node) ? parent : nonNullishOrThrow(parents.get(argument,),);
  return (call.type === 'CallExpression') && treeNodesIn(call.arguments,).includes(argument,)
    && CATCH_READERS.has(calleeNameOf({ call, },),);
}

/**
 Whether a catch clause keeps its error to itself: it throws nothing, calls no
 function named as the package names a helper that throws, binds its error to
 a name rather than taking it apart, and reads that name only to hand it to a
 named reader.

 @param clause - catch clause inside a listed call, each of which a reason
 resting on the call's catches must hold of

 @param parents - each node's parent in the clause's file, through which a
 read of the error is placed

 @returns Whether the error stops there, so the member around it fulfils
 whatever its own call raised

 @throws {@link Error} where the clause holds no body, which the parser never builds

 @example
 ```ts
 const quiet = isQuietCatch({ clause, parents, },);
 ```
 */
function isQuietCatch({ clause, parents, }: { readonly clause: TreeNode; readonly parents: ReadonlyMap<TreeNode, TreeNode>; },): boolean {
  if (!isTreeNode(clause.body,))
    throw new Error(`unreachable: the catch clause at offset ${String(clause.start,)} holds no body`,);
  if (isPattern({ target: clause.param, },))
    return false;
  /**
   Name the clause binds its error to, empty where it binds none.
   */
  const binding = identifierName({ node: clause.param, },);
  return !nodesUnder({ root: clause.body, },).some(function lets(node,): boolean {
    if (node.type === 'ThrowStatement')
      return true;
    if (node.type === 'CallExpression') {
      /**
       Name of the function called.
       */
      const callee = calleeNameOf({ call: node, },);
      return THROWING_PREFIXES.some(function names(prefix,): boolean {
        return callee.startsWith(prefix,);
      },);
    }
    if ((binding === '') || (identifierName({ node, },) !== binding))
      return false;
    /**
     The node holding the identifier, which says whether it reads the error.
     */
    const parent = nonNullishOrThrow(parents.get(node,),);
    if (NAME_POSITIONS.has(positionOf({
      parent,
      field: fieldOf({
        node,
        parent,
      },),
    },),) && (parent.computed !== true))
      return false;
    return !handedToReader({
      node,
      parents,
    },);
  },);
}

/**
 Whether every catch inside a call keeps its error to itself, with at least one there.

 @param call - listed call whose reason rests on its catches, read whole so a
 second `try` beside the one the entry was read against counts too

 @returns Whether the call holds a catch and each one is quiet

 @example
 ```ts
 const holds = holdsQuietCatches({ call, },);
 ```
 */
function holdsQuietCatches({ call, }: { readonly call: FoundCall; },): boolean {
  /**
   Every catch clause inside the call.
   */
  const clauses = nodesUnder({ root: call.call, },).filter(function isClause(node,): boolean {
    return node.type === 'CatchClause';
  },);
  return (clauses.length > 0) && clauses.every(function isQuiet(clause,): boolean {
    return isQuietCatch({
      clause,
      parents: call.parents,
    },);
  },);
}

/**
 Names a statement declares, where it is a variable declaration.

 @param statement - statement or loop head read, whose declared names link it
 to the code that reads them

 @returns Every identifier in its declarators' targets, none for any other statement

 @example
 ```ts
 const declared = declaredNames({ statement, },); // ['sourceRead', 'archiveRead'] for const [sourceRead, archiveRead] = ...
 ```
 */
function declaredNames({ statement, }: { readonly statement: TreeNode; },): readonly string[] {
  /**
   The declaration, under an `export` where one stands.
   */
  const declaration = (statement.type === 'ExportNamedDeclaration') ? statement.declaration : statement;
  if ((!isTreeNode(declaration,)) || (declaration.type !== 'VariableDeclaration'))
    return [];
  return treeNodesIn(declaration.declarations,).flatMap(function targetNames(declarator,): readonly string[] {
    return treeNodesIn(declarator.id,).flatMap(function namesIn(target,): readonly string[] {
      return nodesUnder({ root: target, },)
        .map(function nameOf(node,): string {
          return identifierName({ node, },);
        },)
        .filter(function named(name,): boolean {
          return name !== '';
        },);
    },);
  },);
}

/**
 Names a statement reads, every identifier in it.

 @param statement - statement read, whose read names link it to the statements declaring them

 @returns Names, each once

 @example
 ```ts
 const read = readNames({ statement, },);
 ```
 */
function readNames({ statement, }: { readonly statement: TreeNode; },): ReadonlySet<string> {
  return new Set(nodesUnder({ root: statement, },).map(function nameOf(node,): string {
    return identifierName({ node, },);
  },),);
}

/**
 Whether a node is a statement or a declaration, the unit an anchor is held to.

 @param node - node met on the way out from a call, the first statement among which is the call's own

 @returns Whether it is one; a block is not, since it only groups the
 statements an anchor is held to

 @example
 ```ts
 const statement = isStatement({ node, },);
 ```
 */
function isStatement({ node, }: { readonly node: TreeNode; },): boolean {
  if (node.type === 'BlockStatement')
    return false;
  return node.type.endsWith('Statement',) || node.type.endsWith('Declaration',);
}

/**
 The statements beside a node in the list of statements its parent holds it in.

 @param node - node met on the way out from a call, whose earlier siblings may feed it

 @param parent - node holding it, which may hold it in a statement list

 @returns Whether the parent holds it in a statement list, which a lone
 statement there tells apart from a node held otherwise; the statements before
 it, nearest first; and those after it, in order

 @example
 ```ts
 const { listed, before, after, } = siblingsOf({ node, parent, },);
 ```
 */
function siblingsOf({ node, parent, }: { readonly node: TreeNode; readonly parent: TreeNode; },): {
  readonly listed: boolean;
  readonly before: readonly TreeNode[];
  readonly after: readonly TreeNode[];
} {
  for (const [field, value,] of Object.entries(parent,)) {
    /**
     The nodes the field holds.
     */
    const list = treeNodesIn(value,);
    if (STATEMENT_LISTS.has(positionOf({
      parent,
      field,
    },),) && list.includes(node,)) {
      /**
       Where the node stands among them, which the check has just found.
       */
      const at = list.indexOf(node,);
      return {
        listed: true,
        before: list
          .slice(
          0,
          at,
          )
          .toReversed(),
        after: list.slice(at + 1,),
      };
    }
  }
  return {
    listed: false,
    before: [],
    after: [],
  };
}

/**
 The head of the loop whose body is a node, as code that can feed the body.

 @param node - node met on the way out from a call, which may be a loop's body fed by its head

 @param parent - node holding it, which may be a loop holding it as its body

 @returns Head as one part, or none where the parent is no loop or holds
 the node outside its body

 @example
 ```ts
 earlier.push(...loopHeadOf({ node, parent, },),);
 ```
 */
function loopHeadOf({ node, parent, }: { readonly node: TreeNode; readonly parent: TreeNode; },): readonly ReachPart[] {
  /**
   Fields the loop's head is made of.
   */
  const fields = LOOP_HEADS.get(parent.type,);
  if ((fields === undefined) || (parent.body !== node))
    return [];
  /**
   The head's nodes.
   */
  const nodes = fields.flatMap(function fieldNodes(field,): readonly TreeNode[] {
    return treeNodesIn(parent[field],);
  },);
  return [{
    declares: nodes.flatMap(function declared(head,): readonly string[] {
      return declaredNames({ statement: head, },);
    },),
    nodes,
  },];
}

/**
 The nodes on the way out from a call to the function holding it.

 @param call - listed call, whose holder ends the walk

 @returns Call and each node holding it, innermost first, the holder left out

 @throws {@link Error} where the walk passes the program without meeting the
 holder, which the walk that keyed the call met around it

 @example
 ```ts
 const path = pathOutOf({ call, },);
 ```
 */
function pathOutOf({ call, }: { readonly call: FoundCall; },): readonly TreeNode[] {
  /**
   Nodes passed so far.
   */
  const path: TreeNode[] = [];
  for (let node = call.call; node !== call.holder;) {
    path.push(node,);
    /**
     The node holding this one.
     */
    const parent = call.parents.get(node,);
    if (parent === undefined) {
      throw new Error(`unreachable: the walk out from ${call.place} passed the program without meeting the function `
        + 'holding the call, though the walk that keyed the call met that function around it',);
    }
    node = parent;
  }
  return path;
}

/**
 What an anchor of a listed call is looked for in, gathered on the way out
 from the call to the function holding it.

 @param call - listed call, whose statement and surroundings an anchor is held to

 @returns Call's own statement, code before it that could feed it,
 statements after it in its block, and handlers of the tries around it

 @example
 ```ts
 const { own, earlier, later, handlers, } = surroundingsOf({ call, },);
 ```
 */
function surroundingsOf({ call, }: { readonly call: FoundCall; },): CallSurroundings {
  /**
   The call and each node holding it, innermost first.
   */
  const path = pathOutOf({ call, },);
  /**
   The node holding each node of the path, in the same order, the function last.
   */
  const holders = [
    ...path.slice(1,),
    call.holder,
  ];
  /**
   The innermost statement making the call; where none does, the outermost
   node of the path, an arrow's expression body.
   */
  const own = path.find(function statement(node,): boolean {
    return isStatement({ node, },);
  },) ?? nonNullishOrThrow(path.at(-1,),);
  /**
   Code before the call that could feed it, nearest first.
   */
  const earlier: ReachPart[] = [];
  /**
   For each statement list at or around the call's own statement, innermost
   first, the statements after the node it holds there.
   */
  const afterOwn: (readonly TreeNode[])[] = [];
  /**
   Handlers of the tries whose block holds the call.
   */
  const handlers: TreeNode[] = [];
  /**
   Where the call's own statement stands on the path.
   */
  const ownAt = path.indexOf(own,);
  for (const [index, node,] of path.entries()) {
    /**
     The node holding this one, read from the list built alongside the path.
     */
    const parent = nonNullishOrThrow(holders[index],);
    /**
     The statements beside it, where it stands in a statement list.
     */
    const {
      listed,
      before,
      after,
    } = siblingsOf({
      node,
      parent,
    },);
    earlier.push(
      ...before.map(function asPart(statement,): ReachPart {
        return {
          declares: declaredNames({ statement, },),
          nodes: [statement,],
        };
      },),
      ...loopHeadOf({
        node,
        parent,
      },),
    );
    if (listed && (index >= ownAt))
      afterOwn.push(after,);
    if ((parent.type === 'TryStatement') && (parent.block === node) && isTreeNode(parent.handler,))
      handlers.push(parent.handler,);
  }
  return {
    own,
    earlier,
    // The first list at or around the call's own statement holds the
    // statements after it; for a declaration under an `export` that is the
    // list holding the `export`. None where no list holds it, as for an
    // arrow's expression body.
    later: afterOwn[0] ?? [],
    handlers,
  };
}

/**
 The code a name an entry rests on is looked for in: the innermost statement
 making the call, the code before it that feeds it a variable, directly or
 through other such code, and, where the reason rests on how settled results
 are read, the later statements of its block reading a variable the call's
 statement declares.

 @param call - listed call, whose own statement a name must stand in or be fed from

 @param reason - its listed reason, which decides whether readers of the result count

 @returns Those statements and loop heads' nodes

 @example
 ```ts
 const reach = reachOf({ call, reason: 'cannot-reject', },);
 ```
 */
function reachOf({ call, reason, }: { readonly call: FoundCall; readonly reason: Reason; },): readonly TreeNode[] {
  /**
   The call's statement, and what surrounds it.
   */
  const {
    own,
    earlier,
    later,
  } = surroundingsOf({ call, },);
  /**
   Names the code found so far reads, which earlier code feeds where it
   declares one.
   */
  const needed = new Set(readNames({ statement: own, },),);
  /**
   Earlier code feeding the call's statement, nearest first.
   */
  const feeding: TreeNode[] = [];
  for (const part of earlier) {
    if (part.declares.some(function isNeeded(name,): boolean {
      return needed.has(name,);
    },)) {
      feeding.push(...part.nodes,);
      for (const node of part.nodes) {
        for (const name of readNames({ statement: node, },))
          needed.add(name,);
      }
    }
  }
  /**
   Names the call's statement declares, which a reader of the result reads.
   */
  const results = declaredNames({ statement: own, },);
  /**
   Later statements reading the call's result, where the reason rests on how they read it.
   */
  const reading = (reason === 'settled-in-input-order')
    ? later.filter(function readsResult(candidate,): boolean {
      return results.some(function isRead(name,): boolean {
        return readNames({ statement: candidate, },).has(name,);
      },);
    },)
    : [];
  return [
    own,
    ...feeding,
    ...reading,
  ];
}

/**
 Whether any of some nodes holds an identifier of a name.

 @param roots - code an anchor is looked for in, as its kind of anchor allows

 @param name - name the entry rests on, which must still stand there for its reason to hold

 @returns Whether it stands anywhere under them

 @example
 ```ts
 const named = holdsName({ roots: handlers, name: 'throwIfAborted', },);
 ```
 */
function holdsName({ roots, name, }: { readonly roots: readonly TreeNode[]; readonly name: string; },): boolean {
  return roots.some(function names(root,): boolean {
    return nodesUnder({ root, },).some(function isName(node,): boolean {
      return identifierName({ node, },) === name;
    },);
  },);
}

/**
 Whether a listed call still holds what its entry rests on.

 @param anchor - what the entry rests on, whose kind decides where it must stand

 @param reason - entry's reason, which decides where a name may stand

 @param call - call the entry excuses, never another of its site

 @returns Whether the anchor holds of that call

 @example
 ```ts
 const holds = holdsAnchor({ anchor: 'namesIn', reason: 'cannot-reject', call, },);
 ```
 */
function holdsAnchor(
  {
    anchor,
    reason,
    call,
  }: {
    readonly anchor: Anchor;
    readonly reason: Reason;
    readonly call: FoundCall;
  },
): boolean {
  if (anchor === CATCH_ANCHOR)
    return holdsQuietCatches({ call, },);
  if ((typeof anchor) !== 'string') {
    return holdsName({
      roots: surroundingsOf({ call, },).handlers,
      name: anchor.inHandler,
    },);
  }
  return holdsName({
    roots: reachOf({
      call,
      reason,
    },),
    name: anchor,
  },);
}

/**
 What an entry rests on, in the words a finding uses.

 @param anchor - what the entry rests on, whose kind says where it must stand

 @returns Words, saying where a name must stand where that is no call's statement

 @example
 ```ts
 const resting = anchorWords({ anchor, },); // 'throwIfAborted in the handler of a try around the call'
 ```
 */
function anchorWords({ anchor, }: { readonly anchor: Anchor; },): string {
  if (anchor === CATCH_ANCHOR)
    return 'catches inside the call that neither throw nor hand their error on';
  return ((typeof anchor) === 'string') ? anchor : `${anchor.inHandler} in the handler of a try around the call`;
}

/**
 Key a call and the entries excusing it share.

 @param site - file and function, as an entry names its site and a call is keyed

 @param combinator - combinator called, so one function's calls of two combinators are counted apart

 @returns `path#function: Promise.combinator`

 @example
 ```ts
 const key = keyOf({ site: 'cat.ts#nap', combinator: 'all', },);
 ```
 */
function keyOf({ site, combinator, }: { readonly site: string; readonly combinator: string; },): string {
  return `${site}: Promise.${combinator}`;
}

/**
 The finding of a site whose function holds a different number of calls than
 the list has entries for it.

 @param entries - site's entries, at least one, which name its site and combinator

 @param calls - site's calls, each named by its place, since no entry can
 be paired with the one it was read against

 @returns Finding

 @example
 ```ts
 const text = countMismatch({ entries, calls, },);
 ```
 */
function countMismatch(
  {
    entries,
    calls,
  }: {
    readonly entries: readonly ListedSite[];
    readonly calls: readonly FoundCall[];
  },
): string {
  /**
   The entry naming the site and the combinator, as every entry of the key does.
   */
  const first = nonNullishOrThrow(entries[0],);
  /**
   Where each call stands, where the site holds any.
   */
  const places = (calls.length === 0)
    ? ''
    : `, at ${calls
      .map(function placeOfCall({ place, },): string {
        return place;
      },)
      .join(', ',)}`;
  return `${first.site}: the list excuses ${String(entries.length,)} Promise.${first.combinator} ${
    wordForCount({
      count: entries.length,
      one: 'call',
      many: 'calls',
    },)
  } and the function holds ${String(calls.length,)}${places}, so no entry can be paired with the call it was read `
    + 'against; list the site\'s calls one entry each, in the order they stand';
}

/**
 Findings of a whole package: each read of `Promise` the scan cannot follow,
 each call it cannot key or the list does not excuse, each listed site whose
 function holds more or fewer calls than listed, and each entry whose call no
 longer holds what its reason rests on.

 @param files - files read; tests and fixtures are skipped

 @param listed - calls the scan excuses, the entries of one site in the order its calls stand

 @returns Findings: those of each file in input order and source order, then
 those of the list in list order

 @example
 ```ts
 const found = unexcusedCombinators({ files, listed: LISTED_CALLS, },);
 ```
 */
function unexcusedCombinators(
  {
    files,
    listed,
  }: {
    readonly files: readonly SourceText[];
    readonly listed: readonly ListedSite[];
  },
): readonly string[] {
  /**
   Entries of each site and combinator, in list order.
   */
  const entriesByKey = new Map<string, readonly ListedSite[]>();
  for (const entry of listed) {
    /**
     Key of this entry.
     */
    const key = keyOf(entry,);
    entriesByKey.set(
      key,
      [...(entriesByKey.get(key,) ?? []), entry,],
    );
  }
  /**
   Calls found, by key, read again when the list is checked.
   */
  const callsByKey = new Map<string, readonly FoundCall[]>();
  /**
   Findings of the files.
   */
  const ofFiles: string[] = [];
  for (const file of files.filter(function isSource(candidate,): boolean {
    return !candidate.isTest;
  },)) {
    /**
     The file's keyed calls and findings.
     */
    const reading = combinatorCallsIn({ file, },);
    /**
     This file's findings, each with the offset that orders it.
     */
    const ofFile = [...reading.findings,];
    for (const call of reading.calls) {
      /**
       Key of this call.
       */
      const key = keyOf(call,);
      callsByKey.set(
        key,
        [...(callsByKey.get(key,) ?? []), call,],
      );
      if (!entriesByKey.has(key,)) {
        ofFile.push({
          at: call.call.start,
          text: `${call.place}: Promise.${call.combinator} in ${call.site} is neither the helper's own call nor a `
            + 'listed site outside the family; use allInInputOrder, or list the site with the reason it is outside',
        },);
      }
    }
    ofFiles.push(...ofFile
      .toSorted(function bySource(left, right,): number {
        return left.at - right.at;
      },)
      .map(function textOf({ text, },): string {
        return text;
      },),);
  }
  /**
   Findings of the list: a site holding more or fewer calls than listed, once
   per key, and an anchor the call an entry excuses no longer holds.
   */
  const ofList = [...entriesByKey.entries(),].flatMap(function staleOf([key, entries,],): readonly string[] {
    /**
     Calls of this key, in source order; none where the site holds none.
     */
    const calls = callsByKey.get(key,) ?? [];
    if (calls.length !== entries.length) {
      return [countMismatch({
        entries,
        calls,
      },),];
    }
    return entries.flatMap(function staleEntry(entry, index,): readonly string[] {
      /**
       The call this entry excuses, the one standing where the entry does
       among its site's.
       */
      const call = nonNullishOrThrow(calls[index],);
      return entry.restsOn
        .filter(function isGone(anchor,): boolean {
          return !holdsAnchor({
            anchor,
            reason: entry.reason,
            call,
          },);
        },)
        .map(function staleReason(anchor,): string {
          return `${entry.site}: Promise.${entry.combinator} is listed as ${entry.reason}, resting on ${
            anchorWords({ anchor, },)
          }, which the call at ${call.place} no longer holds`;
        },);
    },);
  },);
  return [
    ...ofFiles,
    ...ofList,
  ];
}

//endregion Combinator calls

//region The list
// The calls the scan excuses: the helper's own, counted, and the sites
// outside the family. Each note was read against the code it excuses, never
// copied from an earlier report.

/**
 Calls the scan excuses, entries of one site in the order its calls stand.
 */
const LISTED_CALLS: readonly ListedSite[] = [
  {
    site: 'document-structure/all-in-input-order.ts#allInInputOrder',
    combinator: 'all',
    reason: 'cannot-reject',
    restsOn: [],
    note: 'the helper\'s own call, counted and resting on nothing the scan reads: the loop over each member\'s '
      + 'outcome has thrown the first failure in input order before it, so every member has fulfilled, and it '
      + 'gives the result the tuple type Promise.all gives',
  },
  {
    site: 'reference/cited-reference-lookup.ts#citedReferenceBlock',
    combinator: 'all',
    reason: 'catches-own',
    restsOn: ['failedPageOutcome',],
    note: 'outcomeFor holds the lookup and the line it builds in a try whose catch answers a named outcome through '
      + 'failedPageOutcome, the caller\'s reason as a stop and any other failure as an unfetched page, so no member '
      + 'rejects; the block writes every page\'s line in order of citation and then throws the first stop\'s reason',
  },
  {
    site: 'probe/corpus-run/attribution-read.ts#gatherAttributionEntries',
    combinator: 'all',
    reason: 'catches-own',
    restsOn: [CATCH_ANCHOR,],
    note: 'readOne holds the read and the parse in a try whose catch answers a malformed artifact named by refusalText',
  },
  {
    site: 'slice-cache/corpus-run/cache-account-slice-report.ts#reportSliceCaches',
    combinator: 'all',
    reason: 'cannot-reject',
    restsOn: ['runsDirsUnder',],
    note: 'each member is runsDirsUnder, whose walk lists every directory through namesIn, which answers an '
      + 'unlistable directory as data; the rest only sorts',
  },
  {
    site: 'slice-cache/corpus-run/cache-account-slices.ts#runsDirsBelow',
    combinator: 'all',
    reason: 'cannot-reject',
    restsOn: ['runsDirsBelow', 'namesIn',],
    note: 'each member recurses into runsDirsBelow, whose one read is namesIn, which answers an unlistable directory '
      + 'as data, and the names the members are made from come from that same read',
  },
  {
    site: 'bench/corpus-run/editor-standing-read-run.ts#reportStandings',
    combinator: 'all',
    reason: 'cannot-reject',
    restsOn: ['standingArtifactPaths',],
    note: 'standingArtifactPaths lists through artifactFilesIn, which answers an unlistable directory as data, then '
      + 'only joins names; the procedure prints what each listing says once all are listed',
  },
  {
    site: 'bench/corpus-run/editor-standing-read-run.ts#reportStandings',
    combinator: 'all',
    reason: 'catches-own',
    restsOn: ['readStandingArtifact',],
    note: 'readStandingArtifact holds the read, the parse and the round check in a try whose catch returns a named '
      + 'outcome with the line it says and rethrows nothing',
  },
  {
    site: 'run/corpus-run/ledger-directory.ts#readLedgerDirectory',
    combinator: 'all',
    reason: 'catches-own',
    restsOn: [CATCH_ANCHOR,],
    note: 'each file is read and parsed in a try whose catch answers a refused outcome through refusalOf',
  },
  {
    site: 'page-check/corpus-run/page-republish.ts#republishSettledPages',
    combinator: 'all',
    reason: 'catches-own',
    restsOn: ['republishOne',],
    note: 'republishOne holds the whole judgement of an entry in a try whose catch answers a left outcome named by '
      + 'errorName',
  },
  {
    site: 'run/corpus-run/runs-lock.ts#lockRunsDir',
    combinator: 'all',
    reason: 'catches-own',
    restsOn: ['hostIdentity', 'startTicksOf',],
    note: 'hostIdentity and startTicksOf each hold their reads in a try whose catch logs and answers an unread kind',
  },
  {
    site: 'stage/overlapped-map.ts#mapOverlapped',
    combinator: 'all',
    reason: 'catches-own',
    restsOn: ['drain',],
    note: 'every lane runs drain, whose catch records a job failure by position, and mapOverlapped throws the lowest '
      + 'position\'s failure once every lane has ended, having written each other failure, stopped or not',
  },
  {
    site: 'provider/provider-meters.ts#readEveryMeter',
    combinator: 'all',
    reason: 'catches-own',
    restsOn: ['meterRecordOf',],
    note: 'each configured meter is read through meterRecordOf, whose catch logs the failure and answers an '
      + 'unreadable state; an unconfigured one is a resolved constant',
  },
  {
    site: 'probe/fidelity-reference-read.ts#readReference',
    combinator: 'allSettled',
    reason: 'settled-in-input-order',
    restsOn: ['sourceRead', 'archiveRead',],
    note: 'the original\'s settlement is checked for a rejection before the archive page\'s, after the caller\'s '
      + 'own abort is checked',
  },
  {
    site: 'slice-reading/image-reading-pair.ts#readImagePair',
    combinator: 'allSettled',
    reason: 'settled-in-input-order',
    restsOn: ['containFailedReaders',],
    note: 'containFailedReaders reads the settlements by position and keeps a failed reader\'s failure as its '
      + 'reading, but raises a refusal that ends the run; an abort is raised from the caller\'s signal',
  },
  {
    site: 'stage/stage-round.ts#runGatherRound',
    combinator: 'allSettled',
    reason: 'cannot-reject',
    restsOn: ['settleWithin',],
    note: 'Promise.allSettled never rejects, and nothing reads its settlements: settleWithin only waits on it for '
      + 'the grace, and the outcomes come from arrived by position',
  },
  {
    site: 'stage/stage-round.ts#nextSettled',
    combinator: 'race',
    reason: 'shared-abort',
    restsOn: [{ inHandler: 'throwIfAborted', },],
    note: 'an ask rejects only once the caller\'s signal has aborted, each with the failure its own call caught, so '
      + 'the asks share no rejection value but share that signal; the catch around the race throws the signal\'s '
      + 'reason through throwIfAborted in place of whichever ask ended first, each ask having logged a failure of '
      + 'its own call as it ended',
  },
  {
    site: 'stage/stage-recovery-round.ts#runRecoveryRound',
    combinator: 'all',
    reason: 'shared-abort',
    restsOn: ['runGatherRound',],
    note: 'each group is one runGatherRound needing no voice, which waits on no ask before its grace and then rejects '
      + 'only through throwIfAborted, with the caller\'s reason every group shares',
  },
  {
    site: 'reference/work-title-lookup.ts#workTitleLookupLines',
    combinator: 'all',
    reason: 'catches-own',
    restsOn: ['failedTitleOutcome',],
    note: 'outcomeFor holds the lookup in a try whose catch answers a named outcome through failedTitleOutcome, the '
      + 'caller\'s reason as a stop and any other failure as a warning, so no member rejects; the lookups write every '
      + 'failure in title order and then throw the first stop\'s reason',
  },
];

//endregion The list

/**
 The helper's own call as a fixture lists it.
 */
const FIXTURE_HELPER: ListedSite = {
  site: 'document-structure/all-in-input-order.ts#allInInputOrder',
  combinator: 'all',
  reason: 'cannot-reject',
  restsOn: [],
  note: 'the helper\'s own call',
};

/**
 The helper as a fixture plants it, making its one call.
 */
const FIXTURE_HELPER_FILE: SourceText = {
  path: 'document-structure/all-in-input-order.ts',
  text: 'export async function allInInputOrder(members) { return await Promise.all(members,); }',
  isTest: false,
};

/**
 A race whose abort check moved into the try block, before the race, where a
 rejection no longer reaches it: the shape a fixture plants under two entries,
 one resting on the name and one on the name in the handler.
 */
const CHECK_MOVED_BEFORE_THE_RACE = [
  'export async function nextSettled({ pending, signal, }) {',
  '  try {',
  '    signal.throwIfAborted();',
  '    return await Promise.race(pending,);',
  '  }',
  '  catch (error) {',
  '    throw error;',
  '  }',
  '}',
].join('\n',);

await describe({
  name: 'Promise combinators in production code',
  children: [
    it({
      name: 'FINDS each combinator called outside the helper and one called by a string key, and passes the helper\'s '
        + 'own call, Promise read as a type, by a member that is no combinator or by new Promise, a mention in a '
        + 'comment or a string, and a test file',
      fn: async () => {
        expect(unexcusedCombinators({
          files: [
            {
              path: 'cat.ts',
              text: [
                'export async function nap() { return await Promise.all([a, b,],); }',
                'export async function purr() { return await Promise.race([a, b,],); }',
                'export async function knead() { return await Promise.any([a, b,],); }',
                'export async function groom() { return await Promise.allSettled([a, b,],); }',
                'export async function pounce() { return await Promise[\'all\']([a,],); }',
                'export async function stretch(): Promise<number> { return await Promise.resolve(a,); }',
                'export function wake(): Promise<void> { return new Promise(function settle(resolve) { resolve(); },); }',
                'export const lap = Promise[\'reject\'];',
                'export type Nap = typeof Promise.all;',
                '// Promise.all([a, b,],) in a comment',
                'export const hiss = \'Promise.race([a,],)\';',
              ].join('\n',),
              isTest: false,
            },
            FIXTURE_HELPER_FILE,
            {
              path: 'ginger.unit.test.ts',
              text: 'export async function run() { const P = Promise; return await P.all([a,],); }',
              isTest: true,
            },
          ],
          listed: [FIXTURE_HELPER,],
        },),).toEqual([
          'cat.ts:1: Promise.all in cat.ts#nap is neither the helper\'s own call nor a listed site outside the family; '
            + 'use allInInputOrder, or list the site with the reason it is outside',
          'cat.ts:2: Promise.race in cat.ts#purr is neither the helper\'s own call nor a listed site outside the '
            + 'family; use allInInputOrder, or list the site with the reason it is outside',
          'cat.ts:3: Promise.any in cat.ts#knead is neither the helper\'s own call nor a listed site outside the '
            + 'family; use allInInputOrder, or list the site with the reason it is outside',
          'cat.ts:4: Promise.allSettled in cat.ts#groom is neither the helper\'s own call nor a listed site outside '
            + 'the family; use allInInputOrder, or list the site with the reason it is outside',
          'cat.ts:5: Promise.all in cat.ts#pounce is neither the helper\'s own call nor a listed site outside the '
            + 'family; use allInInputOrder, or list the site with the reason it is outside',
        ],);
      },
    },),
    it({
      name: 'FINDS EVERY WAY TO REACH A COMBINATOR THAT IS NO PLAIN NAMED CALL: one read uncalled, Promise bound to '
        + 'another name, read off globalThis by a name or a string, globalThis or global itself read, Promise '
        + 'destructured in a declaration, an assignment or a parameter\'s default, read by a variable, a template or '
        + 'an expression as its key, handed on whole, cast, or handed to apply or bind, and a combinator in a '
        + 'sequence or a tagged template, and keys a call through each wrapper a callee can wear (optional, '
        + 'non-null, parentheses)',
      fn: async () => {
        expect(unexcusedCombinators({
          files: [{
            path: 'tom.ts',
            text: [
              'export const settle = Promise.race;',
              'export function chain() { return Promise.all.call(undefined, [a,],); }',
              'const P = Promise;',
              'let Q; Q = Promise;',
              'export const g = globalThis.Promise.all([a,],);',
              'const G = globalThis;',
              'const { all, } = Promise;',
              'const { [\'race\']: settleRace, } = Promise;',
              'let any; ({ any, } = Promise);',
              'export function nap({ allSettled, } = Promise) { return allSettled; }',
              'const key = \'all\'; export const k = Promise[key]([a,],);',
              'export const t = Promise[`all`]([a,],);',
              'export const c = Promise[\'al\' + \'l\']([a,],);',
              'export const r = Reflect.get(Promise, \'all\',);',
              'const H = global;',
              'export const s = globalThis[\'Promise\'].all([a,],);',
              'export const o = Promise.all?.([a,],);',
              'export const q = Promise?.all([a,],);',
              'export const n = Promise.all!([a,],);',
              'export const w = (Promise.all)([a,],);',
              'export const v = (0, Promise.all)([a,],);',
              'export const x = (Promise as PromiseConstructor).all([a,],);',
              'export const y = Promise.all.apply(Promise, [[a,],],);',
              'export const z = Promise.all.bind(Promise,)([a,],);',
              'export const u = Promise.all`a`;',
            ].join('\n',),
            isTest: false,
          },],
          listed: [],
        },),).toEqual([
          'tom.ts:1: Promise.race is read other than called, which the scan cannot follow; call it where it is named',
          'tom.ts:2: Promise.all is read other than called, which the scan cannot follow; call it where it is named',
          'tom.ts:3: Promise is bound to another name, through which the scan cannot follow a combinator; call the '
            + 'combinator on Promise where it is named',
          'tom.ts:4: Promise is bound to another name, through which the scan cannot follow a combinator; call the '
            + 'combinator on Promise where it is named',
          'tom.ts:5: Promise is read off globalThis, through which the scan cannot follow a combinator; call the '
            + 'combinator on Promise where it is named',
          'tom.ts:6: globalThis is read other than by a named member, which can reach Promise where the scan cannot '
            + 'follow it; name the member plainly',
          'tom.ts:7: Promise is destructured in a declaration, which the scan cannot follow; call the combinator on '
            + 'Promise where it is named',
          'tom.ts:8: Promise is destructured in a declaration, which the scan cannot follow; call the combinator on '
            + 'Promise where it is named',
          'tom.ts:9: Promise is destructured in an assignment, which the scan cannot follow; call the combinator on '
            + 'Promise where it is named',
          'tom.ts:10: Promise is destructured as a parameter\'s default, which the scan cannot follow; call the '
            + 'combinator on Promise where it is named',
          'tom.ts:11: Promise is read by a variable as its key, which the scan cannot read; name the member plainly '
            + 'or by a string',
          'tom.ts:12: Promise is read by a template as its key, which the scan cannot read; name the member plainly '
            + 'or by a string',
          'tom.ts:13: Promise is read by an expression as its key, which the scan cannot read; name the member '
            + 'plainly or by a string',
          'tom.ts:14: Promise is read other than by a named member, new Promise or a type, which the scan cannot '
            + 'follow; call the combinator on Promise where it is named',
          'tom.ts:15: global is read other than by a named member, which can reach Promise where the scan cannot '
            + 'follow it; name the member plainly',
          'tom.ts:16: Promise is read off globalThis, through which the scan cannot follow a combinator; call the '
            + 'combinator on Promise where it is named',
          'tom.ts:17: Promise.all in tom.ts#<module> is neither the helper\'s own call nor a listed site outside the '
            + 'family; use allInInputOrder, or list the site with the reason it is outside',
          'tom.ts:18: Promise.all in tom.ts#<module> is neither the helper\'s own call nor a listed site outside the '
            + 'family; use allInInputOrder, or list the site with the reason it is outside',
          'tom.ts:19: Promise.all in tom.ts#<module> is neither the helper\'s own call nor a listed site outside the '
            + 'family; use allInInputOrder, or list the site with the reason it is outside',
          'tom.ts:20: Promise.all in tom.ts#<module> is neither the helper\'s own call nor a listed site outside the '
            + 'family; use allInInputOrder, or list the site with the reason it is outside',
          'tom.ts:21: Promise.all is read other than called, which the scan cannot follow; call it where it is named',
          'tom.ts:22: Promise is read other than by a named member, new Promise or a type, which the scan cannot '
            + 'follow; call the combinator on Promise where it is named',
          'tom.ts:23: Promise.all is read other than called, which the scan cannot follow; call it where it is named',
          'tom.ts:23: Promise is read other than by a named member, new Promise or a type, which the scan cannot '
            + 'follow; call the combinator on Promise where it is named',
          'tom.ts:24: Promise.all is read other than called, which the scan cannot follow; call it where it is named',
          'tom.ts:24: Promise is read other than by a named member, new Promise or a type, which the scan cannot '
            + 'follow; call the combinator on Promise where it is named',
          'tom.ts:25: Promise.all is read other than called, which the scan cannot follow; call it where it is named',
        ],);
      },
    },),
    it({
      name: 'FINDS PROMISE REACHED WITHOUT ITS NAME BEFORE THE COMBINATOR: a member named constructor read off '
        + 'Promise.prototype, a promise, a call\'s result or by a string, a member named Promise read off a chain of '
        + 'the global object\'s members or off a namespace import, a combinator\'s name read off anything but '
        + 'Promise, and each of those names destructured off a value, and passes AbortSignal.any, those names as '
        + 'an object literal\'s keys and a class\'s constructor',
      fn: async () => {
        expect(unexcusedCombinators({
          files: [{
            path: 'manul.ts',
            text: [
              'import * as ns from \'./again.ts\';',
              'import { P, } from \'./again.ts\';',
              'export const a = Promise.prototype.constructor.all([b,],);',
              'export const c = Promise.resolve().constructor.all([b,],);',
              'export async function d(p) { return await p.constructor.all([b,],); }',
              'export const e = (async function nap() { return 1; })().constructor.all([b,],);',
              'export const f = globalThis.globalThis.Promise.all([b,],);',
              'export const g = global.global.Promise.all([b,],);',
              'export const h = globalThis.global.Promise.all([b,],);',
              'export const i = ns.Promise.all([b,],);',
              'export const j = P.all([b,],);',
              'export const k = p[\'constructor\'];',
              'const { constructor: Kind, } = Promise.resolve();',
              'const { Promise: Again, } = ns;',
              'const { race, } = P;',
              'export const s = AbortSignal.any([t,],);',
              'export const toy = { all: 1, race: 2, constructor: 3, Promise: 4, };',
              'export class Cat { constructor() { this.naps = 1; } }',
            ].join('\n',),
            isTest: false,
          },],
          listed: [],
        },),).toEqual([
          'manul.ts:3: constructor is read, which reaches Promise from any promise where the scan cannot follow it; '
            + 'call the combinator on Promise where it is named, or read what is wanted another way',
          'manul.ts:4: constructor is read, which reaches Promise from any promise where the scan cannot follow it; '
            + 'call the combinator on Promise where it is named, or read what is wanted another way',
          'manul.ts:5: constructor is read, which reaches Promise from any promise where the scan cannot follow it; '
            + 'call the combinator on Promise where it is named, or read what is wanted another way',
          'manul.ts:6: constructor is read, which reaches Promise from any promise where the scan cannot follow it; '
            + 'call the combinator on Promise where it is named, or read what is wanted another way',
          'manul.ts:7: Promise is read off an expression, through which the scan cannot follow a combinator; call '
            + 'the combinator on Promise where it is named',
          'manul.ts:8: Promise is read off an expression, through which the scan cannot follow a combinator; call '
            + 'the combinator on Promise where it is named',
          'manul.ts:9: Promise is read off an expression, through which the scan cannot follow a combinator; call '
            + 'the combinator on Promise where it is named',
          'manul.ts:10: Promise is read off ns, through which the scan cannot follow a combinator; call the '
            + 'combinator on Promise where it is named',
          'manul.ts:11: all is read off P, which may be Promise reached where the scan cannot follow it; call the '
            + 'combinator on Promise where it is named',
          'manul.ts:12: constructor is read, which reaches Promise from any promise where the scan cannot follow it; '
            + 'call the combinator on Promise where it is named, or read what is wanted another way',
          'manul.ts:13: constructor is destructured off a value, which the scan cannot follow; call the combinator '
            + 'on Promise where it is named',
          'manul.ts:14: Promise is destructured off a value, which the scan cannot follow; call the combinator on '
            + 'Promise where it is named',
          'manul.ts:15: race is destructured off a value, which the scan cannot follow; call the combinator on '
            + 'Promise where it is named',
        ],);
      },
    },),
    it({
      name: 'KEYS A CALL BY ITS FUNCTION\'S BINDING, OR ITS CLASS AND METHOD, and refuses one in a function with no '
        + 'name and one in a function whose name another function of the file shares',
      fn: async () => {
        expect(unexcusedCombinators({
          files: [{
            path: 'manx.ts',
            text: [
              'export const hiss = async function () { return await Promise.all([nap(),],); };',
              'export const pounce = async () => await Promise.race([nap(),],);',
              'export class Cat { async groom() { return await Promise.any([nap(),],); } }',
              'const toy = { async roll() { return await Promise.allSettled([nap(),],); }, };',
              'export function gather(xs) { return xs.map(async (x) => await Promise.all([nap(x,),],),); }',
              'export function one() { return Promise.all([nap(),],); }',
              'export function outer() { function one() { return 1; } return one(); }',
            ].join('\n',),
            isTest: false,
          },],
          listed: [
            {
              site: 'manx.ts#hiss',
              combinator: 'all',
              reason: 'cannot-reject',
              restsOn: ['nap',],
              note: 'a nap cannot fail',
            },
            {
              site: 'manx.ts#pounce',
              combinator: 'race',
              reason: 'cannot-reject',
              restsOn: ['nap',],
              note: 'a nap cannot fail',
            },
            {
              site: 'manx.ts#Cat.groom',
              combinator: 'any',
              reason: 'cannot-reject',
              restsOn: ['nap',],
              note: 'a nap cannot fail',
            },
            {
              site: 'manx.ts#roll',
              combinator: 'allSettled',
              reason: 'cannot-reject',
              restsOn: ['nap',],
              note: 'a nap cannot fail',
            },
          ],
        },),).toEqual([
          'manx.ts:5: Promise.all sits in a function with no name to key it by; name the function, or bind it to a '
            + 'name',
          'manx.ts:6: Promise.all sits in manx.ts#one, and 2 functions in manx.ts are named one, so no listed site '
            + 'can tell them apart; give each its own name',
        ],);
      },
    },),
    it({
      name: 'HOLDS A REASON RESTING ON A CATCH TO EVERY CATCH INSIDE THE CALL, at least one: a catch that throws, '
        + 'rethrows through a helper, or reads its error other than by handing it to a named reader (an alias, a '
        + 'cast, an array, an object inside an argument, a destructured parameter) is no quiet catch, and one quiet '
        + 'catch beside one that throws does not hold',
      fn: async () => {
        expect(unexcusedCombinators({
          files: [
            {
              path: 'calico.ts',
              text: [
                'export async function roll(names) {',
                '  return await Promise.all(names.map(async function one(name) {',
                '    try { return await read(name,); } catch (error) { throw new Error(\'lost\', { cause: error, },); }',
                '  },),);',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'siamese.ts',
              text: [
                'export async function hunt(names) {',
                '  return await Promise.all(names.map(async function one(name) {',
                '    try { return await read(name,); } catch (error) { rethrowUnlessMissingPath({ error, },); return \'\'; }',
                '  },),);',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'burmese.ts',
              text: [
                'export async function doze(names) {',
                '  return await Promise.all(names.map(async function one(name) {',
                '    try { return await read(name,); } catch (error) { return refusalText({ error, },); }',
                '  },),);',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'birman.ts',
              text: [
                'export async function prowl(names) {',
                '  return await Promise.all(names.map(async function one(name) {',
                '    try { return await read(name,); } catch (error) { const lost = error; fail(lost,); return \'\'; }',
                '  },),);',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'chartreux.ts',
              text: [
                'export async function prowl(names) {',
                '  return await Promise.all(names.map(async function one(name) {',
                '    try { return await read(name,); } catch (error) { fail(error as Error,); return \'\'; }',
                '  },),);',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'devon.ts',
              text: [
                'export async function prowl(names) {',
                '  return await Promise.all(names.map(async function one(name) {',
                '    try { return await read(name,); } catch (error) { fail([error,],); return \'\'; }',
                '  },),);',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'havana.ts',
              text: [
                'export async function prowl(names) {',
                '  return await Promise.all(names.map(async function one(name) {',
                '    try { return await read(name,); } catch (error) { fail({ detail: { error, }, },); return \'\'; }',
                '  },),);',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'javanese.ts',
              text: [
                'export async function prowl(names) {',
                '  return await Promise.all(names.map(async function one(name) {',
                '    try { return await read(name,); } catch ({ message, }) { fail(message,); return \'\'; }',
                '  },),);',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'nebelung.ts',
              text: [
                'export async function prowl(names) {',
                '  return await Promise.all(names.map(async function one(name) {',
                '    try { await warm(name,); } catch (error) { return refusalText({ error, },); }',
                '    try { return await read(name,); } catch (error) { throw new Error(\'lost\', { cause: error, },); }',
                '  },),);',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'minskin.ts',
              text: [
                'export async function prowl(names) {',
                '  return await Promise.all(names.map(async function one(name) { return await read(name,); },),);',
                '}',
              ].join('\n',),
              isTest: false,
            },
          ],
          listed: [
            {
              site: 'calico.ts#roll',
              combinator: 'all',
              reason: 'catches-own',
              restsOn: [CATCH_ANCHOR,],
              note: 'each calico caught its own failure until the catch began to rethrow',
            },
            {
              site: 'siamese.ts#hunt',
              combinator: 'all',
              reason: 'catches-own',
              restsOn: [CATCH_ANCHOR,],
              note: 'each siamese caught its own failure until the catch began to rethrow through a helper',
            },
            {
              site: 'burmese.ts#doze',
              combinator: 'all',
              reason: 'catches-own',
              restsOn: [CATCH_ANCHOR,],
              note: 'each burmese answers its own failure as text',
            },
            {
              site: 'birman.ts#prowl',
              combinator: 'all',
              reason: 'catches-own',
              restsOn: [CATCH_ANCHOR,],
              note: 'each birman kept its failure until the catch handed an alias of it on',
            },
            {
              site: 'chartreux.ts#prowl',
              combinator: 'all',
              reason: 'catches-own',
              restsOn: [CATCH_ANCHOR,],
              note: 'each chartreux kept its failure until the catch handed it on cast',
            },
            {
              site: 'devon.ts#prowl',
              combinator: 'all',
              reason: 'catches-own',
              restsOn: [CATCH_ANCHOR,],
              note: 'each devon kept its failure until the catch handed it on in an array',
            },
            {
              site: 'havana.ts#prowl',
              combinator: 'all',
              reason: 'catches-own',
              restsOn: [CATCH_ANCHOR,],
              note: 'each havana kept its failure until the catch handed it on inside an object',
            },
            {
              site: 'javanese.ts#prowl',
              combinator: 'all',
              reason: 'catches-own',
              restsOn: [CATCH_ANCHOR,],
              note: 'each javanese kept its failure until the catch took it apart and handed a part on',
            },
            {
              site: 'nebelung.ts#prowl',
              combinator: 'all',
              reason: 'catches-own',
              restsOn: [CATCH_ANCHOR,],
              note: 'each nebelung caught its own failure until a second try began to rethrow',
            },
            {
              site: 'minskin.ts#prowl',
              combinator: 'all',
              reason: 'catches-own',
              restsOn: [CATCH_ANCHOR,],
              note: 'each minskin caught its own failure until its try went',
            },
          ],
        },),).toEqual([
          'calico.ts#roll: Promise.all is listed as catches-own, resting on catches inside the call that neither '
            + 'throw nor hand their error on, which the call at calico.ts:2 no longer holds',
          'siamese.ts#hunt: Promise.all is listed as catches-own, resting on catches inside the call that neither '
            + 'throw nor hand their error on, which the call at siamese.ts:2 no longer holds',
          'birman.ts#prowl: Promise.all is listed as catches-own, resting on catches inside the call that neither '
            + 'throw nor hand their error on, which the call at birman.ts:2 no longer holds',
          'chartreux.ts#prowl: Promise.all is listed as catches-own, resting on catches inside the call that '
            + 'neither throw nor hand their error on, which the call at chartreux.ts:2 no longer holds',
          'devon.ts#prowl: Promise.all is listed as catches-own, resting on catches inside the call that neither '
            + 'throw nor hand their error on, which the call at devon.ts:2 no longer holds',
          'havana.ts#prowl: Promise.all is listed as catches-own, resting on catches inside the call that neither '
            + 'throw nor hand their error on, which the call at havana.ts:2 no longer holds',
          'javanese.ts#prowl: Promise.all is listed as catches-own, resting on catches inside the call that neither '
            + 'throw nor hand their error on, which the call at javanese.ts:2 no longer holds',
          'nebelung.ts#prowl: Promise.all is listed as catches-own, resting on catches inside the call that neither '
            + 'throw nor hand their error on, which the call at nebelung.ts:2 no longer holds',
          'minskin.ts#prowl: Promise.all is listed as catches-own, resting on catches inside the call that neither '
            + 'throw nor hand their error on, which the call at minskin.ts:2 no longer holds',
        ],);
      },
    },),
    it({
      name: 'HOLDS A NAMED ANCHOR TO THE INNERMOST STATEMENT MAKING THE CALL and the statements feeding it, found '
        + 'among the earlier statements of each enclosing block and in a loop\'s head (and, for settled results, '
        + 'the later statements reading them, an export after an exported call among them), so a name elsewhere in '
        + 'an if, a for, a try or the function does not hold',
      fn: async () => {
        expect(unexcusedCombinators({
          files: [
            {
              path: 'sphynx.ts',
              text: [
                'export async function sort(dirs, baskets) {',
                '  const first = await Promise.all(dirs.map(function listed(dir) { return readdir(dir,); },),);',
                '  const second = await Promise.all(baskets.map(function named(basket) { return namesIn(basket,); },),);',
                '  return [first, second,];',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'sokoke.ts',
              text: [
                'export async function sort(dirs, baskets) {',
                '  if (dirs.length > 0) {',
                '    const first = await Promise.all(dirs.map(function listed(dir) { return readdir(dir,); },),);',
                '    const second = await Promise.all(baskets.map(function named(basket) { return namesIn(basket,); },),);',
                '    return [first, second,];',
                '  }',
                '  return [];',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'chausie.ts',
              text: [
                'export async function stack(groups) {',
                '  for (const group of groups) {',
                '    await Promise.all(group.map(function listed(dir) { return readdir(dir,); },),);',
                '    await namesIn(group,);',
                '  }',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'toyger.ts',
              text: CHECK_MOVED_BEFORE_THE_RACE,
              isTest: false,
            },
            {
              path: 'persian.ts',
              text: [
                'export async function sleep(dir) {',
                '  const reading = await namesIn({ dir, },);',
                '  const { names, } = reading;',
                '  return await Promise.all(names.map(function each(name) { return sleep(name,); },),);',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'lapperm.ts',
              text: [
                'export async function burrow(dir) {',
                '  const reading = await namesIn({ dir, },);',
                '  if (reading.names.length > 0) {',
                '    return await Promise.all(reading.names.map(function each(name) { return burrow(name,); },),);',
                '  }',
                '  return [];',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'ocicat.ts',
              text: [
                'export async function nest(root) {',
                '  for (const dir of await namesIn(root,)) {',
                '    await Promise.all(dir.names.map(function each(name) { return nest(name,); },),);',
                '  }',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'bengal.ts',
              text: [
                'export async function read() {',
                '  const settled = await Promise.allSettled([nap(), purr(),],);',
                '  return settled.map(function contained(result) { return result.status; },);',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'pixiebob.ts',
              text: [
                'export const settled = await Promise.allSettled([nap(), purr(),],);',
                'export const statuses = settled.map(function contained(result) { return result.status; },);',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'ragdoll.ts',
              text: [
                'export async function read() {',
                '  const settled = await Promise.allSettled([nap(), purr(),],);',
                '  return settled.map(function contained(result) { return result.status; },);',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'tabby.ts',
              text: 'export async function doze(dirs) { return await Promise.all(dirs.map(readdir,),); }',
              isTest: false,
            },
          ],
          listed: [
            {
              site: 'sphynx.ts#sort',
              combinator: 'all',
              reason: 'cannot-reject',
              restsOn: ['namesIn',],
              note: 'namesIn answered an unlistable basket as data until readdir took its place in the first call',
            },
            {
              site: 'sphynx.ts#sort',
              combinator: 'all',
              reason: 'cannot-reject',
              restsOn: ['namesIn',],
              note: 'namesIn answers an unlistable basket as data',
            },
            {
              site: 'sokoke.ts#sort',
              combinator: 'all',
              reason: 'cannot-reject',
              restsOn: ['namesIn',],
              note: 'namesIn answered an unlistable basket as data until readdir took its place in the first call',
            },
            {
              site: 'sokoke.ts#sort',
              combinator: 'all',
              reason: 'cannot-reject',
              restsOn: ['namesIn',],
              note: 'namesIn answers an unlistable basket as data',
            },
            {
              site: 'chausie.ts#stack',
              combinator: 'all',
              reason: 'cannot-reject',
              restsOn: ['namesIn',],
              note: 'namesIn answered an unlistable basket as data until readdir took its place in the call',
            },
            {
              site: 'toyger.ts#nextSettled',
              combinator: 'race',
              reason: 'shared-abort',
              restsOn: ['throwIfAborted',],
              note: 'the catch turned a rejection into the reason until throwIfAborted moved before the race',
            },
            {
              site: 'persian.ts#sleep',
              combinator: 'all',
              reason: 'cannot-reject',
              restsOn: ['namesIn',],
              note: 'the names come from namesIn, which answers an unlistable basket as data',
            },
            {
              site: 'lapperm.ts#burrow',
              combinator: 'all',
              reason: 'cannot-reject',
              restsOn: ['namesIn',],
              note: 'the names come from namesIn, read before the block holding the call',
            },
            {
              site: 'ocicat.ts#nest',
              combinator: 'all',
              reason: 'cannot-reject',
              restsOn: ['namesIn',],
              note: 'the loop walks what namesIn answered',
            },
            {
              site: 'bengal.ts#read',
              combinator: 'allSettled',
              reason: 'settled-in-input-order',
              restsOn: ['contained',],
              note: 'contained reads the settlements by position',
            },
            {
              site: 'pixiebob.ts#<module>',
              combinator: 'allSettled',
              reason: 'settled-in-input-order',
              restsOn: ['contained',],
              note: 'contained reads the exported settlements by position, in the export after the call\'s',
            },
            {
              site: 'ragdoll.ts#read',
              combinator: 'allSettled',
              reason: 'catches-own',
              restsOn: ['contained',],
              note: 'contained reads the settlements, after the call and not inside it',
            },
            {
              site: 'tabby.ts#doze',
              combinator: 'all',
              reason: 'cannot-reject',
              restsOn: ['namesIn',],
              note: 'namesIn answered an unlistable basket as data until readdir took its place',
            },
          ],
        },),).toEqual([
          'sphynx.ts#sort: Promise.all is listed as cannot-reject, resting on namesIn, which the call at sphynx.ts:2 '
            + 'no longer holds',
          'sokoke.ts#sort: Promise.all is listed as cannot-reject, resting on namesIn, which the call at sokoke.ts:3 '
            + 'no longer holds',
          'chausie.ts#stack: Promise.all is listed as cannot-reject, resting on namesIn, which the call at '
            + 'chausie.ts:3 no longer holds',
          'toyger.ts#nextSettled: Promise.race is listed as shared-abort, resting on throwIfAborted, which the call '
            + 'at toyger.ts:4 no longer holds',
          'ragdoll.ts#read: Promise.allSettled is listed as catches-own, resting on contained, which the call at '
            + 'ragdoll.ts:2 no longer holds',
          'tabby.ts#doze: Promise.all is listed as cannot-reject, resting on namesIn, which the call at tabby.ts:1 no '
            + 'longer holds',
        ],);
      },
    },),
    it({
      name: 'HOLDS A NAME WRITTEN IN A HANDLER TO THE HANDLER OF A TRY WHOSE BLOCK HOLDS THE CALL, so a race whose '
        + 'catch turns a rejection into the caller\'s reason passes, and the same name moved into the try block, '
        + 'where no rejection reaches it, does not',
      fn: async () => {
        expect(unexcusedCombinators({
          files: [
            {
              path: 'savannah.ts',
              text: [
                'export async function nextSettled({ pending, signal, }) {',
                '  try {',
                '    return await Promise.race(pending,);',
                '  }',
                '  catch (error) {',
                '    signal.throwIfAborted();',
                '    throw error;',
                '  }',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'cheetoh.ts',
              text: CHECK_MOVED_BEFORE_THE_RACE,
              isTest: false,
            },
          ],
          listed: [
            {
              site: 'savannah.ts#nextSettled',
              combinator: 'race',
              reason: 'shared-abort',
              restsOn: [{ inHandler: 'throwIfAborted', },],
              note: 'the catch around the race throws the caller\'s reason in place of any rejection',
            },
            {
              site: 'cheetoh.ts#nextSettled',
              combinator: 'race',
              reason: 'shared-abort',
              restsOn: [{ inHandler: 'throwIfAborted', },],
              note: 'the catch threw the caller\'s reason until throwIfAborted moved before the race',
            },
          ],
        },),).toEqual([
          'cheetoh.ts#nextSettled: Promise.race is listed as shared-abort, resting on throwIfAborted in the handler of '
            + 'a try around the call, which the call at cheetoh.ts:4 no longer holds',
        ],);
      },
    },),
    it({
      name: 'COUNTS EACH SITE\'S CALLS AGAINST ITS ENTRIES and names every call of a site holding more or fewer than '
        + 'listed, the helper\'s own included, rather than checking an entry against a call it was never read '
        + 'against',
      fn: async () => {
        expect(unexcusedCombinators({
          files: [
            {
              path: 'document-structure/all-in-input-order.ts',
              text: [
                'export async function allInInputOrder(members) {',
                '  await Promise.all(members,);',
                '  return await Promise.all(members,);',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'lykoi.ts',
              text: [
                'export async function sort(dirs) {',
                '  const fresh = await Promise.all(dirs.map(async function two(dir) { try { return await peek(dir,); } '
                  + 'catch (error) { return refusalText({ error, },); } },),);',
                '  return await Promise.all(dirs.map(async function one(dir) { try { return await read(dir,); } '
                  + 'catch (error) { return refusalText({ error, },); } },),);',
                '}',
              ].join('\n',),
              isTest: false,
            },
          ],
          listed: [
            FIXTURE_HELPER,
            {
              site: 'lykoi.ts#sort',
              combinator: 'all',
              reason: 'catches-own',
              restsOn: [CATCH_ANCHOR,],
              note: 'read answers each failure as text',
            },
            {
              site: 'korat.ts#hunt',
              combinator: 'race',
              reason: 'shared-abort',
              restsOn: ['throwIfAborted',],
              note: 'the korat file is gone',
            },
          ],
        },),).toEqual([
          'document-structure/all-in-input-order.ts#allInInputOrder: the list excuses 1 Promise.all call and the function holds 2, at '
            + 'document-structure/all-in-input-order.ts:2, document-structure/all-in-input-order.ts:3, so no entry can be paired with the call it was read '
            + 'against; list the site\'s calls one entry each, in the order they stand',
          'lykoi.ts#sort: the list excuses 1 Promise.all call and the function holds 2, at lykoi.ts:2, lykoi.ts:3, so '
            + 'no entry can be paired with the call it was read against; list the site\'s calls one entry each, in the '
            + 'order they stand',
          'korat.ts#hunt: the list excuses 1 Promise.race call and the function holds 0, so no entry can be paired '
            + 'with the call it was read against; list the site\'s calls one entry each, in the order they stand',
        ],);
      },
    },),
    it({
      name: 'FINDS NO COMBINATOR CALLED OR REACHED ACROSS THE PACKAGE\'S SOURCE outside the listed calls, none it '
        + 'cannot follow or key, and no listed site or reason the source no longer holds',
      fn: async () => {
        expectNoFindings({
          findings: unexcusedCombinators({
            files: await readPackageSource(),
            listed: LISTED_CALLS,
          },),
        },);
      },
    },),
  ],
},);
