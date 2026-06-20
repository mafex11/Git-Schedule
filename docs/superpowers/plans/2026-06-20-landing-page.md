# git-schedule Landing Page Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a single-page, statically-exported Next.js landing site for git-schedule with a dark "Warm Charcoal" terminal aesthetic, an animated hero terminal, and copy-paste install commands.

**Architecture:** Next.js App Router with `output: 'export'` produces static HTML/CSS/JS deployable to Vercel. The page is composed of focused, single-responsibility components under `site/components/`. Pure logic (typing-animation engine, demo sequence data) lives in `site/lib/` and is unit-tested with Vitest. Visual components are verified by a passing `next build` plus a browser screenshot. Styling is Tailwind v4 with the palette wired through CSS variables in `globals.css`.

**Tech Stack:** Next.js 15 (App Router, static export), React 19, TypeScript, Tailwind CSS v4, Vitest + @testing-library/react, Satoshi (self-hosted via next/font/local), JetBrains Mono (next/font/google).

## Global Constraints

- Site root directory: `site/` (Vercel root directory = `site`). All paths below are relative to repo root.
- Static export only: `output: 'export'` in `next.config.ts`; no server components requiring a runtime, no API routes, no `next/image` loader requiring a server (use `images: { unoptimized: true }`).
- Dark theme only. No light/dark toggle.
- Palette is the "Warm Charcoal" system; every color comes from a CSS variable defined once in `globals.css` — never hardcode hex in components.
- Fonts exposed as CSS variables `--font-sans` (Satoshi) and `--font-mono` (JetBrains Mono).
- All animations must respect `prefers-reduced-motion: reduce` with a static fallback.
- Version string is a single constant `VERSION = "0.1.11"` in `site/lib/demo.ts`; reference it everywhere, never re-type it.
- GitHub repo URL: `https://github.com/mafex11/git-schedule`. Homebrew tap: `mafex11/tap/git-schedule`.
- Node 22, npm 10.

---

## File Structure

| File | Responsibility |
|------|----------------|
| `site/package.json` | Deps + scripts (dev, build, test) |
| `site/next.config.ts` | Static export config |
| `site/tsconfig.json` | TypeScript config |
| `site/vitest.config.ts` | Vitest + jsdom setup |
| `site/postcss.config.mjs` | Tailwind v4 PostCSS plugin |
| `site/app/globals.css` | Palette CSS vars, base styles, Tailwind import |
| `site/app/layout.tsx` | Fonts, metadata, body background/glows |
| `site/app/page.tsx` | Composes all sections |
| `site/lib/demo.ts` | VERSION, terminal typing sequence data, ASCII art string |
| `site/lib/typing.ts` | Pure typing-animation state engine (tested) |
| `site/lib/copy.ts` | Clipboard helper (tested) |
| `site/components/Nav.tsx` | Sticky nav |
| `site/components/Hero.tsx` | Centered hero copy + CTAs, hosts Terminal |
| `site/components/Terminal.tsx` | Animated terminal window |
| `site/components/Features.tsx` | Six feature cards |
| `site/components/HowItWorks.tsx` | Four-step timeline |
| `site/components/Install.tsx` | Segmented install block |
| `site/components/CommandReference.tsx` | Command table |
| `site/components/Footer.tsx` | Footer |
| `site/components/CopyButton.tsx` | Reusable copy-to-clipboard button |
| `site/public/fonts/*.woff2` | Self-hosted Satoshi weights |

---

## Task 1: Scaffold the Next.js static-export project

**Files:**
- Create: `site/package.json`, `site/next.config.ts`, `site/tsconfig.json`, `site/postcss.config.mjs`, `site/app/layout.tsx`, `site/app/page.tsx`, `site/app/globals.css`, `site/.gitignore`

**Interfaces:**
- Produces: a buildable Next.js app at `site/` that exports static files to `site/out/`.

- [ ] **Step 1: Create the project with create-next-app**

Run from repo root:
```bash
cd site 2>/dev/null || true
npx create-next-app@latest site --ts --tailwind --eslint --app --src-dir=false --import-alias="@/*" --no-turbopack --use-npm --yes
```
Expected: a `site/` directory with Next.js 15, React 19, Tailwind v4 scaffolding.

- [ ] **Step 2: Configure static export**

Replace `site/next.config.ts` with:
```typescript
import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  images: { unoptimized: true },
};

export default nextConfig;
```

- [ ] **Step 3: Verify the dev build runs and exports**

Run:
```bash
cd site && npm run build
```
Expected: build succeeds and `site/out/index.html` exists. Verify:
```bash
test -f site/out/index.html && echo "EXPORT OK"
```
Expected: `EXPORT OK`

- [ ] **Step 4: Add .gitignore entries**

Ensure `site/.gitignore` contains `node_modules`, `.next`, `out`, `*.tsbuildinfo`.

- [ ] **Step 5: Commit**

```bash
git add site/package.json site/next.config.ts site/tsconfig.json site/postcss.config.mjs site/app site/.gitignore site/package-lock.json site/eslint.config.mjs
git commit -m "Scaffold Next.js static-export site"
```

---

## Task 2: Palette, fonts, and global layout

**Files:**
- Modify: `site/app/globals.css` (replace with palette + base styles)
- Modify: `site/app/layout.tsx` (fonts + metadata + body shell)
- Create: `site/public/fonts/` (Satoshi woff2 files)

**Interfaces:**
- Produces: CSS variables for the full Warm Charcoal palette; CSS vars `--font-sans`, `--font-mono`; a dark body with ambient radial glows. Later components consume these variables.

- [ ] **Step 1: Download Satoshi woff2 weights**

Download Satoshi (400, 500, 700, 900) from Fontshare (https://www.fontshare.com/fonts/satoshi) into `site/public/fonts/` as `Satoshi-Regular.woff2`, `Satoshi-Medium.woff2`, `Satoshi-Bold.woff2`, `Satoshi-Black.woff2`.
```bash
mkdir -p site/public/fonts
# Download the Satoshi zip from Fontshare, extract the woff2 files listed above into site/public/fonts/
ls site/public/fonts/*.woff2
```
Expected: four `.woff2` files listed. (If Fontshare download is unavailable in the environment, note it and fall back to next/font/google "Manrope" as a temporary stand-in, keeping the --font-sans variable name unchanged.)

- [ ] **Step 2: Write globals.css with the palette**

Replace `site/app/globals.css` with:
```css
@import "tailwindcss";

:root {
  --bg: #0c0a09;
  --bg-2: #11100e;
  --panel: #171411;
  --panel-2: #1d1916;
  --border: #2a241f;
  --border-2: #372f28;
  --text: #f3efe9;
  --muted: #a99e90;
  --dim: #6b6055;
  --ink: #1f1405;

  --accent: #f0a93a;
  --accent-hi: #ffc35e;
  --accent-dim: #33260f;
  --accent-bd: #5c4520;

  --c-path: #e0a86a;
  --c-str: #cdd98a;
  --c-flag: #e6b5a0;
  --c-ok: #9ec96f;
  --c-comment: #6b6055;
}

* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body {
  margin: 0;
  background:
    radial-gradient(1100px 520px at 72% -12%, rgba(240,169,58,0.12), transparent 60%),
    radial-gradient(820px 420px at 8% 4%, rgba(240,169,58,0.04), transparent 55%),
    var(--bg);
  color: var(--text);
  font-family: var(--font-sans), system-ui, sans-serif;
  -webkit-font-smoothing: antialiased;
  line-height: 1.6;
  min-height: 100vh;
}
@media (prefers-reduced-motion: reduce) {
  html { scroll-behavior: auto; }
}
```

- [ ] **Step 3: Wire fonts and metadata in layout.tsx**

Replace `site/app/layout.tsx` with:
```tsx
import type { Metadata } from "next";
import localFont from "next/font/local";
import { JetBrains_Mono } from "next/font/google";
import "./globals.css";

const satoshi = localFont({
  variable: "--font-sans",
  display: "swap",
  src: [
    { path: "../public/fonts/Satoshi-Regular.woff2", weight: "400", style: "normal" },
    { path: "../public/fonts/Satoshi-Medium.woff2", weight: "500", style: "normal" },
    { path: "../public/fonts/Satoshi-Bold.woff2", weight: "700", style: "normal" },
    { path: "../public/fonts/Satoshi-Black.woff2", weight: "900", style: "normal" },
  ],
});

const jetbrains = JetBrains_Mono({
  variable: "--font-mono",
  subsets: ["latin"],
  weight: ["400", "500"],
  display: "swap",
});

export const metadata: Metadata = {
  title: "git-schedule — delayed send for your git commits",
  description:
    "Schedule git commits and pull requests for later. Write code at midnight, let it commit at 9 AM — locally or in the cloud. A Rust CLI.",
  openGraph: {
    title: "git-schedule — delayed send for your git commits",
    description: "Schedule git commits and pull requests for later.",
    type: "website",
  },
  twitter: { card: "summary_large_image" },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className={`${satoshi.variable} ${jetbrains.variable}`}>
      <body>{children}</body>
    </html>
  );
}
```

- [ ] **Step 4: Verify build**

Run:
```bash
cd site && npm run build
```
Expected: build succeeds, no font resolution errors.

- [ ] **Step 5: Commit**

```bash
git add site/app/globals.css site/app/layout.tsx site/public/fonts
git commit -m "Add Warm Charcoal palette and fonts"
```

---

## Task 3: Test tooling + demo data

**Files:**
- Create: `site/vitest.config.ts`, `site/vitest.setup.ts`
- Create: `site/lib/demo.ts`
- Test: `site/lib/demo.test.ts`
- Modify: `site/package.json` (add test deps + script)

**Interfaces:**
- Produces:
  - `VERSION: string` — `"0.1.11"`
  - `GITHUB_URL: string`, `BREW_TAP: string`
  - `type DemoLine = { kind: "input" | "output" | "comment"; tokens: Token[] }`
  - `type Token = { text: string; cls: "plain" | "prompt" | "path" | "sub" | "str" | "flag" | "ok" | "muted" | "comment" }`
  - `type DemoStep = { lines: DemoLine[] }`
  - `DEMO_SEQUENCE: DemoStep[]` — the ordered commands the terminal types
  - `ASCII_ART: string` — the owl reveal
  - `fullText(line: DemoLine): string` — concatenation of token texts (used by the typing engine)

- [ ] **Step 1: Install test deps**

```bash
cd site && npm install -D vitest @testing-library/react @testing-library/jest-dom jsdom @vitejs/plugin-react
```

- [ ] **Step 2: Add vitest config and setup**

Create `site/vitest.config.ts`:
```typescript
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  test: { environment: "jsdom", globals: true, setupFiles: ["./vitest.setup.ts"] },
  resolve: { alias: { "@": path.resolve(__dirname, ".") } },
});
```
Create `site/vitest.setup.ts`:
```typescript
import "@testing-library/jest-dom/vitest";
```
Add to `site/package.json` scripts: `"test": "vitest run"`, `"test:watch": "vitest"`.

- [ ] **Step 3: Write the failing test for demo data**

Create `site/lib/demo.test.ts`:
```typescript
import { describe, it, expect } from "vitest";
import { VERSION, DEMO_SEQUENCE, ASCII_ART, fullText } from "./demo";

describe("demo data", () => {
  it("exposes the current version", () => {
    expect(VERSION).toBe("0.1.11");
  });
  it("has at least 3 demo steps", () => {
    expect(DEMO_SEQUENCE.length).toBeGreaterThanOrEqual(3);
  });
  it("every input line begins with a prompt token", () => {
    for (const step of DEMO_SEQUENCE) {
      for (const line of step.lines) {
        if (line.kind === "input") {
          expect(line.tokens[0].cls).toBe("prompt");
        }
      }
    }
  });
  it("fullText concatenates token text", () => {
    const line = { kind: "input" as const, tokens: [
      { text: "a", cls: "plain" as const }, { text: "b", cls: "str" as const },
    ]};
    expect(fullText(line)).toBe("ab");
  });
  it("ASCII art is non-empty multi-line", () => {
    expect(ASCII_ART.split("\n").length).toBeGreaterThan(3);
  });
});
```

- [ ] **Step 4: Run test, verify it fails**

Run: `cd site && npm test -- demo`
Expected: FAIL — cannot resolve `./demo`.

- [ ] **Step 5: Implement demo.ts**

Create `site/lib/demo.ts`:
```typescript
export const VERSION = "0.1.11";
export const GITHUB_URL = "https://github.com/mafex11/git-schedule";
export const BREW_TAP = "mafex11/tap/git-schedule";

export type TokenClass =
  | "plain" | "prompt" | "path" | "sub" | "str" | "flag" | "ok" | "muted" | "comment";
export type Token = { text: string; cls: TokenClass };
export type DemoLine = { kind: "input" | "output" | "comment"; tokens: Token[] };
export type DemoStep = { lines: DemoLine[] };

export function fullText(line: DemoLine): string {
  return line.tokens.map((t) => t.text).join("");
}

const p = (text: string, cls: TokenClass): Token => ({ text, cls });

export const DEMO_SEQUENCE: DemoStep[] = [
  {
    lines: [
      { kind: "input", tokens: [
        p("~/my-project ", "path"), p("❯ ", "prompt"), p("git add feature.rs", "plain"),
      ]},
    ],
  },
  {
    lines: [
      { kind: "input", tokens: [
        p("~/my-project ", "path"), p("❯ ", "prompt"),
        p("git ", "plain"), p("schedule ", "sub"),
        p('"feat: add awesome feature" ', "str"),
        p("--in ", "flag"), p("2h ", "plain"), p("--push", "flag"),
      ]},
      { kind: "output", tokens: [
        p("✓ ", "ok"), p("Scheduled commit for 3:00 PM (in 2 hours)", "muted"),
      ]},
      { kind: "comment", tokens: [
        p("  Files captured and unstaged. Commit happens automatically later.", "comment"),
      ]},
    ],
  },
  {
    lines: [
      { kind: "input", tokens: [
        p("~/my-project ", "path"), p("❯ ", "prompt"),
        p("git ", "plain"), p("schedule ", "sub"),
        p('"fix: patch" ', "str"),
        p("--at ", "flag"), p('"fri 9am" ', "plain"), p("--remote", "flag"),
      ]},
      { kind: "output", tokens: [
        p("✓ ", "ok"), p("Scheduled via GitHub Actions for Fri 9:00 AM", "muted"),
      ]},
    ],
  },
  {
    lines: [
      { kind: "input", tokens: [
        p("~/my-project ", "path"), p("❯ ", "prompt"),
        p("git ", "plain"), p("schedule ", "sub"), p("list", "plain"),
      ]},
      { kind: "output", tokens: [
        p("○ a1b2c3d4  3:00 PM (1h 58m)  feat: add awesome feature  ", "muted"),
        p("[push]", "flag"),
      ]},
    ],
  },
];

export const ASCII_ART = String.raw`
       _ _                  _              _      _
  __ _(_) |_   ___ ___ _  _| |_  ___ __ _ | |_  _| |___
 / _\` | |  _| (_-</ _\` ' \| ' \/ -_) _\` ||  _|/ _\` / -_)
 \__, |_|\__| /__/\__,_||_|_||_\___\__,_| \__|\__,_\___|
 |___/
        schedule your commits like a pro~
`;
```

- [ ] **Step 6: Run tests, verify pass**

Run: `cd site && npm test -- demo`
Expected: PASS (5 tests).

- [ ] **Step 7: Commit**

```bash
git add site/vitest.config.ts site/vitest.setup.ts site/lib/demo.ts site/lib/demo.test.ts site/package.json site/package-lock.json
git commit -m "Add demo sequence data and test tooling"
```

---

## Task 4: Typing-animation engine (pure, tested)

**Files:**
- Create: `site/lib/typing.ts`
- Test: `site/lib/typing.test.ts`

**Interfaces:**
- Consumes: `DemoStep`, `DemoLine`, `fullText` from `./demo`.
- Produces:
  - `type TypingState = { stepIndex: number; lineIndex: number; charCount: number; phase: "typing" | "pausing" | "done" }`
  - `initialTypingState(): TypingState`
  - `advance(state: TypingState, steps: DemoStep[]): TypingState` — advances exactly one tick (one character while typing an input line; whole-line for output/comment lines; loops to start after the last step). This is the pure reducer the React component drives with an interval.
  - `visibleChars(line: DemoLine, charCount: number): { tokens: Token[] }` — returns tokens trimmed to `charCount` visible characters for rendering a partially-typed line.

- [ ] **Step 1: Write the failing test**

Create `site/lib/typing.test.ts`:
```typescript
import { describe, it, expect } from "vitest";
import { initialTypingState, advance, visibleChars } from "./typing";
import { fullText, type DemoStep } from "./demo";

const steps: DemoStep[] = [
  { lines: [
    { kind: "input", tokens: [{ text: "ab", cls: "prompt" }] },
    { kind: "output", tokens: [{ text: "done", cls: "ok" }] },
  ]},
  { lines: [
    { kind: "input", tokens: [{ text: "x", cls: "prompt" }] },
  ]},
];

describe("typing engine", () => {
  it("types input one char per tick", () => {
    let s = initialTypingState();
    s = advance(s, steps); // 1 char of "ab"
    expect(s.charCount).toBe(1);
    s = advance(s, steps); // 2 chars
    expect(s.charCount).toBe(2);
  });

  it("reveals output lines whole, then moves on", () => {
    let s = initialTypingState();
    s = advance(s, steps); // a
    s = advance(s, steps); // ab (input complete)
    s = advance(s, steps); // output line revealed -> next line index
    expect(s.lineIndex).toBe(1);
  });

  it("loops back to step 0 after the final step", () => {
    let s = initialTypingState();
    // exhaust everything: step0 input(2) + output(1) + step1 input(1)
    for (let i = 0; i < 20; i++) s = advance(s, steps);
    expect(s.stepIndex).toBe(0);
  });

  it("visibleChars trims tokens to count", () => {
    const line = { kind: "input" as const, tokens: [
      { text: "git ", cls: "plain" as const },
      { text: "schedule", cls: "sub" as const },
    ]};
    expect(fullText(line)).toBe("git schedule");
    const v = visibleChars(line, 5);
    expect(v.tokens.map((t) => t.text).join("")).toBe("git s");
  });
});
```

- [ ] **Step 2: Run test, verify it fails**

Run: `cd site && npm test -- typing`
Expected: FAIL — cannot resolve `./typing`.

- [ ] **Step 3: Implement typing.ts**

Create `site/lib/typing.ts`:
```typescript
import { fullText, type DemoStep, type DemoLine, type Token } from "./demo";

export type TypingState = {
  stepIndex: number;
  lineIndex: number;
  charCount: number;
  phase: "typing" | "pausing" | "done";
};

export function initialTypingState(): TypingState {
  return { stepIndex: 0, lineIndex: 0, charCount: 0, phase: "typing" };
}

export function advance(state: TypingState, steps: DemoStep[]): TypingState {
  const step = steps[state.stepIndex];
  const line = step.lines[state.lineIndex];
  const isInput = line.kind === "input";
  const target = fullText(line).length;

  // Typing an input line: one char per tick until complete.
  if (isInput && state.charCount < target) {
    return { ...state, charCount: state.charCount + 1 };
  }

  // Line complete (or non-input revealed whole): move to next line.
  const nextLineIndex = state.lineIndex + 1;
  if (nextLineIndex < step.lines.length) {
    return { ...state, lineIndex: nextLineIndex, charCount: 0 };
  }

  // Step complete: move to next step, looping to 0 after the last.
  const nextStepIndex = (state.stepIndex + 1) % steps.length;
  return { stepIndex: nextStepIndex, lineIndex: 0, charCount: 0, phase: "typing" };
}

export function visibleChars(line: DemoLine, charCount: number): { tokens: Token[] } {
  const out: Token[] = [];
  let remaining = charCount;
  for (const tok of line.tokens) {
    if (remaining <= 0) break;
    if (tok.text.length <= remaining) {
      out.push(tok);
      remaining -= tok.text.length;
    } else {
      out.push({ text: tok.text.slice(0, remaining), cls: tok.cls });
      remaining = 0;
    }
  }
  return { tokens: out };
}
```

- [ ] **Step 4: Run tests, verify pass**

Run: `cd site && npm test -- typing`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add site/lib/typing.ts site/lib/typing.test.ts
git commit -m "Add pure typing-animation engine"
```

---

## Task 5: CopyButton component (tested)

**Files:**
- Create: `site/lib/copy.ts`
- Test: `site/lib/copy.test.ts`
- Create: `site/components/CopyButton.tsx`

**Interfaces:**
- Produces:
  - `copyText(text: string): Promise<boolean>` — writes to clipboard, returns success.
  - `<CopyButton text={string} label?={string} />` — client component; shows "copy" then "copied ✓" for ~1.5s.

- [ ] **Step 1: Write the failing test for copyText**

Create `site/lib/copy.test.ts`:
```typescript
import { describe, it, expect, vi, beforeEach } from "vitest";
import { copyText } from "./copy";

describe("copyText", () => {
  beforeEach(() => {
    Object.assign(navigator, {
      clipboard: { writeText: vi.fn().mockResolvedValue(undefined) },
    });
  });
  it("writes the given text and returns true", async () => {
    const ok = await copyText("hello");
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith("hello");
    expect(ok).toBe(true);
  });
  it("returns false when clipboard throws", async () => {
    (navigator.clipboard.writeText as any).mockRejectedValueOnce(new Error("no"));
    const ok = await copyText("x");
    expect(ok).toBe(false);
  });
});
```

- [ ] **Step 2: Run test, verify it fails**

Run: `cd site && npm test -- copy`
Expected: FAIL — cannot resolve `./copy`.

- [ ] **Step 3: Implement copy.ts**

Create `site/lib/copy.ts`:
```typescript
export async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}
```

- [ ] **Step 4: Run test, verify pass**

Run: `cd site && npm test -- copy`
Expected: PASS (2 tests).

- [ ] **Step 5: Implement CopyButton**

Create `site/components/CopyButton.tsx`:
```tsx
"use client";
import { useState } from "react";
import { copyText } from "@/lib/copy";

export function CopyButton({ text, label = "copy" }: { text: string; label?: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      type="button"
      onClick={async () => {
        if (await copyText(text)) {
          setCopied(true);
          setTimeout(() => setCopied(false), 1500);
        }
      }}
      style={{
        fontFamily: "var(--font-mono)",
        fontSize: 12,
        color: copied ? "var(--c-ok)" : "var(--accent)",
        background: "transparent",
        border: "1px solid var(--border-2)",
        borderRadius: 7,
        padding: "5px 10px",
        cursor: "pointer",
      }}
      aria-label={copied ? "Copied" : "Copy to clipboard"}
    >
      {copied ? "copied ✓" : `⧉ ${label}`}
    </button>
  );
}
```

- [ ] **Step 6: Commit**

```bash
git add site/lib/copy.ts site/lib/copy.test.ts site/components/CopyButton.tsx
git commit -m "Add CopyButton and clipboard helper"
```

---

## Task 6: Terminal component (animated)

**Files:**
- Create: `site/components/Terminal.tsx`

**Interfaces:**
- Consumes: `DEMO_SEQUENCE`, `ASCII_ART`, type `Token` from `@/lib/demo`; `initialTypingState`, `advance`, `visibleChars` from `@/lib/typing`.
- Produces: `<Terminal />` — self-contained animated terminal window. No props.

- [ ] **Step 1: Implement Terminal.tsx**

Create `site/components/Terminal.tsx`:
```tsx
"use client";
import { useEffect, useReducer, useState } from "react";
import { DEMO_SEQUENCE, ASCII_ART, type Token, type DemoLine } from "@/lib/demo";
import { initialTypingState, advance, visibleChars } from "@/lib/typing";

const TOKEN_COLOR: Record<Token["cls"], string> = {
  plain: "var(--text)",
  prompt: "var(--accent)",
  path: "var(--c-path)",
  sub: "var(--accent)",
  str: "var(--c-str)",
  flag: "var(--c-flag)",
  ok: "var(--c-ok)",
  muted: "var(--muted)",
  comment: "var(--c-comment)",
};

function Tokens({ tokens }: { tokens: Token[] }) {
  return (
    <>
      {tokens.map((t, i) => (
        <span key={i} style={{ color: TOKEN_COLOR[t.cls] }}>{t.text}</span>
      ))}
    </>
  );
}

function prefersReducedMotion(): boolean {
  if (typeof window === "undefined" || !window.matchMedia) return false;
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function Terminal() {
  const [state, tick] = useReducer(
    (s: ReturnType<typeof initialTypingState>) => advance(s, DEMO_SEQUENCE),
    initialTypingState()
  );
  const [reduced, setReduced] = useState(false);

  useEffect(() => {
    if (prefersReducedMotion()) { setReduced(true); return; }
    const id = setInterval(() => tick(), 55);
    return () => clearInterval(id);
  }, []);

  // Build the lines to render: all completed steps' lines for the current step,
  // plus the in-progress line trimmed to charCount.
  const step = DEMO_SEQUENCE[state.stepIndex];
  const rendered: DemoLine[] = [];
  for (let i = 0; i < state.lineIndex; i++) rendered.push(step.lines[i]);
  const current = step.lines[state.lineIndex];
  const currentTrimmed: DemoLine | null = current
    ? { kind: current.kind, tokens: current.kind === "input"
        ? visibleChars(current, state.charCount).tokens
        : current.tokens }
    : null;

  // Reveal the ASCII art band during the final step of each loop cycle.
  const showArt = state.stepIndex === DEMO_SEQUENCE.length - 1;

  return (
    <div
      style={{
        background: "linear-gradient(180deg, var(--panel), var(--panel-2))",
        border: "1px solid var(--border-2)",
        borderRadius: 14,
        boxShadow: "0 28px 64px -24px rgba(0,0,0,0.75)",
        overflow: "hidden",
        maxWidth: 760,
        margin: "0 auto",
        textAlign: "left",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 7, padding: "12px 14px", background: "rgba(0,0,0,0.25)", borderBottom: "1px solid var(--border)" }}>
        <span style={{ width: 12, height: 12, borderRadius: "50%", background: "#ff5f57" }} />
        <span style={{ width: 12, height: 12, borderRadius: "50%", background: "#febc2e" }} />
        <span style={{ width: 12, height: 12, borderRadius: "50%", background: "#28c840" }} />
        <span style={{ marginLeft: 9, fontFamily: "var(--font-mono)", fontSize: 12, color: "var(--dim)" }}>
          mafex — zsh — 92×24
        </span>
      </div>
      <div style={{ padding: "18px 18px 22px", fontFamily: "var(--font-mono)", fontSize: 13.5, lineHeight: 1.85, minHeight: 220, whiteSpace: "pre-wrap" }}>
        {reduced
          ? DEMO_SEQUENCE.flatMap((s, si) =>
              s.lines.map((l, li) => <div key={`${si}-${li}`}><Tokens tokens={l.tokens} /></div>)
            )
          : (
            <>
              {rendered.map((l, i) => <div key={i}><Tokens tokens={l.tokens} /></div>)}
              {currentTrimmed && <div><Tokens tokens={currentTrimmed.tokens} />
                <span style={{ display: "inline-block", width: 7, height: 15, background: "var(--accent)", marginLeft: 2, verticalAlign: -2, animation: "gsblink 1.1s steps(1) infinite" }} />
              </div>}
              {showArt && (
                <pre style={{ margin: "14px 0 0", color: "var(--accent)", fontSize: 10.5, lineHeight: 1.25, opacity: 0.85 }}>
                  {ASCII_ART}
                </pre>
              )}
            </>
          )}
      </div>
      <style>{`@keyframes gsblink { 50% { opacity: 0; } }`}</style>
    </div>
  );
}
```

- [ ] **Step 2: Temporarily render it to verify build**

Replace `site/app/page.tsx` with a minimal harness:
```tsx
import { Terminal } from "@/components/Terminal";
export default function Home() {
  return <main style={{ padding: 40 }}><Terminal /></main>;
}
```

- [ ] **Step 3: Verify build**

Run: `cd site && npm run build`
Expected: build succeeds (catches SSR/type errors in Terminal).

- [ ] **Step 4: Visual check**

Run `cd site && npm run dev`, open http://localhost:3000, confirm the terminal types commands, loops, and shows the blinking caret. Stop the dev server.

- [ ] **Step 5: Commit**

```bash
git add site/components/Terminal.tsx site/app/page.tsx
git commit -m "Add animated terminal component"
```

---

## Task 7: Nav and Hero

**Files:**
- Create: `site/components/Nav.tsx`, `site/components/Hero.tsx`

**Interfaces:**
- Consumes: `VERSION`, `GITHUB_URL` from `@/lib/demo`; `<Terminal />` from `@/components/Terminal`.
- Produces: `<Nav />`, `<Hero />` (Hero renders the version eyebrow, headline, lede, CTAs, and the Terminal).

- [ ] **Step 1: Implement Nav.tsx**

Create `site/components/Nav.tsx`:
```tsx
import { GITHUB_URL } from "@/lib/demo";

export function Nav() {
  return (
    <nav style={{ position: "sticky", top: 0, zIndex: 10, backdropFilter: "blur(10px)", background: "rgba(12,10,9,0.7)", borderBottom: "1px solid var(--border)" }}>
      <div style={{ maxWidth: 1120, margin: "0 auto", padding: "16px 24px", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 10, fontFamily: "var(--font-mono)", fontWeight: 600, fontSize: 15 }}>
          <span style={{ width: 24, height: 24, borderRadius: 7, background: "linear-gradient(135deg, var(--accent-hi), var(--accent))", display: "grid", placeItems: "center", color: "var(--ink)", fontWeight: 800, fontSize: 13 }}>gs</span>
          git<span style={{ color: "var(--dim)" }}>-</span>schedule
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 24, fontSize: 14 }}>
          <a href="#features" style={{ color: "var(--muted)", textDecoration: "none" }}>Features</a>
          <a href="#how" style={{ color: "var(--muted)", textDecoration: "none" }}>How it works</a>
          <a href="#install" style={{ color: "var(--muted)", textDecoration: "none" }}>Install</a>
          <a href={GITHUB_URL} style={{ fontFamily: "var(--font-mono)", fontSize: 13, color: "var(--text)", textDecoration: "none", border: "1px solid var(--border-2)", padding: "7px 14px", borderRadius: 8 }}>★ Star</a>
        </div>
      </div>
    </nav>
  );
}
```

- [ ] **Step 2: Implement Hero.tsx**

Create `site/components/Hero.tsx`:
```tsx
import { VERSION, GITHUB_URL } from "@/lib/demo";
import { Terminal } from "@/components/Terminal";

export function Hero() {
  return (
    <header style={{ maxWidth: 1120, margin: "0 auto", padding: "96px 24px 72px", textAlign: "center" }}>
      <span style={{ display: "inline-flex", alignItems: "center", gap: 8, fontFamily: "var(--font-mono)", fontSize: 12.5, color: "var(--accent)", background: "var(--accent-dim)", border: "1px solid var(--accent-bd)", padding: "5px 12px", borderRadius: 999, marginBottom: 22 }}>
        <span style={{ width: 6, height: 6, borderRadius: "50%", background: "var(--accent)" }} />
        v{VERSION} · macOS · Linux · Windows
      </span>
      <h1 style={{ fontSize: "clamp(34px, 5vw, 54px)", lineHeight: 1.05, letterSpacing: "-1.5px", fontWeight: 900, margin: 0 }}>
        Delayed send, for your <span style={{ color: "var(--accent)" }}>git commits</span>.
      </h1>
      <p style={{ color: "var(--muted)", fontSize: "clamp(16px, 1.6vw, 18.5px)", margin: "20px auto 0", maxWidth: "44ch" }}>
        Write code at midnight. Let it commit at 9 AM — locally or in the cloud. A Rust CLI for scheduling commits and pull requests.
      </p>
      <div style={{ display: "flex", gap: 12, justifyContent: "center", marginTop: 30, flexWrap: "wrap" }}>
        <a href="#install" style={{ fontFamily: "var(--font-mono)", fontSize: 13.5, fontWeight: 600, padding: "12px 18px", borderRadius: 10, background: "linear-gradient(180deg, var(--accent-hi), var(--accent))", color: "var(--ink)", textDecoration: "none", boxShadow: "0 6px 18px -6px rgba(240,169,58,0.55)" }}>▼ Install</a>
        <a href={GITHUB_URL} style={{ fontFamily: "var(--font-mono)", fontSize: 13.5, fontWeight: 600, padding: "12px 18px", borderRadius: 10, background: "var(--panel)", color: "var(--text)", border: "1px solid var(--border-2)", textDecoration: "none" }}>★ GitHub</a>
      </div>
      <p style={{ fontFamily: "var(--font-mono)", fontSize: 13, color: "var(--muted)", marginTop: 18 }}>Built in Rust · MIT licensed · zero config</p>
      <div style={{ marginTop: 40 }}><Terminal /></div>
    </header>
  );
}
```

- [ ] **Step 3: Wire into page and verify build**

Replace `site/app/page.tsx`:
```tsx
import { Nav } from "@/components/Nav";
import { Hero } from "@/components/Hero";
export default function Home() {
  return <main><Nav /><Hero /></main>;
}
```
Run: `cd site && npm run build`
Expected: build succeeds.

- [ ] **Step 4: Commit**

```bash
git add site/components/Nav.tsx site/components/Hero.tsx site/app/page.tsx
git commit -m "Add nav and hero sections"
```

---

## Task 8: Features and How It Works

**Files:**
- Create: `site/components/Features.tsx`, `site/components/HowItWorks.tsx`

**Interfaces:**
- Produces: `<Features />` (id="features"), `<HowItWorks />` (id="how"). No props; content is inline data arrays.

- [ ] **Step 1: Implement Features.tsx**

Create `site/components/Features.tsx`:
```tsx
const FEATURES = [
  { icon: "⏲", title: "Schedule commits", body: "Relative or absolute time — --in 2h, --at \"fri 3pm\". Up to 7 days out." },
  { icon: "☁", title: "Remote via Actions", body: "Runs on GitHub Actions even when your machine is asleep or off." },
  { icon: "⤴", title: "Scheduled PRs", body: "Open pull requests on a timer with the gh CLI and your staged changes." },
  { icon: "✎", title: "Edit, cancel, undo", body: "Reschedule the time, change the message, or undo your last schedule." },
  { icon: "↻", title: "Failed queue + retry", body: "Missed or failed commits move to a queue. Re-stage the files and retry." },
  { icon: "◔", title: "System notifications", body: "Native success and failure notifications fire when commits run." },
];

export function Features() {
  return (
    <section id="features" style={{ maxWidth: 1120, margin: "0 auto", padding: "40px 24px 80px" }}>
      <div style={{ display: "grid", gap: 18, gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))" }}>
        {FEATURES.map((f) => (
          <div key={f.title} style={{ background: "linear-gradient(180deg, var(--panel), var(--bg-2))", border: "1px solid var(--border)", borderRadius: 12, padding: 22 }}>
            <div style={{ width: 34, height: 34, borderRadius: 9, background: "var(--accent-dim)", border: "1px solid var(--accent-bd)", display: "grid", placeItems: "center", marginBottom: 14, fontFamily: "var(--font-mono)", color: "var(--accent)" }}>{f.icon}</div>
            <h3 style={{ fontSize: 15.5, margin: "0 0 6px", letterSpacing: "-0.2px" }}>{f.title}</h3>
            <p style={{ color: "var(--muted)", fontSize: 13.5, margin: 0 }}>{f.body}</p>
          </div>
        ))}
      </div>
    </section>
  );
}
```

- [ ] **Step 2: Implement HowItWorks.tsx**

Create `site/components/HowItWorks.tsx`:
```tsx
const STEPS = [
  { n: "01", t: "Capture", d: "The CLI snapshots your staged changes as a patch and unstages them so you can keep working." },
  { n: "02", t: "Store", d: "The schedule is saved to ~/.git-schedule/ and a lightweight daemon picks it up." },
  { n: "03", t: "Wait", d: "The daemon waits until the scheduled time — locally, or on GitHub Actions with --remote." },
  { n: "04", t: "Commit", d: "It applies the patch, creates the commit, optionally pushes, and sends a notification." },
];

export function HowItWorks() {
  return (
    <section id="how" style={{ maxWidth: 860, margin: "0 auto", padding: "20px 24px 90px" }}>
      <p style={{ textAlign: "center", color: "var(--dim)", fontFamily: "var(--font-mono)", fontSize: 12, letterSpacing: 2, textTransform: "uppercase", margin: "0 0 6px" }}>how it works</p>
      <h2 style={{ textAlign: "center", fontSize: 26, letterSpacing: "-0.6px", margin: "0 0 40px", fontWeight: 700 }}>Commit on your schedule, not your keyboard's.</h2>
      <div style={{ display: "grid", gap: 14 }}>
        {STEPS.map((s) => (
          <div key={s.n} style={{ display: "flex", gap: 18, alignItems: "flex-start", background: "var(--panel)", border: "1px solid var(--border)", borderRadius: 12, padding: "18px 20px" }}>
            <span style={{ fontFamily: "var(--font-mono)", fontSize: 18, color: "var(--accent)", fontWeight: 700 }}>{s.n}</span>
            <div>
              <h3 style={{ margin: "0 0 4px", fontSize: 16 }}>{s.t}</h3>
              <p style={{ margin: 0, color: "var(--muted)", fontSize: 14 }}>{s.d}</p>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
```

- [ ] **Step 3: Verify build**

Add both to `site/app/page.tsx` (after Hero) and run `cd site && npm run build`.
Expected: build succeeds.

- [ ] **Step 4: Commit**

```bash
git add site/components/Features.tsx site/components/HowItWorks.tsx site/app/page.tsx
git commit -m "Add features and how-it-works sections"
```

---

## Task 9: Install, Command Reference, Footer

**Files:**
- Create: `site/components/Install.tsx`, `site/components/CommandReference.tsx`, `site/components/Footer.tsx`

**Interfaces:**
- Consumes: `<CopyButton />` from `@/components/CopyButton`; `VERSION`, `GITHUB_URL`, `BREW_TAP` from `@/lib/demo`.
- Produces: `<Install />` (id="install"), `<CommandReference />`, `<Footer />`.

- [ ] **Step 1: Implement Install.tsx**

Create `site/components/Install.tsx`:
```tsx
"use client";
import { useState } from "react";
import { CopyButton } from "@/components/CopyButton";

const TABS = [
  { key: "mac", label: "macOS / Linux", cmd: "brew install mafex11/tap/git-schedule" },
  { key: "win", label: "Windows", cmd: "irm https://raw.githubusercontent.com/mafex11/git-schedule/main/install.ps1 | iex" },
  { key: "src", label: "From source", cmd: "git clone https://github.com/mafex11/git-schedule.git\ncd git-schedule && cargo build --release" },
];

export function Install() {
  const [active, setActive] = useState("mac");
  const tab = TABS.find((t) => t.key === active)!;
  return (
    <section id="install" style={{ maxWidth: 760, margin: "0 auto", padding: "20px 24px 90px" }}>
      <p style={{ textAlign: "center", color: "var(--dim)", fontFamily: "var(--font-mono)", fontSize: 12, letterSpacing: 2, textTransform: "uppercase", margin: "0 0 6px" }}>install</p>
      <h2 style={{ textAlign: "center", fontSize: 26, letterSpacing: "-0.6px", margin: "0 0 26px", fontWeight: 700 }}>Up and running in 10 seconds.</h2>
      <div style={{ display: "flex", gap: 8, marginBottom: 14, flexWrap: "wrap", justifyContent: "center" }}>
        {TABS.map((t) => (
          <button key={t.key} type="button" onClick={() => setActive(t.key)} style={{ fontFamily: "var(--font-mono)", fontSize: 13, padding: "8px 14px", borderRadius: 8, cursor: "pointer", background: active === t.key ? "var(--accent-dim)" : "transparent", color: active === t.key ? "var(--accent)" : "var(--muted)", border: `1px solid ${active === t.key ? "var(--accent-bd)" : "var(--border-2)"}` }}>{t.label}</button>
        ))}
      </div>
      <div style={{ background: "var(--panel)", border: "1px solid var(--border-2)", borderRadius: 11, padding: "16px 18px", display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 12 }}>
        <pre style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: 13.5, color: "var(--text)", whiteSpace: "pre-wrap", overflowX: "auto" }}>{tab.cmd}</pre>
        <CopyButton text={tab.cmd} />
      </div>
      <p style={{ textAlign: "center", fontFamily: "var(--font-mono)", fontSize: 12.5, color: "var(--muted)", marginTop: 14 }}>then verify: <span style={{ color: "var(--accent)" }}>git schedule --version</span></p>
    </section>
  );
}
```

- [ ] **Step 2: Implement CommandReference.tsx**

Create `site/components/CommandReference.tsx`:
```tsx
const COMMANDS = [
  { c: 'git schedule "msg" --in 2h', d: "Schedule a commit in relative time" },
  { c: 'git schedule "msg" --at "fri 9am"', d: "Schedule at an absolute time / weekday" },
  { c: "git schedule … --push", d: "Auto-push after the commit lands" },
  { c: "git schedule … --remote", d: "Run via GitHub Actions (PC can be off)" },
  { c: 'git schedule pr "title" --to dev', d: "Schedule a pull request" },
  { c: "git schedule list", d: "List pending schedules" },
  { c: "git schedule status", d: "Daemon status and next commit" },
  { c: "git schedule edit ID --in 3h", d: "Reschedule or edit the message" },
  { c: "git schedule cancel ID", d: "Cancel a schedule" },
  { c: "git schedule undo", d: "Cancel the most recent schedule" },
  { c: "git schedule failed", d: "List failed / missed commits" },
  { c: "git schedule retry ID", d: "Re-stage files from a failed commit" },
];

export function CommandReference() {
  return (
    <section style={{ maxWidth: 860, margin: "0 auto", padding: "0 24px 90px" }}>
      <p style={{ textAlign: "center", color: "var(--dim)", fontFamily: "var(--font-mono)", fontSize: 12, letterSpacing: 2, textTransform: "uppercase", margin: "0 0 6px" }}>commands</p>
      <h2 style={{ textAlign: "center", fontSize: 26, letterSpacing: "-0.6px", margin: "0 0 30px", fontWeight: 700 }}>The whole toolbox.</h2>
      <div style={{ border: "1px solid var(--border)", borderRadius: 12, overflow: "hidden" }}>
        {COMMANDS.map((cmd, i) => (
          <div key={cmd.c} style={{ display: "flex", gap: 16, padding: "12px 18px", borderTop: i === 0 ? "none" : "1px solid var(--border)", alignItems: "baseline", flexWrap: "wrap" }}>
            <code style={{ fontFamily: "var(--font-mono)", fontSize: 13, color: "var(--accent)", flex: "1 1 280px" }}>{cmd.c}</code>
            <span style={{ color: "var(--muted)", fontSize: 13.5, flex: "1 1 220px" }}>{cmd.d}</span>
          </div>
        ))}
      </div>
      <p style={{ textAlign: "center", fontFamily: "var(--font-mono)", fontSize: 13, color: "var(--muted)", marginTop: 16 }}>full reference on <a href="https://github.com/mafex11/git-schedule" style={{ color: "var(--accent)" }}>GitHub →</a></p>
    </section>
  );
}
```

- [ ] **Step 3: Implement Footer.tsx**

Create `site/components/Footer.tsx`:
```tsx
import { VERSION, GITHUB_URL } from "@/lib/demo";

export function Footer() {
  return (
    <footer style={{ borderTop: "1px solid var(--border)", padding: "28px 24px", textAlign: "center", fontFamily: "var(--font-mono)", fontSize: 13, color: "var(--dim)" }}>
      git-schedule v{VERSION} · MIT · built in Rust · <a href={GITHUB_URL} style={{ color: "var(--muted)" }}>GitHub</a>
    </footer>
  );
}
```

- [ ] **Step 4: Verify build**

Add the three components to `site/app/page.tsx` and run `cd site && npm run build`.
Expected: build succeeds.

- [ ] **Step 5: Commit**

```bash
git add site/components/Install.tsx site/components/CommandReference.tsx site/components/Footer.tsx site/app/page.tsx
git commit -m "Add install, command reference, and footer sections"
```

---

## Task 10: Assemble page + final verification

**Files:**
- Modify: `site/app/page.tsx` (final composition)

**Interfaces:**
- Consumes: all section components.

- [ ] **Step 1: Final page.tsx**

Replace `site/app/page.tsx` with:
```tsx
import { Nav } from "@/components/Nav";
import { Hero } from "@/components/Hero";
import { Features } from "@/components/Features";
import { HowItWorks } from "@/components/HowItWorks";
import { Install } from "@/components/Install";
import { CommandReference } from "@/components/CommandReference";
import { Footer } from "@/components/Footer";

export default function Home() {
  return (
    <main>
      <Nav />
      <Hero />
      <Features />
      <HowItWorks />
      <Install />
      <CommandReference />
      <Footer />
    </main>
  );
}
```

- [ ] **Step 2: Run the full test suite**

Run: `cd site && npm test`
Expected: all unit tests pass (demo, typing, copy).

- [ ] **Step 3: Run lint and build**

Run: `cd site && npm run lint && npm run build`
Expected: no lint errors; build succeeds; `site/out/index.html` exists.

- [ ] **Step 4: Visual verification in browser**

Run `cd site && npm run dev`. Open http://localhost:3000 and confirm:
- Hero headline + animated terminal typing/looping with caret and ASCII reveal
- Six feature cards, four how-it-works steps
- Install tabs switch and copy buttons show "copied ✓"
- Command table renders; footer shows version
- Resize to mobile width: terminal and grids reflow without horizontal page scroll
Stop the dev server.

- [ ] **Step 5: Verify reduced-motion fallback**

In browser devtools, emulate `prefers-reduced-motion: reduce`, reload, confirm the terminal shows the full static transcript (no typing animation) and the page is fully readable.

- [ ] **Step 6: Commit**

```bash
git add site/app/page.tsx
git commit -m "Assemble full landing page"
```

---

## Self-Review Notes (resolved)

- **Spec coverage:** nav (T7), centered hero + animated terminal + ASCII (T6, T7), features ×6 (T8), how-it-works ×4 (T8), install tabs + copy (T9), command reference (T9), footer (T9), palette + fonts (T2), static export + Vercel (T1), reduced-motion (T6, T10). SEO/meta covered in T2 layout metadata. OG image deferred (see below).
- **Type consistency:** `Token`/`DemoLine`/`DemoStep` defined in T3, consumed unchanged in T4 and T6; `TypingState` defined T4, used T6; `copyText`/`CopyButton` defined T5, used T9.
- **Deferred (non-blocking, do after first deploy):** generated OG image asset and favicon from the logo mark — the metadata references a Twitter summary_large_image card but the image file itself can be added once the visual design is final. Note this to the user at handoff.
