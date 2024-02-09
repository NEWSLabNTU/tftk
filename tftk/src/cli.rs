use std::ffi::OsString;

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Parser)]
pub enum Cli {
    Convert(Convert),
}

#[derive(Debug, Clone, Parser)]
pub struct Convert {
    #[clap(short = 'f', long)]
    pub input_format: Option<FileFormat>,

    #[clap(short = 't', long)]
    pub output_format: Option<FileFormat>,

    #[clap(short = 'r', long)]
    pub rotation_format: RotationFormat,

    #[clap(short = 'a', long, default_value = "deg")]
    pub angle_format: AngleFormat,

    #[clap(short = 'k', long, default_value = "auto")]
    pub keep_translation: KeepTranslation,

    #[clap(short = 'i', long, default_value = "-")]
    pub input: OsString,

    #[clap(short = 'o', long, default_value = "-")]
    pub output: OsString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum FileFormat {
    Json,
    Json5,
    Yaml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum AngleFormat {
    Deg,
    Rad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum RotationFormat {
    Quat,
    Euler,
    Mat,
    AxisAngle,
    Rodrigues,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum KeepTranslation {
    Auto,
    Always,
    Discard,
}