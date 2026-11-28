/**
 The arms of the forced removal's guest work, as this package names them on a command line, and the two strings a run
 builds from a selection: the nextest filterset and the lines each arm prints when it performed its scene.

 Every name lives here rather than in the driver, because the driver that runs an arm and the person who reads the log
 name the same arm by the same word, and a table in one file is what keeps the two from drifting.
 */

/**
 One arm's name on the command line, the test it selects, and the lines it prints when it performed its scene.
 */
export type RemovalForceArm = {
  /**
   The word `--arms` takes for this arm, such as `4a`.
   */
  readonly only: string;
  /**
   The substring of the test's own name, which is also what nextest's filterset matches.
   */
  readonly test: string;
  /**
   The lines the arm prints when it performed what it claims.
   */
  readonly markers: readonly string[];
};

/**
 Every arm this driver can run, in the order the brief reads them.
 */
export const REMOVAL_FORCE_ARMS: readonly RemovalForceArm[] = [
  {
    only: '1',
    test: 'the_staging_folder_arm',
    markers: [
      'REMOVAL-FORCE arm=1 one-force=unmounted-there-then-destroyed receipt=done staging=gone',
    ],
  },
  {
    only: '2',
    test: 'the_snapshot_batch_arm',
    markers: [
      'REMOVAL-FORCE arm=2 live-proof=pool-list',
    ],
  },
  {
    only: '3',
    test: 'a_clone_the_wrapper_cannot_prove_arm',
    markers: [
      'REMOVAL-FORCE arm=3 refused=none-and-one destroyed=two',
    ],
  },
  {
    only: '4a',
    test: 'with_no_force_arm',
    markers: [
      'REMOVAL-FORCE arm=4a no-force=refused settling=refused plan=kept',
    ],
  },
  {
    only: '4b',
    test: 'with_one_force_arm',
    markers: [
      'REMOVAL-FORCE arm=4b one-force=destroyed',
    ],
  },
  {
    only: '4c',
    test: 'with_two_forces_arm',
    markers: [
      'REMOVAL-FORCE arm=4c two-forces=destroyed',
    ],
  },
  {
    only: '4d',
    test: 'the_clone_gone_arm',
    markers: [
      'REMOVAL-FORCE arm=4d left-with=no-route plan=kept clone=gone settling=refused',
    ],
  },
  {
    only: '5',
    test: 'a_persons_lock_arm',
    markers: [
      'REMOVAL-FORCE arm=5 two-forces=lock-deleted-and-worktree-removed',
    ],
  },
  {
    only: '6a',
    test: 'a_modified_file_arm',
    markers: [
      'REMOVAL-FORCE arm=6a modified=refused-without-force removed-with-one',
    ],
  },
  {
    only: '6b',
    test: 'an_untracked_file_arm',
    markers: [
      'REMOVAL-FORCE arm=6b untracked=refused-without-force removed-with-one',
    ],
  },
  {
    only: '6c',
    test: 'a_staged_change_arm',
    markers: [
      'REMOVAL-FORCE arm=6c staged=ahead-of-git-s-own-rule-and-removed',
    ],
  },
];

/**
 The nextest filterset selecting the arms one run named.

 @param arms - the selected arms

 @returns one filterset matching each selected test, joined by the operator nextest reads as "or"

 @example
 ```ts
 const filterset = filtersetOf({ arms: REMOVAL_FORCE_ARMS.slice(0, 2) }); // 'test(the_staging_folder_arm) | test(the_snapshot_batch_arm)'
 ```
 */
export function filtersetOf({ arms, }: { readonly arms: readonly RemovalForceArm[]; },): string {
  /**
   The one predicate for each selected test, in the order the catalog holds them.
   */
  const tests: string[] = [];
  for (const selected of arms) {
    tests.push(`test(${selected.test})`,);
  }
  return tests.join(' | ',);
}

/**
 The lines the selected arms promise.

 @param arms - the selected arms

 @returns every marker line, in arm order

 @example
 ```ts
 const markers = markersOf({ arms: REMOVAL_FORCE_ARMS.slice(0, 1) });
 ```
 */
export function markersOf({ arms, }: { readonly arms: readonly RemovalForceArm[]; },): readonly string[] {
  /**
   Every line a selected arm prints, in the order the arms were selected.
   */
  const markers: string[] = [];
  for (const selected of arms) {
    markers.push(...selected.markers);
  }
  return markers;
}

/**
 The arms one `--arms` word selects, or every arm when the word is empty.

 @param word - the comma-separated arm names the command line gave

 @returns the selected arms, in the catalog's order

 @throws Error naming a word the catalog does not hold

 @example
 ```ts
 const selected = selectedArms({ word: '1,6a' });
 ```
 */
export function selectedArms({ word, }: { readonly word: string; },): readonly RemovalForceArm[] {
  if (word === '') return REMOVAL_FORCE_ARMS;
  /**
   The arm names the command line gave, split on its commas.
   */
  const wanted: readonly string[] = word.split(',',);
  /**
   The names the catalog does not hold, which a run refuses rather than skipping.
   */
  const unknown: readonly string[] = wanted.filter(function notInTheCatalog(only) {
    return !REMOVAL_FORCE_ARMS.some(function holdsIt(known) {
      return known.only === only;
    },);
  },);
  if (unknown.length > 0) {
    /**
     The arm names the catalog does hold, which a refusal names so a typo is answered with the words that work.
     */
    const known: string = REMOVAL_FORCE_ARMS
      .map(function onlyOf(arm) {
        return arm.only;
      },)
      .join(', ',);
    throw new Error(`unknown arm(s) ${unknown.join(', ',)}; use ${known}`,);
  }
  return REMOVAL_FORCE_ARMS.filter(function wasNamed(known) {
    return wanted.includes(known.only,);
  },);
}
