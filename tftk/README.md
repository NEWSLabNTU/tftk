# The **tftk** Command Line Tool

Install the **tftk** command using Cargo.

```sh
cargo install --git https://github.com/NEWSLabNTU/tftk.git tftk
```

To convert a transform file to Euler format,

```sh
tftk convert -i input.json -o output.json -r euler
```

To convert a transform file to axis-angle format and print the result
on the terminal in YAML,

```sh
tftk convert -i input.json -t yaml -r axis-angle
```

To compute the products of multiple transform files and encode the
result in quaternion,

```sj
tftk compose r1.json r2.json r3.json -i output.json -r quat
```
