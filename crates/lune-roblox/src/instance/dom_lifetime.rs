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

    Call this after the dom's instances have been converted to lua using the
    canonical instance conversion helpers. Copies of Rust `Instance` handles
    are non-owning: they must not outlive the originating Lua state's handles
    or be shared with another state while this cleanup policy is enabled.
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
    use crate::{
        document::{Document, DocumentFormat, DocumentKind},
        instance::{instance_to_lua, instances_to_lua},
    };

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

    fn collect_and_sweep(lua: &Lua) {
        lua.gc_collect().unwrap();
        lua.gc_collect().unwrap();
        free_unreferenced_doms(lua).unwrap();
    }

    #[test]
    fn empty_models_do_not_leave_an_untrackable_dom() {
        let dom = WeakDom::new(InstanceBuilder::new("ROOT"));
        let root_ref = dom.root_ref();
        let doc = Document::from((DocumentKind::Model, DocumentFormat::Xml, dom));
        assert_eq!(doc.into_instance_array().unwrap(), [] as [Instance; 0]);
        // There is no Instance to pass to track_parsed_dom for an empty model.
        assert!(!dom_registry::contains_root(root_ref));
    }

    #[test]
    fn collection_alone_defers_reclamation_until_the_next_sweep() {
        let lua = Lua::new();
        let (dom_id, value) = parsed_model(&lua);
        drop(value);
        lua.gc_collect().unwrap();
        lua.gc_collect().unwrap();
        assert!(dom_registry::contains(dom_id));
        free_unreferenced_doms(&lua).unwrap();
        assert!(!dom_registry::contains(dom_id));
    }

    #[test]
    fn repeated_place_and_model_parses_reclaim_previous_documents() {
        for kind in [DocumentKind::Place, DocumentKind::Model] {
            let lua = Lua::new();
            let mut dom = WeakDom::new(InstanceBuilder::new("ROOT"));
            let root = dom.root_ref();
            let folder = dom.insert(root, InstanceBuilder::new("Folder"));
            dom.insert(folder, InstanceBuilder::new("Part"));
            let doc = Document::from((kind, DocumentFormat::Xml, dom));
            let bytes = doc.to_bytes().unwrap();
            let mut parsed_ids = Vec::new();
            for _ in 0..16 {
                collect_and_sweep(&lua);
                assert!(parsed_ids.iter().all(|&id| !dom_registry::contains(id)));
                let doc = Document::from_bytes(&bytes, kind).unwrap();
                let instances = if kind == DocumentKind::Place {
                    vec![doc.into_data_model_instance().unwrap()]
                } else {
                    doc.into_instance_array().unwrap()
                };
                let first = instances[0];
                let value = instances_to_lua(&lua, instances).unwrap();
                track_parsed_dom(&lua, &first);
                parsed_ids.push(first.dom_id);
                assert!(dom_registry::contains(first.dom_id));
                drop(value);
            }
            collect_and_sweep(&lua);
            assert!(parsed_ids.iter().all(|&id| !dom_registry::contains(id)));
        }
    }

    #[test]
    fn keeps_detached_children_and_clones_after_returned_roots_are_collected() {
        for operation in [
            "kept = model[1]:GetChildren()[1]; kept.Parent = nil",
            "kept = model[1]:Clone()",
        ] {
            let lua = Lua::new();
            let (dom_id, value) = parsed_model(&lua);
            lua.globals().set("model", value).unwrap();
            lua.load(operation).exec().unwrap();
            lua.globals().set("model", LuaValue::Nil).unwrap();
            collect_and_sweep(&lua);
            assert!(dom_registry::contains(dom_id));
            lua.load("assert(kept.Parent == nil); assert(kept.Name ~= '')")
                .exec()
                .unwrap();
            lua.globals().set("kept", LuaValue::Nil).unwrap();
            collect_and_sweep(&lua);
            assert!(!dom_registry::contains(dom_id));
        }
    }

    #[test]
    fn keeps_model_siblings_when_only_the_second_root_is_held() {
        let lua = Lua::new();
        let mut dom = WeakDom::new(InstanceBuilder::new("ROOT"));
        let root = dom.root_ref();
        dom.insert(root, InstanceBuilder::new("Folder"));
        dom.insert(root, InstanceBuilder::new("Part"));
        let doc = Document::from((DocumentKind::Model, DocumentFormat::Xml, dom));
        let instances = doc.into_instance_array().unwrap();
        let first = instances[0];
        let value = instances_to_lua(&lua, instances).unwrap();
        track_parsed_dom(&lua, &first);
        lua.globals().set("model", value).unwrap();
        lua.load("kept = model[2]; model = nil").exec().unwrap();
        collect_and_sweep(&lua);
        assert!(dom_registry::contains(first.dom_id));
        lua.load("assert(kept.Name == 'Part')").exec().unwrap();
        lua.globals().set("kept", LuaValue::Nil).unwrap();
        collect_and_sweep(&lua);
        assert!(!dom_registry::contains(first.dom_id));
    }

    #[test]
    fn reparented_subtrees_keep_the_destination_not_the_source_alive() {
        let lua = Lua::new();
        let (source_id, source) = parsed_model(&lua);
        let (destination_id, destination) = parsed_model(&lua);
        lua.globals().set("source", source).unwrap();
        lua.globals().set("destination", destination).unwrap();
        lua.load(
            "kept = source[1]:GetChildren()[1];
             source[1].Parent = destination[1];
             source = nil; destination = nil",
        )
        .exec()
        .unwrap();
        collect_and_sweep(&lua);
        assert!(!dom_registry::contains(source_id));
        assert!(dom_registry::contains(destination_id));
        lua.load(
            "assert(kept.Name == 'Part');
             assert(kept.Parent.Parent.Name == 'Folder');
             assert(rawequal(kept, kept.Parent:GetChildren()[1]))",
        )
        .exec()
        .unwrap();
        lua.globals().set("kept", LuaValue::Nil).unwrap();
        collect_and_sweep(&lua);
        assert!(!dom_registry::contains(destination_id));
    }

    #[test]
    fn manually_created_child_keeps_its_parsed_destination_alive() {
        let lua = Lua::new();
        let (dom_id, value) = parsed_model(&lua);
        lua.globals().set("model", value).unwrap();
        let child = Instance::new_orphaned("Folder");
        lua.globals()
            .set("child", instance_to_lua(&lua, child).unwrap())
            .unwrap();
        lua.load("child.Parent = model[1]; model = nil")
            .exec()
            .unwrap();
        collect_and_sweep(&lua);
        assert!(dom_registry::contains(dom_id));
        lua.load("assert(child.Parent.Name == 'Folder')")
            .exec()
            .unwrap();
        lua.globals().set("child", LuaValue::Nil).unwrap();
        collect_and_sweep(&lua);
        assert!(!dom_registry::contains(dom_id));
    }

    #[test]
    fn destroying_returned_root_keeps_a_detached_child_alive() {
        let lua = Lua::new();
        let (dom_id, value) = parsed_model(&lua);
        lua.globals().set("model", value).unwrap();
        lua.load(
            "kept = model[1]:GetChildren()[1];
             kept.Parent = nil; model[1]:Destroy(); model = nil",
        )
        .exec()
        .unwrap();
        collect_and_sweep(&lua);
        assert!(dom_registry::contains(dom_id));
        lua.load("assert(kept.Name == 'Part'); kept:Destroy()")
            .exec()
            .unwrap();
        assert!(!dom_registry::contains(dom_id));
        collect_and_sweep(&lua);
        assert!(!dom_registry::contains(dom_id));
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
