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
