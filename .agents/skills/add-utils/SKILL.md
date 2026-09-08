---
name: add-utils
description: Adds a new helper function to src/utils.rs in the wasm extension crate following existing conventions.
---

# Add Utilities

Use this skill when a new shared helper is needed in the extension crate (`src/`).

## Before Writing Code

1. Ask the user:
   - What should the utility do?
   - Is it a single function or a group of related helpers?
   - Which existing type does it extend, if any?
   - Is it needed in the proxy crate (`proxy/`) instead? The proxy is a separate
     native crate and cannot share code with the wasm extension.

## Adding the Utility

1. Open `src/utils.rs`.
2. Decide where the utility belongs:
   - If it extends an existing type, prefer an extension-trait pattern or a
     free function taking the type, matching existing style.
   - Keep visibility minimal: `pub(super)` when only used inside the crate.
3. Add a `///` doc comment explaining parameters, return values, and side effects.
4. Only reach for `zed_extension_api` imports already used elsewhere; don't add
   new dependencies for a helper.

## Conventions

- The file has a `//!` module doc; keep one coherent module.
- Names should be descriptive; receiver/parameter types provide implicit context.
- Code must compile for `wasm32-wasip2` — stay within the extension sandbox
  APIs; no native-only calls, no threads.

## Validation

1. Run `cargo fmt` (repo root).
2. Run `cargo check --target wasm32-wasip2`.

After finishing, summarize the files changed and ask the user if they want to commit.
