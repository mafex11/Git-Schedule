const STEPS = [
  { n: "01", t: "Capture", d: "The CLI snapshots your staged changes as a patch and unstages them so you can keep working." },
  { n: "02", t: "Store", d: "The schedule is saved to ~/.git-schedule/ and a lightweight daemon picks it up." },
  { n: "03", t: "Wait", d: "The daemon waits until the scheduled time — locally, or on GitHub Actions with --remote." },
  { n: "04", t: "Commit", d: "It applies the patch, creates the commit, optionally pushes, and sends a notification." },
];

export function HowItWorks() {
  return (
    <section id="how" style={{ maxWidth: 860, margin: "0 auto", padding: "20px 24px 90px" }}>
      <p style={{ textAlign: "center", color: "var(--dim)", fontFamily: "var(--font-mono)", fontSize: 12, letterSpacing: 2, textTransform: "uppercase", margin: "0 0 6px" }}>how it works</p>
      <h2 style={{ textAlign: "center", fontSize: 26, letterSpacing: "-0.6px", margin: "0 0 40px", fontWeight: 700 }}>Commit on your schedule, not your keyboard's.</h2>
      <div style={{ display: "grid", gap: 14 }}>
        {STEPS.map((s) => (
          <div key={s.n} style={{ display: "flex", gap: 18, alignItems: "flex-start", background: "var(--panel)", border: "1px solid var(--border)", borderRadius: 12, padding: "18px 20px" }}>
            <span style={{ fontFamily: "var(--font-mono)", fontSize: 18, color: "var(--accent)", fontWeight: 700 }}>{s.n}</span>
            <div>
              <h3 style={{ margin: "0 0 4px", fontSize: 16 }}>{s.t}</h3>
              <p style={{ margin: 0, color: "var(--muted)", fontSize: 14 }}>{s.d}</p>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
