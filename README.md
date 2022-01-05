# Epifaneia

Performant code CAD with SDFs.

This project aims to provide enough control over WGSL output using JavaScript
to facilitate code CAD (Don't know what code CAD is? Check out
https://cadhub.xyz). It should allow the user to squeeze every last bit of
juice out of their tooling if the need be, or stick to just simple code.

It is fully extensible and rendering style can be completely swapped out. These
are what I call "raycasters". See `raycasters/toon.js` to understand how simple
it is to do.

Epifaneia uses progressive upscaling to stay fast. This will result in the first
few moments of each camera movement to be low resolution but will reach full
resolution eventually.

A collection of libraries can be found in `lib/` for things like sketching
the outline of a face, which is common in CAD software.

## Working

Create a copy of `template.wgsl.js` and modify to your needs. For each new
project, you should copy this file, as each project will have different needs.
You can remove or add the needed functions / shapes, resulting in the most
optimal WGSL outputs.

I suggest saving files with the naming scheme "my-thing.wgsl.js". This conveys
very clearly what the file contains.

Once happy, you can save the output for rendering:

```
node my-thing.wgsl.js > my-thing.sdf.json
```

and run Epifaneia:

```
epifaneia my-thing.sdf.json
```

Any changes to sdf.json and epifaneia will re-load itself.

Epifaneia will read the JSON structure, which contains the SDF code and
corresponding buffer data (for things like points for `polygon`), and re-compile
the shader and setup the graphics pipeline.

## Exporting

Epifaneia only supports exporting as SVX, a voxel format created by Alan Hudson.

SVX is supported by Shapeways for 3D printing.

Currently there is work to bring SVX support to slicers such as Cura.

SVX can be converted into STL using AbFab3D.

To export run `./tools/svx.sh my-thing.sdf.json my-thing.svx`.

### Examples

Generate a foldable box of any size to place objects into.

![pic](./out.png)

To adjust the size, change the parameters in `boxoconstructus.wgsl.js`.
