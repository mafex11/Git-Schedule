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
