const FEATURES = [
  { icon: "⏲", title: "Schedule commits", body: "Relative or absolute time — --in 2h, --at \"fri 3pm\". Up to 7 days out." },
  { icon: "☁", title: "Remote via Actions", body: "Runs on GitHub Actions even when your machine is asleep or off." },
  { icon: "⤴", title: "Scheduled PRs", body: "Open pull requests on a timer with the gh CLI and your staged changes." },
  { icon: "✎", title: "Edit, cancel, undo", body: "Reschedule the time, change the message, or undo your last schedule." },
  { icon: "↻", title: "Failed queue + retry", body: "Missed or failed commits move to a queue. Re-stage the files and retry." },
  { icon: "◔", title: "System notifications", body: "Native success and failure notifications fire when commits run." },
];

export function Features() {
  return (
    <section id="features" style={{ maxWidth: 1120, margin: "0 auto", padding: "40px 24px 80px" }}>
      <div style={{ display: "grid", gap: 18, gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))" }}>
        {FEATURES.map((f) => (
          <div key={f.title} style={{ background: "linear-gradient(180deg, var(--panel), var(--bg-2))", border: "1px solid var(--border)", borderRadius: 12, padding: 22 }}>
            <div style={{ width: 34, height: 34, borderRadius: 9, background: "var(--accent-dim)", border: "1px solid var(--accent-bd)", display: "grid", placeItems: "center", marginBottom: 14, fontFamily: "var(--font-mono)", color: "var(--accent)" }}>{f.icon}</div>
            <h3 style={{ fontSize: 15.5, margin: "0 0 6px", letterSpacing: "-0.2px" }}>{f.title}</h3>
            <p style={{ color: "var(--muted)", fontSize: 13.5, margin: 0 }}>{f.body}</p>
          </div>
        ))}
      </div>
    </section>
  );
}
