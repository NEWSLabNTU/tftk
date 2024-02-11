# tf-format: The tftk Rust Library

Import the tf-format crate into your Cargo.toml.

```toml
[depenrencies]
tf-format = { git = "https://github.com/NEWSLabNTU/tftk.git" }
```

The library provides two fundamental types: `Rotation` and
`Transform`. Both can ser/deserialized using the `serde` crate and
convert to `nalgebra` types.

## Rotation API Example

```rust
use tf_format::Rotation;
use std::fs;
use nalgebra as na;

// Load a rotation JSON file
let rot: Rotation = serde_json::from_str(&fs::read_to_string("rotation.json")?)?;

// Convert to quaternion format
let rot = rot.into_quaternion_format();

// Convert to degree units
let rot = rot.into_degrees();

// Convert to nalgebra types
let quat: na::UnitQuaternion<f64> = rot.clone().into();

// Encode into YAML text
let yaml_text = serde_yaml::to_string(&rot)?;
```

## Rigid Transform API Example

```rust
use tf_format::Transform;
use std::fs;
use nalgebra as na;

// Load a rotation JSON file
let trans: Transform = serde_json::from_str(&fs::read_to_string("transform.json")?)?;

// Convert to quaternion format
let trans = trans.into_quaternion_format();

// Convert to degree units
let trans = trans.into_degrees();

// Convert to nalgebra types
let iso: na::Isometry3<f64> = trans.clone().into();

// Encode into YAML text
let yaml_text = serde_yaml::to_string(&trans)?;
```

## MaybeTransform

If it's unsure whether a file stores a transform or a rotation. The
`MaybeTransform` can help to guess the content format.

```rust
use tf_format::MaybeTransform;
use std::fs;
use nalgebra as na;

// Load a rotation JSON file
let maybe: MaybeTransform = serde_json::from_str(&fs::read_to_string("transform.json")?)?;

match maybe {
    MaybeTransform::Transform(trans) => { /* ... */ }
    MaybeTransform::Rotation(trans) => { /* ... */ }
}
```
