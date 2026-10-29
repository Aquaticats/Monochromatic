# electron-infra

Reusable infrastructure for no-Vite Electron desktop packages in this monorepo.

It centralizes the machinery that should not live in each demo app:

- staging TypeScript output plus static assets into an Electron app directory;
- building the Linux,
   Windows,
   and macOS x64/arm64 Packager matrix;
- writing dry-run distribution manifests;
- running Electron inside this repo's nested Wayland compositor with `DISPLAY` unset;
- polling machine-readable state files from boundary tests.

Apps keep their own runtime logic and tiny wrapper scripts,
 while this package owns the reusable process,
 filesystem,
 and control-socket code.
