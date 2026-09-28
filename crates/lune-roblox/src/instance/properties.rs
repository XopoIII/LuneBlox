use std::collections::BTreeMap;

use rbx_dom_weak::{
    types::{Attributes as DomAttributes, Variant as DomValue},
    ustr,
};

use super::{Instance, PROPERTY_NAME_ATTRIBUTES, PROPERTY_NAME_TAGS};

impl Instance {
    /**
        Gets a property for the instance, if it exists.
    */
    pub fn get_property(&self, name: impl AsRef<str>) -> Option<DomValue> {
        self.with_dom(|dom| {
            dom.get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document")
                .properties
                .get(&ustr(name.as_ref()))
                .cloned()
        })
    }

    /**
        Sets a property for the instance.

        Note that setting a property here will not fail even if the
        property does not actually exist for the instance class.
    */
    pub fn set_property(&self, name: impl AsRef<str>, value: DomValue) {
        self.with_dom_mut(|dom| {
            dom.get_by_ref_mut(self.dom_ref)
                .expect("Failed to find instance in document")
                .properties
                .insert(ustr(name.as_ref()), value);
        });
    }

    /**
        Gets an attribute for the instance, if it exists.

        ### See Also
        * [`GetAttribute`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetAttribute)
          on the Roblox Developer Hub
    */
    pub fn get_attribute(&self, name: impl AsRef<str>) -> Option<DomValue> {
        self.with_dom(|dom| {
            let inst = dom
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document");
            if let Some(DomValue::Attributes(attributes)) =
                inst.properties.get(&ustr(PROPERTY_NAME_ATTRIBUTES))
            {
                attributes.get(name.as_ref()).cloned()
            } else {
                None
            }
        })
    }

    /**
        Gets all known attributes for the instance.

        ### See Also
        * [`GetAttributes`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetAttributes)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_attributes(&self) -> BTreeMap<String, DomValue> {
        self.with_dom(|dom| {
            let inst = dom
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document");
            if let Some(DomValue::Attributes(attributes)) =
                inst.properties.get(&ustr(PROPERTY_NAME_ATTRIBUTES))
            {
                attributes.clone().into_iter().collect()
            } else {
                BTreeMap::new()
            }
        })
    }

    /**
        Sets an attribute for the instance.

        ### See Also
        * [`SetAttribute`](https://create.roblox.com/docs/reference/engine/classes/Instance#SetAttribute)
          on the Roblox Developer Hub
    */
    pub fn set_attribute(&self, name: impl AsRef<str>, value: DomValue) {
        self.with_dom_mut(|dom| {
            let inst = dom
                .get_by_ref_mut(self.dom_ref)
                .expect("Failed to find instance in document");
            // NOTE: Attributes do not support integers, only floats
            let value = match value {
                DomValue::Int32(i) => DomValue::Float32(i as f32),
                DomValue::Int64(i) => DomValue::Float64(i as f64),
                value => value,
            };
            if let Some(DomValue::Attributes(attributes)) =
                inst.properties.get_mut(&ustr(PROPERTY_NAME_ATTRIBUTES))
            {
                attributes.insert(name.as_ref().to_string(), value);
            } else {
                let mut attributes = DomAttributes::new();
                attributes.insert(name.as_ref().to_string(), value);
                inst.properties.insert(
                    ustr(PROPERTY_NAME_ATTRIBUTES),
                    DomValue::Attributes(attributes),
                );
            }
        });
    }

    /**
        Removes an attribute from the instance.

        Note that this does not have an equivalent in the Roblox engine API,
        but separating this from `set_attribute` lets `set_attribute` be more
        ergonomic and not require an `Option<DomValue>` for the value argument.
        The equivalent in the Roblox engine API would be `instance:SetAttribute(name, nil)`.
    */
    pub fn remove_attribute(&self, name: impl AsRef<str>) {
        self.with_dom_mut(|dom| {
            let inst = dom
                .get_by_ref_mut(self.dom_ref)
                .expect("Failed to find instance in document");
            if let Some(DomValue::Attributes(attributes)) =
                inst.properties.get_mut(&ustr(PROPERTY_NAME_ATTRIBUTES))
            {
                attributes.remove(name.as_ref());
                if attributes.is_empty() {
                    inst.properties.remove(&ustr(PROPERTY_NAME_ATTRIBUTES));
                }
            }
        });
    }

    /**
        Adds a tag to the instance.

        ### See Also
        * [`AddTag`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#AddTag)
          on the Roblox Developer Hub
    */
    pub fn add_tag(&self, name: impl AsRef<str>) {
        self.with_dom_mut(|dom| {
            let inst = dom
                .get_by_ref_mut(self.dom_ref)
                .expect("Failed to find instance in document");
            if let Some(DomValue::Tags(tags)) = inst.properties.get_mut(&ustr(PROPERTY_NAME_TAGS)) {
                tags.push(name.as_ref());
            } else {
                inst.properties.insert(
                    ustr(PROPERTY_NAME_TAGS),
                    DomValue::Tags(vec![name.as_ref().to_string()].into()),
                );
            }
        });
    }

    /**
        Gets all current tags for the instance.

        ### See Also
        * [`GetTags`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#GetTags)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_tags(&self) -> Vec<String> {
        self.with_dom(|dom| {
            let inst = dom
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document");
            if let Some(DomValue::Tags(tags)) = inst.properties.get(&ustr(PROPERTY_NAME_TAGS)) {
                tags.iter().map(ToString::to_string).collect()
            } else {
                Vec::new()
            }
        })
    }

    /**
        Checks if the instance has a specific tag.

        ### See Also
        * [`HasTag`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#HasTag)
          on the Roblox Developer Hub
    */
    pub fn has_tag(&self, name: impl AsRef<str>) -> bool {
        self.with_dom(|dom| {
            let inst = dom
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document");
            if let Some(DomValue::Tags(tags)) = inst.properties.get(&ustr(PROPERTY_NAME_TAGS)) {
                let name = name.as_ref();
                tags.iter().any(|tag| tag == name)
            } else {
                false
            }
        })
    }

    /**
        Removes a tag from the instance.

        ### See Also
        * [`RemoveTag`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#RemoveTag)
          on the Roblox Developer Hub
    */
    pub fn remove_tag(&self, name: impl AsRef<str>) {
        self.with_dom_mut(|dom| {
            let inst = dom
                .get_by_ref_mut(self.dom_ref)
                .expect("Failed to find instance in document");
            if let Some(DomValue::Tags(tags)) = inst.properties.get_mut(&ustr(PROPERTY_NAME_TAGS)) {
                let name = name.as_ref();
                let mut new_tags = tags.iter().map(ToString::to_string).collect::<Vec<_>>();
                new_tags.retain(|tag| tag != name);
                inst.properties
                    .insert(ustr(PROPERTY_NAME_TAGS), DomValue::Tags(new_tags.into()));
            }
        });
    }
}
