all:
	wasm-pack build --no-typescript --target web -d demo-leaflet
	./combine_js > demo-leaflet/temp.js
	mv demo-leaflet/temp.js demo-leaflet/protomaps_rs.js
