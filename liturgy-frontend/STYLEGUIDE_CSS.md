CSS Styleguide & Cleanup Plan

Goal
----
Reduce surprise by consolidating global tokens, layout, and domain styles in a predictable, minimal structure while preserving component-scoped rules.

Canonical structure (recommended)
---------------------------------
- src/assets/global.css  -> global variables, resets, layout tokens, and tiny helpers (already created)
- src/assets/main.css    -> small import file that pulls in global.css and domain css (already created)
- src/styles/liturgical.css -> domain-specific (feasts, day cards, color bars) styles
- component-scoped styles -> per-component styles inside <style scoped> in .vue files
- src/styles/_archive/  -> keep archived/legacy CSS here (not imported by default)

Current inventory
-----------------
(only files under src/)
- src/assets/global.css      (consolidated global tokens & resets)
- src/assets/main.css        (imports global.css and liturgical.css)
- src/assets/base.css        (stub that imports global.css)
- src/assets/archive/base.css (archived copy)
- src/styles/liturgical.css  (domain styles: feasts, cards, controls)
- src/styles/shared.css      (stub that imports global + liturgical)
- src/styles/archive/*       (archived originals)

Principles
----------
- Single source of truth for variables: only `src/assets/global.css` should declare :root tokens.
- Keep layout tokens and responsive breakpoints in `global.css`.
- Domain styles live in `src/styles/liturgical.css` (or split into `feasts.css`, `calendar.css` if desired).
- Component-specific layout and display rules live in each component's `<style scoped>` block.
- Avoid global utility classes except true utilities (e.g., .sr-only, .visually-hidden, .desktop-only). If a class is only used by one component, move it to that component's scoped styles.

Safe cleanup plan (manual, reversible)
--------------------------------------
1) Confirm no runtime imports of `shared.css` or `base.css` remain. We replaced those with stubs, so runtime should be safe.

2) Move archived files to a single archive folder (already done in this cleanup):
   - mkdir -p src/styles/_archive
   - archived content moved to `src/styles/_archive/` and `src/assets/archive/` was removed
   - compatibility stubs (`src/assets/base.css`, `src/styles/shared.css`) remain and import canonical files to avoid breaking runtime while we validate changes

3) Merge any useful duplicate domain rules from archive into `src/styles/liturgical.css`.
   - Do this by copying small sections at a time and running the app/tests to confirm.

4) Remove stub files only after verifying defaults work across pages:
   - git rm src/assets/base.css
   - git rm src/styles/shared.css
   (keep archive until confident)

5) Optional: split `liturgical.css` into smaller files if it grows large (e.g., `feasts.css`, `calendar-grid.css`) and import them from `main.css`.

Commands to help (run locally)
------------------------------
# 1) List all CSS files under src
find src -name "*.css" | sed -e 's|^| - |'

# 2) Run dev server and visually inspect pages
npm install
npm run dev

# 3) Run unit tests + snapshot tests
npm run test:run

# 4) Archive css files (example)
mkdir -p src/styles/_archive
git mv src/styles/archive/* src/styles/_archive/ || true
git mv src/assets/archive/* src/styles/_archive/ || true

Risks and rollbacks
-------------------
- Don't delete files until tests + visual checks pass. Keep an archive folder with original copies.
- If something breaks, `git restore --staged --worktree --source=HEAD <file>` will revert.

Next steps I can do for you
---------------------------
- I can perform the safe archive (move) and update imports automatically.
- I can merge the remaining useful rules from the archived CSS into `liturgical.css` (small chunks, test after each).
- I can add an automated Playwright CI job to capture and compare visual snapshots on PRs.

Tell me which of the "Next steps" you'd like me to run automatically now, or if you want me to open a PR with all of the proposed changes.