#!/usr/bin/env bash
# Read-only: print the first line of each repository location the proposal cites, at commit 8638a53928.
set -u
cd /home/user/Monochromatic || exit 1
commit=8638a53928
pkg=package/linter/monochromatic-lint
show_line() {
  printf '%s:%s => ' "$1" "$2"
  git show "$commit:$1" | sed -n "${2}p" | cut -c1-150
}
show_line $pkg/src/closure_packages.rs 90
show_line $pkg/src/closure_packages.rs 108
show_line $pkg/src/closure_step.rs 102
show_line $pkg/src/closure_findings.rs 178
show_line $pkg/src/run_command.rs 204
show_line $pkg/src/run_command.rs 180
show_line $pkg/src/run_modes_rule_table.rs 116
show_line $pkg/src/run_modes_rule_table.rs 122
show_line $pkg/src/closure_cargo.rs 385
show_line $pkg/src/closure_cargo.rs 400
show_line $pkg/src/closure_cargo.rs 320
show_line $pkg/src/closure_probe.rs 247
for n in 69 110 162 205 249 302; do show_line $pkg/src/closure_driver_probe.rs $n; done
show_line $pkg/src/closure_binary_tests.rs 728
show_line $pkg/src/closure_binary_tests.rs 760
show_line $pkg/README.md 384
show_line $pkg/README.md 513
show_line $pkg/README.md 475
show_line $pkg/README.md 487
show_line $pkg/README.md 529
show_line $pkg/mise.toml 77
show_line package/linter/monochromatic-lint-closure/mise.toml 53
show_line package/linter/monochromatic-lint-closure/.cargo/config.toml 1
show_line monochromatic-lint.config.jsonc 33
show_line package/mpm-shadow/cli/src/toolchain/scope.rs 42
show_line package/mpm-shadow/cli/README.md 797
show_line package/monorepo-task/core-exec/src/repo-tools-setup-archives.ts 45
show_line package/monorepo-task/core-exec/src/repo-tools-setup.ts 138
show_line doc/troubleshooting/ci-steps-do-not-see-the-toolchain-environment.md 25
show_line doc/planning/linter-selective-anonymous-functions.md 150
show_line doc/planning/linter-selective-anonymous-functions.md 277
show_line doc/planning/linter-selective-anonymous-functions.md 304
