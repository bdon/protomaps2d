import init, { wasm_render_tile } from './protomaps_alpha_web.js'

const ratio = window.devicePixelRatio
const tile_size = 2048/ratio
const crs = L.extend({}, L.CRS.EPSG3857, {
    scale:function (zoom) {
    return tile_size * Math.pow(2,zoom)
    },
    zoom :function(scale) {
        return Math.log(scale / tile_size) / Math.LN2
    }
})

const RSinit = async font_faces => {
    await init()
    for (var f of font_faces) {
        const font = new FontFace(f[0],`url(${f[1]})`)
        await font.load()
        document.fonts.add(font)
    }
}

const RSLayer = L.GridLayer.extend({
    initialize: function(tile_url,options) {
        options.tileSize = tile_size
        L.setOptions(this, options);
        this.style = options.style
        this.tile_url = tile_url
        this.renderedTiles = 0
    },

    rerender: function() {
        for (let [k,v] of Object.entries(this._tiles)) {
            const coords = v.coords
            var result = wasm_render_tile(v.el.id,v.el.arr,coords.z,this.style)
        }
    },

    createTile: function(coords,done){
        var error
        var tile = L.DomUtil.create('canvas', 'leaflet-tile')
        tile.deleted = false
        tile.id = "pmap_" + this.renderedTiles
        this.renderedTiles = this.renderedTiles + 1
        tile.width = 2048
        tile.height = 2048

        var tile_url = this.tile_url.replace("{z}",coords.z).replace("{x}",coords.x).replace("{y}",coords.y)
        tile.url = tile_url

        setTimeout(() => {
            if (tile.deleted) return
            fetch(tile_url).then(resp => {
                return resp.arrayBuffer()
            }).then(buf => {
                if (tile.deleted) return
                var arr = new Uint8Array(buf)
                tile.arr = arr
                var result = wasm_render_tile(tile.id,arr,coords.z,this.style)
                done(error,tile)
            })
        },200)
        return tile
    },

    _removeTile: function (key) {
        var tile = this._tiles[key]
        if (!tile) { return }
        tile.el.width = 0
        tile.el.height = 0
        tile.el.deleted = true
        L.DomUtil.remove(tile.el)
        delete this._tiles[key]
        this.fire('tileunload', {
            tile: tile.el,
            coords: this._keyToTileCoords(key)
        })
    },
})

export { crs, RSLayer, RSinit };
