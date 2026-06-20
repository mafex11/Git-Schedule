const COMMANDS = [
  { c: 'git schedule "msg" --in 2h', d: "Schedule a commit in relative time" },
  { c: 'git schedule "msg" --at "fri 9am"', d: "Schedule at an absolute time / weekday" },
  { c: "git schedule … --push", d: "Auto-push after the commit lands" },
  { c: "git schedule … --remote", d: "Run via GitHub Actions (PC can be off)" },
  { c: 'git schedule pr "title" --to dev', d: "Schedule a pull request" },
  { c: "git schedule list", d: "List pending schedules" },
  { c: "git schedule status", d: "Daemon status and next commit" },
  { c: "git schedule edit ID --in 3h", d: "Reschedule or edit the message" },
  { c: "git schedule cancel ID", d: "Cancel a schedule" },
  { c: "git schedule undo", d: "Cancel the most recent schedule" },
  { c: "git schedule failed", d: "List failed / missed commits" },
  { c: "git schedule retry ID", d: "Re-stage files from a failed commit" },
];

export function CommandReference() {
  return (
    <section style={{ maxWidth: 860, margin: "0 auto", padding: "0 24px 90px" }}>
      <p style={{ textAlign: "center", color: "var(--dim)", fontFamily: "var(--font-mono)", fontSize: 12, letterSpacing: 2, textTransform: "uppercase", margin: "0 0 6px" }}>commands</p>
      <h2 style={{ textAlign: "center", fontSize: 26, letterSpacing: "-0.6px", margin: "0 0 30px", fontWeight: 700 }}>The whole toolbox.</h2>
      <div style={{ border: "1px solid var(--border)", borderRadius: 12, overflow: "hidden" }}>
        {COMMANDS.map((cmd, i) => (
          <div key={cmd.c} style={{ display: "flex", gap: 16, padding: "12px 18px", borderTop: i === 0 ? "none" : "1px solid var(--border)", alignItems: "baseline", flexWrap: "wrap" }}>
            <code style={{ fontFamily: "var(--font-mono)", fontSize: 13, color: "var(--accent)", flex: "1 1 280px" }}>{cmd.c}</code>
            <span style={{ color: "var(--muted)", fontSize: 13.5, flex: "1 1 220px" }}>{cmd.d}</span>
          </div>
        ))}
      </div>
      <p style={{ textAlign: "center", fontFamily: "var(--font-mono)", fontSize: 13, color: "var(--muted)", marginTop: 16 }}>full reference on <a href="https://github.com/mafex11/git-schedule" style={{ color: "var(--accent)" }}>GitHub →</a></p>
    </section>
  );
}
