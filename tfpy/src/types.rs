use nalgebra as na;
use ndarray as nd;
use noisy_float::types::R64;
use num::NumCast;
use numpy::{PyArray2, PyReadonlyArray2};
use pyo3::{exceptions::PyValueError, prelude::*};
use tf_format::{
    AxisAngle, Euler, MaybeTransform, Quaternion, Rodrigues, Rotation, RotationMatrix, Translation,
};

const MIN_NORM: f64 = 1e-7;

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyMaybeTransform {
    #[pyo3(get, set)]
    pub t: Option<[f64; 3]>,
    #[pyo3(get, set)]
    pub r: PyRotation,
}

#[pymethods]
impl PyMaybeTransform {
    #[new]
    pub fn new(r: PyRotation, t: Option<[f64; 3]>) -> Self {
        Self { t, r }
    }

    #[staticmethod]
    pub fn from_quat(ijkw: [f64; 4]) -> PyResult<Self> {
        Ok(Self {
            r: PyRotation::from_quat(ijkw)?,
            t: None,
        })
    }

    pub fn form(&self) -> PyRotationForm {
        self.r.form()
    }

    pub fn to_form(&self, form: PyRotationForm) -> Self {
        let r = self.r.to_form(form);
        Self { t: self.t, r }
    }

    pub fn to_quat_form(&self) -> Self {
        let r = self.r.to_quat_form();
        Self { t: self.t, r }
    }

    pub fn to_euler_form(&self) -> Self {
        let r = self.r.to_euler_form();
        Self { t: self.t, r }
    }

    pub fn to_axis_angle_form(&self) -> Self {
        let r = self.r.to_axis_angle_form();
        Self { t: self.t, r }
    }

    pub fn to_rotation_matrix_form(&self) -> Self {
        let r = self.r.to_rotation_matrix_form();
        Self { t: self.t, r }
    }

    pub fn to_rodrigues_form(&self) -> Self {
        let r = self.r.to_rodrigues_form();
        Self { t: self.t, r }
    }

    pub fn get_quat_ijkw(&self) -> [f64; 4] {
        self.r.get_quat_ijkw()
    }

    pub fn get_euler_rpy(&self) -> [f64; 3] {
        self.r.get_euler_rpy()
    }

    pub fn get_axis_angle(&self) -> ([f64; 3], f64) {
        self.r.get_axis_angle()
    }

    pub fn get_rotation_matrix<'py>(&self, py: Python<'py>) -> &'py PyArray2<f64> {
        self.r.get_rotation_matrix(py)
    }

    pub fn get_rodrigues(&self) -> [f64; 3] {
        self.r.get_rodrigues()
    }
}

impl From<MaybeTransform> for PyMaybeTransform {
    fn from(tf: MaybeTransform) -> Self {
        let MaybeTransform { r, t } = tf;
        let t = t.map(|Translation([x, y, z])| [x.raw(), y.raw(), z.raw()]);
        Self { t, r: r.into() }
    }
}

impl TryFrom<PyMaybeTransform> for MaybeTransform {
    type Error = PyErr;

    fn try_from(tf: PyMaybeTransform) -> Result<Self, Self::Error> {
        macro_rules! cast {
            ($val:expr) => {
                R64::try_from($val).map_err(|err| {
                    PyValueError::new_err(format!("invalid value '{}': {err}", $val))
                })
            };
        }

        let PyMaybeTransform { r, t } = tf;

        let t = match t {
            Some([x, y, z]) => Some(Translation([cast!(x)?, cast!(y)?, cast!(z)?])),
            None => None,
        };

        Ok(Self { r: r.into(), t })
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyRotation(Rotation);

#[pymethods]
impl PyRotation {
    #[staticmethod]
    pub fn from_quat(ijkw: [f64; 4]) -> PyResult<Self> {
        let [i, j, k, w] = ijkw;
        let Some(rot) = na::Unit::try_new(na::Quaternion::new(w, i, j, k), MIN_NORM) else {
            return Err(PyValueError::new_err(
                "Almost zero-norm quaternion is not allowed",
            ));
        };
        let rot: Rotation = rot.into();
        Ok(rot.into())
    }

    #[staticmethod]
    pub fn from_axis_angle(axis: [f64; 3], radians: f64) -> PyResult<Self> {
        let [x, y, z] = axis;
        let Some(axis) = na::Unit::try_new(na::Vector3::new(x, y, z), MIN_NORM) else {
            return Err(PyValueError::new_err(
                "Almost zero-norm axis is not allowed",
            ));
        };
        let rot: Rotation = na::UnitQuaternion::from_axis_angle(&axis, radians).into();
        Ok(rot.into())
    }

    #[staticmethod]
    pub fn from_rodrigues(params: [f64; 3]) -> PyResult<Self> {
        macro_rules! cast {
            ($val:expr) => {
                <R64 as NumCast>::from($val).unwrap()
            };
        }

        let [p1, p2, p3] = params;
        let rot: Rotation = Rodrigues {
            params: [cast!(p1), cast!(p2), cast!(p3)],
        }
        .into();
        Ok(rot.into())
    }

    #[staticmethod]
    pub fn from_rotation_matrix<'py>(
        _py: Python<'py>,
        matrix: PyReadonlyArray2<'py, f64>,
    ) -> PyResult<Self> {
        let matrix = matrix.as_array();
        if matrix.shape() != [3, 3] {
            return Err(PyValueError::new_err("Expect a 3x3 matrix"));
        }

        let matrix = matrix.as_standard_layout();
        let matrix = na::Matrix3::from_row_slice(matrix.as_slice().unwrap());

        let rot = na::UnitQuaternion::from_matrix(&matrix);
        let rot: Rotation = rot.into();

        Ok(rot.into())
    }

    pub fn form(&self) -> PyRotationForm {
        match &self.0 {
            Rotation::Euler(_) => PyRotationForm::Euler,
            Rotation::Quaternion(_) => PyRotationForm::Quaternion,
            Rotation::AxisAngle(_) => PyRotationForm::AxisAngle,
            Rotation::RotationMatrix(_) => PyRotationForm::RotationMatrix,
            Rotation::Rodrigues(_) => PyRotationForm::Rodrigues,
        }
    }

    pub fn to_form(&self, form: PyRotationForm) -> Self {
        match form {
            PyRotationForm::Euler => self.to_euler_form(),
            PyRotationForm::Quaternion => self.to_quat_form(),
            PyRotationForm::AxisAngle => self.to_axis_angle_form(),
            PyRotationForm::RotationMatrix => self.to_rotation_matrix_form(),
            PyRotationForm::Rodrigues => self.to_rodrigues_form(),
        }
    }

    pub fn to_quat_form(&self) -> Self {
        let rot: Quaternion = self.0.clone().into();
        Self(rot.into())
    }

    pub fn to_euler_form(&self) -> Self {
        let rot: Euler = self.0.clone().into();
        Self(rot.into())
    }

    pub fn to_axis_angle_form(&self) -> Self {
        let rot: AxisAngle = self.0.clone().into();
        Self(rot.into())
    }

    pub fn to_rotation_matrix_form(&self) -> Self {
        let rot: RotationMatrix = self.0.clone().into();
        Self(rot.into())
    }

    pub fn to_rodrigues_form(&self) -> Self {
        let rot: Rodrigues = self.0.clone().into();
        Self(rot.into())
    }

    pub fn get_euler_rpy(&self) -> [f64; 3] {
        let quat: na::UnitQuaternion<f64> = self.0.clone().into();
        quat.euler_angles().into()
    }

    pub fn get_quat_ijkw(&self) -> [f64; 4] {
        use nalgebra::base::coordinates::IJKW;
        let quat: na::UnitQuaternion<f64> = self.0.clone().into();
        let IJKW { i, j, k, w } = **quat;
        [i, j, k, w]
    }

    pub fn get_axis_angle(&self) -> ([f64; 3], f64) {
        use nalgebra::base::coordinates::XYZ;
        let quat: na::UnitQuaternion<f64> = self.0.clone().into();
        let Some((axis, angle)) = quat.axis_angle() else {
            return ([1.0, 0.0, 0.0], 0.0);
        };
        let XYZ { x, y, z } = **axis;
        ([x, y, z], angle)
    }

    pub fn get_rotation_matrix<'py>(&self, py: Python<'py>) -> &'py PyArray2<f64> {
        let quat: na::UnitQuaternion<f64> = self.0.clone().into();
        let rot = quat.to_rotation_matrix();
        let matrix = rot.matrix();
        let array = nd::Array2::from_shape_fn((3, 3), |(r, c)| *matrix.get((r, c)).unwrap());
        PyArray2::from_owned_array(py, array)
    }

    pub fn get_rodrigues(&self) -> [f64; 3] {
        let Rodrigues {
            params: [r1, r2, r3],
        } = self.0.clone().into();
        [r1.raw(), r2.raw(), r3.raw()]
    }
}

impl From<Rotation> for PyRotation {
    fn from(from: Rotation) -> Self {
        Self(from)
    }
}

impl From<PyRotation> for Rotation {
    fn from(from: PyRotation) -> Self {
        from.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[pyclass]
pub enum PyRotationForm {
    Euler,
    Quaternion,
    AxisAngle,
    RotationMatrix,
    Rodrigues,
}
