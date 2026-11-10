/**
 The five readings that decide whether a design page answers from this tree alone. Each one answers the findings it
 holds, and an empty list means the tree passed that reading.
 */

import { createHash } from 'node:crypto';
import {
  existsSync,
  readdirSync,
  readFileSync,
} from 'node:fs';
import { join } from 'node:path';

import {
  ALLOWED_FONT_FAMILIES,
  SHIPPED_FONT_FAMILIES,
  classifyReference,
  designPages,
  displayPath,
  facesDefinedBy,
  familyNames,
  fontFamilyValues,
  fontShorthandValues,
  pageReferences,
  readText,
  resolveReference,
  stylesheetReferences,
} from './design-page-dependencies.ts';

/** Record naming every vendored build, its provenance and the runtime copies that load them. */
export const BUILD_RECORD = join(
  'vendor',
  'builds.json',
);

/** One vendored build as the record states it. */
type BuildRecord = {
  builds?: {
    file?: string;
    integrity?: string;
    license?: string;
    package?: string;
    version?: string;
  }[];
  runtimes?: string[];
};

/** The vendored-build record, or `null` when the tree holds none a reader can parse. */
function readBuildRecord({ root, }: { root: string; },): BuildRecord | null {
  const text = readText(join(
    root,
    BUILD_RECORD,
  ),);
  if (text === null) {
    return null;
  }
  try {
    const parsed = JSON.parse(text,) as unknown;
    return typeof parsed === 'object' && parsed !== null ? parsed as BuildRecord : null;
  }
  catch {
    return null;
  }
}

/** Names of the vendored builds the record lists. */
function recordedBuildNames(record: BuildRecord | null,): readonly string[] {
  const builds = record?.builds ?? [];
  return builds.map(function name(build,): string {
    return String(build.file,);
  },);
}

/** A reference to a file, without the query string or fragment a browser never sends to the server. */
function withoutQuery(reference: string,): string {
  return reference.split('#',)[0].split('?',)[0];
}

/**
 No page names a host. A page may embed data or link a fragment, because both answer without a network; anything
 carrying a scheme or a protocol-relative host is a dependency this tree cannot serve.
 */
export function externalReferenceFindings({ root, }: { root: string; },): readonly string[] {
  const findings: string[] = [];
  for (const page of designPages(root,)) {
    const html = readText(join(
      root,
      page,
    ),) ?? '';
    for (const reference of pageReferences(html,)) {
      if (classifyReference(reference,) === 'remote') {
        findings.push(`${page}: names ${reference}`);
      }
    }
  }
  return findings;
}

/** Every local name a page makes, and every local name a linked stylesheet makes, resolves to a file in the tree. */
export function unresolvedReferenceFindings({ root, }: { root: string; },): readonly string[] {
  const findings: string[] = [];
  const visited = new Set<string>();
  const pending: string[] = designPages(root,).map(function absolute(page,): string {
    return join(
      root,
      page,
    );
  },);
  while (pending.length > 0) {
    const file = pending.shift();
    if (file === undefined || visited.has(file,)) {
      continue;
    }
    visited.add(file);
    const text = readText(file,) ?? '';
    const references = file.endsWith('.css',)
      ? stylesheetReferences(text,)
      : pageReferences(text,);
    for (const reference of references) {
      if (classifyReference(reference,) !== 'local') {
        continue;
      }
      const target = resolveReference({
        fromFile: file,
        reference: withoutQuery(reference,),
      },);
      if (!existsSync(target,)) {
        findings.push(`${displayPath({
          root,
          file,
        },)}: ${reference} is not in this tree`);
        continue;
      }
      if (target.endsWith('.css',)) {
        pending.push(target,);
      }
    }
  }
  return findings;
}

/** Records a `vendor/` directory holds that are not builds: the record itself and its prose. */
const VENDOR_RECORD_FILES = new Set<string>([
  'README.md',
  'builds.json',
],);

/**
 Every vendored build the record names is present with the digest the record states and its licence text, and no
 file sits in `vendor/` with no record saying where it came from.
 */
export function vendorRecordFindings({ root, }: { root: string; },): readonly string[] {
  const findings: string[] = [];
  const record = readBuildRecord({ root, },);
  if (record === null) {
    return [`${BUILD_RECORD} is missing or unparseable, so no vendored build has a recorded provenance`];
  }
  const accounted = new Set<string>(VENDOR_RECORD_FILES,);
  for (const build of record.builds ?? []) {
    const name = String(build.file,);
    accounted.add(name,);
    const file = join(
      root,
      'vendor',
      name,
    );
    if (!existsSync(file,)) {
      findings.push(`vendor/${name} is recorded and missing`);
      continue;
    }
    const digest = `sha384-${createHash('sha384',).update(readFileSync(file,),).digest('base64',)}`;
    if (digest !== build.integrity) {
      findings.push(`vendor/${name} is ${digest}, and the record states ${String(build.integrity,)}`);
    }
    for (const field of [
      'license',
      'licenseFile',
      'package',
      'version',
    ] as const) {
      if ((build[field] ?? '').length === 0) {
        findings.push(`vendor/${name} records no ${field}`);
      }
    }
    const licenseFile = String(build.licenseFile ?? '',);
    accounted.add(licenseFile,);
    if (licenseFile.length > 0 && !existsSync(join(
      root,
      'vendor',
      licenseFile,
    ),)) {
      findings.push(`vendor/${name} names the licence file ${licenseFile} and no such file is committed`);
    }
  }
  const vendorDirectory = join(
    root,
    'vendor',
  );
  if (existsSync(vendorDirectory,)) {
    for (const entry of readdirSync(vendorDirectory,)) {
      if (!accounted.has(entry,)) {
        findings.push(`vendor/${entry} is committed with no entry in ${BUILD_RECORD}`);
      }
    }
  }
  for (const runtime of record.runtimes ?? []) {
    if (!existsSync(join(
      root,
      String(runtime,),
    ),)) {
      findings.push(`${String(runtime,)} is recorded as a runtime copy and missing`);
    }
  }
  return findings;
}

/**
 The runtime asks this tree for React, ReactDOM and Babel, names every build file the record lists, keeps no
 integrity or cross-origin attribute that a local script does not need and `file://` cannot satisfy, and keeps its
 copies identical apart from the one line saying where `vendor/` sits from that copy.
 */
export function runtimeBuildFindings({ root, }: { root: string; },): readonly string[] {
  const findings: string[] = [];
  const record = readBuildRecord({ root, },);
  if (record === null) {
    return [`${BUILD_RECORD} is missing or unparseable, so no runtime copy is known`];
  }
  const builds = recordedBuildNames(record,);
  const copies = (record.runtimes ?? []).map(function read(name,): {
    name: string;
    text: string;
  } {
    return {
      name: String(name,),
      text: readText(join(
        root,
        String(name,),
      ),) ?? '',
    };
  },);
  if (copies.length < 2) {
    findings.push(`${BUILD_RECORD} names ${copies.length} runtime copies; this guard compares at least two`);
  }
  const normalized = new Set<string>();
  for (const copy of copies) {
    if (copy.text.length === 0) {
      findings.push(`${copy.name}: is recorded and unreadable`);
      continue;
    }
    for (const match of copy.text.matchAll(/https?:\/\/[^\s"')]+/gu,)) {
      findings.push(`${copy.name}: loads ${match[0]} over the network`);
    }
    for (const build of builds) {
      if (!copy.text.includes(build,)) {
        findings.push(`${copy.name}: never asks this tree for ${build}`);
      }
    }
    for (const attribute of [
      'integrity',
      'crossOrigin',
    ]) {
      if (copy.text.includes(attribute,)) {
        findings.push(`${copy.name}: keeps ${attribute}, which a local script does not need and file:// cannot satisfy`);
      }
    }
    normalized.add(copy.text.replace(
      /var VENDOR_BASE = "[^"]*";/u,
      'var VENDOR_BASE = "<vendor base>";',
    ),);
  }
  if (normalized.size > 1) {
    findings.push('the runtime copies differ beyond their vendor base');
  }
  return findings;
}

/**
 A page sets only a typeface this tree ships or one the browser already has, and a shipped typeface it sets is
 defined by a stylesheet it links, so no family silently falls back.
 */
export function shippedFontFindings({ root, }: { root: string; },): readonly string[] {
  const findings: string[] = [];
  const shipped: ReadonlySet<string> = new Set(SHIPPED_FONT_FAMILIES,);
  for (const page of designPages(root,)) {
    const file = join(
      root,
      page,
    );
    const html = readText(file,) ?? '';
    for (const shorthand of fontShorthandValues(html,)) {
      findings.push(`${page}: sets font: ${shorthand}, which hides its family from this guard`);
    }
    const named = new Set<string>();
    for (const value of fontFamilyValues(html,)) {
      for (const name of familyNames(value,)) {
        named.add(name,);
      }
    }
    for (const name of named) {
      if (!ALLOWED_FONT_FAMILIES.has(name,)) {
        findings.push(`${page}: sets ${name}, which this tree does not ship`);
      }
    }
    const defined = new Set<string>();
    for (const reference of pageReferences(html,)) {
      const path = withoutQuery(reference,);
      if (classifyReference(reference,) !== 'local' || !path.endsWith('.css',)) {
        continue;
      }
      const css = readText(resolveReference({
        fromFile: file,
        reference: path,
      },),) ?? '';
      for (const family of facesDefinedBy(css,)) {
        defined.add(family,);
      }
    }
    for (const name of named) {
      if (shipped.has(name,) && !defined.has(name,)) {
        findings.push(`${page}: sets ${name} and links no stylesheet defining it`);
      }
    }
  }
  return findings;
}
