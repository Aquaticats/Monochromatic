# GNU cp 9.10 fails to reflink NOCOW files in a cold-boot Android AVD copy

## Symptom

The coherent Android consumer stopped during private AVD preparation.
GNU `cp 9.10` returned one before emulator launch or any adb operation:

```text
# GNU cp diagnostic, private paths redacted
cp: failed to clone '<private-avd>/snapshots/default_boot/ram.bin' from '<owned-avd>/snapshots/default_boot/ram.bin': Invalid argument
```

The same diagnostic affected snapshot metadata,
snapshot textures,
and temporary adb command files.
The owned snapshot RAM file measured `4,567,597,064` bytes.
`lsattr -l` reported `No_COW` on the measured failing files.
The owned and private paths use the same Btrfs filesystem.
The private systemd unit remained `inactive` and `dead`.

## Root cause

The preparer requested a mandatory reflink of the entire owned AVD,
including state unused by its existing `-no-snapshot` launch.
GNU coreutils tag `v9.10`,
commit `89b2cd58ac895e3fc0d24d8f10e7e4ba132e7fb6`,
was shallow-cloned without source changes.
Its `src/copy.c:229` issues the clone ioctl:

```c
/* coreutils: src/copy.c:229 */
clone_file (MAYBE_UNUSED int dest_fd, MAYBE_UNUSED int src_fd)
{
#ifdef FICLONE
  return ioctl (dest_fd, FICLONE, src_fd);
```

`src/copy.c:691` reports the kernel error and returns failure for `--reflink=always`:

```c
/* coreutils: src/copy.c:691 */
if (reflink_mode == REFLINK_ALWAYS || report_failure)
  error (0, errno, _("failed to clone %s from %s"),
         quoteaf_n (0, dst_name), quoteaf_n (1, src_name));
```

Upstream Linux `v7.2`,
`fs/btrfs/reflink.c:862`,
rejects different data-checksum flags:

```c
/* Linux v7.2: fs/btrfs/reflink.c:862 */
	/* Don't make the dst file partly checksummed */
	if ((inode_in->flags & BTRFS_INODE_NODATASUM) !=
	    (inode_out->flags & BTRFS_INODE_NODATASUM)) {
		return -EINVAL;
	}
```

The host runs `7.2.9-ogc3.1.fc44.x86_64`.
The upstream excerpt explains the matching local fixture behavior;
it is not an exact binary-to-source audit of Fedora patches.
Disposable controls reproduce failure for a NOCOW source and ordinary destination,
while ordinary-to-ordinary and NOCOW-to-NOCOW reflinks succeed.
This rules out a blanket inability to reflink on this filesystem.

## Verification

Working cases:

- Ordinary source to ordinary destination:
  mandatory reflink succeeds and bytes match.
- NOCOW source to a pre-created NOCOW destination:
  mandatory reflink succeeds and bytes match.
- Installed emulator `-help-no-snapshot`:
  explicitly disables snapshot autoload and autosave.

Failing cases:

- NOCOW source to an ordinary destination:
  exit one with `Invalid argument`.
- The initial whole-directory AVD copy:
  the same diagnostic and no device operations.

The fixture task is runnable from a private scratch directory:

```toml
# Private scratch mise.toml: disposable filesystem controls
[tasks.control-nocow-reflink-boundary]
shell = "node --input-type=module-typescript -e"
run = '''
const {execFileSync,spawnSync}=await import('node:child_process');
const {readFileSync,writeFileSync,mkdtempSync,statSync}=await import('node:fs');
const {join}=await import('node:path');
const {strict:assert}=await import('node:assert');
const fixture=mkdtempSync(join(process.cwd(),'nocow-clone-control-'));
const cow=join(fixture,'cow.bin'),nocow=join(fixture,'nocow.bin'),destination=join(fixture,'nocow-destination.bin');
writeFileSync(cow,Buffer.alloc(8192,37));writeFileSync(nocow,'');execFileSync('chattr',['+C',nocow]);writeFileSync(nocow,Buffer.alloc(8192,73));
const rows=[];
for(const [name,source,target,expected] of [['cow-to-cow',cow,join(fixture,'cow-cloned.bin'),0],['nocow-to-cow',nocow,join(fixture,'mismatched.bin'),1]]){const result=spawnSync('cp',['--archive','--reflink=always',source,target],{encoding:'utf8'});assert.equal(result.status,expected,result.stderr);if(expected===0)assert.deepEqual(readFileSync(target),readFileSync(source));else assert(result.stderr.includes('Invalid argument'));rows.push({name,command:['cp','--archive','--reflink=always',source,target],exitCode:result.status,stdout:result.stdout,stderr:result.stderr})}
writeFileSync(destination,'');execFileSync('chattr',['+C',destination]);const matched=spawnSync('cp',['--archive','--reflink=always',nocow,destination],{encoding:'utf8'});assert.equal(matched.status,0,matched.stderr);assert.deepEqual(readFileSync(destination),readFileSync(nocow));rows.push({name:'nocow-to-nocow',exitCode:matched.status,stderr:matched.stderr});
console.log(JSON.stringify(rows));
'''
```

The [coherent early evidence](../audit/depot-coherent-early-results-2026-10-17.evidence.json)
retains the actual controls,
initial failure,
source excerpt identity,
and revised preparer definition.

## Verified workaround and remaining boundary

The preparer now excludes `snapshots` and `tmpAdbCmds` from its cold-boot copy.
Every retained entry still uses `cp --archive --reflink=always`.
The failed private copy is preserved separately;
the owned AVD is unchanged.
Only the staged `prepare` task changed.
The other device task bodies retain their verified hashes.
The revised consumer input names the new local harness digest.

This intentionally omits saved snapshots and transient adb commands.
It preserves the existing full-boot policy rather than adding snapshot support.
Filesystem controls pass.
The revised copy and private AVD metadata checks passed on October 17.
The headful systemd launch booted successfully,
installed the bound APK pair,
and ran the original instrumentation command.
That command reported a `SessionStore` null-context exception;
its source outcome is separate from this copy repair.
Graceful shutdown passed and the private unit is inactive.

## What does not work

The original unconditional whole-directory reflink fails on the measured NOCOW files.
Changing to unrestricted byte-copy fallback was not used:
the saved RAM file is unnecessary for this launch and measures more than four billion bytes.
A successful APK join does not prove AVD preparation or device test execution.

## Upstream filing decision

1.   Fault attribution:
     the repository preparer includes unused state with incompatible clone flags.
     The observed clone failure follows the exercised filesystem constraint.
2.   Fixability:
     a consumer-side cold-boot selection removes the unnecessary files.
     The matching-flag control also demonstrates a separate filesystem route.
3.   Supported scope:
     the existing emulator flag explicitly disables snapshot load and save.
     Snapshot-preserving copy behavior is outside this repair.
4.   Contribution policy:
     no upstream change or submission is proposed;
     contribution eligibility was not investigated.
5.   Maintainer direction:
     no maintainer willingness or refusal is inferred.
6.   Prototype:
     disposable filesystem controls and the staged preparer revision are retained.
     They establish the local repair boundary,
     not an upstream patch.

There is no upstream defect report to add from this evidence.
Repository incident `#1171` is closed after the revised copy and actual cold-boot consumer completed.
All 15 original device assertions ran:
13 passed and two reproduced the source test-context failure in `#1002`.
The private unit is inactive,
the private adb server is stopped,
all private sockets are closed,
and owned ini/config hashes remain unchanged.
The [coherent Android evidence](../audit/depot-coherent-android-2026-10-17.evidence.json)
retains the producer join,
complete device results,
and verified cleanup.

## Node 26.12 recurrence on 2026-10-18

The later private preparer used `fs.cpSync` with `COPYFILE_FICLONE_FORCE`.
It failed with `EINVAL: invalid argument, copyfile` on a 14-byte `tmpAdbCmds` member,
before launching an emulator.
Fresh `lsattr -l` output reported `No_COW` for that source file,
and source and destination parent had the same filesystem device number.
These observations match the recorded clone-flag boundary;
the Node caller is a separate recurrence from the GNU cp invocation.

The verified cold-boot selection was reused:
exclude `snapshots` and `tmpAdbCmds`,
retain mandatory reflinks for selected files,
and disable snapshot load/save for the private emulator.
That copy and the actual headful device consumer completed.
Its frozen source-derived catalog reported 16 cases,
14 passes,
two known context-fixture failures,
and no skips.
Explicit emulator/ADB cleanup completed and original configuration content remained unchanged.
The [2026-10-18 device receipt](../audit/depot-android-consumer-2026-10-18.evidence.json)
retains the APK identities,
raw instrumentation output,
and cleanup results.
No snapshot-preserving copy or new upstream fault is claimed.
