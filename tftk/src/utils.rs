use crate::cli::{AngleFormat, FileFormat, KeepTranslation, RotationFormat};
use anyhow::{bail, Result};
use noisy_float::types::R64;
use num::Zero;
use std::{
    ffi::OsStr,
    fs::File,
    io::{self, prelude::*, BufReader, BufWriter},
    path::Path,
};
use tf_format::{
    AxisAngle, Euler, MaybeTransform, Quaternion, Rodrigues, Rotation, RotationMatrix, Translation,
};

pub fn read_tf_from_path(path: &Path, format: Option<FileFormat>) -> Result<MaybeTransform> {
    let Some(format) = format.or_else(|| guess_format(path.as_os_str())) else {
        bail!(
            "unable to determine the file format for path '{}'",
            path.display()
        );
    };
    let reader = BufReader::new(File::open(path)?);
    read_tf_from_reader(reader, format)
}

pub fn read_tf_from_reader(mut reader: impl Read, format: FileFormat) -> Result<MaybeTransform> {
    // let Some(format) = guess_format(path.as_os_str()) else {
    //     bail!("unable to determine the file format for path '{}'", path.display());
    // };

    // let reader = BufReader::new(File::open(path)?);

    let tf: MaybeTransform = match format {
        FileFormat::Json => serde_json::from_reader(reader)?,
        FileFormat::Json5 => {
            let mut text = String::new();
            reader.read_to_string(&mut text)?;
            json5::from_str(&text)?
        }
        FileFormat::Yaml => serde_yaml::from_reader(reader)?,
    };
    Ok(tf)
}

// pub fn read_tf_from_str(text: &str, format: FileFormat) -> Result<MaybeTransform> {
//     let tf: MaybeTransform = match format {
//         FileFormat::Json => serde_json::from_str(text)?,
//         FileFormat::Json5 => json5::from_str(text)?,
//         FileFormat::Yaml => serde_yaml::from_str(text)?,
//     };
//     Ok(tf)
// }

// pub fn write_tf_to_path(
//     tf: &MaybeTransform,
//     path: &Path,
//     format: Option<FileFormat>,
// ) -> Result<()> {
//     let Some(format) = format.or_else(|| guess_format(path.as_os_str())) else {
//         bail!(
//             "unable to determine the file format for path '{}'",
//             path.display()
//         );
//     };
//     let mut writer = BufWriter::new(File::create(path)?);
//     write_tf_to_writer(tf, &mut writer, format)?;
//     writer.flush()?;
//     Ok(())
// }

pub fn write_tf_to_writer(
    tf: &MaybeTransform,
    mut writer: impl Write,
    format: FileFormat,
    pretty: bool,
) -> Result<()> {
    match (format, pretty) {
        (FileFormat::Json, true) => serde_json::to_writer_pretty(writer, tf)?,
        (FileFormat::Json, false) => serde_json::to_writer(writer, tf)?,
        (FileFormat::Json5, _) => {
            let text = json5::to_string(tf)?;
            write!(writer, "{text}")?;
        }
        (FileFormat::Yaml, _) => serde_yaml::to_writer(writer, tf)?,
    };
    Ok(())
}

// pub fn tf_to_string(tf: &MaybeTransform, format: FileFormat) -> Result<String> {
//     let text = match format {
//         FileFormat::Json => serde_json::to_string(tf)?,
//         FileFormat::Json5 => json5::to_string(tf)?,
//         FileFormat::Yaml => serde_yaml::to_string(tf)?,
//     };
//     Ok(text)
// }

pub fn create_reader(spec: &OsStr) -> io::Result<BufReader<Box<dyn Read + Send + Sync>>> {
    let reader: Box<dyn Read + Send + Sync> = if spec == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(spec)?)
    };
    let reader = BufReader::new(reader);
    Ok(reader)
}

pub fn create_writer(spec: &OsStr) -> Result<BufWriter<Box<dyn Write + Send + Sync>>> {
    let writer: Box<dyn Write + Send + Sync> = if spec == "-" {
        Box::new(io::stdout())
    } else {
        Box::new(File::create(spec)?)
    };
    let writer = BufWriter::new(writer);
    Ok(writer)
}

pub fn guess_format(spec: &OsStr) -> Option<FileFormat> {
    if spec == "-" {
        return None;
    }

    let path = Path::new(spec);
    let Some(ext) = path.extension() else {
        return None;
    };

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

pub fn to_angle_format(rot: Rotation, angle_format: AngleFormat) -> Rotation {
    match angle_format {
        AngleFormat::Deg => rot.into_degrees(),
        AngleFormat::Rad => rot.into_radians(),
    }
}

pub fn to_rotation_format(rot: Rotation, rotation_format: RotationFormat) -> Rotation {
    match rotation_format {
        RotationFormat::Quat => Quaternion::from(rot).into(),
        RotationFormat::Euler => Euler::from(rot).into(),
        RotationFormat::Mat => RotationMatrix::from(rot).into(),
        RotationFormat::AxisAngle => AxisAngle::from(rot).into(),
        RotationFormat::Rodrigues => Rodrigues::from(rot).into(),
    }
}

pub fn keep_or_discard_translation(
    trans: Option<Translation>,
    keep: KeepTranslation,
) -> Option<Translation> {
    match keep {
        KeepTranslation::Auto => trans,
        KeepTranslation::Always => {
            let z = R64::zero();
            Some(trans.unwrap_or(Translation([z; 3])))
        }
        KeepTranslation::Discard => None,
    }
}
