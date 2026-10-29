/**
 Hand-maintained overrides for generated package entries.
 Provides binary names and custom existence-check flags
 that Repology cannot infer.
 
 This file survives index regeneration; only edit this file manually.
 The generated file (`packages.generated.ts`) is rebuilt from Repology data.
 
 Each entry uses `p()` with the same `effname` as the generated entry it overrides.
 The merge function applies `bin` and `check` from these entries onto
 the matching generated entry, preserving the generated manager overrides.
 
 Common check flag patterns:
 - `--version` (default, most tools)
 - `-version` (ImageMagick convert, Java)
 - `-V` (ssh, nmap)
 - `version` (openssl subcommand)
 - `--help` (tools that lack --version but exit 0 for --help)
 */

import type { PackageEntry, } from '../package/types.ts';
import { p, } from '../package/p.ts';

/**
 Package entries with hand-maintained binary names and check flags that Repology cannot infer,
 each built with {@link p}.
 */
export const overrides: readonly PackageEntry[] = [
  //region Binary name differs from effname
  p({
    bin: 'convert',
    check: '-version',
    effname: 'imagemagick',
  },),
  p({
    bin: 'rg',
    effname: 'ripgrep',
  },),
  p({
    bin: '7z',
    effname: 'p7zip',
  },),
  p({
    bin: 'ag',
    effname: 'the-silver-searcher',
  },),
  p({
    bin: 'dig',
    effname: 'bind',
  },),
  p({
    bin: 'ip',
    effname: 'iproute2',
  },),
  p({
    bin: 'ps',
    effname: 'procps',
  },),
  p({
    bin: 'lsblk',
    effname: 'util-linux',
  },),
  p({
    bin: 'ls',
    effname: 'coreutils',
  },),
  p({
    bin: 'clear',
    effname: 'ncurses',
  },),
  p({
    bin: 'gpg',
    effname: 'gnupg',
  },),
  //endregion Binary name differs from effname

  //region Custom check flags (--version outliers)
  p({
    check: 'version',
    effname: 'openssl',
  },),
  p({
    bin: 'ssh',
    check: '-V',
    effname: 'openssh',
  },),
  p({
    bin: 'nmap',
    check: '-V',
    effname: 'nmap',
  },),
  p({
    bin: 'nc',
    check: '--help',
    effname: 'netcat',
  },),
  //endregion Custom check flags (--version outliers)
] as const;
