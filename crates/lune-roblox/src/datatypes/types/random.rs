use std::{
    collections::hash_map::RandomState,
    f64::consts::TAU,
    hash::{BuildHasher, Hasher},
    time::{SystemTime, UNIX_EPOCH},
};

use glam::Vec3;
use mlua::prelude::*;

use lune_utils::{TableBuilder, Truncated};

use crate::exports::LuaExportsTable;

use super::Vector3;

const PCG32_MULTIPLIER: u64 = 6_364_136_223_846_793_005;
// PCG32_INC | 1 in lmathlib.cpp
const PCG32_INCREMENT: u64 = 105;

/**
    An implementation of the [Random](https://create.roblox.com/docs/reference/engine/datatypes/Random)
    Roblox datatype.

    This implements all documented methods & constructors of the `Random` class as of October 2026.

    The generator is PCG32, seeded and stepped as Luau's `math.random` is: `Random.new(seed)`
    followed by `NextNumber()` or `NextInteger(min, max)` gives what `math.randomseed(seed)`
    followed by `math.random()` or `math.random(min, max)` gives, for seeds and ranges
    `math.random` accepts. Whether a live Roblox client gives the same sequence for a seed
    is not verified.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Random {
    state: u64,
}

impl Random {
    fn from_seed(seed: u64) -> Self {
        let mut random = Self { state: 0 };
        random.next_u32();
        random.state = random.state.wrapping_add(seed);
        random.next_u32();
        random
    }

    fn from_entropy() -> Self {
        // RandomState is keyed randomly per process, the clock per call
        let mut hasher = RandomState::new().build_hasher();
        if let Ok(elapsed) = SystemTime::now().duration_since(UNIX_EPOCH) {
            hasher.write_u128(elapsed.as_nanos());
        }
        Self::from_seed(hasher.finish())
    }

    #[allow(
        clippy::cast_possible_truncation,
        reason = "PCG32 keeps 32 of the 64 bits"
    )]
    fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old
            .wrapping_mul(PCG32_MULTIPLIER)
            .wrapping_add(PCG32_INCREMENT);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    #[allow(clippy::cast_precision_loss, reason = "the bits become a fraction")]
    fn next_number(&mut self) -> f64 {
        let low = u64::from(self.next_u32());
        let high = u64::from(self.next_u32());
        // 2^-64, as math.random's ldexp(r, -64)
        ((high << 32) | low) as f64 * (1.0 / 18_446_744_073_709_551_616.0)
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        reason = "two's complement arithmetic on the range, as math.random does it"
    )]
    fn next_integer(&mut self, min: i64, max: i64) -> LuaResult<i64> {
        if min > max {
            return Err(LuaError::runtime(
                "invalid argument #2 to 'NextInteger' (interval is empty)",
            ));
        }
        let span = max.wrapping_sub(min) as u64;
        let offset = if span < u64::from(u32::MAX) {
            // A range math.random takes, picked the way it picks one
            (u64::from(self.next_u32()) * (span + 1)) >> 32
        } else {
            let low = u64::from(self.next_u32());
            let high = u64::from(self.next_u32());
            let bits = u128::from((high << 32) | low);
            match span.checked_add(1) {
                Some(count) => ((bits * u128::from(count)) >> 64) as u64,
                None => bits as u64,
            }
        };
        Ok(min.wrapping_add(offset as i64))
    }

    #[allow(clippy::cast_possible_truncation, reason = "Vector3 holds f32")]
    fn next_unit_vector(&mut self) -> Vector3 {
        let z = self.next_number() * 2.0 - 1.0;
        let angle = self.next_number() * TAU;
        let radius = (1.0 - z * z).sqrt();
        Vector3(Vec3::new(
            (radius * angle.cos()) as f32,
            (radius * angle.sin()) as f32,
            z as f32,
        ))
    }

    fn shuffle(&mut self, table: &LuaTable) -> LuaResult<()> {
        if table.is_readonly() {
            return Err(LuaError::runtime(
                "invalid argument #1 to 'Shuffle' (table is frozen)",
            ));
        }
        let len = table.raw_len();
        for index in 1..=len {
            if table.raw_get::<LuaValue>(index)?.is_nil() {
                return Err(LuaError::runtime(format!(
                    "invalid argument #1 to 'Shuffle' (table has a hole at index {index})"
                )));
            }
        }
        for index in (2..=len).rev() {
            let upper = i64::try_from(index).into_lua_err()?;
            let other = usize::try_from(self.next_integer(1, upper)?).into_lua_err()?;
            if other != index {
                let a: LuaValue = table.raw_get(index)?;
                let b: LuaValue = table.raw_get(other)?;
                table.raw_set(index, b)?;
                table.raw_set(other, a)?;
            }
        }
        Ok(())
    }
}

impl LuaExportsTable for Random {
    const EXPORT_NAME: &'static str = "Random";

    fn create_exports_table(lua: Lua) -> LuaResult<LuaTable> {
        #[allow(
            clippy::cast_sign_loss,
            reason = "a negative seed is sign extended, as in Luau"
        )]
        let random_new = |_: &Lua, seed: Option<Truncated<i64>>| {
            Ok(match seed {
                Some(Truncated(seed)) => Random::from_seed(seed as u64),
                None => Random::from_entropy(),
            })
        };

        TableBuilder::new(lua)?
            .with_function("new", random_new)?
            .build_readonly()
    }
}

impl LuaUserData for Random {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "NextInteger",
            |_, this, (min, max): (Truncated<i64>, Truncated<i64>)| this.next_integer(min.0, max.0),
        );
        methods.add_method_mut(
            "NextNumber",
            |_, this, (min, max): (Option<f64>, Option<f64>)| {
                let min = min.unwrap_or(0.0);
                let max = max.unwrap_or(1.0);
                Ok(min + (max - min) * this.next_number())
            },
        );
        methods.add_method_mut("NextUnitVector", |_, this, ()| Ok(this.next_unit_vector()));
        methods.add_method_mut("Shuffle", |_, this, table: LuaTable| this.shuffle(&table));
        methods.add_method("Clone", |_, this, ()| Ok(*this));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, _, ()| Ok("Random"));
    }
}

#[cfg(test)]
mod tests {
    use super::Random;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Random::from_seed(42);
        let mut b = Random::from_seed(42);
        for _ in 0..100 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn integers_stay_in_range() {
        let mut random = Random::from_seed(7);
        for _ in 0..1000 {
            let n = random.next_integer(-3, 3).unwrap();
            assert!((-3..=3).contains(&n));
        }
        for _ in 0..1000 {
            let n = random.next_integer(i64::MIN, i64::MAX).unwrap();
            assert!((i64::MIN..=i64::MAX).contains(&n));
        }
        assert_eq!(random.next_integer(5, 5).unwrap(), 5);
        assert!(random.next_integer(2, 1).is_err());
    }

    #[test]
    fn numbers_stay_in_unit_interval() {
        let mut random = Random::from_seed(9);
        for _ in 0..1000 {
            let n = random.next_number();
            assert!((0.0..1.0).contains(&n));
        }
    }
}
