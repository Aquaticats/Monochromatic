import {
  mkdir,
  mkdtemp,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

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
  resolveToolchainRuleRoots,
} from '../dist/final/node/opensnitch-toolchain-rules.mjs';

/**
 Host alias the real machine uses for homes.
 */
const HOME_ALIAS = [['/home', '/var/home',],] as const;

/**
 Roots for a fictional user whose home has both spellings.
 */
const ROOTS = {
  dataHomes: pathSpellings({ path: '/var/home/tester/.local/share', aliasPairs: HOME_ALIAS, },),
  cargoHomes: pathSpellings({ path: '/var/home/tester/.cargo', aliasPairs: HOME_ALIAS, },),
  rustupHomes: pathSpellings({ path: '/var/home/tester/.rustup', aliasPairs: HOME_ALIAS, },),
  repositoryRoots: pathSpellings({ path: '/var/home/tester/Monochromatic', aliasPairs: HOME_ALIAS, },),
};

/**
 Reports whether any generated rule matches a process path.

 @param path - Executable path the daemon would compare.

 @returns Whether at least one generated expression matches.

 @example
 ```ts
 isAllowed({ path: '/usr/bin/curl' }); // false
 ```
 */
function isAllowed({ path, }: { readonly path: string; },): boolean {
  return toolchainRuleSpecs({ roots: ROOTS, },).some(function matches(spec,): boolean {
    return new RegExp(spec.pattern,).test(path,);
  },);
}

await describe({
  name: toolchainRuleSpecs.name,
  children: [
    it({
      name: 'matches install roots, exposure links and Rust tools in both home spellings',
      fn: async () => {
        expect(
          [
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
          ].filter((path,) => !isAllowed({ path, },)),
        ).toEqual([],);
      },
    },),
    it({
      name: 'rejects system tools, old mise installs, siblings and lookalike roots',
      fn: async () => {
        expect(
          [
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
            '/var/home/tester/Monochromatic/package/file-enforcer/cli/target/debug/file-enforcer',
            '/var/home/tester/Monochromatic/.claude/worktrees/agent-a1b2c3/package/x/target/debug/x',
          ].filter((path,) => isAllowed({ path, },)),
        ).toEqual([],);
      },
    },),
    it({
      name: 'is case sensitive and uses only RE2 syntax',
      fn: async () => {
        /**
         Specs with lookaround or backreferences,
         which the daemon's RE2 engine cannot compile.
         */
        const specs = toolchainRuleSpecs({ roots: ROOTS, },);
        expect(specs.length,).toBe(3,);
        expect(specs.filter((spec,) => /\(\?[=!<]/v.test(spec.pattern,) || /\\[1-9]/v.test(spec.pattern,)),).toEqual([],);
        expect(specs.filter((spec,) => !spec.pattern.startsWith('^',)),).toEqual([],);
        expect(specs.filter((spec,) => !(spec.pattern.endsWith('$',) || spec.pattern.endsWith('/',))),).toEqual([],);
        expect(
          specs.filter((spec,) =>
            !toolchainRuleFile({ spec, created: 'a', updated: 'b', },).operator.sensitive
          ),
        ).toEqual([],);
      },
    },),
    it({
      name: 'omits a rule whose roots are empty',
      fn: async () => {
        expect(toolchainRuleSpecs({
          roots: { dataHomes: [], cargoHomes: [], rustupHomes: [], repositoryRoots: [], },
        },),).toEqual([],);
        expect(toolchainRuleSpecs({
          roots: { ...ROOTS, repositoryRoots: [], },
        },).map((spec,) => spec.name),).toEqual([
          `${TOOLCHAIN_RULE_NAME_PREFIX}install-roots`,
          `${TOOLCHAIN_RULE_NAME_PREFIX}rust-tools`,
        ],);
      },
    },),
    it({
      name: 'renders precedence allow rules named to sort before auto-saved reject and deny rules',
      fn: async () => {
        for (const spec of toolchainRuleSpecs({ roots: ROOTS, },)) {
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
      name: 'escapes RE2 metacharacters in literal paths',
      fn: async () => {
        expect(escapeRe2Literal({ text: '/a.b+c(d)[e]{f}|g^h$i\\j', },),).toBe(
          '/a\\.b\\+c\\(d\\)\\[e\\]\\{f\\}\\|g\\^h\\$i\\\\j',
        );
        expect(rfc3339({ moment: new Date(Date.UTC(2026, 9, 31, 1, 2, 3, 456,),), },),).toBe('2026-10-31T01:02:03Z',);
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
        const base = await mkdtemp(join(scratchRoot(), 'wg-quicker-rule-roots-',),);
        try {
          await mkdir(join(base, 'real', 'home',), { recursive: true, },);
          await symlink(join(base, 'real',), join(base, 'link',),);
          const home = join(base, 'link', 'home',);
          const roots = await resolveToolchainRuleRoots({ options: { home, repositoryRoots: [], }, },);
          expect(roots.dataHomes,).toContain(join(home, '.local', 'share',),);
          expect(roots.cargoHomes,).toContain(join(home, '.cargo',),);
          expect(roots.rustupHomes,).toContain(join(home, '.rustup',),);
          const spelled = await directorySpellings({ path: home, aliasPairs: [], },);
          expect(spelled,).toEqual([home, join(base, 'real', 'home',),],);
        }
        finally {
          await rm(base, { recursive: true, force: true, },);
        }
      },
    },),
    it({
      name: 'honours explicit data, cargo and rustup homes',
      fn: async () => {
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
        const base = await mkdtemp(join(scratchRoot(), 'wg-quicker-repo-root-',),);
        try {
          const worktree = join(base, 'repo', '.claude', 'worktrees', 'agent-1',);
          await mkdir(join(base, 'repo', 'package', 'wg',), { recursive: true, },);
          await mkdir(join(worktree, 'package', 'wg',), { recursive: true, },);
          await writeFile(join(base, 'repo', 'toolchain.lock.json',), '{}',);
          await writeFile(join(worktree, 'toolchain.lock.json',), '{}',);
          expect(await findRepositoryRoot({ start: join(base, 'repo', 'package', 'wg',), },),).toBe(join(base, 'repo',),);
          expect(await findRepositoryRoot({ start: join(worktree, 'package', 'wg',), },),).toBe(join(base, 'repo',),);
        }
        finally {
          await rm(base, { recursive: true, force: true, },);
        }
      },
    },),
    it({
      name: 'reports no checkout when no ancestor holds the lock file',
      fn: async () => {
        const base = await mkdtemp(join(scratchRoot(), 'wg-quicker-no-repo-',),);
        try {
          expect(await findRepositoryRoot({ start: base, },),).toBeUndefined();
        }
        finally {
          await rm(base, { recursive: true, force: true, },);
        }
      },
    },),
  ],
},);
