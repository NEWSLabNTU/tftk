use crate::Transform;
use nalgebra as na;
use num::NumCast;

#[derive(Debug, thiserror::Error)]
pub enum InsertionError {
    #[error(
        "inconsistent transform
         expect
         {expect:#?}
         but found
         {actual:#?}"
    )]
    InconsistentTransform {
        expect: Box<Transform>,
        actual: Box<Transform>,
    },
    #[error("Unable to insert disjoint coordinates '{src}' and '{dst}'")]
    DisjointCoordinates { src: String, dst: String },
}

impl InsertionError {
    pub fn inconsistent_transform_error<T1, T2>(
        expect: na::Isometry3<T1>,
        actual: na::Isometry3<T2>,
    ) -> Self
    where
        T1: NumCast + na::RealField,
        T2: NumCast + na::RealField,
    {
        let expect = Transform::from(expect)
            .into_axis_angle_format()
            .into_degrees()
            .normalize_rotation();
        let actual = Transform::from(actual)
            .into_axis_angle_format()
            .into_degrees()
            .normalize_rotation();
        InsertionError::InconsistentTransform {
            expect: Box::new(expect),
            actual: Box::new(actual),
        }
    }
}
