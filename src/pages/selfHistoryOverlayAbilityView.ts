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
    .replace(/<br\s*\/?>/gi, "\n")
    .replace(/@[^@]*@/g, "")
    .replace(/%i:[^%]*%/g, "")
    .replace(/<[^>]*>/g, "")
    .replace(/\n{3,}/g, "\n\n")
    .replace(/ +/g, " ")
    .replace(/^\s*s\s*$/i, "")
    .trim();
}

// ── Structured description parser ─────────────────────────────────────────

/** Semantic kind for an inline text span within an ability description. */
export type SpanKind =
  | "text"
  | "magic"
  | "physical"
  | "true"
  | "healing"
  | "status"
  | "bold"
  | "unresolved";

export type DescriptionSpan = { kind: SpanKind; text: string };

export type DescriptionSection = {
  /** null = main section (no label), "Passive" / "Active" = labelled section. */
  label: "Passive" | "Active" | null;
  /** Each inner array is one visual line (split on single <br>). */
  lines: DescriptionSpan[][];
};

// Maps LoL HTML tag names → SpanKind
const TAG_KIND: Record<string, SpanKind> = {
  magicdamage: "magic",
  physicaldamage: "physical",
  truedamage: "true",
  healing: "healing",
  speed: "healing",
  shield: "healing",
  status: "status",
  keywordmajor: "bold",
  keyword: "bold",
  attention: "bold",
  scaleap: "magic",
  scalead: "physical",
  scalehealth: "healing",
  unresolvedtoken: "unresolved",
};

/**
 * Parse a LoL ability description (HTML with resolved @tokens@) into typed
 * sections, each containing coloured inline spans and an optional label.
 *
 * Double <br> → paragraph boundary; single <br> → line break within a section.
 */
export function parseAbilityDescription(html: string): DescriptionSection[] {
  // Strip unresolved template tokens:
  //   @token@   — original LCU format, stripped before reaching resolver
  //   [token]   — diagnostic bracket form emitted by the Rust resolver for
  //               tokens it could not resolve (e.g. @DamageCalc@ without CDragon)
  //
  // In dev builds, [token] brackets are preserved as <unresolvedtoken> elements
  // so they render in red, making token resolution gaps visible during development.
  const cleaned = import.meta.env.DEV
    ? html
        .replace(/@[^@]*@/g, "")
        .replace(/\[(\w[^\]]*)\]/g, (_, name: string) => `<unresolvedtoken>${name}</unresolvedtoken>`)
        .replace(/%i:[^%]*%/g, "")
    : html
        .replace(/@[^@]*@/g, "")
        .replace(/\[\w[^\]]*\]/g, "")
        .replace(/%i:[^%]*%/g, "");

  // Split into paragraphs on double <br>
  const paragraphs = cleaned
    .split(/<br\s*\/?>\s*<br\s*\/?>/gi)
    .map((p) => p.trim())
    .filter(Boolean);

  if (paragraphs.length === 0) return [];

  return paragraphs.flatMap((para): DescriptionSection[] => {
    let label: DescriptionSection["label"] = null;
    let rest = para;

    // Detect section label from an opening <passive> or <active> tag
    // e.g. "<passive>Passive: </passive>text..." or "<active>Active</active> - text"
    const labelMatch = rest.match(/^<(passive|active)>[^<]*<\/(?:passive|active)>\s*-?\s*/i);
    if (labelMatch) {
      label = labelMatch[1].toLowerCase() === "passive" ? "Passive" : "Active";
      rest = rest.slice(labelMatch[0].length).trim();
    }

    // A paragraph that contains a nested <passive>/<active> mid-text should be
    // split so each labelled section becomes its own card row.
    const innerSplitRe = /(<(?:passive|active)>[^<]*<\/(?:passive|active)>)/gi;
    const parts = rest.split(innerSplitRe).filter(Boolean);

    // If the paragraph contains embedded labels we recursively produce multiple sections
    if (parts.length > 1 && label === null) {
      const sections: DescriptionSection[] = [];
      let pendingLabel: DescriptionSection["label"] = null;
      let pendingText = "";

      for (const part of parts) {
        const m = part.match(/^<(passive|active)>[^<]*<\/(?:passive|active)>$/i);
        if (m) {
          if (pendingText.trim()) {
            sections.push(buildSection(null, pendingText));
            pendingText = "";
          }
          pendingLabel = m[1].toLowerCase() === "passive" ? "Passive" : "Active";
        } else {
          pendingText += part;
          // Flush when we have a label waiting
          if (pendingLabel !== null && pendingText.trim()) {
            sections.push(buildSection(pendingLabel, pendingText));
            pendingLabel = null;
            pendingText = "";
          }
        }
      }
      if (pendingText.trim()) sections.push(buildSection(pendingLabel, pendingText));
      return sections;
    }

    return [buildSection(label, rest)];
  });
}

function buildSection(label: DescriptionSection["label"], html: string): DescriptionSection {
  const rawLines = html.split(/<br\s*\/?>/gi);
  const lines = rawLines
    .map((line) => parseInlineSpans(line))
    .filter((spans) => spans.some((s) => s.text.trim().length > 0));
  return { label, lines };
}

function parseInlineSpans(html: string): DescriptionSpan[] {
  const spans: DescriptionSpan[] = [];
  // Match <tag>content</tag> pairs, or plain text runs between tags
  const re = /<(\w+)>([\s\S]*?)<\/\1>|([^<]+)/g;
  let m: RegExpExecArray | null;

  while ((m = re.exec(html)) !== null) {
    if (m[1] !== undefined && m[2] !== undefined) {
      const tag = m[1].toLowerCase();
      // Recursively strip any nested tags inside (e.g. bold inside magic)
      const text = m[2].replace(/<[^>]*>/g, "").replace(/\s+/g, " ").trim();
      if (!text) continue;
      const kind: SpanKind = TAG_KIND[tag] ?? "text";
      spans.push({ kind, text });
    } else if (m[3] !== undefined) {
      const text = m[3].replace(/\s+/g, " ");
      if (text.trim().length > 0) spans.push({ kind: "text", text });
    }
  }

  return spans;
}
