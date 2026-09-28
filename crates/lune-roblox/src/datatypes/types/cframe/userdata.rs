use glam::{EulerRot, Mat4, Quat, Vec3};
use mlua::{Variadic, prelude::*};

use super::{
    super::{super::*, EnumItem, Vector3},
    CFrame,
};

impl LuaUserData for CFrame {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Position", |_, this| Ok(Vector3(this.position())));
        fields.add_field_method_get("Rotation", |_, this| {
            Ok(CFrame(Mat4::from_cols(
                this.0.x_axis,
                this.0.y_axis,
                this.0.z_axis,
                Vec3::ZERO.extend(1.0),
            )))
        });
        fields.add_field_method_get("X", |_, this| Ok(this.position().x));
        fields.add_field_method_get("Y", |_, this| Ok(this.position().y));
        fields.add_field_method_get("Z", |_, this| Ok(this.position().z));
        fields.add_field_method_get("XVector", |_, this| Ok(Vector3(this.orientation().x_axis)));
        fields.add_field_method_get("YVector", |_, this| Ok(Vector3(this.orientation().y_axis)));
        fields.add_field_method_get("ZVector", |_, this| Ok(Vector3(this.orientation().z_axis)));
        fields.add_field_method_get("RightVector", |_, this| {
            Ok(Vector3(this.orientation().x_axis))
        });
        fields.add_field_method_get("UpVector", |_, this| Ok(Vector3(this.orientation().y_axis)));
        fields.add_field_method_get("LookVector", |_, this| {
            Ok(Vector3(-this.orientation().z_axis))
        });
    }

    #[allow(clippy::too_many_lines)]
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Methods
        methods.add_method("Inverse", |_, this, ()| Ok(this.inverse()));
        methods.add_method(
            "Lerp",
            |_, this, (goal, alpha): (LuaUserDataRef<CFrame>, f32)| {
                let quat_this = Quat::from_mat4(&this.0);
                let quat_goal = Quat::from_mat4(&goal.0);
                let translation = this
                    .0
                    .w_axis
                    .truncate()
                    .lerp(goal.0.w_axis.truncate(), alpha);
                let rotation = quat_this.slerp(quat_goal, alpha);
                Ok(CFrame(Mat4::from_rotation_translation(
                    rotation,
                    translation,
                )))
            },
        );
        methods.add_method("Orthonormalize", |_, this, ()| {
            let rotation = Quat::from_mat4(&this.0);
            let translation = this.0.w_axis.truncate();
            Ok(CFrame(Mat4::from_rotation_translation(
                rotation.normalize(),
                translation,
            )))
        });
        methods.add_method(
            "ToWorldSpace",
            |_, this, rhs: Variadic<LuaUserDataRef<CFrame>>| {
                Ok(rhs
                    .into_iter()
                    .map(|cf| *this * *cf)
                    .collect::<Variadic<_>>())
            },
        );
        methods.add_method(
            "ToObjectSpace",
            |_, this, rhs: Variadic<LuaUserDataRef<CFrame>>| {
                let inverse = this.inverse();
                Ok(rhs
                    .into_iter()
                    .map(|cf| inverse * *cf)
                    .collect::<Variadic<_>>())
            },
        );
        methods.add_method(
            "PointToWorldSpace",
            |_, this, rhs: Variadic<LuaUserDataRef<Vector3>>| {
                Ok(rhs
                    .into_iter()
                    .map(|v3| *this * *v3)
                    .collect::<Variadic<_>>())
            },
        );
        methods.add_method(
            "PointToObjectSpace",
            |_, this, rhs: Variadic<LuaUserDataRef<Vector3>>| {
                let inverse = this.inverse();
                Ok(rhs
                    .into_iter()
                    .map(|v3| inverse * *v3)
                    .collect::<Variadic<_>>())
            },
        );
        methods.add_method(
            "VectorToWorldSpace",
            |_, this, rhs: Variadic<LuaUserDataRef<Vector3>>| {
                let result = *this - Vector3(this.position());
                Ok(rhs
                    .into_iter()
                    .map(|v3| result * *v3)
                    .collect::<Variadic<_>>())
            },
        );
        methods.add_method(
            "VectorToObjectSpace",
            |_, this, rhs: Variadic<LuaUserDataRef<Vector3>>| {
                let inverse = this.inverse();
                let result = inverse - Vector3(inverse.position());
                Ok(rhs
                    .into_iter()
                    .map(|v3| result * *v3)
                    .collect::<Variadic<_>>())
            },
        );
        #[rustfmt::skip]
        methods.add_method("GetComponents", |_, this, ()| {
            let pos = this.position();
            let transposed = this.orientation().transpose();
            Ok((
                pos.x,               pos.y,                 pos.z,
                transposed.x_axis.x, transposed.x_axis.y,   transposed.x_axis.z,
                transposed.y_axis.x, transposed.y_axis.y,   transposed.y_axis.z,
                transposed.z_axis.x, transposed.z_axis.y,   transposed.z_axis.z,
            ))
        });
        methods.add_method("ToEulerAnglesXYZ", |_, this, ()| {
            Ok(Quat::from_mat4(&this.0).to_euler(EulerRot::XYZ))
        });
        methods.add_method("ToEulerAnglesYXZ", |_, this, ()| {
            let (ry, rx, rz) = Quat::from_mat4(&this.0).to_euler(EulerRot::YXZ);
            Ok((rx, ry, rz))
        });
        methods.add_method("ToOrientation", |_, this, ()| {
            let (ry, rx, rz) = Quat::from_mat4(&this.0).to_euler(EulerRot::YXZ);
            Ok((rx, ry, rz))
        });
        methods.add_method("ToEulerAngles", |_, this, order: Option<EnumItem>| {
            let order = match &order {
                None => "XYZ",
                Some(e) if e.parent.desc.name == "RotationOrder" => e.name.as_str(),
                Some(_) => {
                    return Err(LuaError::RuntimeError(
                        "Expected argument #1 to be an Enum.RotationOrder".to_string(),
                    ));
                }
            };
            // glam returns the angles in the variant's named order - the bindings
            // below name each by its axis so we can re-emit them as (rx, ry, rz).
            let quat = Quat::from_mat4(&this.0);
            let angles = match order {
                "XYZ" => {
                    let (x, y, z) = quat.to_euler(EulerRot::XYZ);
                    (x, y, z)
                }
                "XZY" => {
                    let (x, z, y) = quat.to_euler(EulerRot::XZY);
                    (x, y, z)
                }
                "YXZ" => {
                    let (y, x, z) = quat.to_euler(EulerRot::YXZ);
                    (x, y, z)
                }
                "YZX" => {
                    let (y, z, x) = quat.to_euler(EulerRot::YZX);
                    (x, y, z)
                }
                "ZXY" => {
                    let (z, x, y) = quat.to_euler(EulerRot::ZXY);
                    (x, y, z)
                }
                "ZYX" => {
                    let (z, y, x) = quat.to_euler(EulerRot::ZYX);
                    (x, y, z)
                }
                _ => {
                    return Err(LuaError::RuntimeError(format!(
                        "Invalid Enum.RotationOrder '{order}'"
                    )));
                }
            };
            Ok(angles)
        });
        methods.add_method("ToAxisAngle", |_, this, ()| {
            let (axis, angle) = Quat::from_mat4(&this.0).to_axis_angle();
            Ok((Vector3(axis), angle))
        });
        methods.add_method(
            "FuzzyEq",
            |_, this, (other, epsilon): (LuaUserDataRef<CFrame>, Option<f32>)| {
                Ok(this.0.abs_diff_eq(other.0, epsilon.unwrap_or(1e-5)))
            },
        );
        methods.add_method("AngleBetween", |_, this, other: LuaUserDataRef<CFrame>| {
            let a = Quat::from_mat4(&this.0);
            let b = Quat::from_mat4(&other.0);
            Ok(a.angle_between(b))
        });
        // Metamethods
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
        methods.add_meta_method(LuaMetaMethod::Mul, |lua, this, rhs: LuaValue| {
            if let LuaValue::UserData(ud) = &rhs {
                if let Ok(cf) = ud.borrow::<CFrame>() {
                    return lua.create_userdata(*this * *cf);
                } else if let Ok(vec) = ud.borrow::<Vector3>() {
                    return lua.create_userdata(*this * *vec);
                }
            }
            Err(LuaError::FromLuaConversionError {
                from: rhs.type_name(),
                to: "userdata".to_string(),
                message: Some(format!(
                    "Expected CFrame or Vector3, got {}",
                    rhs.type_name()
                )),
            })
        });
        methods.add_meta_method(
            LuaMetaMethod::Add,
            |_, this, vec: LuaUserDataRef<Vector3>| Ok(*this + *vec),
        );
        methods.add_meta_method(
            LuaMetaMethod::Sub,
            |_, this, vec: LuaUserDataRef<Vector3>| Ok(*this - *vec),
        );
    }
}
