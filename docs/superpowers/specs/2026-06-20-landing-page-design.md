# git-schedule — Landing Page Design

**Date:** 2026-06-20
**Status:** Approved design, pending implementation plan

## Goal

Build a single-page marketing/landing site for `git-schedule` — a Rust CLI that lets developers schedule git commits and pull requests for later ("delayed send for your code"). The page must communicate the value within ~5 seconds, demonstrate the tool with a polished terminal, and give visitors a copy-paste install command.

## Decisions (locked)

| Choice | Decision |
|--------|----------|
| Scope | Single landing page (no multi-page docs; README stays the docs source) |
| Framework | Next.js (App Router) with `output: 'export'` (static export) |
| Hosting | Vercel (no base-path juggling needed) |
| Styling | Tailwind CSS, palette wired through CSS variables |
| Aesthetic | Dark, terminal / dev-native, professional (not toy-bright) |
| Palette | **Warm Charcoal** — brown-tinged near-black base, earth-toned terminal syntax, gold accent |
| Display/UI font | **Satoshi** (Fontshare, self-hosted via `next/font/local`) |
| Mono font | **JetBrains Mono** (Google, via `next/font/google`) |
| Hero layout | Centered hero (headline + CTAs centered, large animated terminal beneath) |
| Hero animation | Auto-typing command loop + install ASCII-art reveal |

## Color Palette — "Warm Charcoal"

Wired as CSS custom properties (single source of truth; changing accent is a one-line edit).

```
--bg:        #0c0a09   /* page background, brown-tinged near-black */
--bg-2:      #11100e   /* alternating section background */
--panel:     #171411   /* cards, terminal body */
--panel-2:   #1d1916   /* terminal title bar, raised surfaces */
--border:    #2a241f   /* hairline borders */
--border-2:  #372f28   /* stronger borders, button outlines */
--text:      #f3efe9   /* primary text */
--muted:     #a99e90   /* secondary text */
--dim:       #6b6055   /* tertiary text, comments */
--ink:       #1f1405   /* text on amber buttons */

--accent:    #f0a93a   /* gold accent */
--accent-hi: #ffc35e   /* button gradient top, hover */
--accent-dim:#33260f   /* eyebrow / chip background */
--accent-bd: #5c4520   /* eyebrow / chip border */

/* terminal syntax (earth-toned to harmonize with warm base) */
--c-path:    #e0a86a   /* repo path / branch */
--c-str:     #cdd98a   /* string args (commit messages) */
--c-flag:    #e6b5a0   /* flags (--in, --push) */
--c-ok:      #9ec96f   /* success ✓ output */
--c-comment: #6b6055   /* comment lines */
```

Ambient: layered radial-gradient glows (`rgba(240,169,58,.12)` warm wash top-right, subtle second wash top-left), soft shadows on the terminal, gradient primary buttons (`--accent-hi → --accent`).

## Typography

- **Satoshi** — headings, nav, buttons, UI labels. Self-hosted: download woff2 weights (400, 500, 700, 900) from Fontshare into `public/fonts/` (or `app/fonts/`), load with `next/font/local`, `font-display: swap`, size-adjusted fallback to prevent layout shift.
- **JetBrains Mono** — terminal, code snippets, install commands, version badges, monospace labels. Loaded via `next/font/google` (weights 400, 500).
- Both exposed as CSS variables (`--font-sans`, `--font-mono`) on `:root` so Tailwind and raw CSS can reference them.

## Page Structure (top → bottom)

### 1. Sticky nav
- Left: small gradient logo mark (`gs`) + wordmark `git-schedule`.
- Right: anchor links (Features, How it works, Install), and a GitHub star CTA button.
- Sticky, `backdrop-filter: blur`, semi-transparent warm-charcoal background, hairline bottom border.

### 2. Hero (centered — Layout B)
- Pulsing version eyebrow chip: `● v0.1.11 · macOS · Linux · Windows` (version read from a constant; see Open Questions).
- Headline (Satoshi, ~800 weight, tight tracking): **"Delayed send, for your `git commits`."** with `git commits` in accent gold.
- One-line lede (muted): "Write code at midnight. Let it commit at 9 AM — locally or in the cloud."
- CTA row: primary **▼ Install** (gradient, scrolls to install section) + ghost **★ GitHub** (links to repo).
- Sub-line: "Built in Rust · MIT licensed · zero config".
- **Animated terminal** beneath (the centerpiece):
  - Realistic window: traffic-light dots, title `mafex — zsh — 92×24`, copy pill (decorative).
  - JS-driven typing animation (no real execution). Loops through a sequence of examples:
    1. `git add feature.rs`
    2. `git schedule "feat: add awesome feature" --in 2h --push` → `✓ Scheduled commit for 3:00 PM (in 2 hours)` + captured/unstaged comment
    3. `git schedule "fix: patch" --at "fri 9am" --remote` → `✓ Scheduled via GitHub Actions for Fri 9:00 AM`
    4. `git schedule list` → one pending row with `[push]` tag
  - After the loop, reveal the install **ASCII art** (the owl from the README) once, then restart.
  - Blinking amber caret. Per-token syntax highlighting using the palette syntax colors.
  - Respects `prefers-reduced-motion`: falls back to a static fully-rendered terminal.

### 3. Features grid
Six cards (icon, title, one-line description), hover lift:
1. **Schedule commits** — relative or absolute time (`--in 2h`, `--at "fri 3pm"`), up to 7 days out.
2. **Remote via Actions** — runs on GitHub Actions even when your machine is off.
3. **Scheduled PRs** — open pull requests on a timer with the `gh` CLI.
4. **Edit, cancel, undo** — reschedule the time, change the message, or undo the last schedule.
5. **Failed queue + retry** — missed/failed commits move to a queue; re-stage and retry.
6. **System notifications** — native success/failure notifications when commits run.

### 4. How it works
The 4-step flow as a vertical timeline (or 4 connected steps):
1. CLI captures staged changes as a patch, unstages them.
2. Daemon stores the schedule in `~/.git-schedule/`.
3. Daemon waits until the scheduled time.
4. Daemon applies the patch, commits, optionally pushes, sends a notification.

### 5. Install
Segmented/tabbed block with copy-to-clipboard on each snippet:
- **macOS / Linux (Homebrew):** `brew install mafex11/tap/git-schedule`
- **Windows (PowerShell):** `irm https://raw.githubusercontent.com/mafex11/git-schedule/main/install.ps1 | iex`
- **From releases:** download + extract + move binaries.
- **From source:** `cargo build --release` + install.
- Verify line: `git schedule --version`.

### 6. Command reference
Condensed table of the most useful commands (curated subset; "full reference on GitHub →" link):
`schedule --in/--at`, `--push`, `--remote`, `pr --to`, `list`, `status`, `show`, `edit`, `cancel`, `undo`, `failed`, `retry`.

### 7. Footer
MIT · built in Rust · GitHub link · version · small credits (clap, git2, notify-rust).

## Interactions
- Typing animation with loop + ASCII reveal (JS, no execution), reduced-motion fallback.
- Copy-to-clipboard on every install/command snippet (visual "copied" confirmation).
- Subtle scroll-in fade/translate on sections (reduced-motion safe).
- Fully responsive: terminal scales down on mobile (smaller font; guarded horizontal overflow); centered hero stacks naturally.

## Component Breakdown (for the build)
- `app/layout.tsx` — fonts, metadata, global background/glows.
- `app/page.tsx` — composes the sections.
- `components/Nav.tsx`
- `components/Hero.tsx`
- `components/Terminal.tsx` — the animated terminal (owns the typing engine + reduced-motion logic).
- `components/Features.tsx`
- `components/HowItWorks.tsx`
- `components/Install.tsx` — segmented control + `CopyButton`.
- `components/CommandReference.tsx`
- `components/Footer.tsx`
- `components/CopyButton.tsx` — reused.
- `lib/demo.ts` — the typing-sequence data + ASCII art string (single source for terminal content).
- `app/globals.css` — palette CSS variables, base styles, Tailwind layer.

Each component has one clear purpose, takes its content as props/data where reasonable, and can be understood without reading the others.

## Out of Scope (YAGNI)
- In-browser command execution (animation is faked).
- Multi-page documentation site.
- Light/dark theme toggle (dark only).
- Blog, testimonials, analytics, comparison tables, FAQ.

## SEO / Meta
- Title, description, Open Graph + Twitter card tags.
- A generated OG image (static asset) showing the terminal + tagline.
- Favicon from the logo mark.

## Open Questions (resolve during planning)
1. **Version source:** hardcode `v0.1.11` in a constant, or read it from `Cargo.toml` at build time? (Lean: constant in `lib/demo.ts`, easy to bump.)
2. **Where the site lives in the repo:** a `web/` (or `site/`) subdirectory of the git-schedule repo, deployed to Vercel with that as the root directory. (Lean: `web/`.)
3. **ASCII art fit:** the README owl is wide; confirm it renders cleanly in the terminal at the chosen font size, or use a scaled-down variant on mobile.
```
