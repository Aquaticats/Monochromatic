// A step's shell lines: leave the guest running the module this recipe built.
//
// What:  builds the shell a step runs before it touches a pool, mounts a dataset or
//        reads the kernel's type information for the module. It reads whether each of
//        the two modules is loaded and the version each of them publishes, then: says
//        the guest already runs what this recipe built and changes nothing; inserts the
//        built modules where none is loaded; removes another module it can remove and
//        inserts the built ones; or refuses in words naming the version it found, the
//        version it built and what holds the module it could not remove.
// Why:   a guest the same recipe prepared already runs the built module, and a driver
//        that made pools there holds it. `modprobe --remove` then fails with `Module
//        zfs is in use`, the step this replaces passed that over with `|| true`, and
//        the `insmod` after it died with `File exists` (issue 6638).
// Why the words: a person reading the log has to know which module is loaded, which
//        one was built and what holds the loaded one. An imported pool is the holder
//        this recipe meets, so the refusal names the pools the kernel itself lists.
// Why the version is the whole identity: both sides are read from the value the
//        module carries, so two builds of one release read alike. A repeat `prepare`
//        wants exactly that, because the same tarball rebuilt is already there. A step
//        that needs a rebuilt module loaded starts with nothing loaded, which is how
//        the two gate arms work: they destroy every pool and reboot first.

/** The words a step prints when the loaded module is the one this recipe built. */
export const ALREADY_RUNNING_WORDS = 'the guest already runs the module this recipe built';

/** The words a step prints when no module of either kind is loaded. */
export const NOTHING_LOADED_WORDS = 'no zfs or spl module is loaded';

/** The words a step prints when it replaces a module this recipe did not build. */
export const OTHER_MODULE_WORDS = 'the guest runs another module';

/** The words a step prints when it will not insert the module this recipe built. */
export const REFUSAL_WORDS = 'refusing to insert the module this recipe built';

/** The words a step prints once the guest runs the module this recipe built. */
export const NOW_RUNNING_WORDS = 'the guest now runs the module this recipe built';

/** One step's shell lines, and where they read the trees they ask. */
export interface LoadTheBuiltModuleStep {
  /**
   The step's own name, which every line it prints starts with, so a guest log with
   four steps in it says which one spoke.

   A fixed word from this recipe, never text from outside it: it is written into the
   shell this builds.
   */
  readonly label: string;
  /**
   Where the kernel exposes its loaded modules.
   A literal path, never text from outside this recipe; a guest step takes the default.
   */
  readonly sysDirectory?: string;
  /**
   Where the kernel exposes the pools it holds.
   A literal path, never text from outside this recipe; a guest step takes the default.
   */
  readonly procDirectory?: string;
  /**
   Where this recipe installs its modules.
   A literal path, never text from outside this recipe; a guest step takes the default.
   */
  readonly modulesDirectory?: string;
}

/**
 The shell lines that leave the guest running the module this recipe built, or refuse.

 The three directories are parameters because the check beside this file
 (`load-the-built-module-step.check.ts`) runs this shell on a host against stand-in
 trees, which is the only way to hold its states without a guest. No guest step
 passes one: the defaults are the guest's own paths.

 @param label - the step's own name, which every line it prints starts with

 @param sysDirectory - where the kernel exposes its loaded modules

 @param procDirectory - where the kernel exposes the pools it holds

 @param modulesDirectory - where this recipe installs its modules

 @returns the shell lines, to be joined into a script that runs as root in the guest

 @example
 ```ts
 const script = ['set -eu', 'depmod --all', ...loadTheBuiltModuleStep({ label: 'prepare' })].join('\n');
 ```
 */
export function loadTheBuiltModuleStep({
  label,
  sysDirectory = '/sys',
  procDirectory = '/proc',
  modulesDirectory = '/lib/modules',
}: LoadTheBuiltModuleStep,): readonly string[] {
  return [
    // Where this shell reads, which no guest step overrides.
    `module_step_sys=${sysDirectory}`,
    `module_step_proc=${procDirectory}`,
    `module_step_modules=${modulesDirectory}`,
    // Both module files this recipe installs, and the version each one carries.
    // `modinfo` answers from the file this recipe just installed, so "built" is that
    // file's own word and not a constant this recipe could drift from.
    'module_step_kernel=$(uname --kernel-release)',
    'module_step_built_spl=$module_step_modules/$module_step_kernel/extra/spl.ko',
    'module_step_built_zfs=$module_step_modules/$module_step_kernel/extra/zfs.ko',
    'if test ! -f "$module_step_built_spl" || test ! -f "$module_step_built_zfs"; then',
    `  echo "${label}: ${REFUSAL_WORDS}: one of the two module files this recipe installs is not there ($module_step_built_spl, $module_step_built_zfs), so nothing says which version this recipe built" >&2`,
    '  echo "  build and install the module (make install, then depmod --all) and run this step again" >&2',
    '  exit 1',
    'fi',
    // Each answer is captured with its status, so a `modinfo` that fails ends the step
    // in words instead of taking `set -e` out with no sentence.
    'if ! module_step_answered=$(modinfo --field version "$module_step_built_zfs" 2>&1); then',
    `  echo "${label}: ${REFUSAL_WORDS}: modinfo could not read a version from $module_step_built_zfs, and it said: $module_step_answered" >&2`,
    '  exit 1',
    'fi',
    'module_step_built_zfs_version=$module_step_answered',
    'if test -z "$module_step_built_zfs_version"; then',
    `  echo "${label}: ${REFUSAL_WORDS}: $module_step_built_zfs states no version, so nothing here can tell which module a guest runs" >&2`,
    '  exit 1',
    'fi',
    // An spl build may record no version, and then there is no version of it to
    // compare: `none` says so, and only spl may read as `none`.
    'module_step_built_spl_version=none',
    'if ! module_step_answered=$(modinfo --field version "$module_step_built_spl" 2>&1); then',
    `  echo "${label}: ${REFUSAL_WORDS}: modinfo could not read a version from $module_step_built_spl, and it said: $module_step_answered" >&2`,
    '  exit 1',
    'fi',
    'if test -n "$module_step_answered"; then module_step_built_spl_version=$module_step_answered; fi',
    // What is loaded now, in one place: the module's own /sys directory says whether it
    // is there at all, which a version file cannot (a build may publish none), and the
    // version file says which build it is. A refusal after a half-finished removal or
    // insert asks again, so it says what the guest is left running.
    'module_step_read_loaded() {',
    '  module_step_now_zfs=none',
    '  if test -d "$module_step_sys/module/zfs"; then',
    '    module_step_now_zfs=unknown',
    '    if test -f "$module_step_sys/module/zfs/version"; then module_step_now_zfs=$(cat "$module_step_sys/module/zfs/version"); fi',
    '  fi',
    '  module_step_now_spl=none',
    '  if test -d "$module_step_sys/module/spl"; then',
    '    module_step_now_spl=unknown',
    '    if test -f "$module_step_sys/module/spl/version"; then module_step_now_spl=$(cat "$module_step_sys/module/spl/version"); fi',
    '  fi',
    '}',
    'module_step_read_loaded',
    'module_step_zfs_loaded=no',
    'if test "$module_step_now_zfs" != none; then module_step_zfs_loaded=yes; fi',
    'module_step_loaded_zfs_version=$module_step_now_zfs',
    'module_step_spl_loaded=no',
    'if test "$module_step_now_spl" != none; then module_step_spl_loaded=yes; fi',
    'module_step_loaded_spl_version=$module_step_now_spl',
    // The three states the two readings name, and the fourth of a guest that has
    // loaded nothing yet, which a fresh guest reaches. zfs's version is the identity,
    // because that is the release every step of this recipe pins. spl's version is
    // compared where both sides record one, and where the build records none a loaded
    // spl that publishes none too is the only one this can read as the built module:
    // one that publishes another version is another build, and is replaced.
    'module_step_state=nothing',
    'if test "$module_step_zfs_loaded" = yes || test "$module_step_spl_loaded" = yes; then module_step_state=other; fi',
    'if test "$module_step_zfs_loaded" = yes && test "$module_step_loaded_zfs_version" = "$module_step_built_zfs_version" && test "$module_step_spl_loaded" = yes; then',
    '  if test "$module_step_built_spl_version" = none && test "$module_step_loaded_spl_version" = unknown; then module_step_state=already; fi',
    '  if test "$module_step_loaded_spl_version" = "$module_step_built_spl_version"; then module_step_state=already; fi',
    'fi',
    // The built module is loaded: this is the guest a repeat `prepare` meets, and the
    // state the whole step exists to end well in. Nothing is removed and nothing is
    // inserted, so the pools a guest's arms made keep running.
    'if test "$module_step_state" = already; then',
    `  echo "${label}: ${ALREADY_RUNNING_WORDS}: zfs $module_step_loaded_zfs_version against $module_step_built_zfs stating $module_step_built_zfs_version, spl $module_step_loaded_spl_version against $module_step_built_spl stating $module_step_built_spl_version; nothing was removed and nothing was inserted"`,
    'else',
    '  if test "$module_step_state" = nothing; then',
    `    echo "${label}: ${NOTHING_LOADED_WORDS}; inserting the modules this recipe built (zfs $module_step_built_zfs_version, spl $module_step_built_spl_version)"`,
    '  else',
    `    echo "${label}: ${OTHER_MODULE_WORDS} (zfs $module_step_loaded_zfs_version, spl $module_step_loaded_spl_version); removing it before inserting the modules this recipe built (zfs $module_step_built_zfs_version, spl $module_step_built_spl_version)"`,
    // zfs rests on spl, so zfs comes out first, and each removal is checked: a removal
    // that fails is a refusal here, never a line passed over with `|| true` before an
    // insert that would meet a module already there.
    '    module_step_removal_problem=',
    '    module_step_modprobe_output=',
    '    if test "$module_step_zfs_loaded" = yes; then',
    '      if module_step_modprobe_output=$(modprobe --remove zfs 2>&1); then module_step_removal_problem=; else module_step_removal_problem=zfs; fi',
    '      if test -d "$module_step_sys/module/zfs"; then module_step_removal_problem=zfs; fi',
    '      if test -z "$module_step_removal_problem"; then',
    `        echo "${label}: removed zfs $module_step_loaded_zfs_version"`,
    '      fi',
    '    fi',
    '    if test -z "$module_step_removal_problem" && test "$module_step_spl_loaded" = yes; then',
    '      if module_step_modprobe_output=$(modprobe --remove spl 2>&1); then module_step_removal_problem=; else module_step_removal_problem=spl; fi',
    '      if test -d "$module_step_sys/module/spl"; then module_step_removal_problem=spl; fi',
    '      if test -z "$module_step_removal_problem"; then',
    `        echo "${label}: removed spl $module_step_loaded_spl_version"`,
    '      fi',
    '    fi',
    // A module that would not come out is held by something. The holder this recipe
    // measured in its own guest is a mounted dataset: with one dataset mounted the
    // kernel counted one reference, and with only a pool imported it counted none and
    // the module came out (`doc/prepare-in-a-prepared-guest-2026-11-27.md`). So the
    // refusal reads the mounts and the pools from the kernel's own /proc, which needs
    // no userspace of any release and cannot disagree with the module about its
    // version, and names what a person has to release.
    '    if test -n "$module_step_removal_problem"; then',
    '      module_step_read_loaded',
    `      echo "${label}: ${REFUSAL_WORDS}: this guest runs a module this recipe did not build, and module $module_step_removal_problem would not go." >&2`,
    '      echo "  loaded: zfs $module_step_loaded_zfs_version, spl $module_step_loaded_spl_version" >&2',
    '      echo "  built: zfs $module_step_built_zfs_version ($module_step_built_zfs), spl $module_step_built_spl_version ($module_step_built_spl)" >&2',
    '      if test -n "$module_step_modprobe_output"; then',
    '        echo "  modprobe --remove $module_step_removal_problem said: $module_step_modprobe_output" >&2',
    '      else',
    '        echo "  modprobe --remove $module_step_removal_problem printed nothing, and module $module_step_removal_problem is still loaded" >&2',
    '      fi',
    '      module_step_refcount=unknown',
    '      if test -f "$module_step_sys/module/$module_step_removal_problem/refcnt"; then module_step_refcount=$(cat "$module_step_sys/module/$module_step_removal_problem/refcnt"); fi',
    '      echo "  the kernel counts $module_step_refcount references to module $module_step_removal_problem" >&2',
    '      module_step_mounts=',
    '      if test -r "$module_step_proc/self/mountinfo"; then',
    '        while read -r module_step_line; do',
    '          case "$module_step_line" in',
    '            *" - zfs "*)',
    '              module_step_dataset=${module_step_line#* - zfs }',
    '              module_step_dataset=${module_step_dataset%% *}',
    '              module_step_mounts="$module_step_mounts $module_step_dataset"',
    '              ;;',
    '          esac',
    '        done < "$module_step_proc/self/mountinfo"',
    '      fi',
    '      module_step_pools=',
    '      if test -d "$module_step_proc/spl/kstat/zfs"; then',
    '        for module_step_entry in "$module_step_proc"/spl/kstat/zfs/*/; do',
    '          module_step_pool=${module_step_entry%/}',
    '          module_step_pool=${module_step_pool##*/}',
    '          if test -d "$module_step_proc/spl/kstat/zfs/$module_step_pool"; then module_step_pools="$module_step_pools $module_step_pool"; fi',
    '        done',
    '      fi',
    '      echo "  the kernel lists these ZFS datasets mounted:$module_step_mounts" >&2',
    '      echo "  the kernel lists these pools imported:$module_step_pools" >&2',
    '      echo "  a mounted dataset is what holds module zfs: this recipe measured the count at 1 with one dataset mounted, and at 0 with a pool imported and nothing mounted, where modprobe took the module out" >&2',
    '      echo "  to run the module this recipe built: unmount each mounted dataset (zfs unmount <dataset>, or umount its mountpoint), export each pool that may go (zpool export <pool>) or destroy it when what it holds may be destroyed (zpool destroy <pool>), stop anything else holding /dev/zfs (zfs-zed opens it), and run this step again" >&2',
    '      echo "  the guest is left running: zfs $module_step_now_zfs, spl $module_step_now_spl; to start from nothing instead, reboot the guest and run this step again, which reads again what is loaded" >&2',
    '      exit 1',
    '    fi',
    '  fi',
    // The insert, checked: an insert that fails is named with the file it tried and
    // insmod's own words, a module that comes up as another version is a refusal too,
    // and either says what the guest is left running.
    '  module_step_insert_problem=',
    '  module_step_insmod_output=',
    '  if ! module_step_insmod_output=$(insmod "$module_step_built_spl" 2>&1); then',
    '    module_step_insert_problem="insmod $module_step_built_spl failed with: $module_step_insmod_output"',
    '  fi',
    '  if test -z "$module_step_insert_problem"; then',
    '    if ! module_step_insmod_output=$(insmod "$module_step_built_zfs" 2>&1); then',
    '      module_step_insert_problem="insmod $module_step_built_zfs failed with: $module_step_insmod_output"',
    '    fi',
    '  fi',
    '  module_step_read_loaded',
    '  if test -z "$module_step_insert_problem" && test "$module_step_now_spl" = none; then',
    '    module_step_insert_problem="module spl is not loaded after insmod $module_step_built_spl"',
    '  fi',
    '  if test -z "$module_step_insert_problem" && test "$module_step_now_zfs" = none; then',
    '    module_step_insert_problem="module zfs is not loaded after insmod $module_step_built_zfs"',
    '  fi',
    '  if test -z "$module_step_insert_problem" && test "$module_step_now_zfs" != "$module_step_built_zfs_version"; then',
    '    module_step_insert_problem="module zfs reports $module_step_now_zfs where $module_step_built_zfs states $module_step_built_zfs_version"',
    '  fi',
    '  if test -z "$module_step_insert_problem" && test "$module_step_built_spl_version" != none && test "$module_step_now_spl" != "$module_step_built_spl_version"; then',
    '    module_step_insert_problem="module spl reports $module_step_now_spl where $module_step_built_spl states $module_step_built_spl_version"',
    '  fi',
    '  if test -n "$module_step_insert_problem"; then',
    `    echo "${label}: ${REFUSAL_WORDS}: $module_step_insert_problem" >&2`,
    '    echo "  the guest is left running: zfs $module_step_now_zfs, spl $module_step_now_spl" >&2',
    '    exit 1',
    '  fi',
    `  echo "${label}: ${NOW_RUNNING_WORDS} (zfs $module_step_now_zfs, spl $module_step_now_spl, from $module_step_built_zfs and $module_step_built_spl)"`,
    'fi',
  ];
}
