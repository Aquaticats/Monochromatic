# The two memory settings persist across reboots, and the LFS URL mask shows short path segments

## Status

Decided by the owner on 2026-11-24,
through the question tool,
in one round of two questions.
Recorded by the session.

## The memory settings survive a reboot

### The question

`vm.watermark_scale_factor` 125 and `/sys/kernel/mm/lru_gen/min_ttl_ms` 1000 were set at runtime only
(`doc/decision/desktop-freeze-memory-settings-and-disk-swap-owner-answers-2026-11-22.md`),
so a reboot would return them to the values in force at the freeze of 2026-11-22.
The first read under a load of the freeze's shape
(six guests,
a whole gate and Rust builds,
2026-11-23T20:57:22-05:00 to 21:52:20)
held with memory `full` at 5.99% and no blocked task,
while direct reclaim still did most of the scanning,
and no arm without the settings has run
(`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
"The comparison that would verify the change").
The options were to install the drafted files,
to run an arm without the settings first,
or to keep them runtime-only.

### The answer

"Install both files (Recommended)"

### What was done

The two files drafted in the troubleshooting document's "The persistent form as a draft for the owner"
were installed as written,
mode 0644 and owned by root,
on 2026-11-24 at about 01:05-05:00:
`/etc/sysctl.d/90-vm-watermark-scale-factor.conf` and `/etc/tmpfiles.d/lru-gen-min-ttl.conf`.
`systemd-analyze cat-config sysctl.d` lists the first after `70-cachyos-settings.conf`,
and `systemd-tmpfiles --cat-config` lists the second.
Nothing was rebooted,
and the runtime values were already 125 and 1000.
An arm without the settings,
if one is ever run,
reverts them at runtime for that run only.

## Short path segments stay shown in a relayed git-lfs line

### The question

The alpha.45 release pass's L2
(`~/temp/agent/release-pass-wrapper-alpha-45-2026-11-23/commit-push-lfs-lock/keep/review.md`):
the relayed URL mask in `package/git-policy/cli/src/native/lfs/lfs_output_mask.rs` keeps a path segment only when it is
lower-case letters and `-._~`,
so `https://github.com/Org/Repo.git/info/lfs` printed as `https://github.com/<credentials>/<credentials>/info/lfs`.
The owner's earlier answer for this mask was "Hide long words only",
and the mask counts a word over `LONGEST_PLAIN_WORD` (20) characters as long.
The options were to show any segment under that length,
to show capitals but keep hiding digits,
or to keep the rule.

### The answer

"Show any segment under 20 chars (Recommended)"

### What follows

A path segment of at most `LONGEST_PLAIN_WORD` characters prints as written,
whatever its case or digits;
a longer one still prints as `<credentials>`.
The cost the option named:
a short token with digits in a path,
like the test's 14-character `CANARYPATH5521`,
now shows,
so that test changes to a segment longer than the limit.
Userinfo,
query values and fragments keep their own rules.
