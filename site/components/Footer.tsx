import { VERSION, GITHUB_URL } from "@/lib/demo";

export function Footer() {
  return (
    <footer style={{ borderTop: "1px solid var(--border)", padding: "28px 24px", textAlign: "center", fontFamily: "var(--font-mono)", fontSize: 13, color: "var(--dim)" }}>
      git-schedule v{VERSION} · MIT · built in Rust · <a href={GITHUB_URL} style={{ color: "var(--muted)" }}>GitHub</a>
    </footer>
  );
}
