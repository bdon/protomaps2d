# α

A map renderer for the server and WebAssembly.

Try it out at [protomaps.com/experiments/alpha](https://protomaps.com/experiments/alpha)

<img src="examples/5_9_11.png" height="512" width="512" > 

Map data © OpenStreetMap, [Nextzen Tiles](http://developers.nextzen.org)

## How to use
This is an experiment. You almost certainly don't want to use it right now.

Cairo backend:
`cargo run INPUT.mvt OUTPUT.png`

WebAssembly:
`cd alpha-web && wasm-pack build --no-typescript --target web`
