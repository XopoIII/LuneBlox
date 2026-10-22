use mlua::prelude::*;

/*
    Roblox's `Enum`, the same userdata as `require("@lune/roblox").Enum`,
    so an item read through either is the same item.
*/
pub fn create(lua: Lua) -> LuaResult<LuaValue> {
    lune_std_roblox::enum_global(&lua)
}
