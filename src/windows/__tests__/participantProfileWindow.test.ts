import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/event", () => ({ emit: vi.fn(), emitTo: vi.fn() }));
vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: Object.assign(vi.fn(), { getByLabel: vi.fn() }),
}));

const {
  isSelectedParticipant,
  participantProfileHash,
  selectionFromParticipantProfileHash,
  sameParticipant,
} = await import("../participantProfileWindow");

describe("isSelectedParticipant", () => {
  it("returns true for valid selection", () => {
    expect(isSelectedParticipant({ gameId: 123, participantId: 5 })).toBe(true);
  });

  it("returns false for null", () => {
    expect(isSelectedParticipant(null)).toBe(false);
  });

  it("returns false for undefined", () => {
    expect(isSelectedParticipant(undefined)).toBe(false);
  });

  it("returns false for non-object", () => {
    expect(isSelectedParticipant("string")).toBe(false);
  });

  it("returns false for negative gameId", () => {
    expect(isSelectedParticipant({ gameId: -1, participantId: 1 })).toBe(false);
  });

  it("returns false for zero gameId", () => {
    expect(isSelectedParticipant({ gameId: 0, participantId: 1 })).toBe(false);
  });

  it("returns false for non-integer gameId", () => {
    expect(isSelectedParticipant({ gameId: 1.5, participantId: 1 })).toBe(false);
  });

  it("returns false for negative participantId", () => {
    expect(isSelectedParticipant({ gameId: 1, participantId: -1 })).toBe(false);
  });

  it("returns false for missing fields", () => {
    expect(isSelectedParticipant({ gameId: 1 })).toBe(false);
    expect(isSelectedParticipant({ participantId: 1 })).toBe(false);
  });
});

describe("participantProfileHash", () => {
  it("produces correct hash string", () => {
    const hash = participantProfileHash({ gameId: 42, participantId: 7 });
    expect(hash).toBe("#/participant-profile?gameId=42&participantId=7");
  });

  it("encodes special characters", () => {
    const hash = participantProfileHash({ gameId: 1, participantId: 2 });
    expect(hash).toContain("gameId=1");
    expect(hash).toContain("participantId=2");
  });
});

describe("selectionFromParticipantProfileHash", () => {
  it("parses a valid hash", () => {
    const result = selectionFromParticipantProfileHash("#/participant-profile?gameId=42&participantId=7");
    expect(result).toEqual({ gameId: 42, participantId: 7 });
  });

  it("returns null for non-matching prefix", () => {
    expect(selectionFromParticipantProfileHash("#/other?gameId=1&participantId=2")).toBeNull();
  });

  it("returns null for missing gameId", () => {
    expect(selectionFromParticipantProfileHash("#/participant-profile?participantId=7")).toBeNull();
  });

  it("returns null for missing participantId", () => {
    expect(selectionFromParticipantProfileHash("#/participant-profile?gameId=42")).toBeNull();
  });

  it("returns null for non-positive gameId", () => {
    expect(selectionFromParticipantProfileHash("#/participant-profile?gameId=0&participantId=1")).toBeNull();
  });

  it("returns null for non-integer values", () => {
    expect(selectionFromParticipantProfileHash("#/participant-profile?gameId=abc&participantId=1")).toBeNull();
  });

  it("returns null for empty string", () => {
    expect(selectionFromParticipantProfileHash("")).toBeNull();
  });
});

describe("sameParticipant", () => {
  it("returns true for matching participants", () => {
    const left = { gameId: 1, participantId: 2 };
    const right = { gameId: 1, participantId: 2 };
    expect(sameParticipant(left, right)).toBe(true);
  });

  it("returns false for different gameId", () => {
    const left = { gameId: 1, participantId: 2 };
    const right = { gameId: 2, participantId: 2 };
    expect(sameParticipant(left, right)).toBe(false);
  });

  it("returns false for different participantId", () => {
    const left = { gameId: 1, participantId: 2 };
    const right = { gameId: 1, participantId: 3 };
    expect(sameParticipant(left, right)).toBe(false);
  });

  it("returns false when left is null", () => {
    expect(sameParticipant(null, { gameId: 1, participantId: 2 })).toBe(false);
  });
});
