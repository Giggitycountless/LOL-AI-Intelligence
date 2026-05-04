import { describe, expect, it } from "vitest";
import { createTranslator, resolveEffectiveLanguage } from "../i18n";

describe("createTranslator", () => {
  it("returns English translations for 'en'", () => {
    const t = createTranslator("en");
    expect(t("app.name")).toBe("LoL Desktop Assistant");
    expect(t("nav.dashboard")).toBe("Dashboard");
    expect(t("common.save")).toBe("Save");
  });

  it("returns Chinese translations for 'zh'", () => {
    const t = createTranslator("zh");
    expect(t("app.name")).toBe("LoL 桌面助手");
    expect(t("nav.dashboard")).toBe("总览");
    expect(t("common.save")).toBe("保存");
  });

  it("English and Chinese return different values for the same key", () => {
    const tEn = createTranslator("en");
    const tZh = createTranslator("zh");
    expect(tEn("app.name")).not.toBe(tZh("app.name"));
  });
});

describe("resolveEffectiveLanguage", () => {
  it("returns 'zh' when preference is 'zh'", () => {
    expect(resolveEffectiveLanguage("zh")).toBe("zh");
  });

  it("returns 'en' when preference is 'en'", () => {
    expect(resolveEffectiveLanguage("en")).toBe("en");
  });

  it("returns 'zh' for system preference when system language starts with 'zh'", () => {
    expect(resolveEffectiveLanguage("system", "zh-CN")).toBe("zh");
  });

  it("returns 'en' for system preference when system language is English", () => {
    expect(resolveEffectiveLanguage("system", "en-US")).toBe("en");
  });

  it("returns 'en' for system preference when system language is non-Chinese", () => {
    expect(resolveEffectiveLanguage("system", "fr-FR")).toBe("en");
  });

  it("handles case-insensitive system language matching", () => {
    expect(resolveEffectiveLanguage("system", "ZH-TW")).toBe("zh");
  });
});
