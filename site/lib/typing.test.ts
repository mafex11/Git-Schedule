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
