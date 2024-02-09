use crate::{
    cli::Convert,
    utils::{
        create_reader, create_writer, guess_format, keep_or_discard_translation,
        read_tf_from_reader, to_angle_format, to_rotation_format, write_tf_to_writer,
    },
};
use anyhow::{bail, Result};
use std::io::prelude::*;
use tf_format::{MaybeTransform, Transform};

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
        let reader = create_reader(&input)?;
        read_tf_from_reader(reader, input_format)?
    };

    let MaybeTransform { t: trans, r: rot } = input_tf;
    let rot = to_rotation_format(rot, rotation_format);
    let rot = to_angle_format(rot, angle_format);
    let trans = keep_or_discard_translation(trans, keep_translation);

    let output_tf: MaybeTransform = match trans {
        Some(trans) => Transform { t: trans, r: rot }.into(),
        None => rot.into(),
    };

    {
        let mut writer = create_writer(&output)?;
        write_tf_to_writer(&output_tf, &mut writer, output_format)?;
        writer.flush()?;
    }

    Ok(())
}
