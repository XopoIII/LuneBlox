use mlua::prelude::*;

use super::{Instance, dom_registry::DomId};
use rbx_dom_weak::types::Ref as DomRef;

/*
    Userdata interning

    To preserve instance identity from a lua perspective - so that two
    references to the same underlying instance are the *same* userdata, and
    therefore work correctly as table keys and with rawequal - we keep a single
    canonical userdata per `(dom_id, dom_ref)` pair in a per-lua cache.

    The cache table has weak values, so userdata that is no longer referenced
    anywhere in lua can still be collected and will simply be recreated on next
    access (which is fine, since identity only needs to hold while a reference
    is alive).
*/
const INSTANCE_CACHE_KEY: &str = "__lune_roblox_instance_cache";

fn instance_cache(lua: &Lua) -> LuaResult<LuaTable> {
    if let Ok(cache) = lua.named_registry_value::<LuaTable>(INSTANCE_CACHE_KEY) {
        return Ok(cache);
    }
    let cache = lua.create_table()?;
    let meta = lua.create_table()?;
    meta.set("__mode", "v")?;
    cache.set_metatable(Some(meta))?;
    lua.set_named_registry_value(INSTANCE_CACHE_KEY, &cache)?;
    Ok(cache)
}

fn instance_cache_key(inst: Instance) -> String {
    format!("{}:{}", inst.dom_id, inst.dom_ref)
}

/**
    Converts an instance into its canonical lua userdata, creating and caching
    it the first time and returning the same userdata on subsequent calls.

    # Errors

    Errors if creating the userdata or accessing the interning cache fails.
*/
pub fn instance_to_lua(lua: &Lua, inst: Instance) -> LuaResult<LuaValue> {
    let cache = instance_cache(lua)?;
    let key = instance_cache_key(inst);
    if let Ok(LuaValue::UserData(existing)) = cache.get::<LuaValue>(key.as_str()) {
        return Ok(LuaValue::UserData(existing));
    }
    let userdata = lua.create_userdata(inst)?;
    cache.set(key, &userdata)?;
    Ok(LuaValue::UserData(userdata))
}

/**
    Converts an optional instance into lua, returning [`LuaValue::Nil`] for [`None`].

    # Errors

    Errors if creating the userdata or accessing the interning cache fails.
*/
pub fn opt_instance_to_lua(lua: &Lua, inst: Option<Instance>) -> LuaResult<LuaValue> {
    match inst {
        Some(inst) => instance_to_lua(lua, inst),
        None => Ok(LuaValue::Nil),
    }
}

/**
    Converts a list of instances into a lua array table of canonical userdata.

    # Errors

    Errors if creating the table or any of the userdata fails.
*/
pub fn instances_to_lua(lua: &Lua, instances: Vec<Instance>) -> LuaResult<LuaValue> {
    let tab = lua.create_table_with_capacity(instances.len(), 0)?;
    for inst in instances {
        tab.push(instance_to_lua(lua, inst)?)?;
    }
    Ok(LuaValue::Table(tab))
}

/**
    Updates the interning cache after a cross-dom transfer.

    The moved referents keep their values but now live in `new_dom_id`, so we
    re-key any cached userdata and update the `dom_id` stored inside it.
*/
pub(crate) fn rekey_cache_after_transfer(
    lua: &Lua,
    old_dom_id: DomId,
    new_dom_id: DomId,
    moved: &[DomRef],
) -> LuaResult<()> {
    if old_dom_id == new_dom_id {
        return Ok(());
    }
    let cache = instance_cache(lua)?;
    for &moved_ref in moved {
        let old_key = format!("{old_dom_id}:{moved_ref}");
        if let Ok(LuaValue::UserData(ud)) = cache.get::<LuaValue>(old_key.as_str()) {
            if let Ok(mut inst) = ud.borrow_mut::<Instance>() {
                inst.dom_id = new_dom_id;
            }
            let new_key = format!("{new_dom_id}:{moved_ref}");
            cache.set(new_key, &ud)?;
            cache.set(old_key, LuaValue::Nil)?;
        }
    }
    Ok(())
}
