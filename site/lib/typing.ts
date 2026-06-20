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
