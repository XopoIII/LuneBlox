#![allow(clippy::items_after_statements)]

use core::fmt;
use std::ops;

use glam::{Mat3, Mat4, Vec3};
use rbx_dom_weak::types::{CFrame as DomCFrame, Matrix3 as DomMatrix3, Vector3 as DomVector3};

use super::Vector3;

mod exports;
mod userdata;

/**
    An implementation of the [CFrame](https://create.roblox.com/docs/reference/engine/datatypes/CFrame)
    Roblox datatype, backed by [`glam::Mat4`].

    This implements all documented properties, methods & constructors of the `CFrame` class as of May 2026.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CFrame(pub Mat4);

impl CFrame {
    pub const IDENTITY: Self = Self(Mat4::IDENTITY);

    fn position(&self) -> Vec3 {
        self.0.w_axis.truncate()
    }

    fn orientation(&self) -> Mat3 {
        Mat3::from_cols(
            self.0.x_axis.truncate(),
            self.0.y_axis.truncate(),
            self.0.z_axis.truncate(),
        )
    }

    fn inverse(&self) -> Self {
        Self(self.0.inverse())
    }
}

impl fmt::Display for CFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pos = self.position();
        let transposed = self.orientation().transpose();
        write!(
            f,
            "{}, {}, {}, {}",
            Vector3(pos),
            Vector3(transposed.x_axis),
            Vector3(transposed.y_axis),
            Vector3(transposed.z_axis)
        )
    }
}

impl ops::Mul for CFrame {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        CFrame(self.0 * rhs.0)
    }
}

impl ops::Mul<Vector3> for CFrame {
    type Output = Vector3;
    fn mul(self, rhs: Vector3) -> Self::Output {
        Vector3(self.0.project_point3(rhs.0))
    }
}

impl ops::Add<Vector3> for CFrame {
    type Output = Self;
    fn add(self, rhs: Vector3) -> Self::Output {
        CFrame(Mat4::from_cols(
            self.0.x_axis,
            self.0.y_axis,
            self.0.z_axis,
            self.0.w_axis + rhs.0.extend(0.0),
        ))
    }
}

impl ops::Sub<Vector3> for CFrame {
    type Output = Self;
    fn sub(self, rhs: Vector3) -> Self::Output {
        CFrame(Mat4::from_cols(
            self.0.x_axis,
            self.0.y_axis,
            self.0.z_axis,
            self.0.w_axis - rhs.0.extend(0.0),
        ))
    }
}

impl From<DomCFrame> for CFrame {
    fn from(v: DomCFrame) -> Self {
        let transposed = v.orientation.transpose();
        CFrame(Mat4::from_cols(
            Vector3::from(transposed.x).0.extend(0.0),
            Vector3::from(transposed.y).0.extend(0.0),
            Vector3::from(transposed.z).0.extend(0.0),
            Vector3::from(v.position).0.extend(1.0),
        ))
    }
}

impl From<CFrame> for DomCFrame {
    fn from(v: CFrame) -> Self {
        let transposed = v.orientation().transpose();
        DomCFrame {
            position: DomVector3::from(Vector3(v.position())),
            orientation: DomMatrix3::new(
                DomVector3::from(Vector3(transposed.x_axis)),
                DomVector3::from(Vector3(transposed.y_axis)),
                DomVector3::from(Vector3(transposed.z_axis)),
            ),
        }
    }
}

/**
    Creates a matrix at the position `from`, looking towards `to`.

    [`glam`] does provide functions such as [`look_at_lh`], [`look_at_rh`] and more but
    they all create view matrices for camera transforms which is not what we want here.
*/
fn look_at(from: Vec3, to: Vec3, up: Vec3) -> Mat4 {
    let dir = (to - from).normalize();
    let xaxis = dir.cross(up).normalize();
    let yaxis = xaxis.cross(dir).normalize();

    Mat4::from_cols(
        xaxis.extend(0.0),
        yaxis.extend(0.0),
        (-dir).extend(0.0),
        from.extend(1.0),
    )
}

#[cfg(test)]
mod cframe_test {
    use glam::{Mat4, Vec3};
    use rbx_dom_weak::types::{CFrame as DomCFrame, Matrix3 as DomMatrix3, Vector3 as DomVector3};

    use super::CFrame;

    #[test]
    fn dom_cframe_from_cframe() {
        let dom_cframe = DomCFrame::new(
            DomVector3::new(1.0, 2.0, 3.0),
            DomMatrix3::new(
                DomVector3::new(1.0, 2.0, 3.0),
                DomVector3::new(1.0, 2.0, 3.0),
                DomVector3::new(1.0, 2.0, 3.0),
            ),
        );

        let cframe = CFrame(Mat4::from_cols(
            Vec3::new(1.0, 1.0, 1.0).extend(0.0),
            Vec3::new(2.0, 2.0, 2.0).extend(0.0),
            Vec3::new(3.0, 3.0, 3.0).extend(0.0),
            Vec3::new(1.0, 2.0, 3.0).extend(1.0),
        ));

        assert_eq!(CFrame::from(dom_cframe), cframe);
    }

    #[test]
    fn cframe_from_dom_cframe() {
        let cframe = CFrame(Mat4::from_cols(
            Vec3::new(1.0, 2.0, 3.0).extend(0.0),
            Vec3::new(1.0, 2.0, 3.0).extend(0.0),
            Vec3::new(1.0, 2.0, 3.0).extend(0.0),
            Vec3::new(1.0, 2.0, 3.0).extend(1.0),
        ));

        let dom_cframe = DomCFrame::new(
            DomVector3::new(1.0, 2.0, 3.0),
            DomMatrix3::new(
                DomVector3::new(1.0, 1.0, 1.0),
                DomVector3::new(2.0, 2.0, 2.0),
                DomVector3::new(3.0, 3.0, 3.0),
            ),
        );

        assert_eq!(DomCFrame::from(cframe), dom_cframe);
    }
}
