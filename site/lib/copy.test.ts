import { describe, it, expect, vi, beforeEach } from "vitest";
import { copyText } from "./copy";

describe("copyText", () => {
  beforeEach(() => {
    Object.assign(navigator, {
      clipboard: { writeText: vi.fn().mockResolvedValue(undefined) },
    });
  });
  it("writes the given text and returns true", async () => {
    const ok = await copyText("hello");
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith("hello");
    expect(ok).toBe(true);
  });
  it("returns false when clipboard throws", async () => {
    (navigator.clipboard.writeText as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error("no"));
    const ok = await copyText("x");
    expect(ok).toBe(false);
  });
});
