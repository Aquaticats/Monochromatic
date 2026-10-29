/**
 Restores the plain-file fixtures the retired incumbent wrapper left, for `old-format-fixtures.mjs`.

 The fixtures under `native-interop/fixture/` store every `.git` directory as `dot-git`,
 and list the symbolic links the incumbent's shadow repositories hold in the fixture's `links.tsv`
 (path relative to the fixture, a tab, the absolute target).
 Each fixture is restored to the absolute location it was captured at, because its records name absolute paths.
 */
/// <reference types="node" />
import {
  chmod,
  copyFile,
  mkdir,
  readdir,
  readFile,
  rm,
  symlink,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  InteropError,
  isMissingPath,
  namesIn,
  REAL_GIT,
  run,
  runEnvironment,
} from './support.mjs';

/** Where the fixtures sit in the image. */
const FIXTURE_SOURCE = '/fixture/native-interop/fixture';
/** Where each fixture is restored: the location it was captured at. */
const FIXTURE_TARGET = '/work/scratch/fixture';
/** Directory name that stands for `.git` in the stored fixtures. */
const STORED_GIT_NAME = 'dot-git';
/** Mode of every restored directory and of restored scripts. */
const PRIVATE_EXECUTABLE_MODE = 0o700;
/** Mode of every other restored file. */
const PRIVATE_FILE_MODE = 0o600;
/** Empty directories a repository needs but a stored tree cannot hold. */
const REPOSITORY_DIRECTORIES = [
  '.git/objects/pack',
  '.git/objects/info',
  '.git/refs/heads',
  '.git/refs/tags',
  '.git/hooks',
];

/** Suffix of a pack file. */
const PACK_SUFFIX = '.pack';
/** Suffix of the index that belongs to a pack file. */
const INDEX_SUFFIX = '.idx';

/**
 One restored fixture.
 @typedef {{ root: string, repository: string, home: string }} RestoredFixture
 */

/**
 Restore one stored directory tree, renaming `dot-git` entries back to `.git` and applying private modes.

 @param {{ from: string, to: string }} request - stored directory and destination
 */
async function restoreTree({
  from,
  to,
}) {
  await mkdir(
    to,
    {
      recursive: true,
      mode: PRIVATE_EXECUTABLE_MODE,
    },
  );
  await chmod(
    to,
    PRIVATE_EXECUTABLE_MODE,
  );
  const entries = await readdir(
    from,
    { withFileTypes: true },
  );
  // Every entry lands at its own destination, so the copies do not depend on each other.
  await Promise.all(entries.map(async function restoreEntry(entry) {
    const name = entry.name === STORED_GIT_NAME ? '.git' : entry.name;
    const source = join(
      from,
      entry.name,
    );
    const destination = join(
      to,
      name,
    );
    if (entry.isDirectory()) {
      await restoreTree({
        from: source,
        to: destination,
      });
      return;
    }
    await copyFile(
      source,
      destination,
    );
    await chmod(
      destination,
      name.endsWith('.sh') ? PRIVATE_EXECUTABLE_MODE : PRIVATE_FILE_MODE,
    );
  }));
}

/**
 Parse the manifest of symbolic links.

 @param {string} text - manifest text, one `path<TAB>target` pair per line
 @returns {{ path: string, target: string }[]} the links to create
 @throws {InteropError} when a line is not exactly a path and a target
 */
function parseLinks(text) {
  return text
    .split('\n')
    .filter(function isNonEmpty(line) {
      return line.length > 0;
    })
    .map(function parseLink(line) {
      const [path, target, ...rest] = line.split('\t');
      if ((path === undefined) || (target === undefined)
        || (rest.length > 0)
        || (path === '')
        || (target === ''))
        throw new InteropError(`A fixture link manifest line is not a path, a tab and a target: ${line}`);
      return {
        path,
        target,
      };
    });
}

/**
 Read a fixture's link manifest, which a fixture without symbolic links does not have.

 @param {string} manifest - manifest path
 @returns {Promise<{ path: string, target: string }[]>} the links to create, none when the manifest is absent
 @throws {Error} when the manifest cannot be read for any reason other than being absent, or is malformed
 */
async function readLinks(manifest) {
  try {
    return parseLinks(await readFile(
      manifest,
      'utf8',
    ));
  }
  catch (error) {
    if (isMissingPath(error))
      return [];
    throw error;
  }
}

/**
 Rebuild the index of every pack that the stored tree holds without one.

 The repository ignores `*.idx`, so the committed fixtures carry packs without their indexes, and Git cannot read a
 pack without its index. The index is derived from the pack alone, so rebuilding it restores what the incumbent wrote.

 @param {{ repository: string, home: string }} fixture - restored fixture
 @throws {InteropError} when Git cannot index a pack
 */
async function rebuildPackIndexes(fixture) {
  const packDirectory = join(
    fixture.repository,
    '.git/objects/pack',
  );
  const names = await namesIn(packDirectory);
  const unindexed = names.filter(function lacksIndex(name) {
    return name.endsWith(PACK_SUFFIX)
      && (!names.includes(`${name.slice(
        0,
        -PACK_SUFFIX.length,
      )}${INDEX_SUFFIX}`));
  });
  await Promise.all(unindexed.map(async function indexPack(name) {
    const outcome = await run({
      command: REAL_GIT,
      args: [
        'index-pack',
        join(
          packDirectory,
          name,
        ),
      ],
      cwd: fixture.repository,
      env: runEnvironment({ home: fixture.home }),
    });
    if (outcome.status !== 0)
      throw new InteropError(`git index-pack ${name} exited ${String(outcome.status)}: ${outcome.stderr}`);
  }));
}

/**
 Restore one fixture and the empty directories a repository needs but a stored tree cannot hold.

 @param {string} name - fixture name
 @returns {Promise<RestoredFixture>} the restored fixture
 */
export async function restoreFixture(name) {
  const root = join(
    FIXTURE_TARGET,
    name,
  );
  await restoreTree({
    from: join(
      FIXTURE_SOURCE,
      name,
    ),
    to: root,
  });
  const repository = join(
    root,
    'repository',
  );
  const home = join(
    root,
    'home',
  );
  await mkdir(
    home,
    { recursive: true },
  );
  const manifest = join(
    root,
    'links.tsv',
  );
  const links = await readLinks(manifest);
  await Promise.all(links.map(async function restoreLink(link) {
    await symlink(
      link.target,
      join(
        root,
        link.path,
      ),
    );
  }));
  await rm(
    manifest,
    { force: true },
  );
  await Promise.all(REPOSITORY_DIRECTORIES.map(async function ensureDirectory(directory) {
    await mkdir(
      join(
        repository,
        directory,
      ),
      { recursive: true },
    );
  }));
  await rebuildPackIndexes({
    repository,
    home,
  });
  return {
    root,
    repository,
    home,
  };
}

/**
 Read the capture store's sequence number.

 @param {RestoredFixture} fixture - restored fixture
 @returns {Promise<number>} the last allocated sequence
 */
export async function captureSequence(fixture) {
  const text = await readFile(
    join(
      fixture.repository,
      '.git/cli-git-captures/sequence',
    ),
    'utf8',
  );
  return Number(text.trim());
}
