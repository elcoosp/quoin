<!-- Thank you for contributing to quoin! A few pointers before you start:

- Keep the PR focused: one logical change per PR.
- Conventional Commits are enforced by git-cliff for the changelog, e.g.
  `feat(devtools): add live trace summary`, `fix(core): ...`,
  `refactor(gpui): ...`, `test(testkit): ...`, `docs: ...`, `ci: ...`.
- Lint policy: `clippy::pedantic` is enabled workspace-wide and `#[allow]`
  attributes are banned — fix the issue at the source instead.
- Run `just check` (fmt + framework-free clippy) and `just test` before
  pushing; CI runs the same gates per platform/adapter.
-->

## Summary

<!-- What does this PR change and why? Link issues with "Fixes #123". -->

## Changes

-

## Checklist

- [ ] `cargo fmt --all --check` passes
- [ ] `just lint-core` passes (clippy pedantic, no allow attributes)
- [ ] Tests added/updated and passing (`just test`)
- [ ] Docs/doc-tests updated where behavior changed
- [ ] Commit messages follow Conventional Commits
