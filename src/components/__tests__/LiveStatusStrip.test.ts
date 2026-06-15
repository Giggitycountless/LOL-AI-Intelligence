import { describe, expect, it } from "vitest";

import { deriveLiveStatus } from "../LiveStatusStrip";
import type { AutoAcceptStatus, LeagueClientStatus } from "../../backend/types";

function status(phase: LeagueClientStatus["phase"]): LeagueClientStatus {
  return { connection: "connected", phase } as LeagueClientStatus;
}

function autoAccept(state: AutoAcceptStatus["state"]): AutoAcceptStatus {
  return { state } as AutoAcceptStatus;
}

describe("deriveLiveStatus", () => {
  it("prioritises the auto-accept queue state over connection phase", () => {
    expect(deriveLiveStatus(status("connected"), autoAccept("searching"), false)).toMatchObject({
      labelKey: "live.inQueue",
      tone: "active",
      pulse: true,
    });
  });

  it("surfaces a ready check as the most urgent state", () => {
    expect(deriveLiveStatus(status("connected"), autoAccept("readyCheckDetected"), false).labelKey).toBe("live.matchReady");
    expect(deriveLiveStatus(status("connected"), autoAccept("accepting"), false).labelKey).toBe("live.matchReady");
  });

  it("shows accepted once the queue pop is taken", () => {
    expect(deriveLiveStatus(status("connected"), autoAccept("accepted"), false).labelKey).toBe("live.accepted");
  });

  it("falls back to connection phase when auto-accept is idle", () => {
    expect(deriveLiveStatus(status("connected"), autoAccept("disabled"), false).labelKey).toBe("live.online");
    expect(deriveLiveStatus(status("partialData"), null, false).labelKey).toBe("live.online");
    expect(deriveLiveStatus(status("connecting"), null, false)).toMatchObject({ labelKey: "live.connecting", pulse: true });
    expect(deriveLiveStatus(status("notLoggedIn"), null, false).labelKey).toBe("live.notLoggedIn");
    expect(deriveLiveStatus(status("notRunning"), null, false).labelKey).toBe("live.offline");
  });

  it("treats an unknown/absent status as loading or offline", () => {
    expect(deriveLiveStatus(undefined, null, true)).toMatchObject({ labelKey: "live.connecting", tone: "pending" });
    expect(deriveLiveStatus(undefined, null, false)).toMatchObject({ labelKey: "live.offline", tone: "offline" });
  });
});
