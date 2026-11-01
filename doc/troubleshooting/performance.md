# Performance Troubleshooting & Optimizations

Performance troubleshooting documentation has been organized into focused areas for better maintainability
and navigation.

## Performance Troubleshooting Categories

### [Build Performance](performance.build.md)

Build system performance optimizations

- TypeScript scripts replacing shell commands
- File system checks vs binary execution
- Cross-platform optimization strategies

### [tsdown / DTS Bundling](tsdown.md)

Why tsdown builds take 360+ms despite benchmarks showing 36ms

- DTS bundling via `rolldown-plugin-dts` is the bottleneck (~340ms for module-es)
- OXC isolated declarations already active;
   no faster code path available
- JS event loop serialization of NAPI callbacks during the DTS second pass
- Packages that are not libraries should set `dts: false`

### [Logging performance](performance.logging.md)

The proposed function-entry `trace` to `debug` migration was rejected after
source inspection and bounded measurements.
The console suppresses both levels by default;
 verbose `console.trace`
emits a diagnostic stack.
 The linked report records the benchmark and limits.

---

## Quick Reference

### Performance Optimization Key Takeaways

1. **WSL Environments**:
    Avoid executing binaries when file system checks suffice
2. **Logging**:
    Do not migrate entry `trace` calls to `debug` for a presumed
    suppressed-console speedup;
    use the diagnostic level the call requires
3. **Build Scripts**:
    Replace shell commands with TypeScript scripts for better performance and cross-platform compatibility
4. **Caching**:
    Implement file-based checks before expensive operations
5. **Process Creation**:
    Minimize subprocess execution in performance-critical paths

### Common Performance Anti-Patterns

- Using `console.trace()` or `l.trace()` in tight loops or function entry points
- Executing binaries when file existence checks would suffice
- Running shell commands without proper caching mechanisms
- Mixing synchronous I/O operations with performance-critical code paths
- Ignoring WSL-specific performance characteristics
