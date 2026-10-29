/**
 `toolchain:update`: re-resolves the plan's floating requests and channels to
 their current upstream versions and rewrites the resolved data
 (the `pnpm update` analogue `doc/planning/mpm-toolchain-migration.md` names).

 Explicit pins never move: the command reports them as pinned and leaves any
 change to a plan edit. Fixed channel ranges (`java` stays `temurin-21`) move
 only their resolution stamp.

 The steps live in sibling modules (`update.resolve.ts`, `update.report.ts`,
 `update.apply.ts`); this module re-exports them and runs the CLI.

 @example
 ```bash
 node package/file-enforcer/core/src/toolchain/update.ts
 node package/file-enforcer/core/src/toolchain/update.ts --check
 ```
 */

import { ALL_TOOLS, } from './plan.ts';
import { resolveGithubToken, } from './providers.ts';
import {
  globalToolchainPaths,
  isMainModule,
  resolveToolchainPaths,
} from './paths.ts';
import { loadPlanFromDir, } from './plan.load.ts';
import {
  applyResolutions,
  writeResolutions,
} from './update.apply.ts';
import {
  formatReport,
  outcomesOfKind,
} from './update.report.ts';
import { resolveAll, } from './update.resolve.ts';
import type { Fetcher, } from './update.resolvers.ts';

export {
  applyResolutions,
  writeResolutions,
} from './update.apply.ts';
export {
  formatReport,
  outcomesOfKind,
} from './update.report.ts';
export {
  resolveAll,
  type UpdateOutcome,
} from './update.resolve.ts';

//region CLI

/**
 Runs the update flow: resolve, report, and write unless `--check`.

 @throws Error when a resolution fails and `--check` needs a clean verdict

 @example
 ```ts
 await main(); // prints the report and rewrites the plan data
 ```
 */
export async function main(): Promise<void> {
  /**
   Dry-run flag: report only, never write.
   */
  const check = process.argv
    .includes('--check',);
  /**
   Whether this run targets the machine-global scope.
   */
  const global = process.argv
    .includes('--global',);
  /**
   Resolved scope paths.
   */
  const paths = global
    ? globalToolchainPaths({ env: process.env, },)
    : resolveToolchainPaths();
  /**
   Plan entries of the scope being updated.
   */
  const tools = global
    ? await loadPlanFromDir({ dir: paths.worktreeRoot, },)
    : ALL_TOOLS;
  /**
   GitHub token for the GitHub family, absent when none is configured.
   */
  const token = await resolveGithubToken({ env: process.env, },);
  /**
   Network fetcher with GitHub authentication bound in.

   @param url - upstream URL to GET

   @returns response with the body decoded once
   */
  async function fetcher(url: string,): ReturnType<Fetcher> {
    /**
     Request headers with GitHub authentication when the URL needs it.
     */
    const headers: Record<string, string> = { 'user-agent': 'monochromatic-toolchain-update', };
    if (((typeof token) === 'string') && url.startsWith('https://api.github.com/'))
      headers.authorization = `Bearer ${token}`;
    /**
     Response from the upstream source.
     */
    const response = await fetch(
      url,
      { headers, },
    );
    /**
     Body bytes, decoded once (a response body reads a single time).
     */
    const bytes = new Uint8Array(await response.arrayBuffer(),);
    return {
      ok: response.ok,
      status: response.status,
      text: new TextDecoder()
        .decode(bytes),
      bytes,
    };
  }
  /**
   Outcomes for every plan entry.
   */
  const outcomes = await resolveAll({
    tools,
    fetcher,
  },);
  process.stdout
    .write(`${
    formatReport({ outcomes, })
  }\n`,);
  /**
   Entries whose resolution moved.
   */
  const updates = outcomesOfKind({
    outcomes,
    kind: 'updated',
  },);
  /**
   Entries whose resolution failed.
   */
  const failures = outcomesOfKind({
    outcomes,
    kind: 'failed',
  },);
  if (failures.length > 0) {
    process.exitCode = 1;
    return;
  }
  if (check) {
    if (updates.length > 0)
      process.exitCode = 1;
    return;
  }
  if (updates.length === 0)
    return;
  /**
   Entries with the resolutions applied.
   */
  const applied = applyResolutions({
    tools,
    outcomes,
  },);
  /**
   Files the resolution rewrote.
   */
  const written = global
    ? await writeResolutions({
      tools: applied,
      outcomes,
      planDir: paths.worktreeRoot,
      paths,
    },)
    : await writeResolutions({
      tools: applied,
      outcomes,
    },);
  process.stdout
    .write(`toolchain update: wrote ${
    written.join(', ',)
  }\n`,);
}

if (isMainModule(import.meta.filename,))
  await main();

//endregion CLI
