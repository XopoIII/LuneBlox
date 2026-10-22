## What

Give a script run by LuneBlox the Roblox `Enum` global, as a Roblox client has it, so a spec can name `Enum.KeyCode.ButtonA` and compare keys the way the code under test does.

## Why

HudBlox's specs run on LuneBlox. While writing the gamepad chords for HudBlox 0.12.0 (`PadChord`), the rule "which key a held bumper means" could not take `Enum.KeyCode` values in its spec: `Enum` is nil there (`attempt to index nil with 'KeyCode'`). The rule was written over key names instead (`PadChord.match`), which works, but every pad rule to come (D-pad meanings, chords, glyphs) will meet the same wall, and the games' own specs (cn) run their client code under LuneBlox too.

## What would do

- `Enum` as a global, with at least `Enum.KeyCode`, `Enum.UserInputType`, `Enum.UserInputState` and `Enum.ContextActionResult`: items comparable by identity, with `.Name` and `.Value`, `Enum.KeyCode:GetEnumItems()`.
- Ideally the whole of Roblox's enum table, from the same API dump the `roblox` library already reads.

## Not urgent

Nothing is blocked: HudBlox 0.12.0 ships with the name-based rule. Raised by the owner so it is not forgotten.
