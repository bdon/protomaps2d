# protomaps2d

Renders vector geometry tiles to 2D images.

## Motivation

Zoomable maps have traditionally been served as 256x256 image tiles. These static image files have numerous drawbacks:

* Multilingual labels, colors and display of features are difficult to customize in the client.
* Images are designed for a single pixel density, but devices in 2020 have pixel ratios anywhere from 2.0 to 3.0.
* Preparing maps for high zoom levels can demand rendering and storing billions of tiles. 

**Vector map tiles** solve these issues by encoding only geometry and thematic data and enabling the client to render maps in a specified style. Most popular renderers use a graphics API such as WebGL for performance reasons, but programming for these APIs is inherently complex, especially for displaying multilingual text.

**protomaps2d** is a minimalistic library that renders vector tiles to 2D images using simple libraries such as Cairo or Canvas on the web. 

For the web in particular, protomaps2d is integrated with the popular Leaflet library as well as the font loading and complex text layout features standard across web browsers. 

## How to use

**protomaps2d** is in a usable state but missing many important features. It's being developed in concert with the Protomaps web map API. Documentation on how to use protomaps2d with other data sources is forthcoming.

Cairo backend:
`cargo run INPUT.pbf OUTPUT.png ZOOM_LEVEL`
