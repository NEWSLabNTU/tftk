mod cli;

use anyhow::{bail, Result};
use clap::Parser;
use cli::{AngleFormat, Cli, Convert, FileFormat, KeepTranslation, RotationFormat};
use noisy_float::types::R64;
use num::Zero;
use std::{
    ffi::OsStr,
    fs::File,
    io::{self, prelude::*, BufReader, BufWriter},
    path::Path,
};
use tf_format::{
    AxisAngle, Euler, MaybeTransform, Quaternion, Rodrigues, Rotation, RotationMatrix, Transform,
    Translation,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Convert(cli) => convert(cli)?,
    }

    Ok(())
}

pub fn convert(opts: Convert) -> Result<()> {
    let Convert {
        input_format,
        output_format,
        rotation_format,
        angle_format,
        keep_translation,
        input,
        output,
    } = opts;

    let Some(input_format) = input_format.or_else(|| guess_format(&input)) else {
        bail!("Please specify the input file format using --input-format");
    };
    let Some(output_format) = output_format.or_else(|| guess_format(&output)) else {
        bail!("Please specify the input file format using --output-format");
    };

    let input_tf: MaybeTransform = {
        let mut reader = create_reader(&input)?;

        match input_format {
            FileFormat::Json => serde_json::from_reader(reader)?,
            FileFormat::Json5 => {
                let mut text = String::new();
                reader.read_to_string(&mut text)?;
                json5::from_str(&text)?
            }
            FileFormat::Yaml => serde_yaml::from_reader(reader)?,
        }
    };

    let MaybeTransform { t: trans, r: rot } = input_tf;

    let rot: Rotation = match rotation_format {
        RotationFormat::Quat => Quaternion::from(rot).into(),
        RotationFormat::Euler => Euler::from(rot).into(),
        RotationFormat::Mat => RotationMatrix::from(rot).into(),
        RotationFormat::AxisAngle => AxisAngle::from(rot).into(),
        RotationFormat::Rodrigues => Rodrigues::from(rot).into(),
    };
    let rot = match angle_format {
        AngleFormat::Deg => rot.into_degrees(),
        AngleFormat::Rad => rot.into_radians(),
    };

    let trans = match keep_translation {
        KeepTranslation::Auto => trans,
        KeepTranslation::Always => {
            let z = R64::zero();
            Some(trans.unwrap_or(Translation([z; 3])))
        }
        KeepTranslation::Discard => None,
    };

    let output_tf: MaybeTransform = match trans {
        Some(trans) => Transform { t: trans, r: rot }.into(),
        None => rot.into(),
    };

    {
        let mut writer = create_writer(&output)?;

        match output_format {
            FileFormat::Json => serde_json::to_writer_pretty(writer, &output_tf)?,
            FileFormat::Json5 => {
                let text = json5::to_string(&output_tf)?;
                write!(writer, "{text}")?;
            }
            FileFormat::Yaml => serde_yaml::to_writer(writer, &output_tf)?,
        }
    }

    Ok(())
}

fn create_reader(spec: &OsStr) -> io::Result<BufReader<Box<dyn Read + Send + Sync>>> {
    let reader: Box<dyn Read + Send + Sync> = if spec == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(spec)?)
    };
    let reader = BufReader::new(reader);
    Ok(reader)
}

fn create_writer(spec: &OsStr) -> Result<BufWriter<Box<dyn Write + Send + Sync>>> {
    let writer: Box<dyn Write + Send + Sync> = if spec == "-" {
        Box::new(io::stdout())
    } else {
        Box::new(File::create(spec)?)
    };
    let writer = BufWriter::new(writer);
    Ok(writer)
}

fn guess_format(spec: &OsStr) -> Option<FileFormat> {
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