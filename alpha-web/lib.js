import init, { wasm_render_tile } from './protomaps_rs.js'

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
    font_faces = font_faces || [["Inter","https://cdn.protomaps.com/fonts/woff2/Inter.var.woff2"]]
    for (var f of font_faces) {
        const font = new FontFace(f[0],`url(${f[1]})`)
        await font.load()
        document.fonts.add(font)
    }
}

const RSLayer = L.GridLayer.extend({
    initialize: function(tile_url,options) {
        options = options || {}
        options.attribution = options.attribution || '<a href="https://protomaps.com">Protomaps</a> &copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'
        options.style = options.style || {labels:true,name:"name",font:"Inter"} 
        options.maxZoom = options.maxZoom || 19
        options.tileSize = tile_size
        L.setOptions(this, options);
        this.style = options.style
        this.tile_url = tile_url
        this.renderedTiles = 0
    },

    setStyle: function(o) {
        Object.assign(this.style,o)
        this.rerender()
    },

    rerender: function() {
        for (let [k,v] of Object.entries(this._tiles)) {
            const coords = v.coords
            const tile = v.el
            var result = wasm_render_tile(tile.id,tile.arr,coords.z,tile.total,tile.dx,tile.dy,this.style)
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

        tile.total = 1
        tile.dx = 0
        tile.dy = 0
        if (coords.z > 14) {
            const delta_z = coords.z - 14
            const z14_x = Math.floor(coords.x / Math.pow(2,delta_z))
            const z14_y = Math.floor(coords.y / Math.pow(2,delta_z))
            tile.total = Math.pow(2,delta_z)
            tile.dx = coords.x - z14_x * tile.total
            tile.dy = coords.y - z14_y * tile.total
            coords.z = 14
            coords.x = z14_x
            coords.y = z14_y
        }

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
                var result = wasm_render_tile(tile.id,arr,coords.z,tile.total,tile.dx,tile.dy,this.style)
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
