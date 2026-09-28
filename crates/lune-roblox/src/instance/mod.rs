#![allow(clippy::missing_panics_doc)]

use std::{
    fmt,
    hash::{Hash, Hasher},
};

#[cfg(feature = "mlua")]
use mlua::prelude::*;

use rbx_dom_weak::{
    Instance as DomInstance, InstanceBuilder as DomInstanceBuilder, Ustr, WeakDom,
    types::Ref as DomRef, ustr,
};

#[cfg(feature = "mlua")]
use lune_utils::TableBuilder;

use crate::shared::instance::class_is_a;

pub use self::query::{QueryError, QueryResult, query_descendants};
pub use crate::shared::instance::{CustomClassError, register_custom_class};

#[cfg(feature = "mlua")]
use crate::{exports::LuaExportsTable, shared::instance::class_exists};

#[cfg(feature = "mlua")]
pub(crate) mod base;
#[cfg(feature = "mlua")]
pub(crate) mod data_model;
#[cfg(feature = "mlua")]
pub(crate) mod terrain;
#[cfg(feature = "mlua")]
pub(crate) mod workspace;

pub(crate) mod dom_registry;
pub(crate) mod query;

#[cfg(feature = "mlua")]
pub mod registry;

mod hierarchy;
mod properties;

#[cfg(feature = "mlua")]
mod lua_cache;
#[cfg(feature = "mlua")]
pub(crate) use lua_cache::rekey_cache_after_transfer;
#[cfg(feature = "mlua")]
pub use lua_cache::{instance_to_lua, instances_to_lua, opt_instance_to_lua};

use dom_registry::DomId;

const PROPERTY_NAME_ATTRIBUTES: &str = "Attributes";
const PROPERTY_NAME_TAGS: &str = "Tags";

#[derive(Debug, Clone, Copy)]
pub struct Instance {
    #[doc(hidden)]
    pub dom_id: DomId,
    #[doc(hidden)]
    pub dom_ref: DomRef,
    // NOTE: This is not public since we want the accessor to be &str and never Ustr
    pub(crate) class_name: Ustr,
}

impl Instance {
    /**
        Builds an `Instance` from a dom object that has already been
        borrowed out of its dom, without acquiring any new locks.

        This is the preferred constructor to use from inside a
        [`dom::with`] / [`dom::with_mut`] closure.
    */
    pub(crate) fn from_dom_instance(dom_id: DomId, dom_ref: DomRef, inst: &DomInstance) -> Self {
        Self {
            dom_id,
            dom_ref,
            class_name: inst.class,
        }
    }

    /**
        Creates a new `Instance` from a dom id and object ref.

        Panics if the instance does not exist in the given dom,
        or if the given dom object ref points to the dom root.
    */
    #[must_use]
    pub fn new(dom_id: DomId, dom_ref: DomRef) -> Self {
        Self::new_opt(dom_id, dom_ref).expect("Failed to find instance in document")
    }

    /**
        Creates a new `Instance` from a dom id and object ref, if the instance exists.

        Panics if the given dom object ref points to the dom root.
    */
    #[must_use]
    pub fn new_opt(dom_id: DomId, dom_ref: DomRef) -> Option<Self> {
        dom_registry::with(dom_id, |dom| {
            dom.get_by_ref(dom_ref).map(|instance| {
                assert!(
                    !(instance.referent() == dom.root_ref()),
                    "Instances can not be created from dom roots"
                );
                Self::from_dom_instance(dom_id, dom_ref, instance)
            })
        })
        .flatten()
    }

    /**
        Creates a new orphaned `Instance` with a given class name.

        An orphaned instance is an instance at the root of the shared default
        scratch dom used for manually created instances.
    */
    #[must_use]
    pub fn new_orphaned(class_name: impl AsRef<str>) -> Self {
        Self::new_in_dom(dom_registry::default_dom(), class_name)
    }

    /**
        Creates a new orphaned `Instance` with a given class name, directly
        inside the dom with the given id.

        Creating children inside their eventual parent's dom this way avoids
        a needless cross-dom transfer when they are later parented.
    */
    #[must_use]
    pub fn new_in_dom(dom_id: DomId, class_name: impl AsRef<str>) -> Self {
        let class_name = class_name.as_ref();
        let dom_ref = dom_registry::with_mut(dom_id, |dom| {
            let dom_root = dom.root_ref();
            dom.insert(dom_root, DomInstanceBuilder::new(class_name))
        })
        .expect("Failed to find dom to create instance in");
        Self {
            dom_id,
            dom_ref,
            class_name: ustr(class_name),
        }
    }

    /**
        Runs a closure with shared read access to the [`WeakDom`] that this
        instance lives in.

        This is an escape hatch for handing a `&WeakDom` to an external consumer
        that reads it directly through the `rbx_dom_weak` api without copying it
        out. The referents seen inside the closure are the exact same `Ref`s the
        instances were created with, so any `Ref` obtained from the dom maps
        straight back to its [`Instance`].

        # Deadlocks

        **The global dom registry lock is held for the entire duration of `f`,
        and it is not reentrant.** Inside `f` you must work with the `&WeakDom`
        directly and **must not** call back into any method on *this or any
        other* [`Instance`] that touches the dom - `get_name`, `get_parent`,
        `get_children`, `get_property`, `get_attribute`, the `set_*` methods,
        `with_dom`, `with_dom_mut`, and so on. Any such call re-locks the same
        registry lock and **deadlocks the thread**.

        In other words: treat `f` as a pure read of the raw dom, nothing else. Do
        all [`Instance`]-level work before or after the closure, never within it.

        This is `#[doc(hidden)]` precisely because of these sharp edges - it is a
        low-level seam for embedders, not part of the general instance api.
    */
    #[doc(hidden)]
    pub fn with_dom<R>(&self, f: impl FnOnce(&WeakDom) -> R) -> R {
        dom_registry::with(self.dom_id, f).expect("Failed to find dom for instance")
    }

    /**
        Runs a closure with exclusive write access to the [`WeakDom`] that this
        instance lives in.

        The mutable counterpart to [`Instance::with_dom`] - for handing a
        `&mut WeakDom` to an external consumer that edits it directly through the
        `rbx_dom_weak` api. The same referent-stability guarantee applies.

        # Deadlocks

        Carries the **exact same caveat** as [`Instance::with_dom`]: the global,
        non-reentrant dom registry lock is held for the duration of `f`, so the
        closure must operate on the `&mut WeakDom` directly and **must not** call
        back into any dom-accessing [`Instance`] method (doing so deadlocks the
        thread). See [`Instance::with_dom`] for the full explanation.

        This is `#[doc(hidden)]` for the same reasons.
    */
    #[doc(hidden)]
    pub fn with_dom_mut<R>(&self, f: impl FnOnce(&mut WeakDom) -> R) -> R {
        dom_registry::with_mut(self.dom_id, f).expect("Failed to find dom for instance")
    }

    /**
        Clones an instance to an external weak dom.

        This will place the instance as a child of the
        root of the weak dom, and return its referent.
    */
    pub fn clone_into_external_dom(self, external_dom: &mut WeakDom) -> DomRef {
        self.with_dom(|dom| {
            let cloned = dom.clone_into_external(self.dom_ref, external_dom);
            external_dom.transfer_within(cloned, external_dom.root_ref());
            cloned
        })
    }

    /**
        Clones multiple instances from a single dom to an external weak dom.

        This will place the instances as children of the
        root of the weak dom, and return their referents.
    */
    pub fn clone_multiple_into_external_dom(
        dom_id: DomId,
        referents: &[DomRef],
        external_dom: &mut WeakDom,
    ) -> Vec<DomRef> {
        dom_registry::with(dom_id, |dom| {
            let cloned = dom.clone_multiple_into_external(referents, external_dom);
            for referent in &cloned {
                external_dom.transfer_within(*referent, external_dom.root_ref());
            }
            cloned
        })
        .unwrap_or_default()
    }

    /**
        Clones the instance and all of its descendants, and orphans it.

        The clone is placed at the root of the same dom as the original.

        ### See Also
        * [`Clone`](https://create.roblox.com/docs/reference/engine/classes/Instance#Clone)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn clone_instance(&self) -> Self {
        let new_ref = self.with_dom_mut(|dom| dom.clone_within(self.dom_ref));
        let new_inst = Self::new(self.dom_id, new_ref);
        new_inst.set_parent(None);
        new_inst
    }

    /**
        Destroys the instance, removing it completely
        from its dom with no way of recovering it.

        If destroying the instance leaves its dom empty (e.g. destroying the
        root `DataModel` of a parsed place), the entire dom is dropped, freeing
        all of its memory.

        Returns `true` if destroyed successfully, `false` if already destroyed.

        ### See Also
        * [`Destroy`](https://create.roblox.com/docs/reference/engine/classes/Instance#Destroy)
          on the Roblox Developer Hub
    */
    pub fn destroy(&mut self) -> bool {
        if self.is_destroyed() {
            false
        } else {
            self.with_dom_mut(|dom| dom.destroy(self.dom_ref));
            dom_registry::drop_if_empty(self.dom_id);
            true
        }
    }

    fn is_destroyed(&self) -> bool {
        // NOTE: This property can not be cached since instance references
        // other than this one may have destroyed this one, and we don't
        // keep track of all current instance reference structs
        dom_registry::with(self.dom_id, |dom| dom.get_by_ref(self.dom_ref).is_none())
            .unwrap_or(true)
    }

    /**
        Destroys all child instances.

        ### See Also
        * [`Instance::Destroy`] for more info about what happens when an instance gets destroyed
        * [`ClearAllChildren`](https://create.roblox.com/docs/reference/engine/classes/Instance#ClearAllChildren)
          on the Roblox Developer Hub
    */
    pub fn clear_all_children(&mut self) {
        self.with_dom_mut(|dom| {
            if let Some(instance) = dom.get_by_ref(self.dom_ref) {
                let child_refs = instance.children().to_vec();
                for child_ref in child_refs {
                    dom.destroy(child_ref);
                }
            }
        });
    }

    /**
        Checks if the instance matches or inherits a given class name.

        ### See Also
        * [`IsA`](https://create.roblox.com/docs/reference/engine/classes/Instance#IsA)
          on the Roblox Developer Hub
    */
    pub fn is_a(&self, class_name: impl AsRef<str>) -> bool {
        class_is_a(self.class_name, class_name).unwrap_or(false)
    }

    /**
        Gets the class name of the instance.

        This will return the correct class name even if the instance has been destroyed.

        ### See Also
        * [`ClassName`](https://create.roblox.com/docs/reference/engine/classes/Instance#ClassName)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_class_name(&self) -> &str {
        self.class_name.as_str()
    }

    /**
        Gets the name of the instance, if it exists.

        ### See Also
        * [`Name`](https://create.roblox.com/docs/reference/engine/classes/Instance#Name)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_name(&self) -> String {
        self.with_dom(|dom| {
            dom.get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document")
                .name
                .clone()
        })
    }

    /**
        Sets the name of the instance, if it exists.

        ### See Also
        * [`Name`](https://create.roblox.com/docs/reference/engine/classes/Instance#Name)
          on the Roblox Developer Hub
    */
    pub fn set_name(&self, name: impl Into<String>) {
        self.with_dom_mut(|dom| {
            dom.get_by_ref_mut(self.dom_ref)
                .expect("Failed to find instance in document")
                .name = name.into();
        });
    }

    /**
        Gets the parent of the instance, if it exists.

        ### See Also
        * [`Parent`](https://create.roblox.com/docs/reference/engine/classes/Instance#Parent)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_parent(&self) -> Option<Instance> {
        self.with_dom(|dom| {
            let parent_ref = dom.get_by_ref(self.dom_ref)?.parent();
            if parent_ref == dom.root_ref() {
                None
            } else {
                dom.get_by_ref(parent_ref)
                    .map(|parent| Self::from_dom_instance(self.dom_id, parent_ref, parent))
            }
        })
    }

    /**
        Sets the parent of the instance, if it exists.

        If the provided parent is [`None`] the instance will become orphaned
        within its current dom.

        If the parent is in a *different* dom, the instance and all of its
        descendants are transferred into the parent's dom. The transferred
        referents are returned so that callers (such as the lua `Parent`
        setter) can update any cached userdata and `dom_id`s accordingly.

        ### See Also
        * [`Parent`](https://create.roblox.com/docs/reference/engine/classes/Instance#Parent)
          on the Roblox Developer Hub
    */
    #[allow(clippy::must_use_candidate)]
    pub fn set_parent(&self, parent: Option<Instance>) -> Vec<DomRef> {
        let parent = parent.map(|p| (p.dom_id, p.dom_ref));
        dom_registry::reparent(self.dom_id, self.dom_ref, parent)
    }
}

#[cfg(feature = "mlua")]
impl LuaExportsTable for Instance {
    const EXPORT_NAME: &'static str = "Instance";

    fn create_exports_table(lua: Lua) -> LuaResult<LuaTable> {
        let instance_new = |lua: &Lua, class_name: String| {
            if class_exists(&class_name) {
                instance_to_lua(lua, Instance::new_orphaned(class_name))
            } else {
                Err(LuaError::RuntimeError(format!(
                    "Failed to create Instance - '{class_name}' is not a valid class name",
                )))
            }
        };

        TableBuilder::new(lua)?
            .with_function("new", instance_new)?
            .build_readonly()
    }
}

/*
    Here we add inheritance-like behavior for instances by creating
    fields that are restricted to specific classnames / base classes

    Note that we should try to be conservative with how many classes
    and methods we support here - we should only implement methods that
    are necessary for modifying the dom and / or having ergonomic access
    to the dom, not try to replicate Roblox engine behavior of instances

    If a user wants to replicate Roblox engine behavior, they can use the
    instance registry, and register properties + methods from the lua side
*/
#[cfg(feature = "mlua")]
impl LuaUserData for Instance {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        data_model::add_fields(fields);
        workspace::add_fields(fields);
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        base::add_methods(methods);
        data_model::add_methods(methods);
        terrain::add_methods(methods);
    }
}

impl Hash for Instance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dom_id.hash(state);
        self.dom_ref.hash(state);
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            if self.is_destroyed() {
                "<<DESTROYED>>".to_string()
            } else {
                self.get_name()
            }
        )
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        self.dom_id == other.dom_id && self.dom_ref == other.dom_ref
    }
}

impl From<Instance> for DomRef {
    fn from(value: Instance) -> Self {
        value.dom_ref
    }
}
