# The **tfpy** Python Library

## Installation

```sh
# Clone this project
git clone https://github.com/NEWSLabNTU/tftk.git

# Compile the Python module
cd tftk/tfpy
maturin build

# Install the wheel file.
# The file name depends on your system.
pip install ../target/wheels/tfpy-0.1.0-cp310-cp310-manylinux_2_34_x86_64.whl
```

## Example

Use `load_tf()` and `dump_tf()` for file loading and writing.

The `.to_euler_form()` changes the rotation format. Similar methods
include `.to_quat_form()` and `.to_axis_angle_form()`.

```python
import tfpy

tf = tfpy.load_tf('input.json')
tf = tf.to_euler_form()
tfpy.dump_tf(tf, 'output.json')
```

To obtain the rotation parameters in various forms,

```python
axis, angle = tf.get_axis_angle()
i, j, k, w = tf.get_quat_ijkw()
r1, r2, r3 = tf.get_rodrigues()
roll, pitch, yaw = tf.get_euler_rpy()
matrix = tf.get_rotation_matrix()
```

The rotation and translation components can be accessed. the
translation field `t` could be None if the input file does not contain
the translation values.

```python
rotation = tf.r
translation tf.t

if translation is None:
    print("no translation")
```
