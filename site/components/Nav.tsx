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
