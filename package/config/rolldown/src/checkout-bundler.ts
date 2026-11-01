import type { Plugin, } from 'rolldown';

//region Checkout bundler
// THE BUNDLER THAT RUNS A BUILD IS WHICHEVER `rolldown` PATH REACHES FIRST, not
// the one the building checkout installs. A corpus pass launched on 2026-10-19
// handed its unit the session shell's PATH, which led with the main checkout's
// `node_modules/.bin`; that checkout held rolldown 1.2.14 and the worktree's
// lockfile installs 1.2.13. The bundle changed where the bundler wraps CommonJS
// modules, and the pipeline digest over the built files moved with no source
// changed, so the pass ran a build its own checkout could not reproduce.
//
// The config imports `VERSION` from `rolldown`, which Node resolves from the
// config package's own location, so it is the version the building checkout
// installs; `this.meta.rolldownVersion` is the version of the bundler running
// the build. The two disagree exactly when PATH reached another install.

/**
 Name the guard registers under, so a config's plugin list can be checked for it.
 */
export const CHECKOUT_BUNDLER_PLUGIN_NAME = 'monochromatic:checkout-bundler';

/**
 Rolldown plugin refusing a build whose running bundler is not the version the building checkout installs.

 @param installedVersion - version this checkout installs, the `VERSION` its config resolves from `rolldown`;
 passed in so a test can stand for a checkout that installs another

 @returns Rolldown plugin checking the running bundler in `buildStart`, whose error ends the build before any
 output hook runs, wherever the plugin sits in the list

 @throws RolldownError from `buildStart` when the versions differ, so the build and every config after it in the
 same list stop

 @example
 ```ts
 plugins: [checkoutBundlerPlugin({ installedVersion: VERSION, },),],
 ```
 */
export function checkoutBundlerPlugin({ installedVersion, }: {
  readonly installedVersion: string;
},): Plugin {
  return {
    name: CHECKOUT_BUNDLER_PLUGIN_NAME,
    buildStart(): void {
      /**
       Version of the bundler running this build.
       */
      const running = this
        .meta
        .rolldownVersion;
      if (running === installedVersion)
        return;
      this.error(
        `This build is running rolldown ${running}, but this checkout installs rolldown ${installedVersion}. `
          + 'The build runs whichever rolldown comes first on PATH, and that one belongs to another checkout '
          + 'or install, so the built files would differ from what this checkout builds and a pipeline digest '
          + 'over them would name a build this checkout cannot reproduce. Put this checkout\'s '
          + 'node_modules/.bin ahead of every other checkout\'s on PATH, or run the build through this '
          + 'checkout\'s mise task from a shell started in this checkout; a systemd unit started with '
          + '--setenv=PATH carries the launching shell\'s PATH, so name this checkout\'s node_modules/.bin '
          + 'first in it or leave the flag out. If this checkout\'s lockfile moved since its install, '
          + 'reinstall its dependencies so the installed bundler matches it.',
      );
    },
  };
}

//endregion Checkout bundler
