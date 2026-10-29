#!/usr/bin/env node
/**
 * Forge seed CLI.
 *
 * `bun src/cli.ts --repos=N --users=N [--seed=N] [--out=path/to/dir.db] [--max-issues-per-repo=N]`
 *
 * Side-effect imports `data/db.ts` (via the queries import inside `generate.ts`)
 * so the database connection is opened against `--out=` or `DB_PATH`.
 */

import { logger, } from '@monochromatic-dev/logger-core/logger';
import { tagged, } from '@monochromatic-dev/logger-core/tagged';

import { seedDataset, } from './dataset.ts';

/**
 * Tagged logger scoped to the seed CLI.
 */
const l = tagged({
  tag: 'seed',
  l: logger,
},);

/**
 * Decimal radix for `parseInt` calls.
 */
const DECIMAL_RADIX = 10;

/**
 * Default seed value when `--seed=` is not supplied.
 */
const DEFAULT_SEED = 1;

/**
 * Default user count when `--users=` is not supplied.
 */
const DEFAULT_USERS = 10;

/**
 * Default repo count when `--repos=` is not supplied.
 */
const DEFAULT_REPOS = 3;

/**
 * Parses a `--key=value` flag from `process.argv`.
 *
 * @param name - flag name without the leading dashes
 *
 * @returns flag value, or `undefined` if absent
 *
 * @example
 * ```ts
 * const out = getFlag('out');
 * ```
 */
function getFlag(name: string,): string | undefined {
  /**
   * Match-prefix derived from the flag name so the find call locates the right argv entry.
   */
  const prefix = `--${name}=`;
  /**
   * First matching argv entry, or undefined when the flag was not passed.
   */
  const argument = process.argv
    .find(function hasPrefix(entry,) {
    return entry.startsWith(prefix,);
  },);
  return argument?.slice(prefix.length,);
}

/**
 * Parses an integer flag with a default.
 *
 * @param row - flag name and fallback value
 *
 * @returns parsed integer
 *
 * @example
 * ```ts
 * const repos = intFlag({ name: 'repos', fallback: 3 });
 * ```
 */
function intFlag(row: {
  name: string;
  fallback: number;
},): number {
  /**
   * Raw flag string lifted from argv before integer parsing.
   */
  const raw = getFlag(row.name,);
  if (raw === undefined)
    return row.fallback;
  /**
   * Decimal-parsed flag value; reused twice in the finite check and return.
   */
  const parsed = Number.parseInt(
    raw,
    DECIMAL_RADIX,
  );
  return Number.isFinite(parsed,) ? parsed : row.fallback;
}

/**
 * CLI `--out=` arg, used to override `DB_PATH`.
 */
const out = getFlag('out',);
if (out !== undefined)
  process.env
    .DB_PATH = out;

/**
 * Resolved seed flag.
 */
const seed = intFlag({
  name: 'seed',
  fallback: DEFAULT_SEED,
},);

/**
 * Resolved `--users=` flag.
 */
const userCount = intFlag({
  name: 'users',
  fallback: DEFAULT_USERS,
},);

/**
 * Resolved `--repos=` flag.
 */
const repoCount = intFlag({
  name: 'repos',
  fallback: DEFAULT_REPOS,
},);

/**
 * Raw `--max-issues-per-repo=` flag string, before parsing.
 */
const maxIssuesPerRepoRaw = getFlag('max-issues-per-repo',);

/**
 * Parsed `--max-issues-per-repo=`, or `undefined` if absent.
 */
const maxIssuesPerRepo = maxIssuesPerRepoRaw === undefined
  ? undefined
  : Number.parseInt(
    maxIssuesPerRepoRaw,
    DECIMAL_RADIX,
  );

l.info(
  `seeding forge dataset seed=${String(seed,)} users=${String(userCount,)} repos=${
    String(repoCount,)
  } out=${out ?? '(default)'}`,
);

/**
 * Aggregated counts from {@link seedDataset}.
 */
const summary = await seedDataset({
  seed,
  userCount,
  repoCount,
  baseTimestamp: Date.now(),
  ...(maxIssuesPerRepo === undefined
    ? {}
    : { maxIssuesPerRepo, }),
},);

l.info(
  `seed complete users=${String(summary.users,)} repos=${String(summary.repos,)} labels=${
    String(summary.labels,)
  } issues=${String(summary.issues,)} comments=${String(summary.comments,)} milestones=${
    String(summary.milestones,)
  } prs=${String(summary.prs,)} reviews=${String(summary.reviews,)}`,
);
// JSON summary goes through stdout so callers can pipe it into other tools.
process.stdout
  .write(`${
  JSON.stringify(
    summary,
    null,
    2,
  )
}\n`,);
