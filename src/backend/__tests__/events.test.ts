import { describe, expect, it, vi } from "vitest";

const mockListen = vi.fn();
vi.mock("@tauri-apps/api/event", () => ({ listen: mockListen }));

const { listenWithCleanup } = await import("../events");

describe("listenWithCleanup", () => {
  it("returns an unlisten function", () => {
    mockListen.mockResolvedValueOnce(vi.fn());
    const cleanup = listenWithCleanup("test-event", vi.fn());
    expect(typeof cleanup).toBe("function");
    cleanup();
  });

  it("calls unlisten when disposed after registration resolves", async () => {
    const handlerCleanup = vi.fn();
    mockListen.mockResolvedValueOnce(handlerCleanup);

    const cleanup = listenWithCleanup("test-event", vi.fn());
    await vi.waitFor(() => expect(mockListen).toHaveBeenCalled());

    cleanup();
    expect(handlerCleanup).toHaveBeenCalled();
  });

  it("calls handler cleanup immediately when disposed before registration resolves", async () => {
    let resolveListen!: (value: () => void) => void;
    const handlerCleanup = vi.fn();
    mockListen.mockReturnValueOnce(new Promise((r) => { resolveListen = r; }));

    const cleanup = listenWithCleanup("test-event", vi.fn());
    cleanup();

    resolveListen(handlerCleanup);
    await vi.waitFor(() => expect(handlerCleanup).toHaveBeenCalled());
  });

  it("does not throw on listen rejection", async () => {
    mockListen.mockRejectedValueOnce(new Error("no listener"));

    expect(() => listenWithCleanup("test-event", vi.fn())).not.toThrow();
    await vi.waitFor(() => expect(mockListen).toHaveBeenCalled());
  });
});
