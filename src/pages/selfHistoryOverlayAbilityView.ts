export function abilityStatText(values: string[], fallback: string | null | undefined) {
  const cleanedValues = values.map((value) => cleanAbilityText(value)).filter(Boolean);
  if (cleanedValues.length > 0) {
    return cleanedValues.join("/");
  }

  const cleanedFallback = cleanAbilityText(fallback ?? "");
  return cleanedFallback || "-";
}

export function abilityStatDisplay(values: number[], suffix: string) {
  const cleaned = values
    .map((v) => {
      // Show 2 decimal places for fractions, integer for whole numbers
      if (v % 1 === 0) return v.toString();
      return v.toFixed(2).replace(/\.?0+$/, "");
    })
    .join("/");
  return suffix ? `${cleaned}${suffix}` : cleaned;
}

export function abilityTooltipText(summaryDescription: string | null | undefined, description: string | null | undefined) {
  return cleanAbilityText(description ?? "") || cleanAbilityText(summaryDescription ?? "") || "-";
}

function cleanAbilityText(value: string) {
  return value
    .replace(/@[^@]*@/g, "")
    .replace(/%i:[^%]*%/g, "")
    .replace(/<[^>]*>/g, "")
    .replace(/\s+/g, " ")
    .replace(/^\s*s\s*$/i, "")
    .trim();
}
