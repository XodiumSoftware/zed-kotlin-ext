---
name: update-grammar
description: Updates the pinned tree-sitter-kotlin grammar commit in extension.toml and checks the local .scm queries against it.
---

# Update Tree-sitter Grammar

Use this skill when bumping the Kotlin grammar to a newer upstream commit.

## Background

The grammar is pinned by git commit in `extension.toml`:

```toml
[grammars.kotlin]
repository = "https://github.com/fwcd/tree-sitter-kotlin"
commit = "<sha>"
```

Zed fetches and builds the grammar itself; the repo's `languages/kotlin/*.scm`
queries run against whatever grammar version is pinned, so a bump can silently
break captures (highlights, indents, textobjects, runnables, injections,
outline, overrides, brackets).

## Steps

1. Look at commits in [fwcd/tree-sitter-kotlin](https://github.com/fwcd/tree-sitter-kotlin)
   since the current pin; note grammar changes (renamed/removed node kinds).
2. Update the `commit` field in `extension.toml` to the new full SHA.
3. Read `languages/kotlin/*.scm` and cross-check every node kind they reference
   against the new grammar's `node-types.json` (fetch from the upstream repo at
   the pinned commit, or check locally after a dev-install rebuild).
4. Reinstall the dev extension in Zed (`zed: install dev extension`) so the
   grammar is recompiled, open a nontrivial Kotlin file, and verify:
   - highlighting still looks right (including KDoc vs. comments)
   - outline/breadcrumbs populate
   - run/test gutter buttons still appear on `fun main()` and tests
   - Vim text objects (`if`/`af`, `ic`/`ac`) still select
5. Fix any broken captures in the `.scm` files.

Ask the user before reverting unrelated query customizations; the fork has
local tweaks (e.g. KDoc highlighting, textobjects) that upstream lacks.

After finishing, summarize the change and ask the user if they want to commit.
