// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mockUseAppCore = vi.fn();

vi.mock("../state/AppStateProvider", () => ({
  AppStateProvider: ({ children }: { children: React.ReactNode }) => React.createElement(React.Fragment, null, children),
  useAppCore: () => mockUseAppCore(),
}));

vi.mock("../pages/Dashboard", () => ({ Dashboard: () => React.createElement("div", null, "Dashboard Page") }));
vi.mock("../pages/Profile", () => ({ Profile: () => React.createElement("div", null, "Profile Page") }));
vi.mock("../pages/Matches", () => ({ Matches: () => React.createElement("div", null, "Matches Page") }));
vi.mock("../pages/Advisor", () => ({ Advisor: () => React.createElement("div", null, "Advisor Page") }));
vi.mock("../pages/RankedChampions", () => ({ RankedChampions: () => React.createElement("div", null, "Ranked Page") }));
vi.mock("../pages/Activity", () => ({ Activity: () => React.createElement("div", null, "Activity Page") }));
vi.mock("../pages/Settings", () => ({ Settings: () => React.createElement("div", null, "Settings Page") }));
vi.mock("../pages/ParticipantProfileWindow", () => ({
  ParticipantProfileWindow: () => React.createElement("div", null, "Participant Page"),
}));
vi.mock("../pages/SelfHistoryOverlay", () => ({
  SelfHistoryOverlay: () => React.createElement("div", null, "Overlay Page"),
}));
vi.mock("../windows/participantProfileWindow", () => ({
  selectionFromParticipantProfileHash: () => null,
}));
vi.mock("../windows/selfHistoryOverlayWindow", () => ({
  isSelfHistoryOverlayHash: () => false,
}));

function translator(key: string) {
  const labels: Record<string, string> = {
    "nav.dashboard": "Dashboard",
    "nav.profile": "Profile",
    "nav.matches": "Matches",
    "nav.advisor": "Advisor",
    "nav.ranked": "Ranked",
    "nav.activity": "Activity",
    "nav.settings": "Settings",
    "app.name": "LoL Desktop Assistant",
    "app.milestone": "Milestone 5",
    "app.languageToggle": "EN",
    "app.dismiss": "Dismiss",
    "app.loadingState": "Loading",
  };

  return labels[key] ?? key;
}

function createCore(overrides: Record<string, unknown> = {}) {
  return {
    snapshot: {
      settings: {
        startupPage: "dashboard",
        language: "zh",
        compactMode: false,
      },
    },
    feedback: null,
    clearFeedback: vi.fn(),
    refresh: vi.fn().mockResolvedValue(true),
    isLoading: false,
    effectiveLanguage: "zh",
    setLanguagePreference: vi.fn().mockResolvedValue(true),
    t: translator,
    ...overrides,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("AppShell", () => {
  it("applies the configured startup page when the snapshot is ready", async () => {
    mockUseAppCore.mockReturnValue(
      createCore({
        snapshot: {
          settings: {
            startupPage: "activity",
            language: "zh",
            compactMode: false,
          },
        },
      }),
    );

    const { AppShell } = await import("../App");
    render(React.createElement(AppShell));

    expect(await screen.findByText("Activity Page")).toBeDefined();
  });

  it("does not override manual navigation when startup data arrives later", async () => {
    let currentCore = createCore({
      snapshot: null,
      isLoading: true,
    });
    mockUseAppCore.mockImplementation(() => currentCore);

    const { AppShell } = await import("../App");
    const view = render(React.createElement(AppShell));

    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    expect(screen.getByText("Settings Page")).toBeDefined();

    currentCore = createCore({
      snapshot: {
        settings: {
          startupPage: "activity",
          language: "zh",
          compactMode: false,
        },
      },
    });
    view.rerender(React.createElement(AppShell));

    expect(screen.getByText("Settings Page")).toBeDefined();
    expect(screen.queryByText("Activity Page")).toBeNull();
  });

  it("disables the language toggle until startup data is loaded", async () => {
    mockUseAppCore.mockReturnValue(
      createCore({
        snapshot: null,
        isLoading: true,
      }),
    );

    const { AppShell } = await import("../App");
    render(React.createElement(AppShell));

    expect((screen.getByRole("button", { name: "EN" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("shows persistent error with retry button when app state fails to load", async () => {
    const refreshMock = vi.fn().mockResolvedValue(false);
    mockUseAppCore.mockReturnValue(
      createCore({
        snapshot: null,
        isLoading: false,
        feedback: null,
        refresh: refreshMock,
      }),
    );

    const { AppShell } = await import("../App");
    render(React.createElement(AppShell));

    // should show error message, not loading state
    expect(screen.queryByText("Loading")).toBeNull();
    expect(screen.getByText(/failed to load/i)).toBeDefined();

    // retry button should call refresh
    const retryButton = screen.getByRole("button", { name: /retry/i });
    expect(retryButton).toBeDefined();
    fireEvent.click(retryButton);
    expect(refreshMock).toHaveBeenCalled();
  });
});
