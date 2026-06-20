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
