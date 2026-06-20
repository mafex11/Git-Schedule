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
