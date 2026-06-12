// @vitest-environment jsdom
import { render, screen } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mockUseAppCore = vi.fn();
const mockFetchAiAnalysis = vi.fn();
const mockInvoke = vi.fn();

vi.mock("../../state/AppStateProvider", () => ({
  useAppCore: () => mockUseAppCore(),
}));

vi.mock("../../backend/leagueClient", () => ({
  fetchAiAnalysis: (scope: string) => mockFetchAiAnalysis(scope),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (command: string, args?: unknown) => mockInvoke(command, args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

function createSnapshot(aiSettings: { aiBaseUrl?: string; aiApiKey?: string; aiModel?: string } = {}) {
  return {
    settings: {
      startupPage: "dashboard",
      language: "zh",
      compactMode: false,
      aiBaseUrl: aiSettings.aiBaseUrl ?? null,
      aiApiKey: aiSettings.aiApiKey ?? null,
      aiModel: aiSettings.aiModel ?? null,
    },
  };
}

const configuredAi = {
  aiBaseUrl: "https://api.example.com/v1",
  aiApiKey: "sk-test",
  aiModel: "test-model",
};

beforeEach(() => {
  vi.clearAllMocks();
  mockUseAppCore.mockReturnValue({ snapshot: createSnapshot() });
  mockFetchAiAnalysis.mockResolvedValue(null);
  mockInvoke.mockResolvedValue({ recentMatches: [] });
});

describe("Advisor", () => {
  it("shows the unconfigured notice and disables analysis when AI is not set up", async () => {
    const { Advisor } = await import("../Advisor");

    render(React.createElement(Advisor));

    expect(await screen.findByText(/未配置 AI/)).toBeDefined();
    expect(screen.getByText("请先在设置中配置 AI")).toBeDefined();
    const analyzeButton = screen.getByRole("button", { name: "开始分析" });
    expect((analyzeButton as HTMLButtonElement).disabled).toBe(true);
  });

  it("enables analysis and shows the launch hint when AI is configured", async () => {
    mockUseAppCore.mockReturnValue({ snapshot: createSnapshot(configuredAi) });
    const { Advisor } = await import("../Advisor");

    render(React.createElement(Advisor));

    expect(await screen.findByText("选择位置范围，点击「开始分析」")).toBeDefined();
    expect(screen.queryByText(/未配置 AI/)).toBeNull();
    const analyzeButton = screen.getByRole("button", { name: "开始分析" });
    expect((analyzeButton as HTMLButtonElement).disabled).toBe(false);
  });

  it("renders a cached analysis and relabels the button for re-analysis", async () => {
    mockUseAppCore.mockReturnValue({ snapshot: createSnapshot(configuredAi) });
    mockFetchAiAnalysis.mockResolvedValue({
      scope: "all",
      resultText: "**优势**\n补刀稳健，经济领先。",
      gameCountAtAnalysis: 10,
      analyzedAt: "2026-06-01T10:00:00Z",
    });
    const { Advisor } = await import("../Advisor");

    render(React.createElement(Advisor));

    expect(await screen.findByText("补刀稳健，经济领先。")).toBeDefined();
    expect(screen.getByText("优势")).toBeDefined();
    expect(screen.getByRole("button", { name: "重新分析" })).toBeDefined();
  });

  it("prompts a refresh once enough new games accumulate since the last analysis", async () => {
    mockUseAppCore.mockReturnValue({ snapshot: createSnapshot(configuredAi) });
    mockFetchAiAnalysis.mockResolvedValue({
      scope: "all",
      resultText: "分析结果",
      gameCountAtAnalysis: 10,
      analyzedAt: "2026-06-01T10:00:00Z",
    });
    mockInvoke.mockResolvedValue({ recentMatches: new Array(16).fill({}) });
    const { Advisor } = await import("../Advisor");

    render(React.createElement(Advisor));

    expect(await screen.findByText(/已有 6 场新对局/)).toBeDefined();
    expect(mockInvoke).toHaveBeenCalledWith("get_league_self_snapshot", {
      input: { matchLimit: 100 },
    });
  });
});
