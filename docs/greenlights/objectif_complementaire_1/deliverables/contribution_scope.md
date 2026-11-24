# What is open to outside contributors?

Anyone can report a bug, review a pull request or propose a patch. The starting point depends on how many parts of FerrumOS a change affects.

| Area | How to contribute | Why |
| --- | --- | --- |
| Setup instructions, translations and bug reports | Open a documentation PR directly, or file an issue with your OS, revision, command and observed result. | Someone following the README for the first time can spot missing steps the team takes for granted. |
| Tests and isolated fixes in existing modules | Send a focused PR with a test or reproduction. For a console editing change, start with the tests beside [`InputLine`](../../../../src/drivers/console.rs). For framebuffer rendering, see the [display tests](../../../../src/drivers/display/tests.rs). | Contributors can check these behaviors on their own machines before booting a VM. |
| Proofs of concept and benchmarks | Identify the prototype, allocator or scenario in the PR. Run its own checks and describe the result. | Each experiment has a separate build. Results inform decisions about the integrated kernel. |
| Shared kernel interfaces, boot protocol, memory ownership, scheduling policy or a new subsystem | Open a design issue before implementation. Describe affected callers, alternatives and how the change would be tested. Outside contributors can propose and implement the agreed design. | Several modules rely on these decisions. The team coordinates compatibility, safety and integration order. |

## Decisions the team owns

Maintainers decide which designs enter the shared kernel and which milestones get priority. They review changes to the pinned toolchains, required CI checks and release process because these affect everyone's ability to reproduce a build. Repository permissions, final merges, releases and synchronization with Epitech's `main` stay with the team. Contributors submit their change once, to the [public FerrumOS repository](https://github.com/ferrumos-dev/ferrumos).

Maintainers ask for revisions when a fix lacks a reproducible check, changes unrelated behavior or leaves an unsafe operation unexplained. They decline submissions that the author has no right to license, and may defer a feature that depends on an unfinished subsystem. The reviewer explains the reason in the issue or pull request so the author can decide whether to revise it.

These decisions govern the official repository. Developers can maintain independent forks under the [MIT License](./LICENSE). To propose a change here, follow the [contribution guide](./CONTRIBUTING.md).
