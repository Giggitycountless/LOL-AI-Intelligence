import { describe, expect, it } from "vitest";

import { abilityStatText, abilityTooltipText } from "../selfHistoryOverlayAbilityView";

describe("self history overlay ability view helpers", () => {
  it("formats ability stat arrays with slash separators", () => {
    expect(abilityStatText(["9", "8", "7", "6", "5"], null)).toBe("9/8/7/6/5");
  });

  it("uses a dash for missing ability stat values", () => {
    expect(abilityStatText([], null)).toBe("-");
  });

  it("removes raw LCU template tokens from fallback text", () => {
    expect(abilityStatText([], "@Cooldown@")).toBe("-");
    expect(abilityTooltipText("@SpellModifierDescriptionAppend@", "Deals @Damage@ magic damage.")).toBe("Deals magic damage.");
  });
});
