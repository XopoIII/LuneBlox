use glam::{EulerRot, Mat4, Quat, Vec3};
use mlua::prelude::*;

use lune_utils::TableBuilder;

use crate::exports::LuaExportsTable;

use super::{
    super::{EnumItem, Vector3},
    CFrame, look_at,
};

impl LuaExportsTable for CFrame {
    const EXPORT_NAME: &'static str = "CFrame";

    #[allow(clippy::too_many_lines)]
    fn create_exports_table(lua: Lua) -> LuaResult<LuaTable> {
        let cframe_angles = |_: &Lua, (rx, ry, rz): (f32, f32, f32)| {
            Ok(CFrame(Mat4::from_euler(EulerRot::XYZ, rx, ry, rz)))
        };

        let cframe_from_axis_angle = |_: &Lua, (v, r): (LuaUserDataRef<Vector3>, f32)| {
            Ok(CFrame(Mat4::from_axis_angle(v.0, r)))
        };

        let cframe_from_euler_angles_xyz = |_: &Lua, (rx, ry, rz): (f32, f32, f32)| {
            Ok(CFrame(Mat4::from_euler(EulerRot::XYZ, rx, ry, rz)))
        };

        let cframe_from_euler_angles_yxz = |_: &Lua, (rx, ry, rz): (f32, f32, f32)| {
            Ok(CFrame(Mat4::from_euler(EulerRot::YXZ, ry, rx, rz)))
        };

        let cframe_from_matrix = |_: &Lua,
                                  (pos, rx, ry, rz): (
            LuaUserDataRef<Vector3>,
            LuaUserDataRef<Vector3>,
            LuaUserDataRef<Vector3>,
            Option<LuaUserDataRef<Vector3>>,
        )| {
            Ok(CFrame(Mat4::from_cols(
                rx.0.extend(0.0),
                ry.0.extend(0.0),
                rz.map_or_else(|| rx.0.cross(ry.0).normalize(), |r| r.0)
                    .extend(0.0),
                pos.0.extend(1.0),
            )))
        };

        let cframe_from_orientation = |_: &Lua, (rx, ry, rz): (f32, f32, f32)| {
            Ok(CFrame(Mat4::from_euler(EulerRot::YXZ, ry, rx, rz)))
        };

        let cframe_look_at = |_: &Lua,
                              (from, to, up): (
            LuaUserDataRef<Vector3>,
            LuaUserDataRef<Vector3>,
            Option<LuaUserDataRef<Vector3>>,
        )| {
            Ok(CFrame(look_at(
                from.0,
                to.0,
                up.as_deref().unwrap_or(&Vector3(Vec3::Y)).0,
            )))
        };

        let cframe_look_along = |_: &Lua,
                                 (at, direction, up): (
            LuaUserDataRef<Vector3>,
            LuaUserDataRef<Vector3>,
            Option<LuaUserDataRef<Vector3>>,
        )| {
            Ok(CFrame(look_at(
                at.0,
                at.0 + direction.0,
                up.as_deref().unwrap_or(&Vector3(Vec3::Y)).0,
            )))
        };

        let cframe_from_rotation_between_vectors =
            |_: &Lua, (from, to): (LuaUserDataRef<Vector3>, LuaUserDataRef<Vector3>)| {
                Ok(CFrame(Mat4::from_quat(Quat::from_rotation_arc(
                    from.0.normalize(),
                    to.0.normalize(),
                ))))
            };

        let cframe_from_euler_angles =
            |_: &Lua, (rx, ry, rz, order): (f32, f32, f32, Option<EnumItem>)| {
                let order = match &order {
                    None => "XYZ",
                    Some(e) if e.parent.desc.name == "RotationOrder" => e.name.as_str(),
                    Some(_) => {
                        return Err(LuaError::RuntimeError(
                            "Expected argument #4 to be an Enum.RotationOrder".to_string(),
                        ));
                    }
                };
                // glam's `from_euler` consumes the angles in the order named by the
                // variant, so each axis angle is routed to its matching slot.
                let cframe = match order {
                    "XYZ" => CFrame(Mat4::from_euler(EulerRot::XYZ, rx, ry, rz)),
                    "XZY" => CFrame(Mat4::from_euler(EulerRot::XZY, rx, rz, ry)),
                    "YXZ" => CFrame(Mat4::from_euler(EulerRot::YXZ, ry, rx, rz)),
                    "YZX" => CFrame(Mat4::from_euler(EulerRot::YZX, ry, rz, rx)),
                    "ZXY" => CFrame(Mat4::from_euler(EulerRot::ZXY, rz, rx, ry)),
                    "ZYX" => CFrame(Mat4::from_euler(EulerRot::ZYX, rz, ry, rx)),
                    _ => {
                        return Err(LuaError::RuntimeError(format!(
                            "Invalid Enum.RotationOrder '{order}'"
                        )));
                    }
                };
                Ok(cframe)
            };

        // Dynamic args constructor
        type ArgsPos = LuaUserDataRef<Vector3>;
        type ArgsPosLookAt = (LuaUserDataRef<Vector3>, LuaUserDataRef<Vector3>);
        type ArgsLook = (
            LuaUserDataRef<Vector3>,
            LuaUserDataRef<Vector3>,
            Option<LuaUserDataRef<Vector3>>,
        );

        type ArgsPosXYZ = (f32, f32, f32);
        type ArgsPosXYZQuat = (f32, f32, f32, f32, f32, f32, f32);
        type ArgsMatrix = (f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32);

        let cframe_new = |lua: &Lua, args: LuaMultiValue| match args.len() {
            0 => Ok(CFrame(Mat4::IDENTITY)),

            1 => match ArgsPos::from_lua_multi(args, lua) {
                Ok(pos) => Ok(CFrame(Mat4::from_translation(pos.0))),
                Err(err) => Err(err),
            },

            2 => match ArgsPosLookAt::from_lua_multi(args, lua) {
                Ok((pos, look_at_pos)) => Ok(CFrame(look_at(pos.0, look_at_pos.0, Vec3::Y))),
                Err(err) => Err(err),
            },

            3 => {
                if let Ok((from, to, up)) = ArgsLook::from_lua_multi(args.clone(), lua) {
                    Ok(CFrame(look_at(
                        from.0,
                        to.0,
                        up.as_deref().unwrap_or(&Vector3(Vec3::Y)).0,
                    )))
                } else if let Ok((x, y, z)) = ArgsPosXYZ::from_lua_multi(args, lua) {
                    Ok(CFrame(Mat4::from_translation(Vec3::new(x, y, z))))
                } else {
                    // TODO: Make this error message better
                    Err(LuaError::RuntimeError(
                        "Invalid arguments to constructor".to_string(),
                    ))
                }
            }

            7 => match ArgsPosXYZQuat::from_lua_multi(args, lua) {
                Ok((x, y, z, qx, qy, qz, qw)) => Ok(CFrame(Mat4::from_rotation_translation(
                    Quat::from_array([qx, qy, qz, qw]),
                    Vec3::new(x, y, z),
                ))),
                Err(err) => Err(err),
            },

            12 => match ArgsMatrix::from_lua_multi(args, lua) {
                Ok((x, y, z, r00, r01, r02, r10, r11, r12, r20, r21, r22)) => {
                    Ok(CFrame(Mat4::from_cols_array_2d(&[
                        [r00, r10, r20, 0.0],
                        [r01, r11, r21, 0.0],
                        [r02, r12, r22, 0.0],
                        [x, y, z, 1.0],
                    ])))
                }
                Err(err) => Err(err),
            },

            _ => Err(LuaError::RuntimeError(format!(
                "Invalid number of arguments: expected 0, 1, 2, 3, 7, or 12, got {}",
                args.len()
            ))),
        };

        TableBuilder::new(lua)?
            .with_function("Angles", cframe_angles)?
            .with_value("identity", CFrame(Mat4::IDENTITY))?
            .with_function("fromAxisAngle", cframe_from_axis_angle)?
            .with_function("fromEulerAngles", cframe_from_euler_angles)?
            .with_function("fromEulerAnglesXYZ", cframe_from_euler_angles_xyz)?
            .with_function("fromEulerAnglesYXZ", cframe_from_euler_angles_yxz)?
            .with_function("fromMatrix", cframe_from_matrix)?
            .with_function("fromOrientation", cframe_from_orientation)?
            .with_function(
                "fromRotationBetweenVectors",
                cframe_from_rotation_between_vectors,
            )?
            .with_function("lookAlong", cframe_look_along)?
            .with_function("lookAt", cframe_look_at)?
            .with_function("new", cframe_new)?
            .build_readonly()
    }
}
