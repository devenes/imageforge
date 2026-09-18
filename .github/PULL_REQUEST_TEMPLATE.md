## Description

<!-- Provide a brief explanation of the problem solved or feature introduced. -->

## Changes Proposed

- 
- 

## Invariants Checklist

Please verify that your pull request respects the application's core invariants:

- [ ] **Dimension Invariant**: Pixel dimensions of processed images are preserved.
- [ ] **Non-Destructive**: Original images are never overwritten; collision avoidance is preserved.
- [ ] **Privacy**: Zero outbound network requests, analytics, or telemetry introduced.
- [ ] **Atomic Writes**: Outputs are written via atomic rename (`atomic_write_file`).

## Quality Verification

- [ ] `pnpm check` passes with 0 errors and 0 warnings.
- [ ] `pnpm build` succeeds.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` passes with 0 warnings.
- [ ] `cargo test --all-targets` passes all tests.

## Visual / UI Changes (if applicable)

<!-- Attach before/after screenshots or screen recordings if applicable. -->
