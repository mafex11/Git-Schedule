import { GITHUB_URL } from "@/lib/demo";

export function Nav() {
  return (
    <nav style={{ position: "sticky", top: 0, zIndex: 10, backdropFilter: "blur(10px)", background: "rgba(12,10,9,0.7)", borderBottom: "1px solid var(--border)" }}>
      <div className="gs-nav-inner" style={{ maxWidth: 1120, margin: "0 auto", padding: "16px 24px", display: "flex", alignItems: "center", justifyContent: "space-between", gap: 12 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 10, fontFamily: "var(--font-mono)", fontWeight: 600, fontSize: 15, whiteSpace: "nowrap" }}>
          <span style={{ width: 24, height: 24, borderRadius: 7, background: "linear-gradient(135deg, var(--accent-hi), var(--accent))", display: "grid", placeItems: "center", color: "var(--ink)", fontWeight: 800, fontSize: 13, flexShrink: 0 }}>gs</span>
          git<span style={{ color: "var(--dim)" }}>-</span>schedule
        </div>
        <div className="gs-nav-links" style={{ display: "flex", alignItems: "center", gap: 24, fontSize: 14 }}>
          <a className="gs-nav-link" href="#features" style={{ color: "var(--muted)", textDecoration: "none", whiteSpace: "nowrap" }}>Features</a>
          <a className="gs-nav-link" href="#how" style={{ color: "var(--muted)", textDecoration: "none", whiteSpace: "nowrap" }}>How it works</a>
          <a className="gs-nav-link" href="#install" style={{ color: "var(--muted)", textDecoration: "none", whiteSpace: "nowrap" }}>Install</a>
          <a href={GITHUB_URL} style={{ fontFamily: "var(--font-mono)", fontSize: 13, color: "var(--text)", textDecoration: "none", border: "1px solid var(--border-2)", padding: "7px 14px", borderRadius: 8, whiteSpace: "nowrap", flexShrink: 0 }}>★ Star</a>
        </div>
      </div>
      <style>{`
        @media (max-width: 640px) {
          .gs-nav-links { gap: 14px; }
          .gs-nav-link { display: none; }
        }
      `}</style>
    </nav>
  );
}
