use crate::PyMaybeTransform;
use pyo3::{exceptions::PyValueError, prelude::*};
use std::{
    fs::File,
    io::{prelude::*, BufReader, BufWriter, Read},
    path::{Path, PathBuf},
    str::FromStr,
};
use tf_format::MaybeTransform;

#[pyfunction]
pub fn load_tf(path: PathBuf, format: Option<String>) -> PyResult<PyMaybeTransform> {
    let format: Option<FileFormat> = match format {
        Some(format) => Some(format.parse()?),
        None => None,
    };
    let Some(format) = format.or_else(|| guess_format(&path)) else {
        return Err(PyValueError::new_err(
            "Unable to guess file format. Please specify the format explicitly.",
        ));
    };

    let mut reader = BufReader::new(File::open(&path)?);

    let tf: MaybeTransform = {
        macro_rules! parse_error {
            ($err:expr) => {
                PyValueError::new_err(format!("{}", $err))
            };
        }

        match format {
            FileFormat::Json => serde_json::from_reader(reader).map_err(|err| parse_error!(err))?,
            FileFormat::Json5 => {
                let mut text = String::new();
                reader.read_to_string(&mut text)?;
                json5::from_str(&text).map_err(|err| parse_error!(err))?
            }
            FileFormat::Yaml => serde_yaml::from_reader(reader).map_err(|err| parse_error!(err))?,
        }
    };

    Ok(tf.into())
}

#[pyfunction]
pub fn loads_tf(string: String, format: String) -> PyResult<PyMaybeTransform> {
    let format: FileFormat = format.parse()?;

    let tf: MaybeTransform = {
        macro_rules! parse_error {
            ($err:expr) => {
                PyValueError::new_err(format!("{}", $err))
            };
        }

        match format {
            FileFormat::Json => serde_json::from_str(&string).map_err(|err| parse_error!(err))?,
            FileFormat::Json5 => json5::from_str(&string).map_err(|err| parse_error!(err))?,
            FileFormat::Yaml => serde_yaml::from_str(&string).map_err(|err| parse_error!(err))?,
        }
    };

    Ok(tf.into())
}

#[pyfunction]
pub fn dump_tf(tf: &PyMaybeTransform, path: PathBuf, format: Option<String>) -> PyResult<()> {
    let format: Option<FileFormat> = match format {
        Some(format) => Some(format.parse()?),
        None => None,
    };
    let Some(format) = format.or_else(|| guess_format(&path)) else {
        return Err(PyValueError::new_err(
            "Unable to guess file format. Please specify the format explicitly.",
        ));
    };
    let tf: MaybeTransform = tf.clone().try_into()?;

    let mut writer = BufWriter::new(File::create(&path)?);

    {
        macro_rules! serialize_error {
            ($err:expr) => {
                PyValueError::new_err(format!("{}", $err))
            };
        }

        match format {
            FileFormat::Json => {
                serde_json::to_writer(writer, &tf).map_err(|err| serialize_error!(err))?
            }
            FileFormat::Json5 => {
                let text = json5::to_string(&tf).map_err(|err| serialize_error!(err))?;
                write!(writer, "{text}")?;
            }
            FileFormat::Yaml => {
                serde_yaml::to_writer(writer, &tf).map_err(|err| serialize_error!(err))?
            }
        }
    };

    Ok(())
}

#[pyfunction]
pub fn dumps_tf(tf: &PyMaybeTransform, format: String) -> PyResult<String> {
    let format: FileFormat = format.parse()?;
    let tf: MaybeTransform = tf.clone().try_into()?;

    let text = {
        macro_rules! serialize_error {
            ($err:expr) => {
                PyValueError::new_err(format!("{}", $err))
            };
        }

        match format {
            FileFormat::Json => serde_json::to_string(&tf).map_err(|err| serialize_error!(err))?,
            FileFormat::Json5 => json5::to_string(&tf).map_err(|err| serialize_error!(err))?,
            FileFormat::Yaml => serde_yaml::to_string(&tf).map_err(|err| serialize_error!(err))?,
        }
    };

    Ok(text)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FileFormat {
    Json,
    Json5,
    Yaml,
}

impl FromStr for FileFormat {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let format = match s {
            "json" => Self::Json,
            "json5" => Self::Json5,
            "Yaml" => Self::Yaml,
            _ => return Err(PyValueError::new_err(format!("unrecognized format '{s}'"))),
        };
        Ok(format)
    }
}

fn guess_format(path: &Path) -> Option<FileFormat> {
    let ext = path.extension()?;

    let format = if ext == "json" {
        FileFormat::Json
    } else if ext == "json5" {
        FileFormat::Json5
    } else if ext == "yaml" {
        FileFormat::Yaml
    } else {
        return None;
    };
    Some(format)
}
