use std::collections::VecDeque;

use rbx_dom_weak::Instance as DomInstance;

use super::Instance;

impl Instance {
    /**
        Gets all of the current children of this `Instance`.

        Note that this is a somewhat expensive operation and that other
        operations using weak dom referents should be preferred if possible.

        ### See Also
        * [`GetChildren`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetChildren)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_children(&self) -> Vec<Instance> {
        self.with_dom(|dom| {
            let instance = dom
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document");
            instance
                .children()
                .iter()
                .filter_map(|child_ref| {
                    dom.get_by_ref(*child_ref)
                        .map(|child| Self::from_dom_instance(self.dom_id, *child_ref, child))
                })
                .collect()
        })
    }

    /**
        Gets all of the current descendants of this `Instance` using a breadth-first search.

        Note that this is a somewhat expensive operation and that other
        operations using weak dom referents should be preferred if possible.

        ### See Also
        * [`GetDescendants`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetDescendants)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_descendants(&self) -> Vec<Instance> {
        self.with_dom(|dom| {
            let mut descendants = Vec::new();
            let mut queue = VecDeque::from_iter(
                dom.get_by_ref(self.dom_ref)
                    .expect("Failed to find instance in document")
                    .children(),
            );

            while let Some(queue_ref) = queue.pop_front() {
                if let Some(queue_inst) = dom.get_by_ref(*queue_ref) {
                    descendants.push(Self::from_dom_instance(self.dom_id, *queue_ref, queue_inst));
                    for queue_ref_inner in queue_inst.children().iter().rev() {
                        queue.push_back(queue_ref_inner); // NOTE: push_back for breadth-first
                    }
                }
            }

            descendants
        })
    }

    /**
        Gets all of the current descendants of this `Instance` using a
        depth-first preorder search (a parent appears before its children,
        and children appear in order).

        This is the traversal order used by `QueryDescendants`, and differs
        from [`Instance::get_descendants`] which is breadth-first.
    */
    #[must_use]
    pub fn get_descendants_preorder(&self) -> Vec<Instance> {
        self.with_dom(|dom| {
            let mut descendants = Vec::new();
            let mut queue = VecDeque::from_iter(
                dom.get_by_ref(self.dom_ref)
                    .expect("Failed to find instance in document")
                    .children(),
            );

            while let Some(queue_ref) = queue.pop_front() {
                if let Some(queue_inst) = dom.get_by_ref(*queue_ref) {
                    descendants.push(Self::from_dom_instance(self.dom_id, *queue_ref, queue_inst));
                    for queue_ref_inner in queue_inst.children().iter().rev() {
                        queue.push_front(queue_ref_inner); // NOTE: push_front for depth-first
                    }
                }
            }

            descendants
        })
    }

    /**
        Gets the "full name" of this instance.

        This will be a path composed of instance names from the top-level
        ancestor of this instance down to itself, in the following format:

        `Ancestor.Child.Descendant.Instance`

        ### See Also
        * [`GetFullName`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetFullName)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_full_name(&self) -> String {
        self.with_dom(|dom| {
            let dom_root = dom.root_ref();

            let mut parts = Vec::new();
            let mut instance_ref = self.dom_ref;

            while let Some(instance) = dom.get_by_ref(instance_ref) {
                if instance_ref != dom_root && instance.class != "DataModel" {
                    instance_ref = instance.parent();
                    parts.push(instance.name.clone());
                } else {
                    break;
                }
            }

            parts.reverse();
            parts.join(".")
        })
    }

    /**
        Finds a child of the instance using the given predicate callback.

        ### See Also
        * [`FindFirstChild`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstChild) on the Roblox Developer Hub
        * [`FindFirstChildOfClass`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstChildOfClass) on the Roblox Developer Hub
        * [`FindFirstChildWhichIsA`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstChildWhichIsA) on the Roblox Developer Hub
    */
    pub fn find_child<F>(&self, predicate: F) -> Option<Instance>
    where
        F: Fn(&DomInstance) -> bool,
    {
        self.with_dom(|dom| {
            let children = dom
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document")
                .children();

            children.iter().find_map(|child_ref| {
                let child_inst = dom.get_by_ref(*child_ref)?;
                if predicate(child_inst) {
                    Some(Self::from_dom_instance(self.dom_id, *child_ref, child_inst))
                } else {
                    None
                }
            })
        })
    }

    /**
        Finds an ancestor of the instance using the given predicate callback.

        ### See Also
        * [`FindFirstAncestor`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstAncestor) on the Roblox Developer Hub
        * [`FindFirstAncestorOfClass`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstAncestorOfClass) on the Roblox Developer Hub
        * [`FindFirstAncestorWhichIsA`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstAncestorWhichIsA) on the Roblox Developer Hub
    */
    pub fn find_ancestor<F>(&self, predicate: F) -> Option<Instance>
    where
        F: Fn(&DomInstance) -> bool,
    {
        self.with_dom(|dom| {
            let mut ancestor_ref = dom
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document")
                .parent();

            while let Some(ancestor) = dom.get_by_ref(ancestor_ref) {
                if predicate(ancestor) {
                    return Some(Self::from_dom_instance(self.dom_id, ancestor_ref, ancestor));
                }
                ancestor_ref = ancestor.parent();
            }

            None
        })
    }

    /**
        Finds a descendant of the instance using the given
        predicate callback and a breadth-first search.

        ### See Also
        * [`FindFirstDescendant`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstDescendant) on the Roblox Developer Hub
    */
    pub fn find_descendant<F>(&self, predicate: F) -> Option<Instance>
    where
        F: Fn(&DomInstance) -> bool,
    {
        self.with_dom(|dom| {
            let mut queue = VecDeque::from_iter(
                dom.get_by_ref(self.dom_ref)
                    .expect("Failed to find instance in document")
                    .children(),
            );

            while let Some(queue_ref) = queue.pop_front() {
                if let Some(queue_item) = dom.get_by_ref(*queue_ref) {
                    if predicate(queue_item) {
                        return Some(Self::from_dom_instance(self.dom_id, *queue_ref, queue_item));
                    }
                    queue.extend(queue_item.children());
                }
            }

            None
        })
    }
}
