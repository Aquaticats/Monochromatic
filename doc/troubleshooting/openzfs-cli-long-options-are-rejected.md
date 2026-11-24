# zfs 2.4.4 rejects `--output` and `--recursive` with `invalid option ' '`

OpenZFS's `zfs` command takes long options only where a subcommand's own table declares them,
and today those tables hold two names, `--json` and `--json-int`.
Every other long form a GNU habit reaches for,
`--output`, `--recursive`, `--depth`, `--help`,
is refused with a message whose offending character is a NUL byte,
so it reads as a blank space.

## Symptom

Read on this host, 2026-11-23, with the userspace that reports `zfs-2.4.4-1` and the module
`/sys/module/zfs/version` reading `2.4.4-1`:

```text
$ zfs list --recursive zpcachyos/ROOT/cos/home
invalid option ' '
usage:
	list [-Hp] [-j [--json-int]] [-r|-d max] [-o property[,...]] [-s property]...
	    [-S property]... [-t type[,...]] [filesystem|volume|snapshot] ...
...
```

The same message, with the same usage dump, comes from each of these:

```text
zfs list --recursive --output name,mountpoint <dataset>
zfs list --output=name,mountpoint <dataset>
zfs list --recursive=on <dataset>
zfs list --help
zfs get --output name,property,value mounted <dataset>
```

The patterns that work cleanly, on the same host and on a guest's `2.4.4-1` build:

```text
$ zfs list -r -o name,canmount zpcachyos/ROOT/cos/home
NAME                     CANMOUNT
zpcachyos/ROOT/cos/home  on
$ zfs get -o name,property,value mounted zpcachyos/ROOT/cos/home
$ zfs list --json zpcachyos/ROOT/cos/home
{"output_version":{"command":"zfs list","vers_major":0,"vers_minor":1},"datasets":{...
```

The exit status is non-zero for every failing form, and the usage text is printed to standard error,
so a script that pipes `zfs list` output into a parser sees an empty result plus a non-zero status rather than a
partial listing.

## Root cause

`zfs` uses the C library's `getopt_long`:

```c
/* cmd/zfs/zfs_main.c:43 */
#include <getopt.h>
```

Each subcommand declares its own table of long names.
For `list`, the whole table is two entries:

```c
/* cmd/zfs/zfs_main.c:3900-3904, inside zfs_do_list, which starts at :3885 */
	struct option long_options[] = {
		{"json", no_argument, NULL, 'j'},
		{"json-int", no_argument, NULL, ZFS_OPTION_JSON_NUMS_AS_INT},
		{0, 0, 0, 0}
	};

	/* check options */
	while ((c = getopt_long(argc, argv, "jHS:d:o:prs:t:", long_options,
	    NULL)) != -1) {
```

`get` does the same, with the same two names (`:2201-2205`).
Everything else that the manual and `usage()` print is a short flag,
so `--output` is not a name this table knows even though `-o` is.

An unknown long option makes `getopt_long` return `'?'` and leaves `optopt` at its zero value,
because there is no offending *character* to report.
The handler prints that zero as a character:

```c
/* cmd/zfs/zfs_main.c:3996-3998, zfs_do_list's switch */
		case '?':
			(void) fprintf(stderr, gettext("invalid option '%c'\n"),
			    optopt);
			usage(B_FALSE);
```

`%c` of `0` writes a NUL byte, which a terminal or a log renders as nothing or as a space,
so the message reads `invalid option ' '`.
`usage(B_FALSE)` then prints the subcommand's flag summary and exits non-zero.

The `=` form fails for the same reason and not a different one:
`--output=name` is still an unknown name,
and the failure happens while options are parsed, before any dataset is opened.

## Verification

- Release under test: OpenZFS `2.4.4`,
  module and userspace both reporting `2.4.4-1`,
  built from `zfs-2.4.4.tar.gz`
  (sha256 `2a3c70d55a37cc71618a95a60e81ad66530201eb118d37741dc92efcf848c8b1`)
  in a throwaway `mvm` guest on kernel `6.8.0-147-generic`,
  and the same failing and working forms on this host with its own `2.4.4-1` userspace.
- The guest's catalogs are in that round folder's
  `keep/guest-logs/openzfs-cli-catalog.log`, and the host's own failing and working forms are in
  `keep/host-reads-for-the-provider-2026-11-23.log`.
- The round folder is
  `~/temp/agent/zfs-step-19-checkout-boot-mount-provider-and-ordering-2026-11-23/`.
- A runnable check of the whole claim, needing only a pool or dataset name:

```sh
zfs list -r -o name             "$DATASET"   # works
zfs list --recursive --output name "$DATASET" # invalid option '<NUL>'
zfs list --json                 "$DATASET"   # works, a long name the table has
```

- Two catalogs, one per outcome:
  working, `-r`, `-o name,canmount`, `-H`, `-d 1`, `-t filesystem`, `--json`;
  failing, `--output name`, `--output=name`, `--recursive`, `--recursive=on`, `--depth 1`, `--help`,
  and the same three shapes on `zfs get`.
  Every failing pattern produced the identical message and usage dump, so this is one failure mode and not several.

## Verified workarounds

- Use the short flags in scripts, unit files and documentation:
  `-r` for recursion, `-o <properties>` for the column list, `-H` for scripted output without a header,
  `-d <depth>` for the depth bound, `-s` and `-S` for sorting, `-t <types>` for the type filter.
  The tradeoff: nothing changes in the output, and the only cost is that the caller cannot write the GNU form
  they may expect; a reader of the script sees the older spelling.
- Where a structured output is wanted, `-j` and `--json` both exist, as does `--json-int`.
  The tradeoff: the JSON schema is the command's own (`output_version`, `datasets`), not the property table's,
  so a consumer written against `-o` columns must be rewritten rather than adapted.
- Ask the command what it accepts with `zfs help <subcommand>` or the `usage()` text a failure prints,
  never with `--help`, which is itself an unknown name here.
  The tradeoff: the usage text is the only authority for a given build, and a long form it does not show does not
  exist in that build.

## What does not work

- `--output=<list>` and `--output <list>`: identical failure, so the equals form is not a workaround.
- `--recursive`, `--recursive=on`, `--depth`, `--sort`, `--type`: none is declared for any subcommand this was tried
  on, so guessing further GNU names only produces the same NUL message.
- `zfs list --help`: unknown, so it prints the NUL message and the usage dump rather than help.
- Reading the usage dump's property list as a promise of long options: that block lists *properties* for `-o`,
  which is a different namespace from the option names.

## Upstream filing decision

Nothing to file. Walking the six constraints:

1.  Is it upstream's fault?
    It is a usability wart and a diagnostic wart: an unknown option should say which one, and `optopt` is zero for a
    long option, so printing it as a character hides the name. The behaviour itself,
    "long names exist only where declared", is a design choice, not a bug.
2.  Can upstream fix it?
    Yes: the message could quote `argv[optind - 1]`, and long aliases could be added per subcommand.
    Neither is hard.
3.  Are they supporting this use case?
    The manual documents short flags only, so a caller writing `--output` is outside documented usage.
4.  Would the repo welcome our contribution?
    Not checked: this round did not read OpenZFS's `CONTRIBUTING.md`, its issue templates or its tracker policy,
    and no duplicate search was run, so no filing may be drafted from this round.
5.  Would a filing be news to them?
    Unknown for the same reason.
6.  Is it ours to file?
    Unknown for the same reason.

The default is not to file, and the two unchecked constraints keep it there.
What is left for a later round with a source read and a tracker search is the message-quality half only,
since nothing is broken by the option table itself.

## Limits

- No permalink to the upstream repository was opened in a browser this round,
  so the citations above are the file and line numbers of the `zfs-2.4.4` release tarball,
  reproducible from the sha256 named under "Verification",
  rather than a commit-pinned URL whose landing on the quoted text was confirmed.
- The local copies of that tarball were in a throwaway guest, destroyed at the end of the round,
  so a reader reproduces them by downloading the pinned release.
- `zpool`, `zed`, `zdb` and `ztest` were not tried; this document claims nothing about their option tables.
- Only `2.4.3` and earlier were not tested, and no claim is made about which release added `--json`.
