import { beforeEach, describe, expect, it, vi } from "vitest";

const mockGetByLabel = vi.fn();
const mockWebviewWindow = vi.fn();

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: Object.assign(mockWebviewWindow, {
    getByLabel: mockGetByLabel,
  }),
}));

vi.mock("../../backend/commands", () => ({
  callBackend: vi.fn().mockResolvedValue(true),
}));

describe("openSelfHistoryOverlayWindow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("does not throw when WebviewWindow.getByLabel rejects", async () => {
    mockGetByLabel.mockRejectedValue(new Error("window not found"));

    const { openSelfHistoryOverlayWindow } = await import(
      "../selfHistoryOverlayWindow"
    );

    // should not throw
    let threw = false;
    try {
      await openSelfHistoryOverlayWindow();
    } catch {
      threw = true;
    }

    expect(threw).toBe(false);
  });

  it("does not throw when new WebviewWindow throws", async () => {
    mockGetByLabel.mockResolvedValue(null);
    mockWebviewWindow.mockImplementation(() => {
      throw new Error("failed to create window");
    });

    const { openSelfHistoryOverlayWindow } = await import(
      "../selfHistoryOverlayWindow"
    );

    let threw = false;
    try {
      await openSelfHistoryOverlayWindow();
    } catch {
      threw = true;
    }

    expect(threw).toBe(false);
  });

  it("resets overlayOpenPromise after failure so retries work", async () => {
    mockGetByLabel.mockRejectedValue(new Error("fail"));

    const { openSelfHistoryOverlayWindow } = await import(
      "../selfHistoryOverlayWindow"
    );

    await openSelfHistoryOverlayWindow();

    // second call should not return a settled promise — it should try again
    mockGetByLabel.mockResolvedValue(null);

    // create a mock window that succeeds
    const mockWindow = {
      once: vi.fn(),
    };
    mockWebviewWindow.mockReturnValue(mockWindow);

    const result = await openSelfHistoryOverlayWindow();
    expect(result).toBe(true);
    expect(mockWebviewWindow).toHaveBeenCalled();
  });
});
