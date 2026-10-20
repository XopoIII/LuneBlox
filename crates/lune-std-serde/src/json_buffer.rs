/*!
    Buffers in JSON, written and read the way Roblox's `HttpService:JSONEncode`
    and `JSONDecode` write and read them, so that a value a game stored -
    in a `DataStore`, say - decodes here and the other way around.

    Roblox writes a buffer as an object, `{"m":null,"t":"buffer","base64":"..."}`,
    or with `zbase64` in place of `base64` when the bytes compressed with zstd are
    shorter. Both are read; `zbase64` is written when it is shorter, as on Roblox.
*/

use std::collections::HashMap;
use std::ffi::c_void;

use data_encoding::BASE64;
use futures_lite::future::block_on;
use mlua::prelude::*;
use serde_json::Value as JsonValue;

use crate::compress_decompress::{CompressDecompressFormat, compress, decompress};

const KEY_META: &str = "m";
const KEY_TYPE: &str = "t";
const KEY_BASE64: &str = "base64";
const KEY_ZBASE64: &str = "zbase64";
const TYPE_BUFFER: &str = "buffer";

/**
    Replaces every buffer in the value with the table Roblox writes for it.

    A table holding no buffer, at any depth, is returned as it is; one that does
    is copied, so the caller's tables are never changed. A table reached twice is
    converted once, and one that contains itself is left for the encoder to reject.
*/
pub(crate) fn buffers_to_tables(lua: &Lua, value: LuaValue) -> LuaResult<LuaValue> {
    let mut converted = HashMap::new();
    let mut path = Vec::new();
    convert_value(lua, value, &mut converted, &mut path)
}

fn convert_value(
    lua: &Lua,
    value: LuaValue,
    converted: &mut HashMap<*const c_void, LuaValue>,
    path: &mut Vec<*const c_void>,
) -> LuaResult<LuaValue> {
    match value {
        LuaValue::Buffer(buffer) => buffer_to_table(lua, &buffer.to_vec()).map(LuaValue::Table),
        LuaValue::Table(table) => {
            let pointer = table.to_pointer();
            if let Some(done) = converted.get(&pointer) {
                return Ok(done.clone());
            }
            if path.contains(&pointer) {
                return Ok(LuaValue::Table(table));
            }
            path.push(pointer);
            let result = convert_table(lua, &table, converted, path);
            path.pop();
            let result = result?;
            converted.insert(pointer, result.clone());
            Ok(result)
        }
        other => Ok(other),
    }
}

fn convert_table(
    lua: &Lua,
    table: &LuaTable,
    converted: &mut HashMap<*const c_void, LuaValue>,
    path: &mut Vec<*const c_void>,
) -> LuaResult<LuaValue> {
    let mut pairs = Vec::new();
    let mut changed = false;
    for pair in table.pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair?;
        let new_value = convert_value(lua, value.clone(), converted, path)?;
        changed |= new_value != value;
        pairs.push((key, new_value));
    }
    if !changed {
        return Ok(LuaValue::Table(table.clone()));
    }
    let copy = lua.create_table_with_capacity(0, pairs.len())?;
    for (key, value) in pairs {
        copy.raw_set(key, value)?;
    }
    copy.set_metatable(table.metatable())?;
    Ok(LuaValue::Table(copy))
}

fn buffer_to_table(lua: &Lua, bytes: &[u8]) -> LuaResult<LuaTable> {
    let plain = BASE64.encode(bytes);
    let compressed = BASE64.encode(&block_on(compress(
        bytes,
        CompressDecompressFormat::Zstd,
        None,
    ))?);

    let table = lua.create_table_with_capacity(0, 3)?;
    table.raw_set(KEY_META, lua.null())?;
    table.raw_set(KEY_TYPE, TYPE_BUFFER)?;
    if compressed.len() < plain.len() {
        table.raw_set(KEY_ZBASE64, compressed)?;
    } else {
        table.raw_set(KEY_BASE64, plain)?;
    }
    Ok(table)
}

/**
    Converts decoded JSON to a Lua value, turning every object Roblox writes
    for a buffer back into a buffer.

    Values holding no such object are converted by `convert`, the conversion
    used for all other JSON, so they come out exactly as they always did.
*/
pub(crate) fn json_to_lua(
    lua: &Lua,
    value: &JsonValue,
    convert: &impl Fn(&JsonValue) -> LuaResult<LuaValue>,
) -> LuaResult<LuaValue> {
    if let Some(bytes) = json_buffer_bytes(value)? {
        return lua.create_buffer(bytes).map(LuaValue::Buffer);
    }
    match value {
        JsonValue::Array(items) if contains_buffer(value) => {
            let table = lua.create_table_with_capacity(items.len(), 0)?;
            for (index, item) in items.iter().enumerate() {
                table.raw_set(index + 1, json_to_lua(lua, item, convert)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        JsonValue::Object(entries) if contains_buffer(value) => {
            let table = lua.create_table_with_capacity(0, entries.len())?;
            for (key, item) in entries {
                table.raw_set(key.as_str(), json_to_lua(lua, item, convert)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        other => convert(other),
    }
}

fn contains_buffer(value: &JsonValue) -> bool {
    match value {
        JsonValue::Array(items) => items.iter().any(contains_buffer),
        JsonValue::Object(entries) => {
            is_json_buffer(value) || entries.values().any(contains_buffer)
        }
        _ => false,
    }
}

fn is_json_buffer(value: &JsonValue) -> bool {
    let JsonValue::Object(entries) = value else {
        return false;
    };
    let encoded = [KEY_BASE64, KEY_ZBASE64]
        .iter()
        .filter(|key| entries.get(**key).is_some_and(JsonValue::is_string))
        .count();
    entries.get(KEY_TYPE).and_then(JsonValue::as_str) == Some(TYPE_BUFFER)
        && encoded == 1
        && entries
            .keys()
            .all(|key| matches!(key.as_str(), KEY_META | KEY_TYPE | KEY_BASE64 | KEY_ZBASE64))
}

fn json_buffer_bytes(value: &JsonValue) -> LuaResult<Option<Vec<u8>>> {
    if !is_json_buffer(value) {
        return Ok(None);
    }
    let JsonValue::Object(entries) = value else {
        return Ok(None);
    };
    if let Some(encoded) = entries.get(KEY_BASE64).and_then(JsonValue::as_str) {
        let bytes = BASE64.decode(encoded.as_bytes()).map_err(|e| {
            LuaError::runtime(format!("Failed to decode the base64 of a buffer: {e}"))
        })?;
        return Ok(Some(bytes));
    }
    let encoded = entries
        .get(KEY_ZBASE64)
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let compressed = BASE64
        .decode(encoded.as_bytes())
        .map_err(|e| LuaError::runtime(format!("Failed to decode the zbase64 of a buffer: {e}")))?;
    let bytes = block_on(decompress(compressed, CompressDecompressFormat::Zstd))?;
    Ok(Some(bytes))
}
