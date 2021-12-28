## Epifaneia

CAD with SDFs, using JavaScript, WGSL and Rust.

This project aims to provide enough control over WGSL output using JavaScript
to facilitate code CAD. It should allow the user to squeeze every last bit
of juice out of their tooling if the need be, or stick to just simple code.

It is fully extensible and rendering style can be completely swapped out. These
are what I call "raycasters". See `raycasters/toon.js` to understand how simple
it is to do.

### Examples

Generate a foldable box of any size to place objects into.

![pic](./out.png)

To adjust the size, change the parameters in `main.curv`.
