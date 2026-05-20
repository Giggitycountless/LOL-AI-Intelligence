# Known Unresolved Spell Tokens

Generated from `audit_token_coverage` (last run: 2026-05-19).  
Tokens that survive all four resolution layers. Most are naming mismatches between
LCU description templates and CDragon bin data, or involve unsupported slot formats.

Re-run after any CDragon data update, resolver change, or user feedback about
`[TokenName]` brackets appearing in ability tooltips.

## How to re-run

```sh
cargo test -p adapters -- --ignored audit_token_coverage --nocapture 2>&1 | tee audit.txt
```

## Token list (sorted by frequency)

| Rank | Count | Pattern | Token | Champions |
|------|-------|---------|-------|-----------|
| 1 | 1 | plain-name | `ADDamageCalc` | Katarina |
| 2 | 1 | plain-name | `AllDamageHit` | Mel |
| 3 | 1 | plain-name | `Calc_Base_Heal` | Shyvana |
| 4 | 1 | plain-name | `Calc_Max_Health_Dragon_Damage` | Shyvana |
| 5 | 1 | plain-name | `Calc_Missing_Health_Heal` | Shyvana |
| 6 | 1 | plain-name | `Cost` | Cassiopeia |
| 7 | 1 | plain-name | `EmpoweredDamageMultCalcModified` | Pantheon |
| 8 | 1 | plain-name | `EmpoweredLightningBonusMax` | Udyr |
| 9 | 1 | plain-name | `MaxDamageDisplay` | Kai'Sa |
| 10 | 1 | multiplied-name | `SlowPercent.0*100` | Heimerdinger |
| 11 | 1 | plain-name | `TotalADDamageCalc` | Katarina |
| 12 | 1 | fN-effect-index | `f11.1` | Kai'Sa |
| 13 | 1 | cross-spell | `spell.HweiQE:Duration` | Hwei |
| 14 | 1 | cross-spell | `spell.HweiQE:SlowPercent` | Hwei |
| 15 | 1 | cross-spell | `spell.HweiWW:ToolTipAllyMod*100` | Hwei |
| 16 | 1 | cross-spell | `spell.KhazixQ:Effect4Amount` | Kha'Zix |
| 17 | 1 | cross-spell | `spell.KhazixW:Effect3Amount` | Kha'Zix |
| 18 | 1 | cross-spell | `spell.NaafiriP:PackmateTauntDuration` | Naafiri |
| 19 | 1 | cross-spell | `spell.SmolderP:EBonusDamage` | Smolder |
| 20 | 1 | cross-spell | `spell.SmolderP:Passive_QDamageIncrease` | Smolder |
| 21 | 1 | cross-spell | `spell.SmolderP:Passive_WDamageIncrease` | Smolder |
| 22 | 1 | cross-spell | `spell.ZyraP:PlantDamage` | Zyra |
| 23 | 1 | cross-spell | `spell.ZyraP:PlantDuration` | Zyra |

**Total: 23 distinct token types across 13 champions.**

## Root causes by group

### `cross-spell` tokens with unsupported slot keys (Hwei)

`spell.HweiQE:Duration`, `spell.HweiQE:SlowPercent`, `spell.HweiWW:ToolTipAllyMod*100`  
Hwei uses compound spell keys (`QQ`, `QW`, `QE`, `WQ`, `WW`, `WE`, `EQ`, `EW`, `EE`).
The resolver only indexes `Q / W / E / R / Passive`. These six-character slot refs are not
recognised by `extract_slot_from_spell_ref`.

### `cross-spell` tokens referencing LCU effectAmount keys (Kha'Zix)

`spell.KhazixQ:Effect4Amount`, `spell.KhazixW:Effect3Amount`  
Cross-spell lookup searches the referenced slot's CDragon DataValues and calculations but
does not have access to that slot's LCU `effectAmounts` HashMap. `Effect4Amount` is an LCU
key, not a CDragon DataValue name, so it fails.

### `cross-spell` tokens targeting Passive sub-data (Naafiri, Smolder, Zyra)

`spell.NaafiriP:PackmateTauntDuration`, `spell.SmolderP:Passive_QDamageIncrease`, etc.  
The CDragon Passive spell for these champions has a different structure than expected (data
may be under a sub-spell path not indexed) or the DataValue/calculation name is a mismatch.

### `plain-name` naming mismatches (Katarina, Shyvana, Pantheon, Udyr, Kai'Sa, Cassiopeia, Mel)

Tokens like `ADDamageCalc`, `Calc_Base_Heal`, `Cost` — the LCU description template uses a
token name that does not correspond to any DataValue or calculation name in any slot's CDragon
bin data for that champion. Likely renamed between patches.

### `SlowPercent.0*100` (Heimerdinger)

The token body is `SlowPercent.0*100`. The `*` multiplier parser splits on `*` first,
giving `name="SlowPercent.0"` and `factor=100`. The `.0` suffix in the name does not
trigger the fN decimal-strip logic (that only fires after a leading `f`/`F`), so no
DataValue named `SlowPercent.0` is found.

### `f11.1` (Kai'Sa)

Parses as `Effect11Amount` after decimal strip. Kai'Sa has at most 10 standard effect
slots; `Effect11Amount` does not exist in her CDragon DataValues.
