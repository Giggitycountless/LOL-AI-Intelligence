# ADR: Spell Token Resolver Architecture

**Status:** Superseded by [tencent-api-primary-spell-numerics.md](tencent-api-primary-spell-numerics.md)  
**Code:** *(removed — see git history)*  
**Last updated:** 2026-05-20

---

## Why this ADR is superseded

The Tencent LoL API provides pre-resolved spell numerics and Simplified Chinese descriptions with `【value】` bracket notation, eliminating the need for `@Token@` resolution on the primary path. With Tencent covering all champions globally, the four-layer resolver became dead code and was removed in full. CDragon is retained only as a fallback source for per-rank `DataValues` (ability stats display), not for description rendering.

---

## Background (historical)

League of Legends ability descriptions are stored in the LCU (League Client Update) as
HTML templates containing `@TokenName@` placeholders. CommunityDragon (CDragon) provides
a `bin.json` per champion that maps token names to numeric arrays (`DataValues`) and to
calculation formulas (`mSpellCalculations`). The resolver substitutes each placeholder
with a slash-separated per-rank value string (e.g. `"50/75/100/125/150"`), or leaves it
as `[TokenName]` (shown in red in dev builds) when resolution fails.

The entry point from outside the module is `resolve_spell_tokens_with_fallbacks` in
`champion_mapping.rs`.

---

## Four-layer resolution chain

`resolve_token_text` tries four paths in order, returning the result of the first success.

### Layer 1 — Current-spell DataValues (+ LCU effectAmounts)

```
data_value_numbers(name, spell.data_values, lcu_effect_amounts, rank_count)
```

- Searches the CDragon spell's `DataValues` / `mDataValues` arrays.
- Also searches **direct `mXxx` numeric array properties** on `mSpell` (e.g.
  `mAmmoRechargeTime`) — extracted as synthetic DataValues during `parse_bin_json`.
- Falls back to LCU `effectAmounts` (e.g. `Effect1Amount`) when not in CDragon.
- **`@fN@` / `@fN.M@` LCU effectBurn format:** `f2` resolves to `Effect2Amount`.
  The `.M` decimal suffix is a LCU formatting artefact and is stripped before parsing.

### Layer 2 — Current-spell calculations

```
find_calculation(name, spell.calculations) → calculation_text(…)
```

- Searches `mSpellCalculations` in the current spell (case-insensitive).
- `calculation_text` follows `mModifiedGameCalculation` forward references recursively.
- Formula parts are dispatched by `process_one_formula_part` (see §Part types).
- The `@Name*factor@` multiplier is applied to DataValue-backed flat values only;
  calculation results are returned as-is.

### Layer 3 — Explicit cross-spell reference (`spell.X:Y`)

```
try_cross_spell_token("spell.GnarE:MiniASDuration", multiplier, bin_data)
```

- Triggered when the token body starts with `spell.` (case-insensitive, so `Spell.` also matches).
- Parses `spell.<ChampionSlot>:<TokenName>`, extracts the slot suffix by checking
  suffixes `["Passive","R","Q","W","E"]` longest-first.
- Searches the referenced slot's DataValues then calculations, using that slot's own `rank_count`.

### Layer 4 — Implicit sibling-spell fallback

```
try_sibling_spell_token(name, multiplier, bin_data)
```

- Last resort for bare names (e.g. `@TotalDamage@`) that are defined in a different
  spell slot than the one whose description references them.
- Iterates `Q → W → E → R → Passive`. For each slot: tries DataValues, then calculations.
  Returns the first match, using that slot's own `rank_count`.
- Only reached after layers 1–3 all return `None`, minimising false-positive risk.

---

## Supported formula part types

`calculation_text` calls `process_one_formula_part` for each entry in `mFormulaParts`.

| CDragon `__type` | Key field(s) | Resolution |
|---|---|---|
| `NamedDataValueCalculationPart` | `mDataValue` | DataValue lookup → flat value |
| `NumberCalculationPart` | `mNumber` | Scalar constant → flat value |
| `LevelCalculationPart` | `mLevel1Value` | Level-1 constant → flat value |
| `ByCharLevelInterpolationCalculationPart` | `mStartValue`, `mEndValue` | `mStartValue` used as level-1 baseline |
| `StatByCoefficientCalculationPart` | `mStat`, `mCoefficient` | Coefficient → `+X%` in percent list |
| `StatByNamedDataValueCalculationPart` | `mStat`, `mDataValue` | DataValue as coefficient → `+X%` |
| `SumOfSubPartsCalculationPart` | `mSubparts: […]` | Recurse into each sub-part; preserves flat / `+X%` split |
| `ProductOfSubPartsCalculationPart` | `mPart1`, `mPart2` | Element-wise multiply; scalar side broadcasts across ranks |

Additional rules:
- `mDisplayAsPercent: true` on a calculation causes flat values to be multiplied ×100
  (raw `0.3` → displayed as `"30"`).
- `mModifiedGameCalculation` chains are followed recursively until a formula with
  `mFormulaParts` is found.

---

## Token body classification (`classify_token_body`)

Every unresolved token is classified and annotated in `[lcu-adapter]` log output:

| Category | Detection rule | Example |
|---|---|---|
| `fN-effect-index` | Leading `f`/`F` + digits (optional `.M` suffix stripped) | `@f1@`, `@f10.1@` |
| `cross-spell` | Starts with `spell.` (any case) and contains `:` | `@spell.GnarE:MiniASDuration@` |
| `lcu-effect-amount` | Matches `Effect\d+Amount` case-insensitively | `@Effect1Amount@` |
| `multiplied-name` | Contains `*` | `@BaseDamage*100@` |
| `plain-name` | None of the above | `@TotalDamage@` |

Multiplier suffix `@Name*factor@` is parsed before classification: `factor` is stripped
for name matching; it scales only DataValue-backed results, not calculation results.

`@SpellModifierDescriptionAppend@` is silently discarded before resolution reaches the
resolver (see `NON_DISPLAY_TOKENS` constant).

---

## Known unsupported corner cases

These account for the ~23 tokens in `docs/known-unresolved-tokens.md`.

| Case | Champion(s) | Root cause |
|---|---|---|
| **Hwei compound slot keys** — `spell.HweiQE:*` | Hwei | Hwei's spells use compound keys (`QQ`, `QW`, `QE`, …). `extract_slot_from_spell_ref` only recognises single-slot suffixes `Q/W/E/R/Passive`. |
| **Cross-spell LCU effectAmount reference** — `spell.KhazixQ:Effect4Amount` | Kha'Zix | `try_cross_spell_token` passes `&[]` as fallback_values; LCU effectAmounts for the target slot are unavailable, so `Effect4Amount` (an LCU key, not a CDragon DataValue name) is not found. |
| **LCU ↔ CDragon naming mismatch** — `ADDamageCalc`, `Calc_*`, `Cost`, etc. | Katarina, Shyvana, Cassiopeia, Pantheon, Udyr, Mel, Kai'Sa | LCU template uses a token name with no matching DataValue or calculation in any CDragon slot — likely renamed across patches. |
| **`SlowPercent.0*100`** — period in DataValue name | Heimerdinger | Multiplier split gives `name = "SlowPercent.0"`. The `.0` is not stripped (fN decimal-strip logic only applies after a leading `f`/`F`). No DataValue named `SlowPercent.0` exists. |
| **`f11.1`** — effect index beyond available slots | Kai'Sa | Correctly parses as `Effect11Amount`, but Kai'Sa has at most 10 standard effect slots. |
| **`mMultiplier` on calculations** | Various | Calculations like `RTotalDamage` carry an `mMultiplier` (crit scaling). The display shows the base formula only; the multiplier is not applied. |
| **Champion-level attributes** | Potentially any | Values at `Characters/{name}/CharacterRecords/Root` are outside the five spell paths indexed by `parse_bin_json`. The `mXxx` direct-field extraction only covers in-spell properties. |
| **CDragon unavailable** | All | Without `bin.json`, only positional LCU effectAmounts resolve. Named calculations and DataValues that come exclusively from CDragon are unavailable; `cdragonAvailable: false` is set on the ability. |

---

## Continuous monitoring

The offline coverage audit downloads CDragon data for all ~191 champions and runs the full
resolver pipeline (no LCU needed):

```sh
cargo test -p adapters -- --ignored audit_token_coverage --nocapture 2>&1 | tee audit.txt
```

Re-run after:
- Adding new resolver logic or formula part handlers
- A CDragon patch (new champions, reworked calculations, renamed DataValues)
- User feedback about `[TokenName]` brackets appearing in ability descriptions

The test is marked `#[ignore]` and is **not** run in normal CI (requires ~170 network
requests). The frontend shows unresolved tokens in red in dev builds
(`import.meta.env.DEV`) so they're visible during manual testing.
