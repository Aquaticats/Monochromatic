/**
 Changelog line generator for changesets, replacing `@changesets/changelog-git`.
 
 The stock generator prefixes each entry with `<hash>: <summary>`, which the
 repository markdown lint rejects (semantic line breaks want a break after
 that colon). This generator keeps the summary lines exactly as written in
 the changeset, one sentence per line, and records the commit on its own
 line underneath.
 
 @module
 */

/**
 Subset of a changeset the release line needs: its summary text and the
 commit that added it, when changesets could attribute one.
 */
type ChangesetWithCommit = {
  readonly summary: string;
  readonly commit?: string;
};

/**
 Subset of an updated dependency the dependency line needs.
 */
type UpdatedDependency = {
  readonly name: string;
  readonly newVersion: string;
};

/**
 Length of the short commit hash shown in changelog entries.
 */
const SHORT_HASH_LENGTH = 7;

/**
 Indents one changelog continuation line under its bullet.
 
 @param line - Continuation line.
 
 @returns Line prefixed with two spaces.
 */
function indent(line: string,): string {
  return `  ${line}`;
}

/**
 Renders the commit attribution sentence for a changeset, or nothing when
 changesets could not attribute a commit.
 
 @param changeset - Changeset being released.
 
 @returns Zero or one sentence naming the short commit hash.
 */
function commitLines(changeset: ChangesetWithCommit,): string[] {
  if (changeset.commit === undefined)
    return [];
  /**
   Short hash shown in the entry.
   */
  const shortHash = changeset.commit
    .slice(
    0,
    SHORT_HASH_LENGTH,
    );
  return [`Commit \`${shortHash}\`.`,];
}

/**
 Changelog functions in the shape changesets expects from a changelog module.
 Both are function expressions with the argument lists changesets dictates.
 */
const changelogFunctions = {
  /**
   Renders one changeset as a changelog bullet. The first summary line
   becomes the bullet text; later lines are indented under it unchanged; the
   commit, when known, closes the entry as its own indented sentence.
   
   @param changeset - Changeset being released.
   
   @returns Markdown bullet for the changelog.
   
   @example
   ```ts
   getReleaseLine({ summary: 'First release.\nSecond sentence.', commit: 'abcdef0123' });
   // => '- First release.\n  Second sentence.\n  Commit `abcdef0`.'
   ```
   */
  getReleaseLine: function getReleaseLine(changeset: ChangesetWithCommit,): string {
    /**
     Summary lines with trailing whitespace removed and blank lines dropped.
     */
    const lines = changeset.summary
      .split('\n',)
      .map(function trimEnd(line,) {
        return line.trimEnd();
      },)
      .filter(function isNotBlank(line,) {
        return line.length > 0;
      },);
    /**
     Bullet text: the first summary line, or a placeholder for an empty summary.
     */
    const [
      first = '(no summary)',
      ...rest
    ] = lines;
    /**
     Indented continuation lines, plus the commit line when a commit is known.
     */
    const continuation = [
      ...rest,
      ...commitLines(changeset,),
    ]
      .map(indent,);
    return [
      `- ${first}`,
      ...continuation,
    ]
      .join('\n',);
  },

  /**
   Renders the "updated dependencies" bullet for a release, one nested bullet
   per bumped workspace dependency.
   
   @param changesets - Changesets that caused the dependency bumps; unused
   beyond signalling that at least one exists.
   
   @param dependenciesUpdated - Workspace dependencies whose version changed.
   
   @returns Markdown bullet with nested dependency bullets, or an empty string
   when nothing was updated.
   
   @example
   ```ts
   getDependencyReleaseLine([], [{ name: 'a', newVersion: '1.2.3' }]);
   // => '- Updated dependencies:\n  - a\@1.2.3'
   ```
   */
  getDependencyReleaseLine: function getDependencyReleaseLine(
    changesets: readonly ChangesetWithCommit[],
    dependenciesUpdated: readonly UpdatedDependency[],
  ): string {
    if ((changesets.length === 0) || (dependenciesUpdated.length === 0))
      return '';
    /**
     One nested bullet per updated dependency.
     */
    const nested = dependenciesUpdated.map(function toBullet(dependency,) {
      return `  - ${dependency.name}@${dependency.newVersion}`;
    },);
    return [
      '- Updated dependencies:',
      ...nested,
    ]
      .join('\n',);
  },
};

export default changelogFunctions;
