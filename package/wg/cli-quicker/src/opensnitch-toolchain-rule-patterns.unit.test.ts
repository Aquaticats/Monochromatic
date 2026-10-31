import {
  mkdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  escapeRe2Literal,
  pathSpellings,
  rfc3339,
  TOOLCHAIN_RULE_NAME_PREFIX,
  toolchainRuleFile,
  toolchainRuleSpecs,
} from '../dist/final/node/opensnitch-toolchain-rule-patterns.mjs';
import {
  directorySpellings,
  findRepositoryRoot,
  REPOSITORY_ROOT_ABSENT,
  resolveToolchainRuleRoots,
} from '../dist/final/node/opensnitch-toolchain-rules.mjs';
import {
  createScratch,
  HOME_ALIAS,
  ROOTS,
} from './fixture.toolchain-rules.ts';

/**
 Executable paths the allowance must cover,
 in both home spellings.
 */
const ALLOWED_PATHS = [
  '/var/home/tester/.local/share/monochromatic-dev/toolchain/node/26.13.0/bin/node',
  '/home/tester/.local/share/monochromatic-dev/toolchain/bin/pnpm',
  '/var/home/tester/.local/share/monochromatic-dev/toolchain-global/uv/0.13.6/uv',
  '/home/tester/.local/share/monochromatic-dev/toolchain-global/launchers/mpm-shadow',
  '/var/home/tester/Monochromatic/.cache/toolchain-bin/node',
  '/home/tester/Monochromatic/.cache/toolchain-launchers/mpm-shadow',
  '/var/home/tester/Monochromatic/.claude/worktrees/agent-a1b2c3/.cache/toolchain-launchers/cargo-binstall',
  '/var/home/tester/.cargo/bin/rustup',
  '/home/tester/.cargo/bin/cargo',
  '/var/home/tester/.cargo/bin/cargo-binstall',
  '/var/home/tester/.rustup/toolchains/nightly-2026-10-11-x86_64-unknown-linux-gnu/bin/cargo',
  '/home/tester/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin/rustc',
];

/**
 Executable paths the allowance must not cover:
 system tools, old mise installs, sibling directories, lookalike roots and build outputs.
 */
const REJECTED_PATHS = [
  '/usr/bin/curl',
  '/var/home/tester/.local/share/mise/installs/node/26.10.0/bin/node',
  '/var/home/tester/.local/share/monochromatic-dev/toolchain-evil/bin/x',
  '/var/home/tester/.local/share/monochromatic-dev/toolchain',
  '/var/home/tester/.local/share/monochromatic-dev/tool/x',
  '/tmp/var/home/tester/.local/share/monochromatic-dev/toolchain/bin/x',
  '/var/home/other/.local/share/monochromatic-dev/toolchain/bin/x',
  '/var/home/tester/.cargo/bin/cargo-evil',
  '/var/home/tester/.cargo/bin/other-tool',
  '/var/home/tester/.cargo/bin/rustup/child',
  '/var/home/tester/.cargo/registry/bin/cargo',
  '/var/home/other/.cargo/bin/rustup',
  '/var/home/tester/.rustup/toolchains/nightly/lib/rustlib/x',
  '/var/home/tester/.rustup/toolchains/nightly/bin/sub/x',
  '/var/home/tester/.rustup/toolchains/bin/cargo',
  '/var/home/tester/Monochromatic/.cache/toolchain-bin/sub/x',
  '/var/home/tester/Monochromatic/.cache/toolchain-bin2/x',
  '/var/home/tester/Monochromatic/.cache/other/x',
  '/var/home/tester/monochromatic/.cache/toolchain-bin/x',
  '/var/home/tester/Monochromatic/.claude/worktrees//.cache/toolchain-bin/x',
  '/var/home/tester/Monochromatic/package/file-enforcer/cli/target/debug/file-enforcer',
  '/var/home/tester/Monochromatic/.claude/worktrees/agent-a1b2c3/package/x/target/debug/x',
];

/**
 Compiles a generated daemon pattern the way OpenSnitch does for a case-sensitive rule.

 @param pattern - Anchored RE2 expression.

 @returns Matcher over a process path.

 @example
 ```ts
 daemonMatcher({ pattern: '^/usr/' })('/usr/bin/curl'); // true
 ```
 */
function daemonMatcher({ pattern, }: { readonly pattern: string; },): (path: string) => boolean {
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- The test compiles the generated daemon pattern, built from escaped literals without nested quantifiers, over short fixed paths.
  const expression = new RegExp(pattern, 'u',);
  return function matches(path,): boolean {
    return expression.test(path,);
  };
}

/**
 Reports whether any generated rule's daemon expression matches a process path.

 @param path - Executable path the daemon would compare.

 @returns Whether at least one generated expression matches.

 @example
 ```ts
 isAllowedByPattern({ path: '/usr/bin/curl' }); // false
 ```
 */
function isAllowedByPattern({ path, }: { readonly path: string; },): boolean {
  return toolchainRuleSpecs({ roots: ROOTS, },).some(function matches(spec,): boolean {
    return daemonMatcher({ pattern: spec.pattern, },)(path,);
  },);
}

/**
 Reports whether any generated rule's plain-string coverage matches a process path.

 @param path - Executable path the daemon would compare.

 @returns Whether at least one generated predicate covers the path.

 @example
 ```ts
 isAllowedByCoverage({ path: '/usr/bin/curl' }); // false
 ```
 */
function isAllowedByCoverage({ path, }: { readonly path: string; },): boolean {
  return toolchainRuleSpecs({ roots: ROOTS, },).some(function covers(spec,): boolean {
    return spec.covers(path,);
  },);
}

await describe({
  name: toolchainRuleSpecs.name,
  children: [
    it({
      name: 'matches install roots, exposure links and Rust tools in both home spellings',
      fn: async () => {
        expect(ALLOWED_PATHS.filter((path,) => !isAllowedByPattern({ path, },)),).toEqual([],);
      },
    },),
    it({
      name: 'rejects system tools, old mise installs, siblings, lookalike roots and build outputs',
      fn: async () => {
        expect(REJECTED_PATHS.filter((path,) => isAllowedByPattern({ path, },)),).toEqual([],);
      },
    },),
    it({
      name: 'agrees between the daemon expression and the plain-string coverage on every fixture path',
      fn: async () => {
        for (const path of [...ALLOWED_PATHS, ...REJECTED_PATHS,]) {
          expect(isAllowedByCoverage({ path, },),).toBe(isAllowedByPattern({ path, },),);
        }
        expect(ALLOWED_PATHS.filter((path,) => !isAllowedByCoverage({ path, },)),).toEqual([],);
      },
    },),
    it({
      name: 'uses only RE2 syntax, anchors every expression and compares case-sensitively',
      fn: async () => {
        /**
         Generated specs.
         */
        const specs = toolchainRuleSpecs({ roots: ROOTS, },);
        expect(specs.length,).toBe(3,);
        expect(
          specs.filter((spec,) =>
            ['(?=', '(?!', '(?<', ...'123456789'.split('',).map((digit,) => `\\${digit}`),]
              .some((fragment,) => spec.pattern.includes(fragment,))
          ),
        ).toEqual([],);
        expect(specs.filter((spec,) => !spec.pattern.startsWith('^',)),).toEqual([],);
        expect(specs.filter((spec,) => !(spec.pattern.endsWith('$',) || spec.pattern.endsWith('/',))),).toEqual([],);
        expect(
          specs.filter((spec,) => !toolchainRuleFile({ spec, created: 'a', updated: 'b', },).operator.sensitive),
        ).toEqual([],);
      },
    },),
    it({
      name: 'omits a rule whose roots are empty',
      fn: async () => {
        expect(toolchainRuleSpecs({
          roots: { dataHomes: [], cargoHomes: [], rustupHomes: [], repositoryRoots: [], },
        },),).toEqual([],);
        expect(toolchainRuleSpecs({ roots: { ...ROOTS, repositoryRoots: [], }, },).map((spec,) => spec.name),).toEqual([
          `${TOOLCHAIN_RULE_NAME_PREFIX}install-roots`,
          `${TOOLCHAIN_RULE_NAME_PREFIX}rust-tools`,
        ],);
      },
    },),
    it({
      name: 'renders precedence allow rules named to sort before auto-saved reject and deny rules',
      fn: async () => {
        for (const spec of toolchainRuleSpecs({ roots: ROOTS, },)) {
          /**
           Rule rendered from the spec.
           */
          const rule = toolchainRuleFile({ spec, created: 'c', updated: 'u', },);
          expect(rule.precedence,).toBe(true,);
          expect(rule.action,).toBe('allow',);
          expect(rule.duration,).toBe('always',);
          expect(rule.enabled,).toBe(true,);
          expect(rule.operator.operand,).toBe('process.path',);
          expect(rule.operator.type,).toBe('regexp',);
          expect(rule.name < 'reject-always-simple-var-home-x',).toBe(true,);
          expect(rule.name < 'deny-always-simple-var-home-x',).toBe(true,);
          expect(rule.name < 'allow-always-simple-var-home-x',).toBe(true,);
        }
      },
    },),
  ],
},);

await describe({
  name: pathSpellings.name,
  children: [
    it({
      name: 'maps /home and /var/home in both directions without duplicates',
      fn: async () => {
        expect(pathSpellings({ path: '/home/me', aliasPairs: HOME_ALIAS, },),).toEqual([
          '/home/me',
          '/var/home/me',
        ],);
        expect(pathSpellings({ path: '/var/home/me/', aliasPairs: HOME_ALIAS, },),).toEqual([
          '/var/home/me',
          '/home/me',
        ],);
        expect(pathSpellings({ path: '/opt/x', aliasPairs: HOME_ALIAS, },),).toEqual(['/opt/x',],);
        expect(pathSpellings({ path: '/homework/me', aliasPairs: HOME_ALIAS, },),).toEqual(['/homework/me',],);
      },
    },),
    it({
      name: 'escapes RE2 metacharacters in literal paths and renders RFC 3339 times',
      fn: async () => {
        expect(escapeRe2Literal({ text: String.raw`/a.b+c(d)[e]{f}|g^h$i\j`, },),).toBe(
          String.raw`/a\.b\+c\(d\)\[e\]\{f\}\|g\^h\$i\\j`,
        );
        expect(rfc3339({ moment: new Date('2026-10-31T01:02:03.456Z',), },),).toBe('2026-10-31T01:02:03Z',);
      },
    },),
  ],
},);

await describe({
  name: resolveToolchainRuleRoots.name,
  children: [
    it({
      name: 'adds the real path of a symlinked home and defaults data, cargo and rustup under it',
      fn: async () => {
        await using scratch = await createScratch({ prefix: 'wg-quicker-rule-roots-', },);
        await mkdir(join(scratch.path, 'real', 'home',), { recursive: true, },);
        await symlink(join(scratch.path, 'real',), join(scratch.path, 'link',),);
        /**
         Home spelled through the symlink.
         */
        const home = join(scratch.path, 'link', 'home',);
        /**
         Roots resolved from the symlinked home.
         */
        const roots = await resolveToolchainRuleRoots({ options: { home, repositoryRoots: [], }, },);
        expect(roots.dataHomes,).toContain(join(home, '.local', 'share',),);
        expect(roots.cargoHomes,).toContain(join(home, '.cargo',),);
        expect(roots.rustupHomes,).toContain(join(home, '.rustup',),);
        expect(await directorySpellings({ path: home, aliasPairs: [], },),).toEqual([
          home,
          join(scratch.path, 'real', 'home',),
        ],);
      },
    },),
    it({
      name: 'honours explicit data, cargo and rustup homes',
      fn: async () => {
        /**
         Roots resolved from explicit overrides.
         */
        const roots = await resolveToolchainRuleRoots({
          options: {
            home: '/nonexistent/home',
            dataHome: '/nonexistent/data',
            cargoHome: '/nonexistent/cargo',
            rustupHome: '/nonexistent/rustup',
            repositoryRoots: ['/nonexistent/repo',],
          },
        },);
        expect(roots.dataHomes,).toContain('/nonexistent/data',);
        expect(roots.cargoHomes,).toContain('/nonexistent/cargo',);
        expect(roots.rustupHomes,).toContain('/nonexistent/rustup',);
        expect(roots.repositoryRoots,).toContain('/nonexistent/repo',);
        expect(roots.dataHomes.some((path,) => path.includes('/nonexistent/home',)),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: findRepositoryRoot.name,
  children: [
    it({
      name: 'finds the checkout above a package directory and the main checkout above a worktree',
      fn: async () => {
        await using scratch = await createScratch({ prefix: 'wg-quicker-repo-root-', },);
        /**
         Linked worktree inside the main checkout.
         */
        const worktree = join(scratch.path, 'repo', '.claude', 'worktrees', 'agent-1',);
        await mkdir(join(scratch.path, 'repo', 'package', 'wg',), { recursive: true, },);
        await mkdir(join(worktree, 'package', 'wg',), { recursive: true, },);
        await writeFile(join(scratch.path, 'repo', 'toolchain.lock.json',), '{}',);
        await writeFile(join(worktree, 'toolchain.lock.json',), '{}',);
        expect(await findRepositoryRoot({ start: join(scratch.path, 'repo', 'package', 'wg',), },),).toBe(
          join(scratch.path, 'repo',),
        );
        expect(await findRepositoryRoot({ start: join(worktree, 'package', 'wg',), },),).toBe(
          join(scratch.path, 'repo',),
        );
      },
    },),
    it({
      name: 'reports no checkout when no ancestor holds the lock file',
      fn: async () => {
        await using scratch = await createScratch({ prefix: 'wg-quicker-no-repo-', },);
        expect(await findRepositoryRoot({ start: scratch.path, },),).toBe(REPOSITORY_ROOT_ABSENT,);
      },
    },),
  ],
},);
