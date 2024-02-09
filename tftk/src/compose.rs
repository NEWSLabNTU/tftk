use crate::{
    cli::Compose,
    utils::{
        create_writer, guess_format, keep_or_discard_translation, read_tf_from_path,
        to_angle_format, to_rotation_format, write_tf_to_writer,
    },
};
use anyhow::{bail, Result};
use nalgebra as na;
use std::io::prelude::*;
use tf_format::{MaybeTransform, Rotation, Translation};

pub fn compose(cli: Compose) -> Result<()> {
    let Compose {
        output_format,
        rotation_format,
        angle_format,
        keep_translation,
        pretty,
        output,
        input_files,
    } = cli;
    let Some(output_format) = output_format.or_else(|| guess_format(&output)) else {
        bail!("Please specify the input file format using --output-format");
    };

    let (prod, has_trans) = input_files.iter().try_fold(
        (na::Isometry3::identity(), false),
        |(prod, has_trans), path| -> Result<_> {
            let tf = read_tf_from_path(path, None)?;
            let has_trans = has_trans | tf.t.is_some();
            let iso: na::Isometry3<f64> = tf.to_na_isometry3();
            let prod = prod * iso;
            Ok((prod, has_trans))
        },
    )?;

    let rot: Rotation = prod.rotation.into();
    let trans: Option<Translation> = has_trans.then(|| prod.translation.into());

    let rot = to_rotation_format(rot, rotation_format);
    let rot = to_angle_format(rot, angle_format);
    let trans = keep_or_discard_translation(trans, keep_translation);
    let output_tf = MaybeTransform { r: rot, t: trans };

    let mut writer = create_writer(&output)?;
    write_tf_to_writer(&output_tf, &mut writer, output_format, pretty)?;
    writer.flush()?;

    Ok(())
}
