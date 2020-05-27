use piet::{
    Color, FontBuilder, FontStyle, RenderContext, Text, TextLayoutBuilder, TextLayout
};
use piet::kurbo::{ Affine,Vec2 };

extern crate quick_protobuf;
use quick_protobuf::{MessageRead, BytesReader};

mod vector_tile;
use vector_tile::vector_tile::Tile;
use crate::vector_tile::vector_tile::mod_Tile::{GeomType};

pub mod label;
pub mod draw;
pub mod tile;

extern crate log;

#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
pub struct Style {
    pub labels: bool,
    pub name: String,
    pub font: String,
    pub text_scale: f64
}

#[derive(Serialize)]
pub struct Result {
    pub feature_count: u64
}

pub fn highway_size(zoom:u32) -> (f64,f64) {
    match zoom {
        15 => return (16.0,30.0),
        14 => return (10.0,20.0),
        13 => return (8.0,16.0),
        12 => return (7.0,12.0),
        9..=11 => return (5.0,10.0),
        6..=8 => return (4.0,8.0),
        _ => return (1.0,0.0)
    }
}


pub fn render_tile(rc:&mut impl RenderContext, buf:&[u8], zoom:u32,total:u32,dx:u32,dy:u32,style:&Style,_logger:&dyn Fn(&String)) -> Result {
    rc.clear(Color::rgba8(0xF6,0xE7,0xD4,0xFF));
    rc.save();
    rc.transform(Affine::translate(Vec2{x:-2048.0*(dx as f64),y:-2048.0*(dy as f64)}));
    let text = rc.solid_brush(Color::rgba8(0x44, 0x44, 0x44, 0xFF));
    let text_halo = rc.solid_brush(Color::rgba8(0xFF, 0xFF, 0xFF, 0xFF));
    let mid_gray = rc.solid_brush(Color::rgba8(0x55, 0x55, 0x55, 0xFF));


    let road_0 = rc.solid_brush(Color::rgba8(0xFF,0xFF,0xFF,0xFF));
    //let road_0_buf = rc.solid_brush(Color::rgba8(0xE4,0x6B,0x8D,0xFF));

    let park = rc.solid_brush(Color::rgba8(0xBB, 0xDB, 0xC4, 0xFF));
    let water = rc.solid_brush(Color::rgba8(0xA5,0xBC,0xCB,0xFF));
    let buildings = rc.solid_brush(Color::rgba8(0xCC,0xCC,0xCC,0xFF));

    let mut reader = BytesReader::from_bytes(&buf);
    let tile = Tile::from_reader(&mut reader, &buf).expect("Cannot read Tile");

    // preprocess tile into a thing with hashmaps for lookup

    let x = draw::Xform{extent:4096.0,total:total,dx:dx,dy:dy};

    for layer in &tile.layers {
        if layer.name == "landuse" {
            for feature in &layer.features {
                if feature.type_pb != GeomType::POLYGON {
                    continue
                }
                rc.fill(draw::path(&feature.geometry,&x), &park);
            }
        }
    }
    for layer in &tile.layers {
        if layer.name == "natural" {
            for feature in &layer.features {
                if feature.type_pb != GeomType::POLYGON {
                    continue
                }
                rc.fill(draw::path(&feature.geometry,&x), &park);
            }
        }
    }

    // draw water polygons
    for layer in &tile.layers {
        if layer.name == "water" {
            for feature in &layer.features {
                if feature.type_pb != GeomType::POLYGON {
                    continue
                }
                rc.fill(draw::path(&feature.geometry,&x), &water);
            }
        }
    }


    // get the road features in order
    for layer in &tile.layers {
        if layer.name != "roads" {
            continue;
        }
        for feature in &layer.features {
            if feature.type_pb != GeomType::LINESTRING {
                continue
            }

            rc.stroke(draw::path(&feature.geometry,&x), &road_0, 1.0);
        }
    };

    for layer in &tile.layers {
        if layer.name == "buildings" {
            for feature in &layer.features {
                if feature.type_pb != GeomType::POLYGON {
                    continue
                }
                rc.fill(draw::path(&feature.geometry,&x), &buildings);
            }
        }
    }

    for layer in &tile.layers {
        if layer.name == "admin" {
            for feature in &layer.features {
                if feature.type_pb != GeomType::LINESTRING {
                    continue
                }
                rc.stroke(draw::path(&feature.geometry,&x), &mid_gray, 1.0);
            }
        }
    }

    let mut collider = label::Collider{bboxes:Vec::new()};
    let font_size_big = 18.0 * style.text_scale;
    let font_big = rc.text().new_font_by_name(&style.font, font_size_big,100, FontStyle::Normal).build().unwrap();

    let font_size_small = 12.0 * style.text_scale;
    let font_small = rc.text().new_font_by_name(&style.font, font_size_small,400, FontStyle::Normal).build().unwrap();

    let poi_font = rc.text().new_font_by_name(&style.font, 12.0 * style.text_scale,400, FontStyle::Normal).build().unwrap();

    if style.labels == true {

        for layer in &tile.layers {
            if layer.name != "places" {
                continue
            }
            for feature in &layer.features {
                let cursor_x = draw::de_zig_zag(&x,feature.geometry[1]);
                let cursor_y = draw::de_zig_zag(&x,feature.geometry[2]);
                let logical_x = cursor_x - 2048.0 * dx as f64;
                let logical_y = cursor_y - 2048.0 * dy as f64;

                let nam = tile::taggetstr(layer,feature,&style.name);

                let kind_val = tile::taggetstr(layer,feature,"place");
                if nam.is_some() {
                    if kind_val.is_some() && kind_val.unwrap() == "country" {
                        let layout = rc.text().new_text_layout(&font_big, &nam.unwrap(),None).build().unwrap();
                        if (logical_y-font_size_big < 0.0) || (logical_x - layout.width()/2.0 < 0.0) || (logical_x + layout.width()/2.0 > 2048.0) || (logical_y > 2048.0) {
                            continue;
                        }
                        if !collider.add((cursor_x-layout.width()/2.0,cursor_y-font_size_big),(cursor_x+layout.width()/2.0,cursor_y)) {
                            continue;
                        }
                        rc.stroke_text(&layout, (cursor_x-layout.width()/2.0,cursor_y), &text_halo,6.0);
                        rc.draw_text(&layout, (cursor_x-layout.width()/2.0,cursor_y), &text);
                    } else if kind_val.is_some() && kind_val.unwrap() == "city" {
                        let layout = rc.text().new_text_layout(&font_small, &nam.unwrap(),None).build().unwrap();
                        if (logical_y-font_size_small < 0.0) || (logical_x + layout.width() > 2048.0) || (logical_y > 2048.0) {
                            continue;
                        }
                        if !collider.add((cursor_x,cursor_y-font_size_small),(cursor_x+layout.width(),cursor_y)) {
                            continue;
                        }
                        rc.stroke_text(&layout, (cursor_x,cursor_y), &text_halo,6.0);
                        rc.draw_text(&layout, (cursor_x,cursor_y), &text);
                    } else {
                        let layout = rc.text().new_text_layout(&font_small, &nam.unwrap(),None).build().unwrap();
                        if (logical_y-font_size_small < 0.0) || (logical_x + layout.width() > 2048.0) || (logical_y > 2048.0) {
                            continue;
                        }
                        if !collider.add((cursor_x,cursor_y-font_size_small),(cursor_x+layout.width(),cursor_y)) {
                            continue;
                        }
                        rc.stroke_text(&layout, (cursor_x,cursor_y), &text_halo,    6.0);
                        rc.draw_text(&layout, (cursor_x,cursor_y), &text);
                    }
                }
            }
        } 

        for layer in &tile.layers {
            if layer.name != "poi" {
                continue
            }
            for feature in &layer.features {
                let cursor_x = draw::de_zig_zag(&x,feature.geometry[1]);
                let cursor_y = draw::de_zig_zag(&x,feature.geometry[2]);
                let nam = tile::taggetstr(layer,feature,"name");

                let logical_x = cursor_x - 2048.0 * dx as f64;
                let logical_y = cursor_y - 2048.0 * dy as f64;

                if nam.is_some() {
                    let layout = rc.text().new_text_layout(&poi_font, &nam.unwrap(),None).build().unwrap();
                    if (logical_x-font_size_big < 0.0) || (logical_x - layout.width()/2.0 < 0.0) || (logical_x + layout.width()/2.0 > 2048.0) || (logical_y > 2048.0) {
                        continue;
                    }
                    if !collider.add((cursor_x-layout.width()/2.0,cursor_y-font_size_big),(cursor_x+layout.width()/2.0,cursor_y)) {
                        continue;
                    }
                    rc.stroke_text(&layout, (cursor_x-layout.width()/2.0,cursor_y), &text_halo,4.0);
                    rc.draw_text(&layout, (cursor_x-layout.width()/2.0,cursor_y), &text);
                }
            }
        } 
    }

    rc.restore();

    let result = Result{feature_count:1};
    return result;
}