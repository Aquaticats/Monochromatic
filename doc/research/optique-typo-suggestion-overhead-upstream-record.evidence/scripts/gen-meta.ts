// Collects the evidence that pins this measurement: tarball digests, publish
// times, JSR version-to-commit strings, and whether each installed tree holds
// the deferred-failure module pull request 997 added.
// Run on the host: node scripts/gen-meta.ts

import { execFileSync, } from 'node:child_process';
import { createHash, } from 'node:crypto';
import {
  existsSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';

//region Config

/**
 * Every measured version, in the order the report lists them.
 */
const VERSIONS: readonly string[] = [
  '1.1.1',
  '1.3.1',
  '1.3.2',
  '1.3.6',
  '1.4.0-dev.2664',
  '1.4.0-dev.2665',
  '1.4.0-dev.2667',
  '1.4.0-dev.2971',
];

/**
 * Scratch root, one level above the scripts directory.
 */
const ROOT = new URL('..', import.meta.url,).pathname;

/**
 * Version the migration commit used, as recorded in the migration record.
 */
const MIGRATION_VERSION = '1.1.1';

//endregion Config

//region Evidence

/**
 * Digests one file.
 *
 * @param path - File to digest.
 *
 * @returns Base64 SHA-512 and hex SHA-1 digests.
 */
function digestFile(path: string,): { sha512: string; sha1: string; } {
  /** File contents. */
  const bytes = readFileSync(path,);

  return {
    sha512: `sha512-${createHash('sha512',).update(bytes,).digest('base64',)}`,
    sha1: createHash('sha1',).update(bytes,).digest('hex',),
  };
}

/**
 * Reads the per-version publish times npm reports.
 *
 * @returns Publish timestamp per version.
 */
function npmPublishTimes(): Record<string, string> {
  /** Raw `npm view` output for the time map. */
  const raw = execFileSync(
    'npm',
    [
      'view',
      '@optique/core',
      'time',
      '--json',
    ],
    {
      encoding: 'utf8',
      maxBuffer: 32 * 1024 * 1024,
    },
  );

  /** Publish time per version. */
  const parsed = JSON.parse(raw,) as Record<string, string>;
  /** Subset covering the measured versions. */
  const selected: Record<string, string> = {};

  for (const version of VERSIONS) {
    /** Timestamp npm reports for this version. */
    const time = parsed[version];

    if (time === undefined) {
      throw new Error(`npm reports no publish time for ${version}`,);
    }
    selected[version] = time;
  }

  return selected;
}

/**
 * Reads JSR's version strings, which carry the source commit after a `+`.
 *
 * @returns Source commit per measured version, where JSR published one.
 */
function jsrSourceCommits(): Record<string, string> {
  /** JSR metadata for the package. */
  const meta = JSON.parse(
    readFileSync(
      join(
        ROOT,
        'ctx/jsr-core-meta.json',
      ),
      'utf8',
    ),
  ) as { versions?: Record<string, unknown>; };

  /** Source commit per version. */
  const commits: Record<string, string> = {};

  for (const versionWithCommit of Object.keys(meta.versions ?? {},)) {
    /** Version prefix before the commit suffix. */
    const [version, commit,] = versionWithCommit.split('+',);

    if (version === undefined || commit === undefined) {
      continue;
    }
    if (VERSIONS.includes(version,)) {
      commits[version] = commit;
    }
  }

  return commits;
}

/**
 * Reports whether an installed tree holds the deferred-failure module and its
 * exported factory, which is what pull request 997 introduced.
 *
 * @param version - Installed version to inspect.
 *
 * @returns Presence evidence for the module and the factory.
 */
function deferredFailureEvidence(version: string,): {
  failureModule: boolean;
  createDeferredFailure: boolean;
  filesUsingIt: readonly string[];
} {
  /** Unpacked package root for this version. */
  const packageRoot = join(
    ROOT,
    'ctx/unpack',
    version,
    'package',
  );
  /** Candidate module paths, ESM and CJS. */
  const modulePaths = [
    join(
      packageRoot,
      'dist/internal/failure.js',
    ),
    join(
      packageRoot,
      'dist/internal/failure.cjs',
    ),
  ];
  /** Whether either shipped module exists. */
  const failureModule = modulePaths.some(function exists(path,) {
    return existsSync(path,);
  },);
  /** Whether any shipped file defines or imports the factory. */
  const usingFiles: string[] = [];

  if (existsSync(packageRoot,)) {
    /** Every shipped dist file. */
    const distFiles = readdirSync(
      join(
        packageRoot,
        'dist',
      ),
      { recursive: true, },
    ).map(function asString(entry,) {
      return String(entry,);
    },);

    for (const file of distFiles) {
      if (!file.endsWith('.js',)) {
        continue;
      }
      /** File text, for a plain substring search. */
      const text = readFileSync(
        join(
          packageRoot,
          'dist',
          file,
        ),
        'utf8',
      );

      if (text.includes('createDeferredFailure',)) {
        usingFiles.push(file,);
      }
    }
  }

  return {
    failureModule,
    createDeferredFailure: usingFiles.length > 0,
    filesUsingIt: usingFiles.sort(),
  };
}

//endregion Evidence

//region Entry point

/**
 * Writes `ctx/meta.json` with every pinned fact about the measured versions.
 */
function main(): void {
  /** Publish times from npm. */
  const publishTimes = npmPublishTimes();
  /** Source commits from JSR version strings. */
  const sourceCommits = jsrSourceCommits();
  /** Evidence per version. */
  const versions = VERSIONS.map(function describe(version,) {
    /** Tarball path for this version. */
    const tarball = join(
      ROOT,
      'ctx/tarballs',
      `optique-core-${version}.tgz`,
    );

    return {
      version,
      tarballSha512: digestFile(tarball,).sha512,
      tarballSha1: digestFile(tarball,).sha1,
      npmPublishedAt: publishTimes[version],
      jsrSourceCommit: sourceCommits[version] ?? null,
      shipped: deferredFailureEvidence(version,),
      installedPackageJsonVersion: JSON.parse(
        readFileSync(
          join(
            ROOT,
            'ctx/unpack',
            version,
            'package/package.json',
          ),
          'utf8',
        ),
      ).version,
    };
  },);

  /** Meta document baked into the image. */
  const meta = {
    purpose: 'Measure the old git-policy commit parser against @optique/core versions.',
    parserSource: {
      repository: 'Aquaticat/Monochromatic',
      migrationCommit: '4879a44e6a2460ab3c2531744e24a1f2aef27aeb',
      parentCommit: '1e53f52e02989b152576c0683f6f228b24c40140',
      migrationVersionUsed: MIGRATION_VERSION,
      files: [
        'package/git-policy/cli/src/parser/commit.ts',
        'package/git-policy/cli/src/parser/commit-normalise.ts',
        'package/git-policy/cli/src/parser/commit-flag-aliases.ts',
        'package/git-policy/cli/src/escape-hatch.ts',
        'package/git-policy/cli/src/abbrev.ts',
      ],
    },
    upstream: {
      issue988: 'https://github.com/dahlia/optique/issues/988',
      issue989: 'https://github.com/dahlia/optique/issues/989',
      issue990: 'https://github.com/dahlia/optique/issues/990',
      issue991: 'https://github.com/dahlia/optique/issues/991',
      pull992: 'https://github.com/dahlia/optique/pull/992',
      pull997: 'https://github.com/dahlia/optique/pull/997',
      pull997MergeCommit: '68b5edd17d0644f60aab8a6e30a4bc88c3594bf5',
      pull997OnlyCommit: '868a9d77139433608178eb3453bf9aaafccd0066',
      pull997BaseCommit: '6f25829653f',
    },
    ownedParser: {
      description: 'Baseline arm: the repository-owned parser that replaced Optique, no Optique at all.',
      commit: '4879a44e6a2460ab3c2531744e24a1f2aef27aeb',
      files: [
        'package/git-policy/cli/src/parser/commit.ts',
        'package/git-policy/cli/src/parser/argv.ts',
        'package/git-policy/cli/src/parser/commit-normalise.ts',
        'package/git-policy/cli/src/parser/commit-flag-aliases.ts',
        'package/git-policy/cli/src/escape-hatch.ts',
        'package/git-policy/cli/src/abbrev.ts',
      ],
      runtimeDependency: {
        package: 'valibot',
        version: '1.4.2',
        chosenBecause: 'pnpm-lock.yaml resolves the catalog range at that commit to this version',
        tarballSha512: digestFile(
          join(
            ROOT,
            'ctx/tarballs/valibot-1.4.2.tgz',
          ),
        ).sha512,
      },
    },
    versions,
  };

  writeFileSync(
    join(
      ROOT,
      'ctx/meta.json',
    ),
    `${JSON.stringify(
      meta,
      null,
      2,
    )}\n`,
  );

  for (const entry of versions) {
    console.log(
      `[meta] ${entry.version} npm=${entry.npmPublishedAt} jsr=${
        entry.jsrSourceCommit ?? '(none)'
      } failureModule=${entry.shipped.failureModule} createDeferredFailure=${entry.shipped.createDeferredFailure} users=${
        entry.shipped.filesUsingIt.length
      }`,
    );
  }
}

main();

//endregion Entry point
