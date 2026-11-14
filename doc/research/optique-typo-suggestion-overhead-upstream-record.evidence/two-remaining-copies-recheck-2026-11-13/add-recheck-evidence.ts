// Adds the records of the recheck of the two-further-copies prototypes to the evidence archive beside the research
// document, from an explicit list of files. Nothing else in this scratch folder goes in: it also holds process and
// unit listings of this machine and extracts of an agent's feed, which are no evidence for the documents.
// Files kept local on the owner's decision and the review reports are left out, as before. The words that mark a
// file as local-only are read from a local list that is itself not archived.
//
// Run: node add-recheck-evidence.ts

import { spawnSync, } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, } from 'node:fs';

const HOME = process.env['HOME'];

if (HOME === undefined) throw new Error('HOME is not set',);

const HERE = import.meta.dirname;
const AGENT_KEEP = `${HOME}/temp/agent/upstream-prototype.3lsGY9v7/keep`;
const ARCHIVE = '/var/mnt/encrypted/repository/Monochromatic/doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip';
const FOLDER = 'two-remaining-copies-recheck-2026-11-13';
const STAGE = `${HERE}/evidence-stage`;

const OWN = [
  'Containerfile.recheck',
  'Containerfile.annotated',
  'Containerfile.mixed-threshold',
  'Containerfile.kept-patch',
  'Containerfile.node-and-bun',
  'run-node-and-bun-timing-check.ts',
  'image/run-node-and-bun-timing.ts',
  'suite/build-arm-packages-inside.ts',
  'splice-timing-section.ts',
  'keep/arm-packages-build.jsonl',
  'keep/timing-round-node-and-bun.jsonl',
  'keep/upstream-core-suite-on-the-kept-patch-with-its-length-descriptor-fixed.jsonl',
  'run-kept-patch-timing-check.ts',
  'run-probes-on-the-kept-patch.ts',
  'verify-new-link.ts',
  'add-recheck-evidence.ts',
  'image/run-kept-patch-timing.ts',
  'probe/write-aftermath.ts',
  'keep/write-aftermath.jsonl',
  'keep/behaviour-probes-on-the-kept-patch.jsonl',
  'keep/timing-round-kept-patch.jsonl',
  'keep/upstream-core-suite-on-the-kept-patch-and-with-its-threshold-at-two.jsonl',
  'keep/multiple-state-fix-with-recheck-changes.patch',
  'keep/verify-new-link.jsonl',
  'stage-arms-for-recheck.ts',
  'derive-views-from-two-arm.ts',
  'make-final-multiple-arm.ts',
  'run-in-recheck-image.ts',
  'run-suites.ts',
  'run-view-facts.ts',
  'run-freeze-aftermath.ts',
  'run-annotated-check.ts',
  'run-mixed-threshold-check.ts',
  'run-remaining-timing-rounds.ts',
  'run-repeat-one-bun-test-file.ts',
  'summarise-timing.ts',
  'compact-timing-view.ts',
  'read-upstream-state.ts',
  'image/mixed-scaling.ts',
  'image/annotated-scaling.ts',
  'image/run-timing.ts',
  'image/run-annotated.ts',
  'image/run-mixed-threshold.ts',
  'image/view-facts.ts',
  'probe/behaviour-probe.ts',
  'probe/long-input-probe.ts',
  'probe/compare-builds-in-container.ts',
  'probe/freeze-aftermath.ts',
  'suite/run-suites-inside.ts',
  'suite/repeat-one-bun-test-file.ts',
  'keep/stage-arms-for-recheck.jsonl',
  'keep/derive-views-from-two-arm.jsonl',
  'keep/make-final-multiple-arm.jsonl',
  'keep/behaviour-probes-on-all-arms.jsonl',
  'keep/upstream-core-suite-on-three-runtimes.jsonl',
  'keep/upstream-core-suite-with-views-from-two-items.jsonl',
  'keep/upstream-core-suite-on-the-final-multiple-arm.jsonl',
  'keep/upstream-core-suite-on-the-final-multiple-arm-second-run.jsonl',
  'keep/upstream-core-suite-on-the-final-multiple-arm-third-run-and-concat.jsonl',
  'keep/upstream-core-suite-on-the-final-multiple-arm-with-the-freeze-refusal.jsonl',
  'keep/one-bun-test-file-repeated-on-two-arms.jsonl',
  'keep/view-facts.jsonl',
  'keep/freeze-aftermath.jsonl',
  'keep/annotated-parse-timing.jsonl',
  'keep/timing-round-1.jsonl',
  'keep/timing-round-1-large-base.jsonl',
  'keep/timing-rounds-2-to-5.jsonl',
  'keep/timing-round-mixed-threshold.jsonl',
  'keep/timing-summary.jsonl',
  'keep/timing-summary.txt',
  'keep/upstream-state.jsonl',
];
const AGENT = [
  'report.md',
  'diffs/01-array-view.patch',
  'diffs/02-multiple-state.patch',
  'diffs/03-input-buffer.patch',
  'diffs/04-both-combined.patch',
  'changelog/multiple-item-state-sharing.md',
  'changelog/consumed-argument-buffer.md',
  'commit-messages/01-array-view.txt',
  'commit-messages/02-multiple-state.txt',
  'commit-messages/03-input-buffer.txt',
  'measurements/matrix-summary.jsonl',
];

rmSync(STAGE, { recursive: true, force: true, },);

for (const [root, names, into,] of [[HERE, OWN, `${STAGE}/${FOLDER}`,], [AGENT_KEEP, AGENT, `${STAGE}/${FOLDER}/prototype-agent`,],] as const) {
  for (const name of names) {
    if (!existsSync(`${root}/${name}`,)) throw new Error(`missing: ${root}/${name}`,);

    mkdirSync(`${into}/${name}`.split('/',).slice(0, -1,).join('/',), { recursive: true, },);
    cpSync(`${root}/${name}`, `${into}/${name}`,);
  }
}

// Words that would mark a file as one that must stay out of a public archive, one regular expression per line.
const WORDS = `${HOME}/temp/agent/optique-2026-11-13/words-that-mark-a-local-only-file.txt`;
const scan = spawnSync('rg', ['--line-number', '--no-heading', '--ignore-case', '--file', WORDS, STAGE,], { encoding: 'utf8', },);

console.log(JSON.stringify({ linesNamingAWordToCheck: scan.stdout.trim().split('\n',).filter((line,) => line.length > 0).map((line,) => line.slice(STAGE.length + 1, STAGE.length + 260,)), },),);

if (process.argv.includes('--write',)) {
  const zip = spawnSync('zip', ['--recurse-paths', '--quiet', ARCHIVE, FOLDER,], { cwd: STAGE, encoding: 'utf8', },);

  if (zip.status !== 0) throw new Error(`zip failed: ${zip.stderr.slice(0, 300,)}`,);

  const listing = spawnSync('unzip', ['-l', ARCHIVE,], { encoding: 'utf8', },);
  const lines = listing.stdout.split('\n',);

  console.log(JSON.stringify({
    entriesInTheFolder: lines.filter((line,) => line.includes(`${FOLDER}/`,)).length,
    total: lines.at(-2,)?.trim(),
    namesOfLocalOnlyFiles: lines.filter((line,) => new RegExp(readFileSync(WORDS, 'utf8',).trim().split('\n',).join('|',), 'iu',).test(line,)).length,
    namesMentioningReviewReport: lines.filter((line,) => /review-report/iu.test(line,)).length,
  },),);
}
