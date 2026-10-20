/*!
    `utf8.graphemes`, which Roblox adds to Luau's `utf8` library and
    open-source Luau does not have.
*/

use std::cell::Cell;

use bstr::ByteSlice;
use mlua::prelude::*;

/**
    Adds `graphemes` to the `utf8` table. It must run before the VM is
    sandboxed, which makes the built-in libraries read-only.
*/
pub fn inject_utf8_graphemes(lua: &Lua) -> LuaResult<()> {
    let utf8 = lua.globals().get::<LuaTable>("utf8")?;
    utf8.set("graphemes", lua.create_function(utf8_graphemes)?)?;
    Ok(())
}

/**
    Returns an iterator over the grapheme clusters of `str` between the byte
    positions `i` and `j`, which default to the whole string and count from
    its end when negative, as in `string.sub`. Each step returns the first and
    last byte position of a cluster, so `str:sub(first, last)` is the cluster.
*/
fn utf8_graphemes(
    lua: &Lua,
    (s, i, j): (LuaString, Option<f64>, Option<f64>),
) -> LuaResult<LuaFunction> {
    let bytes = s.as_bytes();
    let (start, end) = byte_range(bytes.len(), i, j);
    let slice = &bytes[start..end];
    if let Err(e) = std::str::from_utf8(slice) {
        return Err(LuaError::runtime(format!(
            "invalid UTF-8 code at position {}",
            start + e.valid_up_to() + 1
        )));
    }
    let clusters = slice
        .grapheme_indices()
        .map(|(first, last, _)| (start + first + 1, start + last))
        .collect::<Vec<_>>();
    let next = Cell::new(0);
    lua.create_function(move |_, _: LuaMultiValue| {
        let index = next.get();
        next.set(index + 1);
        Ok(clusters.get(index).copied().unzip())
    })
}

/**
    The half-open byte range `string.sub(s, i, j)` would take.
*/
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    reason = "positions are clamped to the string before they are cast"
)]
fn byte_range(len: usize, i: Option<f64>, j: Option<f64>) -> (usize, usize) {
    let len_f = len as f64;
    let position = |n: f64| {
        let n = n.trunc();
        if n < 0.0 {
            (len_f + n + 1.0).max(0.0)
        } else {
            n
        }
    };
    let first = position(i.unwrap_or(1.0)).max(1.0);
    let last = position(j.unwrap_or(-1.0)).min(len_f);
    if first > last {
        (0, 0)
    } else {
        (first as usize - 1, last as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::byte_range;

    #[test]
    fn ranges_match_string_sub() {
        assert_eq!(byte_range(5, None, None), (0, 5));
        assert_eq!(byte_range(5, Some(2.0), None), (1, 5));
        assert_eq!(byte_range(5, Some(-2.0), None), (3, 5));
        assert_eq!(byte_range(5, Some(2.0), Some(3.0)), (1, 3));
        assert_eq!(byte_range(5, Some(0.0), Some(10.0)), (0, 5));
        assert_eq!(byte_range(5, Some(-10.0), Some(-4.0)), (0, 2));
        assert_eq!(byte_range(5, Some(4.0), Some(2.0)), (0, 0));
        assert_eq!(byte_range(0, None, None), (0, 0));
    }
}
