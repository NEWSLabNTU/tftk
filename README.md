# **tftk**: Rigid Transformation Toolkit

**tftk** is a tiny toolkit to process rigid transformation in various
formats. It provides an easy-to-install command line tool, and Rust
and Python libraries for programming. It supports the file formats:

- File formats: JSON, JSON5 and YAML.
- Rotation notations: quaternion, Euler, axis-angle, rotation matrix
  and Rodrigues.


## The Command Line Tool

The command line tool is used to convert the file format and geometric
computation.

```sh
# Convert to Euler format
tftk convert -i input.json -o output.json -r euler

# Compute the product of transforms
tftk compose r1.json r2.json r3.json -i output.json -r quat
```

See the [README](tftk/README.md) to learn more details.

## The Python Library

Here is an example to get rotation parameters.

```python
import tfpy
tf = tfpy.load_tf('input.json')
i, j, k, w = tf.get_quat_ijkw()
roll, pitch, yaw = tf.get_euler_rpy()
matrix = tf.get_rotation_matrix()
```

See the [README](tfpy/README.md) to learn to install the Python
package.


## The Rust Library

The Rust library is used for transform data serialization and
computation. It cooperates with `serde` and `nalgebra` crates.

```rust
use tf_format::Transform;

// Serialization using serde
let trans: Transform = serde_json::from_str(&std::fs::read_to_string("transform.json")?)?;
let yaml_text = serde_yaml::to_string(&trans)?;

let trans = trans.into_quaternion_format();               // Convert to quaternion format
let iso: nalgebra::Isometry3<f64> = trans.clone().into(); // Convert to nalgebra types
```

See the [README](tf-format/README.md) to learn more details.


## File Format

### Angle Unit

Here is a 3D rotation encoded in roll, pitch and yaw angles,
 respecting the "rpy" order in this JSON file. The "d" and "deg"
 suffix in the angle values denotes the degree unit.

```json
{
    "format": "euler",
    "order": "rpy",
    "angles": ["180d", "-5deg", "0d"]
}
```

Use "r" and "rad" for raidan unit.

```json
{
    "format": "euler",
    "order": "rpy",
    "angles": ["3.1415926r", "-5deg", "0rad"]
}
```

### Rotation Format

- Euler

```json
{
    "format": "euler",
    "order": "rpy",
    "angles": ["10d", "-5d", "3d"]
}
```

- Quaternion

```json
{
    "format": "quaternion",
    "ijkw": [0.0, 0.0, 0.0, 1.0]
}
```

- Axis-angle

```json
{
    "format": "axis-angle",
    "axis": [0.6, -0.8, 0.0],
    "angle": "45d"
}
```

- Rodrigues

```json
{
    "format": "rodrigues",
    "params": [1.5707963267948966, 0.0, 0.0]
}
```

- Rotation matrix

```json
{
    "format": "rotation-matrix",
    "matrix": [
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0]
    ]
}
```

### Rigid Transformation Format

The transformation is the combination of a rotation and a 3-value
translation, respectively written in "r" and "t" fields.

```json
{
    "r": {
        "format": "euler",
        "order": "ypr",
        "angles": ["-3.14r", "27d", "-30rad"]
    },
    "t": [1.0, -2.0, 0.3]
}
```

More examples can be found at [directory](tf-format/example_config).
