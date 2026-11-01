import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { classifyLeftovers, } from './leftover-fixture.ts';

await describe({
  name: classifyLeftovers.name,
  children: [
    it({
      name: 'keeps persistent roots and ignores objects',
      fn: async () => {
        expect(classifyLeftovers([
          'HEAD',
          'cli-git-transactions',
          'cli-git',
          'cli-git/shadow',
          'cli-git-captures',
          'cli-git-captures/worktree-id',
          'cli-git-captures/sequence',
          'cli-git-captures/landed',
          'worktrees/linked/cli-git-captures/sequence',
          'objects/pack/tmp.lock',
          'refs/heads/main',
        ],),).toEqual([],);
      },
    },),
    it({
      name: 'reports locks, lock directories, transactions, staging directories, legacy directories, shadow repositories, and transient capture-store entries once each',
      fn: async () => {
        expect(classifyLeftovers([
          'index.lock',
          'next-index-12.lock',
          'refs/heads/main.lock',
          'cli-git-transactions/landing.lock',
          'cli-git-transactions/landing.lock/owner.json',
          'cli-git-transactions/0a1b',
          'cli-git-transactions/0a1b/owner.json',
          'cli-git-transactions/.staging-3',
          'cli-git-transaction',
          'cli-git-transaction/journal.json',
          'cli-git/shadow/0a1b',
          'cli-git/shadow/0a1b/HEAD',
          'worktrees/linked/index.lock',
          'worktrees/linked/cli-git-transactions/9f/owner.json',
          'cli-git-captures/capture.lock',
          'cli-git-captures/capture.lock/owner.json',
          'cli-git-captures/sequence.0c.tmp',
          'cli-git-captures/landed/abc.json',
          'worktrees/linked/cli-git-captures/landed/def.json',
        ],),).toEqual([
          'cli-git-captures/capture.lock',
          'cli-git-captures/landed/abc.json',
          'cli-git-captures/sequence.0c.tmp',
          'cli-git-transaction',
          'cli-git-transactions/.staging-3',
          'cli-git-transactions/0a1b',
          'cli-git-transactions/landing.lock',
          'cli-git/shadow/0a1b',
          'index.lock',
          'next-index-12.lock',
          'refs/heads/main.lock',
          'worktrees/linked/cli-git-captures/landed/def.json',
          'worktrees/linked/cli-git-transactions/9f',
          'worktrees/linked/index.lock',
        ],);
      },
    },),
  ],
},);
