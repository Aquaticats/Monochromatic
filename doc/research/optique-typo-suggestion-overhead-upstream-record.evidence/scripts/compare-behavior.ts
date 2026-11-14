// Compares the exact CommitRegion record each arm returns for every recorded
// input, and the two upstream issue reproductions, then reports every input
// where versions disagree with both outputs.
//
// Host-side script; run with: node scripts/compare-behavior.ts

import { readFileSync, writeFileSync, } from 'node:fs';

//region Types

/**
 * One recorded fact record for one arm and one input.
 */
type BehaviorRecord = {
  readonly kind: 'behavior';
  readonly arm: string;
  readonly case: string;
  readonly outcome: Record<string, unknown>;
};

/**
 * One arm's outcomes for the upstream reproductions.
 */
type UpstreamRecord = {
  readonly kind: 'upstream';
  readonly arm: string;
  readonly probes: Record<string, unknown>;
};

//endregion Types

//region Entry point

/**
 * Groups every case by result and prints the cases where versions differ.
 */
function main(): void {
  /** Every parsed JSONL line. */
  const lines = readFileSync(
    'results/behavior.jsonl',
    'utf8',
  )
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line.trim() !== '';
    },)
    .map(function parse(line,) {
      return JSON.parse(line,) as BehaviorRecord | UpstreamRecord;
    },);
  /** Fact records grouped per case and arm. */
  const factsByCase = new Map<string, Map<string, string>>();

  for (const line of lines) {
    if (line.kind !== 'behavior') {
      continue;
    }

    /** Per-arm result text for this case. */
    const perArm = factsByCase.get(line.case,) ?? new Map<string, string>();

    perArm.set(
      line.arm,
      JSON.stringify(line.outcome,),
    );
    factsByCase.set(
      line.case,
      perArm,
    );
  }

  /** Cases that disagree between versions. */
  const differences = [...factsByCase.entries(),]
    .map(function asCase(entry,) {
      /** Distinct result texts for this case. */
      const distinct = new Map<string, string[]>();

      for (const [arm, text,] of entry[1]) {
        /** Arms producing this text. */
        const arms = distinct.get(text,) ?? [];

        arms.push(arm,);
        distinct.set(
          text,
          arms,
        );
      }

      return {
        case: entry[0],
        distinctResults: distinct.size,
        groups: [...distinct.entries(),].map(function asGroup(group,) {
          return {
            arms: group[1].sort(),
            outcome: JSON.parse(group[0],) as Record<string, unknown>,
          };
        },),
      };
    },)
    .filter(function differ(entry,) {
      return entry.distinctResults > 1;
    },);

  /** Upstream probe outcomes per arm. */
  const upstream: Record<string, Record<string, unknown>> = {};

  for (const line of lines) {
    if (line.kind === 'upstream') {
      upstream[line.arm] = line.probes;
    }
  }

  /** Distinct upstream outcome sets. */
  const upstreamGroups = new Map<string, string[]>();

  for (const [arm, probes,] of Object.entries(upstream,)) {
    /** Result text for this arm. */
    const text = JSON.stringify(probes,);
    /** Arms sharing this text. */
    const arms = upstreamGroups.get(text,) ?? [];

    arms.push(arm,);
    upstreamGroups.set(
      text,
      arms,
    );
  }

  writeFileSync(
    'results/behavior-comparison.json',
    `${JSON.stringify(
      {
        cases: factsByCase.size,
        differences,
        upstream,
        upstreamGroups: [...upstreamGroups.entries(),].map(function asGroup(group,) {
          return {
            arms: group[1].sort(),
            probes: JSON.parse(group[0],) as Record<string, unknown>,
          };
        },),
      },
      null,
      2,
    )}\n`,
  );

  console.log(`cases recorded: ${factsByCase.size}`);
  console.log(`cases with differing results: ${differences.length}`,);

  for (const difference of differences) {
    console.log(`\ncase: ${difference.case}`);
    for (const group of difference.groups) {
      console.log(`  ${group.arms.join(', ')}`);
      console.log(`    ${JSON.stringify(group.outcome,)}`,);
    }
  }

  console.log(`\nupstream probe outcome sets: ${upstreamGroups.size}`);

  for (const group of upstreamGroups.entries()) {
    console.log(`  arms: ${group[1].sort().join(', ')}`,);
    for (const [probe, outcome,] of Object.entries(
      JSON.parse(group[0],) as Record<string, unknown>,
    )) {
      console.log(`    ${probe}: ${JSON.stringify(outcome,)}`,);
    }
  }
}

main();

//endregion Entry point
