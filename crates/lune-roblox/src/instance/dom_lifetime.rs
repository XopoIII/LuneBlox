use std::collections::HashSet;

use mlua::prelude::*;

use super::{
    Instance,
    dom_registry::{self, DomId},
    lua_cache::live_dom_ids,
};

/*
    Freeing parsed doms

    A parsed place or model is registered as a dom of its own, and an instance
    is only a `(dom_id, dom_ref)` pair: nothing tells the registry when the last
    instance of a dom is gone from lua, so a dom stayed in memory until its
    root was destroyed. Each lua state therefore records the doms it parsed,
    and before parsing another one frees those that no instance userdata in its
    interning cache points into any more. The cache has weak values, so an
    instance lua dropped leaves it once the garbage collector has run.

    Only doms parsed into this lua state are freed here, so a runtime never
    frees another's doms, nor one a Rust caller built and holds on its own.
    When the lua state closes, every dom it parsed goes with it.
*/
#[derive(Debug, Default)]
struct ParsedDoms(HashSet<DomId>);

impl Drop for ParsedDoms {
    fn drop(&mut self) {
        for &dom_id in &self.0 {
            dom_registry::remove(dom_id);
        }
    }
}

/**
    Records that the dom the given instance lives in was parsed into the given
    lua state, so that it is freed once lua holds none of its instances, or
    when the lua state closes.

    Call this after the dom's instances have been converted to lua.
*/
pub fn track_parsed_dom(lua: &Lua, inst: &Instance) {
    let dom_id = inst.dom_id;
    if let Some(mut parsed) = lua.app_data_mut::<ParsedDoms>() {
        parsed.0.insert(dom_id);
        return;
    }
    lua.set_app_data(ParsedDoms(HashSet::from([dom_id])));
}

/**
    Frees every dom parsed into the given lua state that no instance in lua
    points into any more.

    # Errors

    Errors if the interning cache can not be read.
*/
pub fn free_unreferenced_doms(lua: &Lua) -> LuaResult<()> {
    let Some(mut parsed) = lua.app_data_mut::<ParsedDoms>() else {
        return Ok(());
    };
    let live = live_dom_ids(lua)?;
    parsed.0.retain(|&dom_id| {
        let keep = live.contains(&dom_id) && dom_registry::contains(dom_id);
        if !keep {
            dom_registry::remove(dom_id);
        }
        keep
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use rbx_dom_weak::{InstanceBuilder, WeakDom};

    use super::*;
    use crate::instance::{instance_to_lua, instances_to_lua};

    fn parsed_model(lua: &Lua) -> (DomId, LuaValue) {
        let mut dom = WeakDom::new(InstanceBuilder::new("DataModel"));
        let root = dom.root_ref();
        let folder = dom.insert(root, InstanceBuilder::new("Folder"));
        dom.insert(folder, InstanceBuilder::new("Part"));
        let dom_id = dom_registry::register(dom);
        let inst = Instance::new(dom_id, folder);
        let value = instances_to_lua(lua, vec![inst]).unwrap();
        track_parsed_dom(lua, &inst);
        (dom_id, value)
    }

    #[test]
    fn frees_a_dom_lua_dropped() {
        let lua = Lua::new();
        let (dom_id, value) = parsed_model(&lua);
        drop(value);
        lua.gc_collect().unwrap();
        lua.gc_collect().unwrap();
        free_unreferenced_doms(&lua).unwrap();
        assert!(!dom_registry::contains(dom_id));
    }

    #[test]
    fn keeps_a_dom_lua_holds_any_instance_of() {
        let lua = Lua::new();
        let (dom_id, value) = parsed_model(&lua);
        // Keep only a descendant, not the instance that was returned
        let LuaValue::Table(list) = value else {
            panic!("expected a table");
        };
        let folder = list.get::<LuaAnyUserData>(1).unwrap();
        let child = folder.borrow::<Instance>().unwrap().get_children()[0];
        let child = instance_to_lua(&lua, child).unwrap();
        drop((list, folder));
        lua.gc_collect().unwrap();
        lua.gc_collect().unwrap();
        free_unreferenced_doms(&lua).unwrap();
        assert!(dom_registry::contains(dom_id));
        drop(child);
    }

    #[test]
    fn frees_every_dom_when_lua_closes() {
        let lua = Lua::new();
        let (dom_id, value) = parsed_model(&lua);
        drop(value);
        drop(lua);
        assert!(!dom_registry::contains(dom_id));
    }

    #[test]
    fn leaves_doms_other_lua_states_parsed() {
        let lua = Lua::new();
        let other = Lua::new();
        let (dom_id, value) = parsed_model(&other);
        free_unreferenced_doms(&lua).unwrap();
        assert!(dom_registry::contains(dom_id));
        drop((value, other));
    }
}
