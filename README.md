# Epifaneia

Performant code CAD with SDFs.

This project aims to provide enough control over WGSL output using JavaScript
to facilitate code CAD (Don't know what code CAD is? Check out
https://cadhub.xyz). It should allow the user to squeeze every last bit of
juice out of their tooling if the need be, or stick to just simple code.

It is fully extensible and rendering style can be completely swapped out. These
are what I call "raycasters". See `raycasters/toon.js` to understand how simple
it is to do.

## Usage

Create a copy of `template.wgsl.js` and modify to your needs. For each new
project, you should copy this file, as each project will have different needs.
You can remove or add the needed functions / shapes, resulting in the most
optimal WGSL outputs.

I suggest saving files with the naming scheme "my-thing.wgsl.js". This conveys
very clearly what the file contains.

Once happy, you can save the output for rendering later:

```
node template.wgsl.js > sdf.json
```

or you can reload the current instance of Epifaneia:

```
node wgsl.js > /tmp/epifaneia/json
# Alternatively "cat sdf.json > /tmp/epifaneia/json"
```

Epifaneia will read the JSON structure, which contains the SDF code and
corresponding buffer data (for things like points for `polygon`), and re-compile
the shader and setup the graphics pipeline.

### Examples

Generate a foldable box of any size to place objects into.

![pic](./out.png)

To adjust the size, change the parameters in `boxoconstructus.wgsl.js`.
