# dd rejects the fractional region operands a qcow2 cluster boundary produces

## Symptom

Incremental vmsync sync from the KVM overlay to the vhdx failed on ordinary
change patterns with:

```text
dd: invalid number: '0.0625'
```

The same arithmetic silently over-read wherever the division came out whole:
`count = ceil(length / 1 MiB)` copied up to a full MiB past the reported region
end,
and `conv=notrunc` then wrote bytes the block map never marked as changed.

## Root cause

`qemu-img map` reports regions at qcow2 cluster boundaries,
and the default cluster size is 64 KiB.
`patchBlocks` passed `bs=1048576` and computed `skip` and `seek` as
`region.start / 1048576`,
so a region starting at 65536 asked dd to skip `0.0625` blocks.
GNU dd validates its numeric operands before opening either file,
so nothing is written and the transfer aborts.

The only alignment the old code checked was `SECTOR_SIZE` (512),
which a region start at a cluster boundary always satisfies.
Nothing constrained the block-size division that followed.

## Probe

GNU dd rejects the operand outright,
at coreutils 9.11 on this host:

```text
$ dd if=/dev/zero of=/dev/null bs=1048576 skip=0.5 count=1
dd: invalid number: '0.5'
```

The same command with `skip=0.0625`,
which is what a 64 KiB-aligned region start produces,
fails identically.

## What works

Byte-offset operands keep the 1 MiB I/O block while taking an exact byte range:

```text
bs=1048576 iflag=skip_bytes,count_bytes oflag=seek_bytes skip=65536 seek=65536 count=524288
```

A probe on a 4 MiB file pair copied exactly the 524288 bytes at offset 65536,
left every other byte at its original value,
and produced a byte-identical file to the sector-block alternative:

```text
bs=512 skip=128 seek=128 count=1024
```

Both are correct for any region the 512-byte alignment check accepts.
The byte-offset form is the one `ddTransferArgs` in
`package/vmsync/cli/src/nbd.ts` builds,
because it keeps dd's 1 MiB read and write size instead of issuing one syscall
pair per sector.
`status=none`,
which the argument list already passed,
is itself a GNU extension,
so `iflag=skip_bytes`,
`count_bytes` and `oflag=seek_bytes` add no portability constraint the module
did not already have.

## Limits

- The operands assume GNU dd;
  BSD dd lacks `status=none` and the byte-offset flags.
  `nbd.ts` loads the Linux `nbd` kernel module,
  so the module is Linux-only by construction.
- Region alignment is still validated at 512 bytes.
  A region that is not sector-aligned throws before dd runs.
- `count` is the region length in bytes,
  never a rounded-up block count,
  so a region length that misses the block size transfers exactly its own bytes.

## Where the coverage lives

`package/vmsync/cli/src/nbd.unit.test.ts` asserts the exact argument list for a
64 KiB-aligned region,
that every numeric operand is an integer,
that `count` equals the region length when it misses a block boundary,
and that both alignment rejections throw.
