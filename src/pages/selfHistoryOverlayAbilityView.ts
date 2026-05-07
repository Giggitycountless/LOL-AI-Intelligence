export function abilityStatText(values: string[], fallback: string | null | undefined) {
  const cleanedValues = values.map((value) => cleanAbilityText(value)).filter(Boolean);
  if (cleanedValues.length > 0) {
    return cleanedValues.join("/");
  }

  const cleanedFallback = cleanAbilityText(fallback ?? "");
  return cleanedFallback || "-";
}

export function abilityTooltipText(summaryDescription: string | null | undefined, description: string | null | undefined) {
  return cleanAbilityText(summaryDescription ?? "") || cleanAbilityText(description ?? "") || "-";
}

function cleanAbilityText(value: string) {
  return value
    .replace(/@[^@]*@/g, "")
    .replace(/<[^>]*>/g, "")
    .replace(/\s+/g, " ")
    .trim();
}
