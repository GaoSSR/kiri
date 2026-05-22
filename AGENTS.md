# AGENTS.md

Instructions for AI coding agents working in this repository.

## Global Coding Rules

- Default to real, executable code rather than pseudocode.
- Do not provide pseudocode unless explicitly requested.
- For any programming language, default to production-style, maintainable implementations.
- When giving code examples, prefer runnable code with clear structure, maintainable naming, and practical error handling where appropriate.
- If an example is illustrative and cannot run as-is, explicitly state what is omitted and why.

## General Engineering Lessons

- Do not treat "real-world sample passes" as sufficient verification for parser, serializer, markdown, formatting, or round-trip fixes.
- For any text-rewrite, markdown, parser, serializer, or round-trip bugfix, always define explicit invariants before changing code:
  - what must be fixed
  - what must remain unchanged
  - what contexts are allowed to trigger the rewrite
  - what contexts must never trigger the rewrite
- Assume any regex-based formatting fix can overmatch until proven otherwise.
- If a fix changes syntax-sensitive text, review it as if it were parsing code, not just cleaning strings.

## Required Test Strategy For Text/Format Fixes

- For parser, markdown, serializer, round-trip, escaping, and formatting fixes, always add all three categories of tests:
  - positive cases: prove the reported bug is fixed
  - negative cases: prove nearby literal text is not accidentally rewritten
  - contextual cases: prove the rewrite only happens when structural context is valid
- Near-miss counterexamples are mandatory. Do not wait for code review to supply them.
- When fixing delimiter-sensitive logic, always add tests for ambiguous literal inputs such as math-like, prose-like, or partially formatted text.
- When fixing indentation-sensitive logic, always add tests for:
  - true nested structure
  - top-level content that only looks nested
  - blank-line-separated edge cases

## Required Review Heuristics For Regex Or Rewrite Logic

- Before submitting any regex-based fix, explicitly check:
  - Can the pattern consume delimiter characters that should remain literal?
  - Can it trigger inside a larger delimiter pair?
  - Can two passes interact and create a false positive that neither pass creates alone?
  - Does it rely on local syntax when the real condition is structural context?
- If a rewrite depends on nesting, hierarchy, or surrounding structure, do not key it off the current line alone; inspect the nearest meaningful context.
- Prefer the narrowest possible transformation that satisfies the invariant.
- If the fix starts accumulating more and more regex exceptions, stop and consider a parser/tokenizer-level approach.

## Review And PR Discipline

- External code review comments must be technically verified, not blindly accepted and not emotionally resisted.
- If a review comment identifies a valid edge case, convert it into a regression test before or alongside the code change.
- For bugfix PRs, include a short root-cause statement and explicitly call out any unrelated existing test failures rather than hiding them.
- When a patch touches parser or formatter behavior, add a short note in the PR description about:
  - the exact false positive avoided
  - the exact structural case normalized
  - the boundaries intentionally left unchanged

## Imported Karpathy-Inspired Rules

Source: https://github.com/multica-ai/andrej-karpathy-skills/blob/main/CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

Tradeoff: These guidelines bias toward caution over speed. For trivial tasks, use judgment.

### 1. Think Before Coding

Do not assume. Do not hide confusion. Surface tradeoffs.

Before implementing:

- State assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them. Do not pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what is confusing. Ask.

### 2. Simplicity First

Minimum code that solves the problem. Nothing speculative.

- No features beyond what was asked.
- No abstractions for single-use code.
- No flexibility or configurability that was not requested.
- No error handling for impossible scenarios.
- If 200 lines could be 50, rewrite it.

Ask: would a senior engineer say this is overcomplicated? If yes, simplify.

### 3. Surgical Changes

Touch only what must be touched. Clean up only changes caused by the current work.

When editing existing code:

- Do not improve adjacent code, comments, or formatting.
- Do not refactor things that are not broken.
- Match existing style, even if a different style seems preferable.
- If unrelated dead code is noticed, mention it instead of deleting it.

When changes create orphans:

- Remove imports, variables, and functions made unused by the current changes.
- Do not remove pre-existing dead code unless explicitly asked.

Every changed line should trace directly to the request.

### 4. Goal-Driven Execution

Define success criteria. Loop until verified.

Transform tasks into verifiable goals:

- "Add validation" means write tests for invalid inputs, then make them pass.
- "Fix the bug" means write a test that reproduces it, then make it pass.
- "Refactor X" means ensure tests pass before and after.

For multi-step tasks, state a brief plan:

```text
1. [Step] -> verify: [check]
2. [Step] -> verify: [check]
3. [Step] -> verify: [check]
```

Strong success criteria allow independent execution loops. Weak criteria such as "make it work" require clarification.

These guidelines are working if diffs contain fewer unnecessary changes, fewer rewrites due to overcomplication, and clarifying questions happen before implementation rather than after mistakes.

## Port Whisperer Refactor Guardrails

- Do not begin the Rust rewrite until the target product shape is agreed: CLI-only first, desktop wrapper later, or both.
- Preserve the current core behavior before changing presentation: default `ports` must show developer-relevant listeners, and `ports --all` must show all listeners.
- Treat process detection, platform collection, Docker mapping, framework detection, and terminal rendering as separate modules.
- Terminal output must be readable on both light and dark backgrounds. Do not use low-contrast dim gray or white text for primary table data.
- If borrowing ideas from `/Users/gaossr/CodingProject/claude-code`, copy the interaction principle and rendering strategy, then adapt it to this project and its license constraints. Do not blindly paste unrelated framework code.
- Every behavior-preserving refactor must have before/after verification commands.
