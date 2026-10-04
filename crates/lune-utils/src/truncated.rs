use mlua::prelude::*;

/**
    An integer taken from Lua the way Roblox takes one: a number with a
    fractional part is truncated toward zero instead of being rejected.

    Since 0.12.2, mlua refuses to convert a fractional number to a Rust integer.
    Wrapping the integer type keeps the previous behavior - the fraction is
    dropped first, and mlua still checks the range, so `NaN`, infinities and
    values that do not fit the integer type remain errors.
*/
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Truncated<T>(pub T);

impl<T: FromLua> FromLua for Truncated<T> {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let value = match value {
            LuaValue::Number(n) => LuaValue::Number(n.trunc()),
            LuaValue::String(_) => match lua.coerce_number(value.clone())? {
                Some(n) => LuaValue::Number(n.trunc()),
                None => value,
            },
            other => other,
        };
        T::from_lua(value, lua).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval<T: FromLua>(lua: &Lua, source: &str) -> LuaResult<T> {
        lua.load(source).eval()
    }

    #[test]
    fn truncates_toward_zero() -> LuaResult<()> {
        let lua = Lua::new();
        assert_eq!(eval::<Truncated<i32>>(&lua, "1.9")?, Truncated(1));
        assert_eq!(eval::<Truncated<i32>>(&lua, "-1.9")?, Truncated(-1));
        assert_eq!(eval::<Truncated<u8>>(&lua, "-0.5")?, Truncated(0));
        assert_eq!(eval::<Truncated<i32>>(&lua, "7")?, Truncated(7));
        assert_eq!(eval::<Truncated<i32>>(&lua, "'2.7'")?, Truncated(2));
        Ok(())
    }

    #[test]
    fn keeps_nil_optional() -> LuaResult<()> {
        let lua = Lua::new();
        assert_eq!(eval::<Option<Truncated<i32>>>(&lua, "nil")?, None);
        assert_eq!(
            eval::<Option<Truncated<i32>>>(&lua, "1.5")?,
            Some(Truncated(1))
        );
        Ok(())
    }

    #[test]
    fn rejects_what_does_not_fit() {
        let lua = Lua::new();
        assert!(eval::<Truncated<i32>>(&lua, "0 / 0").is_err());
        assert!(eval::<Truncated<i32>>(&lua, "1 / 0").is_err());
        assert!(eval::<Truncated<u8>>(&lua, "300").is_err());
        assert!(eval::<Truncated<u8>>(&lua, "-1").is_err());
        assert!(eval::<Truncated<i32>>(&lua, "false").is_err());
        assert!(eval::<Truncated<i32>>(&lua, "'abc'").is_err());
    }
}
