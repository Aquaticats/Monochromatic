/**
 Guards against a caught value turned into text whole, in production source.

 THE FAMILY. A caught value's message can carry what must not travel: a
 runtime `fetch` rejection for an unsendable header quotes the header value
 (an API key), a `JSON.parse` refusal quotes the opening of the text it
 refused (a model's reply, a provider's body), and an HTTP status failure
 carries an excerpt of the provider's response body on purpose. A log line, a
 finding or a stored record that holds such a message whole has copied it
 somewhere nothing bounds. `refusalText` prints a message only for a class
 that declares `messageNamesOnly` and names the class otherwise, and
 `exchangeFailureText` adds the HTTP status to that.

 WHAT IT FLAGS. An identifier named like a caught value (`error`, `caught`,
 `reason`, `cause`, `failure`, `thrown`, or the binding of an enclosing
 catch clause) that is: the argument of `String`, `caughtValueText`,
 `caughtValueStack`, `JSON.stringify` or a logger method; inside a template
 literal; or the object of `.message`, `.stack` or `.toString`; or the
 initializer of a declaration destructuring `message` or `stack` off it. A
 member named `reason` (a settled promise's rejection) is flagged under the
 same calls, but not in a template; the plain names `reason` and `failure`
 are not flagged in a template either, where each is as often an authored
 phrase as a caught value.

 WHAT IT CANNOT SEE. A caught value copied into another binding, one
 carried across a function boundary under another name, a rejection read off
 `Promise.allSettled` under a name outside that list, and any text that
 reaches a sink through a callback it hands the error to. Review reads
 those.

 THE HELD LIST names each function that may keep the form and why, keyed by
 file and enclosing function. An entry that no longer matches fails, so the
 list cannot outlive its reasons. Two limits of the key: a function name
 shared by several classes or functions in one file (every class's
 `constructor`) is one key, and a held function stays held for every form in
 it, not only the form its reason was written for.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each form. Fixtures are cat-themed.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  ancestorsOf,
  childNodes,
  identifierName,
  isTreeNode,
  memberName,
  parentsOf,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Names a caught value goes by outside a catch clause's own binding.
 */
const CAUGHT_NAMES: ReadonlySet<string> = new Set(['caught', 'cause', 'error', 'failure', 'reason', 'thrown',],);

/**
 Names that are as often an authored phrase as a caught value (a constructor's
 `reason`, a `failure` kind), so a template holding one is not a finding.
 */
const PHRASE_NAMES: ReadonlySet<string> = new Set(['failure', 'reason',],);

/**
 Functions that turn their argument into text whole.
 */
const TEXT_CALLEES: ReadonlySet<string> = new Set(['String', 'caughtValueText', 'caughtValueStack',],);

/**
 Method names a logger call goes through.
 */
const LOG_METHODS: ReadonlySet<string> = new Set(['debug', 'error', 'info', 'trace', 'warn',],);

/**
 Node kinds that wrap one expression without changing the value it holds.
 */
const WRAPPER_KINDS: ReadonlySet<string> = new Set(['ParenthesizedExpression', 'TSAsExpression', 'TSNonNullExpression',],);

/**
 Node kinds that open a function or a method, whose name the finding carries.
 */
const NAMING_KINDS: ReadonlySet<string> = new Set(['FunctionDeclaration', 'FunctionExpression', 'MethodDefinition',],);

/**
 Forms a rejection member (`result.reason`) is flagged under.
 */
const MEMBER_FORMS: ReadonlySet<string> = new Set([
  'String',
  'caughtValueText',
  'caughtValueStack',
  'JSON.stringify',
  'logger call',
],);

/**
 Functions whose handling of a caught value was read and stands, keyed
 `path#function`, each with the reason it is not a disclosure.
 */
const HELD: Readonly<Record<string, string>> = {
  'artifact/artifact-change-sets.ts#readCheckedSets':
    'reads the message of an AssemblyContractError, the only class the clause lets through, which declares it safe',
  'artifact-pool/corpus-run/artifact-two-lane-read-comparison.ts#deriveComparison':
    'reads the message of an ArtifactComparisonError, the only class the clause lets through, which declares it safe',
  'artifact/corpus-run/artifact-two-lane-read-row-relations.ts#assertRowsCoherent':
    'text of a WordingCoherenceError or a DeliveryCoherenceError, the only classes the clause lets through, both marked',
  'run/corpus-run/cap-census-walk.ts#reportCapCensusUnreadable':
    'a directory listing or log read failure printed to the terminal of the cap-census command; no provider is called',
  'run/corpus-run/cli-refusal.ts#framesOf':
    'destructures the stack and the message, but the message is only searched for in the stack to find where its '
      + 'header ends, and only the lines after that end are printed',
  'coverage-census/corpus-run/coverage-census-steps.ts#commandExit':
    'a child process failure whose message is the command line and exit code (nano-spawn result.js), the '
      + 'output going to a log file; no provider is called',
  'coverage-census/corpus-run/coverage-probe-pair.ts#readPair':
    'a corpus read failure on the probe\'s own command; the corpus is public and no provider is called',
  'bench/corpus-run/editor-standing-read-artifact.ts#readStandingArtifact':
    'reads the message of an OffRosterModelError, which declares it safe and names a path and a model id',
  'slice-cache/corpus-run/entry-pictures.ts#gather':
    'reads the message of a CorpusReadError that isMissingCorpusObject narrowed, a marked class',
  'glossary/corpus-run/git-command.ts#detectGit':
    'an access check of one fixed system path, which fails with a filesystem code and that path',
  'probe/corpus-run/judge-fidelity-probe-run.ts#requestedSpecs':
    'reads the message of a FidelityReferenceError, the only class its catch lets through, which declares it safe',
  'slice-cache/corpus-run/pass-eligibility.ts#readSide':
    'reads the message of a CorpusReadError that isMissingCorpusObject narrowed, a marked class',
  'slice-cache/corpus-run/cache-account-search-reach.ts#reachOfRoot':
    'a real-path lookup failure of a root the audit command was given, written to the log beside that root and the '
      + 'runbook\'s runs root; the text names the path the lookup stopped at, which lies on the way to one of those '
      + 'two roots, and no provider is called',
  'slice-cache/corpus-run/cache-account-search-reach.ts#runbookRootReading':
    'a real-path lookup failure of the runbook\'s runs root, written to the log beside that root; the text names the '
      + 'path the lookup stopped at, which lies on the way to it, and no provider is called',
  'bench/corpus-run/producer-calibrate-run.ts#refuseUnjudgeableRoster':
    'reads the message of a ProducerRosterError, the only class its catch lets through, which declares it safe',
  'page-check/corpus-run/publish-defects.ts#defectOf':
    'reads the message of the refusal class a check names, which the step type holds to a class declaring '
      + 'messageNamesOnly',
  'run/corpus-run/run-timing-report-run.ts#reportRunTiming':
    'reads the message of a NothingInFlightError, the only class the clause lets through, which declares it safe',
  'insertion/corpus-run/runner-closure.ts#readEntryText':
    'a read failure of the built entry the run is executing, which quotes that path; no provider is called',
  'bench/corpus-run/translate-probe-run.ts#probeTranslate':
    'printed to the terminal of a probe command whose operator owns the provider keys, not logged or stored',
  'bench/corpus-run/window-trial-probe-draw.ts#readPairTexts':
    'a missing-object corpus read narrowed by isMissingCorpusObject; the corpus is public',
  'document-structure/refusal-text.ts#refusalText':
    'reads the message only of a class that declares messageNamesOnly, which is what this function exists to do',
  'shared-contract/run-json-read.ts#parseRunJson':
    'copies the digits after the position phrase and nothing else of the message (offsetIn)',
  'provider/transient-retry.ts#sleepBackoff':
    'the rejection of a timers/promises sleep under an aborted signal: an AbortError whose message is the fixed '
      + 'phrase "The operation was aborted" and whose cause, not its message, holds the abort reason (measured on Node '
      + 'with a custom abort reason)',
};

/**
 The form a node takes as the operand of its parent, empty where it turns
 into no text whole.

 @param node - identifier or member read

 @param parents - each node's parent

 @returns Form name, such as `String` or `template`, empty for none

 @example
 ```ts
 const form = formOf({ node: identifier, parents, },); // 'String' for String(error)
 ```
 */
function formOf(
  {
    node,
    parents,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): string {
  /**
   The nodes holding the node, nearest first.
   */
  const chain = ancestorsOf({
    node,
    parents,
  },);
  /**
   How many wrappers sit directly around the node.
   */
  const wrapped = chain.findIndex(function isNoWrapper(ancestor,): boolean {
    return !WRAPPER_KINDS.has(ancestor.type,);
  },);
  /**
   The node or the outermost wrapper around it.
   */
  const operand = (wrapped <= 0) ? node : nonNullishOrThrow(chain[wrapped - 1],);
  /**
   The first node that is no wrapper.
   */
  const parent = chain[wrapped];
  if (parent === undefined)
    return '';
  if (parent.type === 'TemplateLiteral')
    return 'template';
  if ((parent.type === 'MemberExpression') && (parent.object === operand)) {
    /**
     Property read off the operand.
     */
    const property = memberName({ node: parent, },);
    return ['message', 'stack', 'toString',].includes(property,) ? property : '';
  }
  if ((parent.type !== 'CallExpression') || (!Array.isArray(parent.arguments,)) || (!parent.arguments.includes(operand,)))
    return '';
  /**
   What the call goes through.
   */
  const { callee, } = parent;
  if (!isTreeNode(callee,))
    return '';
  if (TEXT_CALLEES.has(identifierName({ node: callee, },),))
    return identifierName({ node: callee, },);
  if (callee.type !== 'MemberExpression')
    return '';
  if ((identifierName({ node: callee.object, },) === 'JSON') && (memberName({ node: callee, },) === 'stringify'))
    return 'JSON.stringify';
  return LOG_METHODS.has(memberName({ node: callee, },),) ? 'logger call' : '';
}

/**
 Whether a node is the declared or accessed name of something rather than a
 value read.

 @param node - identifier read

 @param parents - each node's parent

 @returns Whether it names a property, a key or a declaration

 @example
 ```ts
 const named = namesSomething({ node: identifier, parents, },);
 ```
 */
function namesSomething(
  {
    node,
    parents,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): boolean {
  /**
   The node holding it.
   */
  const parent = parents.get(node,);
  if (parent === undefined)
    return true;
  if ((parent.type === 'MemberExpression') && (parent.property === node) && (parent.computed !== true))
    return true;
  if ((parent.type === 'Property') && (parent.key === node) && (parent.shorthand !== true))
    return true;
  return false;
}

/**
 The message or stack a declaration destructures off a caught value.

 @param node - node read

 @param bound - names the clauses around it caught

 @returns `destructured message of error` and the like; empty for any other node

 @example
 ```ts
 const found = destructuredOf({ node, bound: new Set(['error',],), },); // for const { stack, } = error;
 ```
 */
function destructuredOf(
  {
    node,
    bound,
  }: {
    readonly node: TreeNode;
    readonly bound: ReadonlySet<string>;
  },
): string {
  if ((node.type !== 'VariableDeclarator') || (!isTreeNode(node.id,)) || (node.id.type !== 'ObjectPattern'))
    return '';
  /**
   Name the declaration reads from, empty for anything but an identifier.
   */
  const name = identifierName({ node: node.init, },);
  if ((name === '') || (!(CAUGHT_NAMES.has(name,) || bound.has(name,))))
    return '';
  /**
   Names the pattern picks out.
   */
  const picked = new Set(childNodes({ node: node.id, },).map(function keyOf(property,): string {
    return identifierName({ node: property.key, },);
  },),);
  /**
   First of the two fields that is picked, absent for neither.
   */
  const field = ['message', 'stack',].find(function isPicked(candidate,): boolean {
    return picked.has(candidate,);
  },);
  return (field === undefined) ? '' : `destructured ${field} of ${name}`;
}

/**
 Findings in one file, as `path#function: form of name`.

 @param file - file read

 @returns Findings, unsorted and possibly repeated

 @example
 ```ts
 const findings = findingsIn({ file, },);
 ```
 */
function findingsIn({ file, }: { readonly file: SourceText; },): readonly string[] {
  const { program, } = parseSource({ file, },);
  /**
   Each node's parent.
   */
  const parents = parentsOf({ program, },);
  /**
   Findings so far.
   */
  const found: string[] = [];
  /**
   Nodes still to visit, each with its enclosing function's name and the
   names caught by the clauses around it.
   */
  const pending: {
    readonly node: TreeNode;
    readonly site: string;
    readonly bound: ReadonlySet<string>;
  }[] = [{
    node: program,
    site: '<module>',
    bound: new Set(),
  },];
  while (pending.length > 0) {
    const next = pending.pop();
    if (next === undefined)
      break;
    const {
      node,
      site,
      bound,
    } = next;
    /**
     Enclosing function's name for this node's children.
     */
    const here = NAMING_KINDS.has(node.type,) && (identifierName({ node: node.id ?? node.key, },) !== '')
      ? identifierName({ node: node.id ?? node.key, },)
      : site;
    /**
     Names a catch clause binds, for its children.
     */
    const inside = (node.type === 'CatchClause') && (identifierName({ node: node.param, },) !== '')
      ? new Set([...bound, identifierName({ node: node.param, },),],)
      : bound;
    pending.push(...childNodes({ node, },).map(function withContext(child,) {
      return {
        node: child,
        site: here,
        bound: inside,
      };
    },),);
    /**
     Message or stack a declaration destructures off a caught value, empty
     for none.
     */
    const destructured = destructuredOf({
      node,
      bound,
    },);
    if (destructured !== '')
      found.push(`${file.path}#${here}: ${destructured}`,);
    /**
     Name this node reads, empty where it reads no caught value.
     */
    const name = readName({
      node,
      parents,
      bound,
    },);
    if (name === '')
      continue;
    /**
     Form it is turned into text under, empty where it is not.
     */
    const form = formOf({
      node,
      parents,
    },);
    if ((form === '') || ((node.type === 'MemberExpression') && (!MEMBER_FORMS.has(form,))))
      continue;
    if ((form === 'template') && PHRASE_NAMES.has(name,))
      continue;
    found.push(`${file.path}#${here}: ${form} of ${name}`,);
  }
  return found;
}

/**
 The caught value a node reads, if it reads one.

 @param node - node read

 @param parents - each node's parent

 @param bound - names the clauses around it caught

 @returns Name, or `<object>.reason` for a rejection member; empty for neither

 @example
 ```ts
 const name = readName({ node, parents, bound: new Set(['error',],), },);
 ```
 */
function readName(
  {
    node,
    parents,
    bound,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
    readonly bound: ReadonlySet<string>;
  },
): string {
  if ((node.type === 'MemberExpression') && (memberName({ node, },) === 'reason'))
    return `${identifierName({ node: node.object, },)}.reason`;
  /**
   Name an identifier carries, empty for any other node.
   */
  const name = identifierName({ node, },);
  if ((name === '') || namesSomething({
    node,
    parents,
  },))
    return '';
  return (CAUGHT_NAMES.has(name,) || bound.has(name,)) ? name : '';
}

/**
 Findings across files that the held list does not cover, and held entries
 that matched nothing.

 @param files - files read; tests and fixtures are skipped

 @param held - functions held, keyed `path#function`

 @returns Findings and stale entries, sorted

 @example
 ```ts
 const open = openFindings({ files, held: HELD, },);
 ```
 */
function openFindings(
  {
    files,
    held,
  }: {
    readonly files: readonly SourceText[];
    readonly held: Readonly<Record<string, string>>;
  },
): readonly string[] {
  /**
   Every finding, once each.
   */
  const all = new Set(files
    .filter(function isSource(file,): boolean {
      return !file.isTest;
    },)
    .flatMap(function read(file,): readonly string[] {
      return findingsIn({ file, },);
    },),);
  /**
   Keys the findings matched.
   */
  const matched = new Set([...all,].map(function keyOf(finding,): string {
    return finding.slice(
      0,
      finding.indexOf(': ',),
    );
  },),);
  return [
    ...[...all,].filter(function isOpen(finding,): boolean {
      return !(finding.slice(
        0,
        finding.indexOf(': ',),
      ) in held);
    },),
    ...Object.keys(held,)
      .filter(function isStale(key,): boolean {
        return !matched.has(key,);
      },)
      .map(function stale(key,): string {
        return `${key}: held but matches nothing`;
      },),
  ].toSorted();
}

await describe({
  name: 'caught value text',
  children: [
    it({
      name: 'FINDS a caught value turned into text whole by String, a template, message, stack, JSON.stringify, '
        + 'caughtValueText, toString, a logger call, a destructured message or stack and a rejection member, and '
        + 'leaves refusalText, a cause, a class test, a field of the value, tests and a held function',
      fn: async () => {
        expect(openFindings({
          files: [
            {
              path: 'cat.ts',
              text: [
                'export async function control(): Promise<void> {',
                '  try { await risky(); } catch (error) {',
                '    l.warn(`one @{String(error)}`);',
                '    l.warn(`two @{error}`);',
                '    l.warn(`three @{(error as Error).message}`);',
                '    l.warn(`four @{(error as Error).stack}`);',
                '    l.warn(`five @{JSON.stringify(error)}`);',
                '    l.warn(`six @{caughtValueText(error)}`);',
                '    l.warn(`seven @{error.toString()}`);',
                '    l.warn(error);',
                '    const { stack, } = error;',
                '  }',
                '  for (const result of await Promise.allSettled([risky()]))',
                '    if (result.status === \'rejected\') l.warn(`eight @{String(result.reason)}`);',
                '}',
                'export async function calm(): Promise<void> {',
                '  try { await risky(); } catch (error) {',
                '    l.warn(`one @{refusalText({ error, })}`);',
                '    l.warn(`two @{error.name} @{String(error.charsSeen)}`);',
                '    const { code, } = error;',
                '    if (error instanceof TypeError) throw new Failure({ cause: error, });',
                '    l.warn(`three @{result.reason}`);',
                '  }',
                '}',
                'export function say(reason: string): string { return `cat @{reason}`; }',
                'export async function pause(): Promise<void> {',
                '  try { await risky(); } catch (stumble) { l.warn(`nine @{String(stumble)}`); }',
                '}',
                'export async function nap(): Promise<void> {',
                '  try { await risky(); } catch (error) { l.warn(`ten @{String(error)}`); }',
                '}',
              ]
                .join('\n',)
                // The fixture writes `@{` where a placeholder opens, so no string literal here holds a whole one.
                .replaceAll(
                '@{',
                '${',
                ),
              isTest: false,
            },
            {
              path: 'cat.unit.test.ts',
              text: 'try { JSON.parse(\'{\'); } catch (error) { console.log(String(error)); }',
              isTest: true,
            },
          ],
          held: {
            'cat.ts#nap': 'the cat sleeps and says nothing',
            'cat.ts#groom': 'a held entry that matches nothing',
          },
        },),).toEqual([
          'cat.ts#control: JSON.stringify of error',
          'cat.ts#control: String of error',
          'cat.ts#control: String of result.reason',
          'cat.ts#control: caughtValueText of error',
          'cat.ts#control: destructured stack of error',
          'cat.ts#control: logger call of error',
          'cat.ts#control: message of error',
          'cat.ts#control: stack of error',
          'cat.ts#control: template of error',
          'cat.ts#control: toString of error',
          'cat.ts#groom: held but matches nothing',
          'cat.ts#pause: String of stumble',
        ].toSorted(),);
      },
    },),
    it({
      name: 'FINDS NO CAUGHT VALUE TURNED INTO TEXT WHOLE across the package outside the held list',
      fn: async () => {
        /**
         Every package file, tests among them to be skipped.
         */
        const files = await readPackageSource();
        expect(files.some(function isSource(file,): boolean {
          return !file.isTest;
        },),).toBe(true,);
        expectNoFindings({
          findings: openFindings({
            files,
            held: HELD,
          },),
        },);
      },
    },),
  ],
},);
