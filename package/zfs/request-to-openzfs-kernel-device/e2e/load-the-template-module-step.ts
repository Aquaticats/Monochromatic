// A template guest's step: leave the guest running the module the template installs.
//
// What:  builds the shell the custom-template `prepare` runs before it checks the
//        userspace and the module's type information. It reads which modules are loaded
//        and the version each publishes, asks `modprobe` for `zfs` with its output kept,
//        then: says the guest already ran the module this template installs and changes
//        nothing, says which module `modprobe` loaded, or refuses in words naming the
//        version loaded, the version the file this template installs states, the release
//        this recipe pins and `modprobe`'s own words.
// Why a step of its own: the template installs its module as a package
//        (`linux-cachyos-zfs`), in `extramodules/`, and builds none, so the step that
//        `prepare` runs on the Ubuntu image would look for `extra/zfs.ko` and insert
//        with `insmod`, which cannot load the package's compressed files at all. This
//        step is the smaller one: it inserts nothing, removes nothing, and leaves the
//        template's own package route (`modprobe`) to load what it ships.
// Why the failed load is not hidden: the step it replaces was
//        `modprobe --quiet zfs 2>/dev/null || true`, so a load that failed said nothing
//        and the `cat /sys/module/zfs/version` after it failed without saying why
//        (issue 6644). `modprobe`'s output is kept here, its status is checked, and a
//        failure names it.
// Why the file's version is the identity: `modinfo --field filename zfs` names the file
//        `modprobe` will load and `modinfo --field version zfs` states what that file
//        carries, so "the module the template was built with" is read from the template
//        rather than from a constant this recipe could drift from. The release this
//        recipe pins is compared too, so a template rebuilt with another release is
//        refused in words instead of failing a bare `test` after it.

/** The words a step prints when no zfs module was loaded before it ran. */
export const NOTHING_LOADED_WORDS = 'no zfs module was loaded';

/** The words a step prints about the module `modprobe` loaded. */
export const LOADED_WORDS = 'loaded the module this template installs';

/** The words a step prints when the guest already ran the module this template installs. */
export const ALREADY_RUNNING_WORDS = 'the guest already ran the module this template installs';

/** The words a step prints when it will not go on with the module the guest runs. */
export const REFUSAL_WORDS = 'refusing to go on';

/** One template guest's step, and where it reads what it asks. */
export interface LoadTheTemplateModuleStep {
  /**
   The step's own name, which every line it prints starts with, so a guest log with
   several steps in it says which one spoke.

   A fixed word from this recipe, never text from outside it: it is written into the
   shell this builds.
   */
  readonly label: string;
  /**
   The OpenZFS release this recipe pins, which the module the template installs must
   state and the loaded module must report.

   A fixed word from this recipe's own constants, never text from outside it.
   */
  readonly release: string;
  /**
   Where the kernel exposes its loaded modules.
   A literal path, never text from outside this recipe; a guest step takes the default.
   */
  readonly sysDirectory?: string;
}

/**
 The shell lines that leave a template guest running the module the template installs,
 or refuse in words.

 The sys directory is a parameter because the check beside this file
 (`load-the-template-module-step.check.ts`) runs this shell on a host against stand-in
 trees, which is the only way to hold its states without a guest. No guest step passes
 one: the default is the guest's own path.

 @param label - the step's own name, which every line it prints starts with

 @param release - the OpenZFS release this recipe pins

 @param sysDirectory - where the kernel exposes its loaded modules

 @returns the shell lines, to be joined into a script that runs as root in the guest

 @example
 ```ts
 const script = ['set -eu', ...loadTheTemplateModuleStep({ label: 'prepare', release: '2.4.4-1' })].join('\n');
 ```
 */
export function loadTheTemplateModuleStep({
  label,
  release,
  sysDirectory = '/sys',
}: LoadTheTemplateModuleStep,): readonly string[] {
  return [
    // Where this shell reads, and the release this recipe pins, which no guest step overrides.
    `template_module_sys=${sysDirectory}`,
    `template_module_release=${release}`,
    // What is loaded now, in one place: the module's own /sys directory says whether it
    // is there at all, which a version file cannot (a build may publish none), and the
    // version file says which build it is.
    'template_module_read_loaded() {',
    '  template_module_now_zfs=none',
    '  if test -d "$template_module_sys/module/zfs"; then',
    '    template_module_now_zfs=unknown',
    '    if test -f "$template_module_sys/module/zfs/version"; then template_module_now_zfs=$(cat "$template_module_sys/module/zfs/version"); fi',
    '  fi',
    '  template_module_now_spl=none',
    '  if test -d "$template_module_sys/module/spl"; then',
    '    template_module_now_spl=unknown',
    '    if test -f "$template_module_sys/module/spl/version"; then template_module_now_spl=$(cat "$template_module_sys/module/spl/version"); fi',
    '  fi',
    '}',
    'template_module_read_loaded',
    'template_module_before_zfs=$template_module_now_zfs',
    'template_module_before_spl=$template_module_now_spl',
    // `modprobe` is the template's own route: the package put the module where
    // `modules.dep` names it. Its status and its words are kept, so a load that failed is
    // named here instead of being passed over before a version read that would fail with
    // no reason.
    'template_module_modprobe_status=0',
    'template_module_modprobe_output=',
    'if ! template_module_modprobe_output=$(modprobe zfs 2>&1); then template_module_modprobe_status=1; fi',
    'template_module_read_loaded',
    // The file the template installs, and the version it states, read from `modinfo`,
    // which resolves the same `modules.dep` `modprobe` just used. A `modinfo` that fails
    // is kept as words rather than taken out with `set -e`.
    'template_module_problem=',
    'template_module_file_zfs=',
    'if template_module_answer=$(modinfo --field filename zfs 2>&1) && test -n "$template_module_answer"; then',
    '  template_module_file_zfs=$template_module_answer',
    'elif test -z "$template_module_answer"; then',
    '  template_module_problem="modinfo --field filename zfs printed nothing, so nothing names the file this template installs"',
    'else',
    '  template_module_problem="modinfo --field filename zfs said: $template_module_answer"',
    'fi',
    'template_module_file_zfs_version=',
    'if test -z "$template_module_problem"; then',
    '  if template_module_answer=$(modinfo --field version zfs 2>&1) && test -n "$template_module_answer"; then',
    '    template_module_file_zfs_version=$template_module_answer',
    '  elif test -z "$template_module_answer"; then',
    '    template_module_problem="modinfo --field version zfs printed nothing, so nothing says which release the file this template installs carries"',
    '  else',
    '    template_module_problem="modinfo --field version zfs said: $template_module_answer"',
    '  fi',
    'fi',
    // An spl build may record no version, and then there is no version of it to compare:
    // `none` says so, and only spl may read as `none`.
    'template_module_file_spl_version=none',
    'if test -z "$template_module_problem"; then',
    '  if template_module_answer=$(modinfo --field version spl 2>&1); then',
    '    if test -n "$template_module_answer"; then template_module_file_spl_version=$template_module_answer; fi',
    '  else',
    '    template_module_problem="modinfo --field version spl said: $template_module_answer"',
    '  fi',
    'fi',
    // A load that failed: `modprobe`'s own words, what is loaded now, the file this
    // template installs and the release this recipe pins, so a person reading the log
    // knows what was asked for and what answered.
    'if test "$template_module_modprobe_status" != 0; then',
    `  echo "${label}: ${REFUSAL_WORDS}: modprobe zfs could not load the module this template installs." >&2`,
    '  if test -n "$template_module_modprobe_output"; then',
    '    echo "  modprobe zfs said: $template_module_modprobe_output" >&2',
    '  else',
    '    echo "  modprobe zfs printed nothing and exited with a failure" >&2',
    '  fi',
    '  echo "  loaded now: zfs $template_module_now_zfs, spl $template_module_now_spl" >&2',
    '  echo "  this recipe pins OpenZFS $template_module_release" >&2',
    '  if test -n "$template_module_file_zfs"; then',
    '    echo "  the file this template installs: $template_module_file_zfs stating ${template_module_file_zfs_version:-no version}" >&2',
    '  fi',
    '  if test -n "$template_module_problem"; then',
    '    echo "  and reading it: $template_module_problem" >&2',
    '  fi',
    '  echo "  to load it: check that the module file this template installs and its spl module are there and that depmod --all has recorded them, then run this step again; a guest made from the template again starts with them as the template shipped them" >&2',
    '  exit 1',
    'fi',
    // Nothing readable says which module this template installs, so nothing here can
    // tell the guest's module from another build.
    'if test -n "$template_module_problem"; then',
    `  echo "${label}: ${REFUSAL_WORDS}: nothing here says which module this template installs, and the guest now runs zfs $template_module_now_zfs, spl $template_module_now_spl" >&2`,
    '  echo "  $template_module_problem" >&2',
    '  echo "  this recipe pins OpenZFS $template_module_release" >&2',
    '  exit 1',
    'fi',
    // A template rebuilt with another release is another template. The `test` this step
    // replaces failed here with no words; this one names both releases and the file.
    'if test "$template_module_file_zfs_version" != "$template_module_release"; then',
    `  echo "${label}: ${REFUSAL_WORDS}: the file this template installs states zfs $template_module_file_zfs_version where this recipe pins OpenZFS $template_module_release" >&2`,
    '  echo "  the file: $template_module_file_zfs" >&2',
    '  echo "  loaded now: zfs $template_module_now_zfs, spl $template_module_now_spl" >&2',
    '  echo "  a template that carries another release is another template: make the image with OpenZFS $template_module_release, or move the release this recipe pins, and run this step again" >&2',
    '  exit 1',
    'fi',
    // `modprobe` that reports success and loads nothing is not a guest running the
    // template's module, and the version test after the old line read that as a missing
    // file rather than as the load that did nothing.
    'if test "$template_module_now_zfs" = none; then',
    `  echo "${label}: ${REFUSAL_WORDS}: modprobe zfs exited 0 and no zfs module is loaded, so nothing says which module this guest runs" >&2`,
    '  echo "  loaded now: zfs $template_module_now_zfs, spl $template_module_now_spl" >&2',
    '  echo "  the file this template installs: $template_module_file_zfs stating $template_module_file_zfs_version" >&2',
    '  echo "  this recipe pins OpenZFS $template_module_release" >&2',
    '  exit 1',
    'fi',
    // A loaded module that is not the one the file states: this step removes nothing, so
    // the guest it leaves is the guest it found, and the refusal names both versions and
    // the ways out.
    'template_module_mismatch=',
    'if test "$template_module_now_zfs" != "$template_module_file_zfs_version"; then template_module_mismatch=yes; fi',
    'if test "$template_module_file_spl_version" = none; then',
    '  if test "$template_module_now_spl" != unknown; then template_module_mismatch=yes; fi',
    'else',
    '  if test "$template_module_now_spl" != "$template_module_file_spl_version"; then template_module_mismatch=yes; fi',
    'fi',
    'if test -n "$template_module_mismatch"; then',
    `  echo "${label}: ${REFUSAL_WORDS}: the guest runs a module this template did not install, and this step loads and removes nothing" >&2`,
    '  echo "  loaded: zfs $template_module_now_zfs, spl $template_module_now_spl" >&2',
    '  echo "  the file this template installs: $template_module_file_zfs stating zfs $template_module_file_zfs_version, spl $template_module_file_spl_version" >&2',
    '  echo "  this recipe pins OpenZFS $template_module_release" >&2',
    '  echo "  to run the module this template installs: reboot the guest, which loads the module the template ships, or take the other one out when nothing holds it (modprobe --remove zfs, then modprobe --remove spl) and run this step again; a guest made from the template again starts with the template\'s own module" >&2',
    '  exit 1',
    'fi',
    // The state the step ends well in, said so a log with one line in it answers which
    // module the guest runs and where that module came from.
    'if test "$template_module_before_zfs" = none; then',
    `  echo "${label}: ${NOTHING_LOADED_WORDS}; modprobe zfs ${LOADED_WORDS} (zfs $template_module_now_zfs from $template_module_file_zfs stating $template_module_file_zfs_version, spl $template_module_now_spl stating $template_module_file_spl_version)"`,
    'else',
    `  echo "${label}: ${ALREADY_RUNNING_WORDS} (zfs $template_module_before_zfs, spl $template_module_before_spl), which is what $template_module_file_zfs states (zfs $template_module_file_zfs_version, spl $template_module_file_spl_version); modprobe zfs had nothing to load, nothing was removed and nothing was inserted"`,
    'fi',
  ];
}
