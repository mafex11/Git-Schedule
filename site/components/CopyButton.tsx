"use client";
import { useState } from "react";
import { copyText } from "@/lib/copy";

export function CopyButton({ text, label = "copy" }: { text: string; label?: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      type="button"
      onClick={async () => {
        if (await copyText(text)) {
          setCopied(true);
          setTimeout(() => setCopied(false), 1500);
        }
      }}
      style={{
        fontFamily: "var(--font-mono)",
        fontSize: 12,
        color: copied ? "var(--c-ok)" : "var(--accent)",
        background: "transparent",
        border: "1px solid var(--border-2)",
        borderRadius: 7,
        padding: "5px 10px",
        cursor: "pointer",
      }}
      aria-label={copied ? "Copied" : "Copy to clipboard"}
    >
      {copied ? "copied ✓" : `⧉ ${label}`}
    </button>
  );
}
