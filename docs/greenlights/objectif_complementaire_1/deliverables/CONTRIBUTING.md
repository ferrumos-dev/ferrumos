# Contributing to FerrumOS

For a first patch, try following the setup instructions on your OS or adding a test for a driver edge case. You can also contribute by reviewing an open pull request or filing a reproducible bug report. [Setup and tests](./README.md#run-the-host-tests) · [Scope of contributions](./contribution_scope.md)

## Propose a change

Search the [public issue tracker](https://github.com/ferrumos-dev/ferrumos/issues) for the same behavior. If you pick an existing issue, leave a comment saying what you plan to try. A small fix, test or documentation edit can go straight to a pull request.

For a bug report, give the commit (`git rev-parse --short HEAD`), your OS, the component, the shortest steps that reproduce it and the expected and actual results. Include the relevant output. For a design proposal, explain the problem, the interface you would change, which callers depend on it and how you would verify it. Discuss changes to shared kernel contracts or build toolchains with the team before implementing them.

## Prepare and submit a patch

1. [Fork FerrumOS](https://github.com/ferrumos-dev/ferrumos/fork). In the clone made with the README, run `git remote add fork https://github.com/YOUR-USERNAME/ferrumos.git`. Replace `YOUR-USERNAME` with your GitHub username.
2. Fetch the latest public base with `git fetch origin main`. Start a topic branch from it, for example `git switch -c docs/windows-setup origin/main`. Keep each pull request (PR) focused on one behavior, correction or decision.
3. Run the checks for the affected component (see below). Commit with a title in the form `[TAG] Short description`. Use `[ADD]` for new code or tests, `[FIX]` for corrections, `[DOC]` for documentation and `[MOVE]` for a relocation. For example, `[DOC] Explain how to find QEMU on Windows`.
4. Push your branch to your fork with `git push -u fork HEAD`. On GitHub, choose **Compare & pull request**. Set the base to `ferrumos-dev/ferrumos:main` and the head to your fork's topic branch. Link the issue if there is one.
5. Fill in the [PR template](../../../../.github/pull_request_template.md) with the behavior changed, commands run, target and observed results. Mention any check you could not run. For a docs-only fix, name the instructions you followed or the links you checked.

## Code and documentation conventions

- Follow the pinned `rustfmt`. Use `snake_case` for functions and modules, `UpperCamelCase` for types and traits, and `SCREAMING_SNAKE_CASE` for constants.
- Keep the kernel usable with `no_std`. If you add a dependency, explain why it is needed and how you verified that it builds for the bare-metal target.
- Keep portable kernel policy in `src/kernel/`, which forbids `unsafe`. Put CPU-specific mechanisms in `src/arch/` and device access in `src/drivers/`. Explain each unsafe block’s invariant with a `// SAFETY:` comment. Document the caller’s obligations in a `# Safety` section on public unsafe functions.
- Document public interfaces, units and error cases. If a kernel contract changes, update the contract and its callers, then add a test for the new behavior. [Module map](../../../../docs/architecture.md) · [Rust conventions](../../../../docs/greenlights/objecitf_obligatoire_2/md/normes_rust.md) · [Kernel contracts](../../../../docs/greenlights/objecitf_obligatoire_2/md/contrats_noyau.md)
- Put setup instructions for the integrated kernel in the [contributor README](./README.md) and prototype instructions in the corresponding `POC/` README. Use short `snake_case` names for new documentation files, relative links and `%20` for spaces in paths. Update the relevant index. Greenlight records belong under the corresponding objective, with objective 2 material in its `md/` and `pdf/` directories. [Documentation index](../../../README.md)

## Check your work

| Change | What to verify |
| --- | --- |
| Setup or documentation | Follow the changed steps from the stated directory. Open the links you edited. |
| Portable Rust code | Run formatting, Clippy and the Rust host tests from the root. Cover the changed behavior or regression. [Quality check commands](../../../../README.md#quality-checks) |
| Boot, hardware input or display | Run the Python boot-script tests and the QEMU smoke test. For a visible change, describe what appeared on screen. [Boot and diagnostics](../../../../docs/boot.md) |
| Isolated prototype, allocator or benchmark | Run the affected component’s own checks. Name the manifest, target and allocator or scenario in your report. [Prototype entry points](../../../../README.md#build-and-run) · [Allocator benchmark](../../../../research/heap-allocator/README.md) |

For a test report, record the revision, target and date. Attach build images and long logs as artifacts and link them in the PR.

## Review and response time

We aim to give a **first human reply within three working days** of a new issue or pull request. The reply should identify a reviewer, ask a specific question or explain the next step. If that window passes, a follow-up comment in the same thread is welcome.

A maintainer reviews the patch and another team member reviews a maintainer’s own patch. Before merging, the reviewer checks the applicable tests on the revision being merged and confirms that open review comments are resolved. If a change needs revision, the reviewer points to the behavior or test that needs work. If the team declines or defers it, the reason goes in the thread. The [scope of contributions](./contribution_scope.md) explains the acceptance criteria and who decides what ships.

Submit original work you can contribute under the project’s [MIT License](./LICENSE). When reusing third-party material, identify its source and license and preserve its notices.
