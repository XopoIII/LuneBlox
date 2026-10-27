use mlua::prelude::*;
use rbx_dom_weak::types::{ContentType, Variant as DomValue, VariantType as DomType};
use rbx_reflection::PropertyMigration;

use super::Instance;
use crate::{
    datatypes::types::{BrickColor, Color3},
    shared::instance::{PropertyInfo, find_property_info},
};

pub(super) fn migrated_property_get(this: &Instance, info: &PropertyInfo) -> Option<DomValue> {
    let migration = info.migration?;
    let new_name = migration.new_property_names().first()?;
    let new_value = this.get_property(new_name).or_else(|| {
        find_property_info(this.class_name, new_name)?
            .value_default
            .cloned()
    })?;
    attempt_reverse_migration(info.value_type?, new_value)
}

pub(super) fn migrated_property_set(
    this: &Instance,
    prop_name: &str,
    migration: &PropertyMigration,
    value: &DomValue,
) -> LuaResult<()> {
    let migrated = migration.perform(value).map_err(|e| {
        LuaError::RuntimeError(format!("Failed to set property '{prop_name}' - {e}"))
    })?;
    for new_name in migration.new_property_names() {
        this.set_property(new_name, migrated.clone());
    }
    this.remove_property(prop_name);
    Ok(())
}

fn attempt_reverse_migration(old_type: DomType, new_value: DomValue) -> Option<DomValue> {
    if new_value.ty() == old_type {
        Some(new_value)
    } else {
        match (old_type, new_value) {
            (DomType::BrickColor, DomValue::Color3uint8(color)) => Some(DomValue::BrickColor(
                BrickColor::from(Color3::from(color)).into(),
            )),
            (DomType::BrickColor, DomValue::Color3(color)) => Some(DomValue::BrickColor(
                BrickColor::from(Color3::from(color)).into(),
            )),
            (DomType::ContentId, DomValue::Content(content)) => {
                Some(DomValue::ContentId(match content.value() {
                    ContentType::Uri(uri) => uri.as_str().into(),
                    _ => "".into(),
                }))
            }
            _ => None,
        }
    }
}
