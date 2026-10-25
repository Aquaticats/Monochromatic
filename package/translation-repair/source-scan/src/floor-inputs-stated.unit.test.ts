/**
 Guards against a default standing in for a floor input its caller holds
 (ledger B29): a production call to a package function whose first parameter
 defaults, or types as optional, one of the inputs that decide what a floor
 refuses, what a sheet shows or whether a text may ship must state it, or be
 named here with the reason it may leave it out. The final polish dropped the slice's disputed
 wordings this way and shipped one; the chunk's checker check dropped the
 refiners; the consolidation writer's sheet dropped its candidates' break
 counts.

 WHAT THE SCAN READS. A call whose first argument is an object literal, a
 conditional between literals, or a `const` in the same file bound to one;
 a spread of any of those counts its keys, a conditional spread
 (`...((x === undefined) ? {} : { x, })`) counts `x` as stated. A spread or
 argument naming a parameter of an enclosing function forwards the caller's
 own argument and is taken as stating everything: the call that built that
 argument is read where it is a literal. Anything else is unreadable and has
 to be named. Out of reach: a key nested inside a named type (a `subject`
 or a settlement `identity`), a parameter typed by a named alias rather
 than a literal, whose optional keys the scan cannot see, and a literal a
 function returns for its caller to spread. A name is read as a forwarded
 parameter before any same-file constant, so a local constant shadowing an
 enclosing function's parameter would be taken as forwarded; none does.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each kind (ledger M21). Fixtures are cat-themed;
 the package case reads this package's own source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  childNodes,
  isTreeNode,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import {
  expectFindingsAsListed,
  expectNoFindings,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Floor inputs

/**
 Parameter names that carry what a floor refuses or what a sheet shows, and
 the gate flags that decide whether a text may ship: most of those default
 to the permissive answer (`eligible = true`, `standingMayShip = true`), so a
 caller leaving one out lets a text through that its own evidence refused.
 */
const FLOOR_KEYS: ReadonlySet<string> = new Set([
  'archiveDispute',
  'archiveDisputeNote',
  'attestedDetails',
  'attestedLines',
  'choiceMayShip',
  'declared',
  'declaredNamePairs',
  'disputedWordings',
  'eligible',
  'identityContext',
  'incumbentEligible',
  'incumbentWithheld',
  'insertionCarried',
  'lineStructured',
  'pageText',
  'pictureContext',
  'referenceContext',
  'refinerModelIds',
  'renderings',
  'runoffOverStanding',
  'selfCertificationPermitted',
  'standingEligible',
  'standingFlawedByMajority',
  'standingMayShip',
  'standingValid',
  'withdrawnSliceIndices',
],);

/**
 Marker a key list carries where a parameter of an enclosing function is
 forwarded whole.
 */
const FORWARDED = '<forwarded>';

/**
 Prefix of the marker a key list carries where the scan cannot read what an
 expression holds.
 */
const OPAQUE = '<opaque:';

/**
 Deepest chain of `const` names the scan follows to a literal.
 */
const MAX_CONST_DEPTH = 4;

/**
 A package call the scan found leaving floor inputs out, as
 `path#site -> callee: omits|unreadable [keys]`.
 */
type Omission = string;

/**
 A function and the floor inputs its first parameter defaults or leaves
 optional.
 */
type FloorKeyed = {
  /**
   File declaring it.
   */
  readonly path: string;

  /**
   Floor inputs it lets a caller leave out.
   */
  readonly keys: readonly string[];
};

/**
 Name an identifier, a string key or a numeric key carries.

 @param node - key or identifier node

 @returns Its name

 @example
 ```ts
 const name = nameOf({ node: property.key, },);
 ```
 */
function nameOf({ node, }: { readonly node: TreeNode; },): string {
  /**
   What the node names itself by, whichever field carries it.
   */
  const { name, value, } = node;
  if ((typeof name) === 'string')
    return name as string;
  if (((typeof value) === 'string') || ((typeof value) === 'number'))
    return String(value,);
  return '';
}

/**
 The nodes a list field holds; none where the field is absent or no list.

 @param value - field of a node

 @returns Its nodes, in order

 @example
 ```ts
 const params = nodesIn({ value: fn.params, },);
 ```
 */
function nodesIn({ value, }: { readonly value: unknown; },): readonly TreeNode[] {
  if (!Array.isArray(value,))
    return [];
  return value.filter(function isNode(item: unknown,): item is TreeNode {
    return isTreeNode(item,);
  },);
}

/**
 The expression inside any parentheses and type assertions.

 @param node - expression as written

 @returns Expression it wraps

 @example
 ```ts
 const inner = unwrapped({ node: argument, },);
 ```
 */
function unwrapped({ node, }: { readonly node: TreeNode; },): TreeNode {
  /**
   Expression under the wrappers read so far.
   */
  let inner = node;
  while (['ParenthesizedExpression', 'TSAsExpression', 'TSSatisfiesExpression', 'TSNonNullExpression',].includes(inner.type,))
    inner = inner.expression as TreeNode;
  return inner;
}

/**
 Floor inputs a function's first parameter defaults, or leaves optional in
 the type literal that annotates it, directly or as the argument of a generic.

 @param fn - function node

 @returns Floor inputs a caller may leave out

 @example
 ```ts
 const keys = floorKeysOf({ fn: declaration, },);
 ```
 */
function floorKeysOf({ fn, }: { readonly fn: TreeNode; },): readonly string[] {
  /**
   First parameter, when there is one.
   */
  const [first,] = nodesIn({ value: fn.params, },);
  if (first === undefined)
    return [];
  /**
   Its pattern, under any default for the whole argument.
   */
  const pattern = (first.type === 'AssignmentPattern') ? (first.left as TreeNode) : first;
  if (pattern.type !== 'ObjectPattern')
    return [];
  /**
   Keys the pattern gives a default.
   */
  const defaulted = (pattern.properties as readonly TreeNode[])
    .filter(function hasDefault(property,): boolean {
      return (property.type === 'Property') && isTreeNode(property.value,) && (property.value.type === 'AssignmentPattern');
    },)
    .map(function keyOf(property,): string {
      return nameOf({ node: property.key as TreeNode, },);
    },);
  /**
   Nodes of the annotation still to search for its type literal.
   */
  const pending: TreeNode[] = isTreeNode(pattern.typeAnnotation,) ? [pattern.typeAnnotation,] : [];
  /**
   Keys the first type literal found marks optional.
   */
  const optional: string[] = [];
  while ((pending.length > 0) && (optional.length === 0)) {
    /**
     Node searched now.
     */
    const node = pending.shift() as TreeNode;
    if (node.type !== 'TSTypeLiteral') {
      pending.push(...childNodes({ node, },),);
      continue;
    }
    optional.push(...(node.members as readonly TreeNode[])
      .filter(function isOptional(member,): boolean {
        return (member.type === 'TSPropertySignature') && (member.optional === true);
      },)
      .map(function keyOf(member,): string {
        return nameOf({ node: member.key as TreeNode, },);
      },),);
    // A literal with no optional member still ends the search: it is the
    // parameter's own type.
    break;
  }
  return [...new Set([...defaulted, ...optional,],),]
    .filter(function isFloorKey(key,): boolean {
      return FLOOR_KEYS.has(key,);
    },)
    .toSorted();
}

/**
 Functions and function constants declared anywhere in a program, with the
 floor inputs each lets a caller leave out, where there are any.

 @param program - parsed program

 @returns Name and floor inputs of each such function

 @example
 ```ts
 const keyed = floorKeyedFunctions({ program, },);
 ```
 */
function floorKeyedFunctions({ program, }: { readonly program: TreeNode; },): readonly (readonly [string, readonly string[]])[] {
  /**
   Functions found so far.
   */
  const found: (readonly [string, readonly string[]])[] = [];
  /**
   Nodes still to visit.
   */
  const pending: TreeNode[] = [program,];
  while (pending.length > 0) {
    /**
     Node visited now.
     */
    const node = pending.pop() as TreeNode;
    pending.push(...childNodes({ node, },),);
    /**
     The function and its name, where the node declares one.
     */
    const declared = ((node.type === 'FunctionDeclaration') && isTreeNode(node.id,))
      ? { name: nameOf({ node: node.id, },), fn: node, }
      : ((node.type === 'VariableDeclarator') && isTreeNode(node.id,) && isTreeNode(node.init,)
        && ((node.init.type === 'ArrowFunctionExpression') || (node.init.type === 'FunctionExpression')))
        ? { name: nameOf({ node: node.id, },), fn: node.init, }
        : undefined;
    if (declared === undefined)
      continue;
    /**
     Floor inputs it lets a caller leave out.
     */
    const keys = floorKeysOf({ fn: declared.fn, },);
    if (keys.length > 0)
      found.push([declared.name, keys,],);
  }
  return found;
}

/**
 Initialisers of the names a program binds once with `const` or `let`; a
 name bound twice resolves to nothing, since the scan cannot tell which
 binding a use reads.

 @param program - parsed program

 @returns Initialiser per name bound once

 @example
 ```ts
 const inits = singleBindings({ program, },);
 ```
 */
function singleBindings({ program, }: { readonly program: TreeNode; },): ReadonlyMap<string, TreeNode> {
  /**
   Initialisers of the names bound so far.
   */
  const inits = new Map<string, TreeNode>();
  /**
   Names bound more than once, or once without an initialiser.
   */
  const unresolved = new Set<string>();
  /**
   Nodes still to visit.
   */
  const pending: TreeNode[] = [program,];
  while (pending.length > 0) {
    /**
     Node visited now.
     */
    const node = pending.pop() as TreeNode;
    pending.push(...childNodes({ node, },),);
    if ((node.type !== 'VariableDeclarator') || (!isTreeNode(node.id,)) || (node.id.type !== 'Identifier'))
      continue;
    /**
     Name bound.
     */
    const name = nameOf({ node: node.id, },);
    if (inits.has(name,) || unresolved.has(name,) || (!isTreeNode(node.init,))) {
      inits.delete(name,);
      unresolved.add(name,);
      continue;
    }
    inits.set(name, node.init,);
  }
  return inits;
}

/**
 Names a function's parameters bind: plain parameters, the properties an
 object pattern destructures, and a rest element.

 @param fn - function node

 @returns Names bound

 @example
 ```ts
 const bound = parameterNames({ fn, },);
 ```
 */
function parameterNames({ fn, }: { readonly fn: TreeNode; },): readonly string[] {
  return nodesIn({ value: fn.params, },).flatMap(function boundBy(param,): readonly string[] {
    /**
     Pattern under any default.
     */
    const pattern = (param.type === 'AssignmentPattern') ? (param.left as TreeNode) : param;
    if (pattern.type === 'Identifier')
      return [nameOf({ node: pattern, },),];
    if (pattern.type !== 'ObjectPattern')
      return [];
    return (pattern.properties as readonly TreeNode[]).flatMap(function propertyBinding(property,): readonly string[] {
      /**
       What the property binds.
       */
      const target = (property.type === 'RestElement') ? property.argument : property.value;
      if (!isTreeNode(target,))
        return [];
      /**
       Binding under any default.
       */
      const binding = (target.type === 'AssignmentPattern') ? (target.left as TreeNode) : target;
      return (binding.type === 'Identifier') ? [nameOf({ node: binding, },),] : [];
    },);
  },);
}

/**
 Keys an argument expression states, with a marker where it forwards a
 parameter whole or holds what the scan cannot read.

 @param expression - argument or spread operand

 @param inits - the file's single bindings

 @param parameters - names the enclosing functions' parameters bind

 @param depth - `const` names followed so far

 @returns Keys stated, and markers

 @example
 ```ts
 const stated = statedKeys({ expression: argument, inits, parameters, depth: 0, },);
 ```
 */
function statedKeys(
  {
    expression,
    inits,
    parameters,
    depth,
  }: {
    readonly expression: TreeNode;
    readonly inits: ReadonlyMap<string, TreeNode>;
    readonly parameters: ReadonlySet<string>;
    readonly depth: number;
  },
): readonly string[] {
  /**
   Expression under its wrappers.
   */
  const inner = unwrapped({ node: expression, },);
  if (inner.type === 'ObjectExpression') {
    return (inner.properties as readonly TreeNode[]).flatMap(function keysOfProperty(property,): readonly string[] {
      return (property.type === 'Property')
        ? [nameOf({ node: property.key as TreeNode, },),]
        : statedKeys({
          expression: property.argument as TreeNode,
          inits,
          parameters,
          depth,
        },);
    },);
  }
  if (inner.type === 'ConditionalExpression') {
    return [inner.consequent, inner.alternate,]
      .filter(function isNode(branch: unknown,): branch is TreeNode {
        return isTreeNode(branch,);
      },)
      .flatMap(function keysOfBranch(branch,): readonly string[] {
        return statedKeys({
          expression: branch,
          inits,
          parameters,
          depth,
        },);
      },);
  }
  if (inner.type === 'Identifier') {
    /**
     Name the expression reads.
     */
    const name = nameOf({ node: inner, },);
    if (parameters.has(name,))
      return [FORWARDED,];
    /**
     Literal the name is bound to, where it is bound once.
     */
    const init = inits.get(name,);
    if ((init !== undefined) && (depth < MAX_CONST_DEPTH)) {
      return statedKeys({
        expression: init,
        inits,
        parameters,
        depth: depth + 1,
      },);
    }
    return [`${OPAQUE}${name}>`,];
  }
  return [`${OPAQUE}${inner.type}>`,];
}

/**
 Every production call to a floor-keyed function that leaves a floor input
 out, or whose argument the scan cannot read, in each file given.

 @param files - files read; tests and fixtures are skipped

 @returns Omissions, sorted; a name declared with floor inputs in two places
 is reported as ambiguous, since calls to it cannot be told apart

 @example
 ```ts
 const omissions = floorInputOmissions({ files, },);
 ```
 */
function floorInputOmissions({ files, }: { readonly files: readonly SourceText[]; },): readonly Omission[] {
  /**
   Package source parsed once.
   */
  const parsed = files
    .filter(function isSource(file,): boolean {
      return !file.isTest;
    },)
    .map(function parse(file,) {
      return {
        path: file.path,
        program: parseSource({ file, },).program,
      };
    },);
  /**
   Floor-keyed functions by name, with every place one is declared.
   */
  const keyed = new Map<string, FloorKeyed[]>();
  for (const { path, program, } of parsed) {
    for (const [name, keys,] of floorKeyedFunctions({ program, },))
      keyed.set(name, [...(keyed.get(name,) ?? []), { path, keys, },],);
  }
  /**
   Findings so far.
   */
  const found: Omission[] = [...keyed,]
    .filter(function declaredTwice([, where,],): boolean {
      return where.length > 1;
    },)
    .map(function ambiguous([name, where,],): Omission {
      return `${name}: ambiguous, declared in ${where.map(function pathOf(entry,): string {
        return entry.path;
      },)
        .join(' | ',)}`;
    },);
  for (const { path, program, } of parsed) {
    /**
     The file's single bindings.
     */
    const inits = singleBindings({ program, },);
    /**
     Nodes still to visit, each with its enclosing function's name and the
     parameter names in scope.
     */
    const pending: {
      readonly node: TreeNode;
      readonly site: string;
      readonly parameters: ReadonlySet<string>;
    }[] = [{ node: program, site: '<module>', parameters: new Set(), },];
    while (pending.length > 0) {
      /**
       Node visited now, with its context.
       */
      const { node, site, parameters, } = pending.pop() as (typeof pending)[number];
      /**
       Whether the node opens a function scope.
       */
      const opensFunction = ['FunctionDeclaration', 'FunctionExpression', 'ArrowFunctionExpression',].includes(node.type,);
      /**
       Enclosing function's name for the node's children.
       */
      const here = (opensFunction && isTreeNode(node.id,)) ? nameOf({ node: node.id, },) : site;
      /**
       Parameter names in scope for the node's children.
       */
      const inScope = opensFunction
        ? new Set([...parameters, ...parameterNames({ fn: node, },),],)
        : parameters;
      pending.push(...childNodes({ node, },).map(function withContext(child,) {
        return {
          node: child,
          site: here,
          parameters: inScope,
        };
      },),);
      if ((node.type !== 'CallExpression') || (!isTreeNode(node.callee,)) || (node.callee.type !== 'Identifier'))
        continue;
      /**
       Function called.
       */
      const callee = nameOf({ node: node.callee, },);
      /**
       Its floor inputs, where it has some and one declaration.
       */
      const declaration = keyed.get(callee,);
      if ((declaration === undefined) || (declaration.length !== 1))
        continue;
      /**
       First argument, when there is one.
       */
      const [argument,] = node.arguments as readonly TreeNode[];
      /**
       Keys the call states.
       */
      const stated = (argument === undefined)
        ? []
        : statedKeys({
          expression: argument,
          inits,
          parameters,
          depth: 0,
        },);
      if (stated.includes(FORWARDED,))
        continue;
      /**
       Floor inputs the call leaves out.
       */
      const missing = (declaration[0] as FloorKeyed).keys.filter(function unstated(key,): boolean {
        return !stated.includes(key,);
      },);
      if (missing.length === 0)
        continue;
      /**
       What the scan could not read, if anything.
       */
      const opaque = stated.filter(function isOpaque(key,): boolean {
        return key.startsWith(OPAQUE,);
      },);
      found.push(`${path}#${site} -> ${callee}: ${(opaque.length === 0) ? 'omits' : `unreadable ${opaque.join(' ',)}`} [${
        missing.join(', ',)
      }]`,);
    }
  }
  return found.toSorted();
}

//endregion Floor inputs

//region Named omissions

/**
 Measurement files: quota-spending bins `rolldown.node.config.ts` builds, or
 helpers only such bins import (the package barrels export them for their
 tests). Each measures a stage on cases it draws itself, without the pass's
 page context, and its recorded readings were bought that way; any omission
 in one is theirs.
 */
const MEASUREMENT_FILES: Readonly<Record<string, string>> = {
  'bench/corpus-run/checker-sensitivity-run.ts': 'asks whether the resolution checkers can say no, on cat-themed sheets',
  'coverage-census/corpus-run/coverage-control.ts': 'asks the coverage roster about a passage before and after its anchors are cut',
  'bench/corpus-run/editor-calibrate-lane.ts': 'ranks every model on the editor\'s job over drawn slices',
  'probe/corpus-run/editor-fidelity-probe-read.ts': 'sizes the sheet a seeded editor trial sends, which carries no declared names or cited references',
  'probe/corpus-run/editor-fidelity-trial.ts': 'seats one editor card alone over a seeded defect, a slice that carries no declared names or cited references',
  'bench/corpus-run/editor-width-arm.ts': 'one arm of the editor-width probe',
  'bench/corpus-run/editor-width-contest.ts': 'the editor-width probe\'s head-to-head between its arms\' winners',
  'bench/corpus-run/editor-width-input.ts': 'the critic and panel findings the editor-width probe\'s arms share',
  'probe/corpus-run/probe-relabel-case.ts': 'locates a relabelled probe case in its drawn entry',
  'probe/corpus-run/probe-relabel-control.ts': 'gathers the relabel probe\'s control cases',
  'bench/corpus-run/producer-calibrate-round.ts': 'ranks producers on drawn bare slices, every seat filled',
  'bench/corpus-run/roster-bench-round.ts': 'benches roster widths on drawn bare slices',
  'bench/corpus-run/translate-probe-run.ts': 'prints the translate sheet for a drawn slice',
  'bench/corpus-run/window-trial-probe-draw.ts': 'draws the entries the judging-window trial runs on',
  'bench/corpus-run/window-trial-slice.ts': 'runs one slice of the judging-window trial in both arms',
};

/**
 A pipeline call that leaves a floor input out on purpose.
 */
type NamedOmission = {
  /**
   The call as the scan reports it.
   */
  readonly omission: Omission;

  /**
   Why leaving the input out changes nothing the call decides.
   */
  readonly why: string;
};

/**
 Pipeline calls that leave a floor input out on purpose, each with the
 reason, as the scan reports them.
 */
const NAMED_OMISSIONS: readonly NamedOmission[] = [
  {
    omission: 'archive/archive-dispute.ts#archiveDisputeNotesOf -> archiveDisputesOf: omits [withdrawnSliceIndices]',
    why: 'the note states the accepted claims alone; the withdrawn slices decide only whether the repair text may stand '
      + 'in, which no note reads',
  },
  {
    omission: 'lane-contest/lane-contest-eligibility.ts#candidateEligibility -> validateTranslatedSlice: omits [declared, disputedWordings, '
      + 'lineStructured]',
    why: 'the front-matter branch reads neither the line structure nor the declared names, and no dispute exists at a '
      + 'front-matter slice (front-matter-slice.unit.test.ts, "RAISES NO ISSUE")',
  },
  {
    omission: 'lane-contest/lane-contest-eligibility.ts#excluded -> validateTranslatedSlice: omits [declared, disputedWordings, lineStructured]',
    why: 'the front-matter branch reads neither the line structure nor the declared names, and no dispute exists at a '
      + 'front-matter slice (front-matter-slice.unit.test.ts, "RAISES NO ISSUE")',
  },
  {
    omission: 'lane-contest/lane-contest-eligibility.ts#laneContestChoiceVerdict -> validateTranslatedSlice: omits [disputedWordings]',
    why: 'the verdict decides only whether the contest ballots persist and what the log says; the consolidation re-reads '
      + 'every contested slice\'s standing against the disputed wordings, endorsed or not (consolidate-driver.ts, '
      + 'readStandingVerdict)',
  },
  {
    omission: 'repair/repair-entry.ts#repairTranslation -> prepareDocumentPair: omits [attestedDetails, referenceContext]',
    why: 'the library entry takes no cited references or attestations; the corpus pass fetches and attests them '
      + '(corpus-run/pass-prepare.ts)',
  },
  {
    omission: 'probe/editor-fidelity-context.ts#editorSheetContextOf -> prepareDocumentPair: omits [attestedDetails, referenceContext]',
    why: 'the editor fidelity probe slices the pair only to read the sections either side of a seeded passage and the '
      + 'declared names; cited references and attestations need keyed web lookups the probe does not make, and its '
      + 'report states that limit (editor-fidelity-context.ts header)',
  },
  {
    omission: 'lane-contest/seed-detection.ts#gradeSeedDetection -> prepareDocumentPair: omits [attestedDetails, referenceContext]',
    why: 'grading a seeded-error benchmark reads the pair and the seeds alone',
  },
  {
    omission: 'artifact/corpus-run/artifact-two-lane-rebuild.ts#rebuildPreparation -> prepareDocumentPair: omits [attestedDetails, '
      + 'referenceContext]',
    why: 'the rebuild re-carves the slices an artifact recorded, which neither input moves, and artifacts record neither',
  },
  {
    omission: 'repair/refine-slice-settle.ts#settleRefinedSlice -> runRefineStage: omits [archiveDisputeNote]',
    why: 'the repair lane\'s own refinement passes neither the dispute note nor the accepted claims, which only the '
      + 'final polish is given, as its gate is (ledger B805); its sheet is what it was '
      + '(refine-prompt.unit.test.ts, "LEAVES THE SHEET UNCHANGED WHEN THE SLICE IS NOT DISPUTED")',
  },
  {
    omission: 'translate-lane/translate-slice.ts#settleTranslateSlice -> runTranslateStage: unreadable <opaque:stageInput> [archiveDisputeNote, '
      + 'attestedLines, declared, disputedWordings, identityContext, pictureContext, referenceContext]',
    why: 'stageInput is translateSliceInput\'s stage input, whose literal states each of them (translate-slice-input.ts); '
      + 'this scan cannot read a returned literal, so translate-slice-input.unit.test.ts and the stage-threading tests '
      + 'pin each input (b29-stage-input-mutants.json caught all seven)',
  },
];

//endregion Named omissions

/**
 Whether an omission sits in a measurement file.

 @param omission - omission as reported

 @returns Whether its file is a measurement file

 @example
 ```ts
 const measured = inMeasurementFile({ omission, },);
 ```
 */
function inMeasurementFile({ omission, }: { readonly omission: Omission; },): boolean {
  return Object.hasOwn(MEASUREMENT_FILES, omission.slice(0, omission.indexOf('#',),),);
}

await describe({
  name: 'floor inputs stated (ledger B29)',
  children: [
    it({
      name: 'FINDS a defaulted and an optional floor input left out, and an argument it cannot read, and leaves a key '
        + 'stated plainly, through a conditional spread or a local constant, a forwarded parameter, tests, and keys '
        + 'that are not floor inputs',
      fn: async () => {
        expect(floorInputOmissions({
          files: [
            {
              path: 'bowl.ts',
              text: [
                'export function fill({ kibble, declared = [], lineStructured, pictureContext, }: '
                  + '{ readonly kibble: string; readonly declared?: readonly string[]; readonly lineStructured: boolean; '
                  + 'readonly pictureContext?: string; readonly treats?: number; },): string { return kibble; }',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'cat.ts',
              text: [
                'import { fill, } from \'./bowl.ts\';',
                'export function breakfast(): string { return fill({ kibble: \'fish\', lineStructured: true, },); }',
                'export function lunch({ pictureContext, }: { readonly pictureContext?: string; },): string {',
                '  const fragment = { declared: [], };',
                '  return fill({ kibble: \'chicken\', lineStructured: false, ...fragment, '
                  + '...((pictureContext === undefined) ? {} : { pictureContext, }), },);',
                '}',
                'export function dinner(meal: Parameters<typeof fill>[0],): string { return fill(meal,); }',
                'export function supper(): string { return fill(makeMeal(),); }',
                'function makeMeal(): Parameters<typeof fill>[0] { return { kibble: \'tuna\', lineStructured: true, }; }',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'cat.unit.test.ts',
              text: 'import { fill, } from \'./bowl.ts\';\nfill({ kibble: \'beef\', lineStructured: true, },);',
              isTest: true,
            },
          ],
        },),).toEqual([
          'cat.ts#breakfast -> fill: omits [declared, pictureContext]',
          'cat.ts#supper -> fill: unreadable <opaque:CallExpression> [declared, pictureContext]',
        ],);
      },
    },),
    it({
      name: 'FINDS a name declared with floor inputs in two files, whose calls it cannot tell apart',
      fn: async () => {
        expect(floorInputOmissions({
          files: [
            {
              path: 'cat.ts',
              text: 'export function groom({ declared = [], }: { readonly declared?: readonly string[]; },): number { return declared.length; }',
              isTest: false,
            },
            {
              path: 'kitten.ts',
              text: 'export function groom({ pageText = \'\', }: { readonly pageText?: string; },): number { return pageText.length; }',
              isTest: false,
            },
          ],
        },),).toEqual(['groom: ambiguous, declared in cat.ts | kitten.ts',],);
      },
    },),
    it({
      name: 'FINDS NO FLOOR INPUT LEFT OUT across the package but the named ones and the measurement files\'',
      fn: async () => {
        /**
         Every package file, tests among them to be skipped.
         */
        const files = await readPackageSource();
        /**
         Every omission the scan finds.
         */
        const found = floorInputOmissions({ files, },);
        expectFindingsAsListed({
          findings: found.filter(function outsideMeasurement(omission,): boolean {
            return !inMeasurementFile({ omission, },);
          },),
          listed: NAMED_OMISSIONS
            .map(function omissionOf(named,): Omission {
              return named.omission;
            },)
            .toSorted(),
        },);
      },
    },),
    it({
      name: 'NAMES ONLY MEASUREMENT FILES THAT LEAVE AN INPUT OUT, so a file that no longer does keeps no exemption',
      fn: async () => {
        /**
         Every omission the scan finds.
         */
        const found = floorInputOmissions({ files: await readPackageSource(), },);
        expectNoFindings({
          findings: Object.keys(MEASUREMENT_FILES,).filter(function stale(path,): boolean {
            return !found.some(function inFile(omission,): boolean {
              return omission.startsWith(`${path}#`,);
            },);
          },),
        },);
      },
    },),
  ],
},);
