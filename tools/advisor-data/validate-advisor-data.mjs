import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const VALID_LANES = new Map([
  ["top", "top"],
  ["jungle", "jungle"],
  ["middle", "middle"],
  ["mid", "middle"],
  ["bottom", "bottom"],
  ["bot", "bottom"],
  ["adc", "bottom"],
  ["support", "support"],
  ["sup", "support"],
]);
const REQUIRED_CANONICAL_LANES = ["top", "jungle", "middle", "bottom", "support"];
const VALID_SLOTS = new Set(["Q", "W", "E", "R"]);
const filePath = resolve(process.argv[2] ?? "data/advisor/latest.json");
const errors = [];

const json = await readFile(filePath, "utf8");
const data = parseJson(json);
if (data) {
  validateDocument(data);
}

if (errors.length > 0) {
  console.error(`Advisor data is invalid: ${filePath}`);
  for (const error of errors) {
    console.error(`- ${error}`);
  }
  process.exit(1);
}

const laneCounts = countLanes(data.champions);
console.log(
  `Advisor data OK: ${data.champions.length} champions, ` +
    REQUIRED_CANONICAL_LANES.map((lane) => `${lane}=${laneCounts.get(lane) ?? 0}`).join(", "),
);

function parseJson(value) {
  try {
    return JSON.parse(value);
  } catch (error) {
    errors.push(`JSON parse failed: ${error.message}`);
    return null;
  }
}

function validateDocument(value) {
  if (!isObject(value)) {
    errors.push("Root value must be an object");
    return;
  }
  if (value.formatVersion !== 1) {
    errors.push("formatVersion must be 1");
  }
  for (const key of ["source", "patch", "region", "queue", "tier", "generatedAt"]) {
    if (key in value && !isNonEmptyString(value[key])) {
      errors.push(`${key} must be a non-empty string when present`);
    }
  }
  if (!Array.isArray(value.champions) || value.champions.length === 0) {
    errors.push("champions must be a non-empty array");
    return;
  }

  const seen = new Set();
  const lanes = new Set();
  value.champions.forEach((entry, index) => validateChampion(entry, index, seen, lanes));
  for (const lane of REQUIRED_CANONICAL_LANES) {
    if (!lanes.has(lane)) {
      errors.push(`champions must include at least one ${lane} entry`);
    }
  }
}

function validateChampion(entry, index, seen, lanes) {
  const prefix = `champions[${index}]`;
  if (!isObject(entry)) {
    errors.push(`${prefix} must be an object`);
    return;
  }

  if (!Number.isInteger(entry.championId) || entry.championId <= 0) {
    errors.push(`${prefix}.championId must be a positive integer`);
  }
  if (!isNonEmptyString(entry.championName)) {
    errors.push(`${prefix}.championName must be a non-empty string`);
  }
  if ("championAlias" in entry && !isNonEmptyString(entry.championAlias)) {
    errors.push(`${prefix}.championAlias must be a non-empty string when present`);
  }

  const lane = isNonEmptyString(entry.lane) ? VALID_LANES.get(entry.lane.trim().toLowerCase()) : null;
  if (!lane) {
    errors.push(`${prefix}.lane is invalid`);
  } else {
    lanes.add(lane);
  }
  const duplicateKey = `${entry.championId}:${lane ?? entry.lane}`;
  if (seen.has(duplicateKey)) {
    errors.push(`${prefix} duplicates championId/lane ${duplicateKey}`);
  }
  seen.add(duplicateKey);

  if (!Number.isInteger(entry.games) || entry.games < 0) {
    errors.push(`${prefix}.games must be a non-negative integer`);
  }
  for (const key of ["winRate", "pickRate", "banRate"]) {
    if (!isRate(entry[key])) {
      errors.push(`${prefix}.${key} must be a finite number from 0 to 100`);
    }
  }
  if ("overallScore" in entry && !isRate(entry.overallScore)) {
    errors.push(`${prefix}.overallScore must be a finite number from 0 to 100 when present`);
  }

  validateRunes(entry.runes, `${prefix}.runes`);
  validateNamedList(entry.summonerSpells, `${prefix}.summonerSpells`);
  validateSkillOrder(entry.skillOrder, `${prefix}.skillOrder`);
  validateItemBuild(entry.itemBuild, `${prefix}.itemBuild`);
  validateMatchups(entry.strongAgainst, `${prefix}.strongAgainst`);
  validateMatchups(entry.weakAgainst, `${prefix}.weakAgainst`);
  validatePowerSpikes(entry.powerSpikes, `${prefix}.powerSpikes`);
  for (const key of ["laneAdvice", "teamfightAdvice"]) {
    if (!isNonEmptyString(entry[key])) {
      errors.push(`${prefix}.${key} must be a non-empty string`);
    }
  }
}

function validateRunes(value, prefix) {
  if (!isObject(value)) {
    errors.push(`${prefix} must be an object`);
    return;
  }
  for (const key of ["primaryStyle", "secondaryStyle"]) {
    if (!isNonEmptyString(value[key])) {
      errors.push(`${prefix}.${key} must be a non-empty string`);
    }
  }
  validateNamedList(value.primaryRunes, `${prefix}.primaryRunes`);
  validateNamedList(value.secondaryRunes, `${prefix}.secondaryRunes`);
  validateStringList(value.statShards, `${prefix}.statShards`);
}

function validateSkillOrder(value, prefix) {
  if (!isObject(value)) {
    errors.push(`${prefix} must be an object`);
    return;
  }
  for (const key of ["maxOrder", "earlyOrder"]) {
    if (!Array.isArray(value[key]) || value[key].length === 0 || value[key].some((slot) => !VALID_SLOTS.has(slot))) {
      errors.push(`${prefix}.${key} must contain ability slots`);
    }
  }
}

function validateItemBuild(value, prefix) {
  if (!isObject(value)) {
    errors.push(`${prefix} must be an object`);
    return;
  }
  for (const key of ["starter", "core", "boots", "late", "situational"]) {
    validateNamedList(value[key], `${prefix}.${key}`);
  }
}

function validateNamedList(values, prefix) {
  if (!Array.isArray(values) || values.length === 0) {
    errors.push(`${prefix} must be a non-empty array`);
    return;
  }
  values.forEach((value, index) => {
    if (!isObject(value)) {
      errors.push(`${prefix}[${index}] must be an object`);
      return;
    }
    if ("id" in value && value.id !== null && (!Number.isInteger(value.id) || value.id <= 0)) {
      errors.push(`${prefix}[${index}].id must be a positive integer when present`);
    }
    if (!isNonEmptyString(value.name)) {
      errors.push(`${prefix}[${index}].name must be a non-empty string`);
    }
  });
}

function validateMatchups(values, prefix) {
  if (!Array.isArray(values)) {
    errors.push(`${prefix} must be an array`);
    return;
  }
  values.forEach((value, index) => {
    if (!isObject(value)) {
      errors.push(`${prefix}[${index}] must be an object`);
      return;
    }
    if (!Number.isInteger(value.championId) || value.championId <= 0) {
      errors.push(`${prefix}[${index}].championId must be a positive integer`);
    }
    if (!isNonEmptyString(value.championName)) {
      errors.push(`${prefix}[${index}].championName must be a non-empty string`);
    }
    if (!isNonEmptyString(value.note)) {
      errors.push(`${prefix}[${index}].note must be a non-empty string`);
    }
    if ("winRateDelta" in value && value.winRateDelta !== null && typeof value.winRateDelta !== "number") {
      errors.push(`${prefix}[${index}].winRateDelta must be a number when present`);
    }
  });
}

function validatePowerSpikes(values, prefix) {
  if (!Array.isArray(values) || values.length === 0) {
    errors.push(`${prefix} must be a non-empty array`);
    return;
  }
  values.forEach((value, index) => {
    if (!isObject(value)) {
      errors.push(`${prefix}[${index}] must be an object`);
      return;
    }
    for (const key of ["timing", "label", "description"]) {
      if (!isNonEmptyString(value[key])) {
        errors.push(`${prefix}[${index}].${key} must be a non-empty string`);
      }
    }
  });
}

function validateStringList(values, prefix) {
  if (!Array.isArray(values) || values.length === 0 || values.some((value) => !isNonEmptyString(value))) {
    errors.push(`${prefix} must contain non-empty strings`);
  }
}

function countLanes(champions) {
  const counts = new Map();
  for (const champion of champions) {
    const lane = isNonEmptyString(champion.lane) ? VALID_LANES.get(champion.lane.trim().toLowerCase()) : null;
    if (lane) {
      counts.set(lane, (counts.get(lane) ?? 0) + 1);
    }
  }
  return counts;
}

function isObject(value) {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isNonEmptyString(value) {
  return typeof value === "string" && value.trim().length > 0;
}

function isRate(value) {
  return typeof value === "number" && Number.isFinite(value) && value >= 0 && value <= 100;
}
