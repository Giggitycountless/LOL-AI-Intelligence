import { describe, expect, it, vi } from "vitest";

const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: mockInvoke }));

const { callBackend, isCommandError } = await import("../commands");

describe("callBackend", () => {
  it("returns result on success", async () => {
    mockInvoke.mockResolvedValueOnce(42);
    const result = await callBackend<number>("my-command");
    expect(result).toBe(42);
  });

  it("passes args to invoke", async () => {
    mockInvoke.mockResolvedValueOnce(null);
    await callBackend("cmd", { foo: "bar", count: 3 });
    expect(mockInvoke).toHaveBeenCalledWith("cmd", { foo: "bar", count: 3 });
  });

  it("normalizes a CommandError-shaped rejection", async () => {
    const error = { code: "storage", message: "disk full" };
    mockInvoke.mockRejectedValueOnce(error);
    await expect(callBackend("cmd")).rejects.toEqual({ code: "storage", message: "disk full" });
  });

  it("normalizes a plain Error into a CommandError", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("boom"));
    await expect(callBackend("cmd")).rejects.toEqual({ code: "internal", message: "boom" });
  });

  it("normalizes a non-Error value into a CommandError", async () => {
    mockInvoke.mockRejectedValueOnce("string error");
    await expect(callBackend("cmd")).rejects.toEqual({ code: "internal", message: "Command failed" });
  });
});

describe("isCommandError", () => {
  it("returns true for valid CommandError", () => {
    expect(isCommandError({ code: "internal", message: "oops" })).toBe(true);
  });

  it("returns true for all valid code values", () => {
    const codes = ["validation", "storage", "clientUnavailable", "clientAccess", "integration", "internal"];
    for (const code of codes) {
      expect(isCommandError({ code, message: "msg" })).toBe(true);
    }
  });

  it("returns false for null", () => {
    expect(isCommandError(null)).toBe(false);
  });

  it("returns false for undefined", () => {
    expect(isCommandError(undefined)).toBe(false);
  });

  it("returns false when code is missing", () => {
    expect(isCommandError({ message: "oops" })).toBe(false);
  });

  it("returns false when message is missing", () => {
    expect(isCommandError({ code: "internal" })).toBe(false);
  });

  it("returns false when code is not a string", () => {
    expect(isCommandError({ code: 123, message: "oops" })).toBe(false);
  });

  it("returns false when message is not a string", () => {
    expect(isCommandError({ code: "internal", message: 123 })).toBe(false);
  });

  it("returns false for a plain string", () => {
    expect(isCommandError("error")).toBe(false);
  });

  it("returns false for a number", () => {
    expect(isCommandError(42)).toBe(false);
  });
});
