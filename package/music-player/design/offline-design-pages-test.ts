/**
 Proves the music player's design pages answer from this tree alone: no page names a scheme this tree cannot serve,
 every local name resolves to a committed file, the Design Component runtime loads its vendored builds, and every
 typeface a page sets is one the design system ships.
 
 The browser half of the same promise, that each page renders with no request leaving this tree, lives in
 `offline-design-pages-render-test.ts`.
 */

import { rmSync } from 'node:fs';

import { designPages, designRoot } from './design-page-dependencies.ts';
import {
  externalReferenceFindings,
  runtimeBuildFindings,
  shippedFontFindings,
  unresolvedReferenceFindings,
  vendorRecordFindings,
} from './design-page-offline-checks.ts';
import {
  scratchDirectory,
  writeControls,
} from './design-page-offline-controls.ts';

//region Guards

/** Guards this file holds the tree to, each named as the finding it reports. */
const CHECKS = [
  {
    name: 'no-external-reference',
    run: externalReferenceFindings,
  },
  {
    name: 'references-resolve',
    run: unresolvedReferenceFindings,
  },
  {
    name: 'vendor-builds-recorded',
    run: vendorRecordFindings,
  },
  {
    name: 'runtime-loads-vendored-builds',
    run: runtimeBuildFindings,
  },
  {
    name: 'shipped-fonts-only',
    run: shippedFontFindings,
  },
] as const;

/** One defect the control fixture carries, and the finding text that proves a guard still reports it. */
const CONTROLS = [
  {
    defect: 'example.org',
    run: externalReferenceFindings,
  },
  {
    defect: 'absent.css is not in this tree',
    run: unresolvedReferenceFindings,
  },
  {
    defect: '../fonts/absent.woff2 is not in this tree',
    run: unresolvedReferenceFindings,
  },
  {
    defect: 'and the record states',
    run: vendorRecordFindings,
  },
  {
    defect: 'committed with no entry',
    run: vendorRecordFindings,
  },
  {
    defect: 'loads https://example.org',
    run: runtimeBuildFindings,
  },
  {
    defect: 'differ beyond their vendor base',
    run: runtimeBuildFindings,
  },
  {
    defect: 'sets Noto Sans, which this tree does not ship',
    run: shippedFontFindings,
  },
  {
    defect: 'sets Google Sans Text and links no stylesheet defining it',
    run: shippedFontFindings,
  },
  {
    defect: 'hides its family from this guard',
    run: shippedFontFindings,
  },
] as const;

//endregion Guards

//region Report

const base = scratchDirectory();
try {
  const control = writeControls(base,);
  const failures: string[] = [];
  for (const entry of CONTROLS) {
    const reported = entry.run({ root: control.rejected, },);
    if (!reported.some(function mentions(finding,): boolean {
      return finding.includes(entry.defect,);
    },)) {
      failures.push(`control: ${entry.defect} went unreported in ${reported.length} findings`);
    }
  }
  const acceptedFindings = CHECKS.flatMap(function controls(check,): readonly string[] {
    return check.run({ root: control.accepted, },);
  },);
  if (acceptedFindings.length > 0) {
    failures.push(`control: a clean tree reported ${acceptedFindings.join('; ')}`);
  }

  const findings = CHECKS.flatMap(function read(check,): readonly string[] {
    return check.run({ root: designRoot, },).map(function label(finding,): string {
      return `${check.name}: ${finding}`;
    },);
  },);

  const lines = [
    ...failures,
    ...findings,
  ];
  if (lines.length > 0) {
    throw new Error(`Offline design pages: ${lines.length} findings\n${lines.join('\n')}`);
  }
  console.log(
    `Offline design pages: ${designPages(designRoot,).length} pages and ${CHECKS.length} guards, each proven able to fail.`,
  );
}
finally {
  rmSync(
    base,
    { recursive: true, force: true, },
  );
}

//endregion Report
