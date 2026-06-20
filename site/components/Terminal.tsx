"use client";
import { useEffect, useReducer, useSyncExternalStore } from "react";
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

function subscribeReducedMotion(callback: () => void) {
  if (typeof window === "undefined" || !window.matchMedia) return () => {};
  const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
  mq.addEventListener("change", callback);
  return () => mq.removeEventListener("change", callback);
}

function getReducedMotionSnapshot() {
  if (typeof window === "undefined" || !window.matchMedia) return false;
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function Terminal() {
  const [state, tick] = useReducer(
    (s: ReturnType<typeof initialTypingState>) => advance(s, DEMO_SEQUENCE),
    initialTypingState()
  );
  const reduced = useSyncExternalStore(subscribeReducedMotion, getReducedMotionSnapshot, () => false);

  useEffect(() => {
    if (reduced) return;
    const id = setInterval(() => tick(), 55);
    return () => clearInterval(id);
  }, [reduced]);

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
