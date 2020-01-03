
use piet::{
    Color, FontBuilder, RenderContext, Text, TextLayoutBuilder, TextLayout
};

extern crate quick_protobuf;
use quick_protobuf::{MessageRead, BytesReader};

mod vector_tile;
use vector_tile::Tile;
use crate::vector_tile::mod_Tile::GeomType;

pub mod label;
pub mod draw;

use std::borrow::Cow;
#[macro_use]

extern crate log;

fn tagmatch(layer:&vector_tile::mod_Tile::Layer,feature:&vector_tile::mod_Tile::Feature,key:&str,value:&str) -> bool {
    for x in (0..feature.tags.len()).step_by(2) {
        if layer.keys[feature.tags[x] as usize] == key {
            let val = layer.values[feature.tags[x+1] as usize].string_value.as_ref();
            if val.is_some() && val.unwrap() == value {
                return true;
            }
        }
    }
    return false;
}

fn taggetstr<'l,'f>(layer:&'l vector_tile::mod_Tile::Layer,feature:&'f vector_tile::mod_Tile::Feature,key:&str) -> Option<&'l Cow<'l,str>> {
    for x in (0..feature.tags.len()).step_by(2) {
        if layer.keys[feature.tags[x] as usize] == key {
            return layer.values[feature.tags[x+1] as usize].string_value.as_ref();
        }
    }
    return None;
}

fn taggetint<'l,'f>(layer:&'l vector_tile::mod_Tile::Layer,feature:&'f vector_tile::mod_Tile::Feature,key:&str) -> Option<i64> {
    for x in (0..feature.tags.len()).step_by(2) {
        if layer.keys[feature.tags[x] as usize] == key {
            return layer.values[feature.tags[x+1] as usize].int_value;
        }
    }
    return None;
}

pub fn small_size(zoom:u32) -> f64 {
    if zoom < 8 {
        return 24.0;
    }
    return 36.0;
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


pub fn render_tile(rc:&mut impl RenderContext, buf:&Vec<u8>, zoom:u32) {
    rc.clear(Color::rgba8(0xF2,0xEF,0xE9,0xFF));
    let park = rc.solid_brush(Color::rgba8(0xC8, 0xFA, 0xCC, 0xFF));
    let text = rc.solid_brush(Color::rgba8(0x44, 0x44, 0x44, 0xFF));
    let text_halo = rc.solid_brush(Color::rgba8(0xFF, 0xFF, 0xFF, 0xFF));
    let mid_gray = rc.solid_brush(Color::rgba8(0x55, 0x55, 0x55, 0xFF));

    let road_0 = rc.solid_brush(Color::rgba8(0xE8,0x92,0xA2,0xFF));
    let road_0_buf = rc.solid_brush(Color::rgba8(0xE4,0x6B,0x8D,0xFF));
    let water = rc.solid_brush(Color::rgba8(0xAA,0xD3,0xDF,0xFF));
    let buildings = rc.solid_brush(Color::rgba8(0xD9,0xD0,0xC9,0xFF));

    let mut reader = BytesReader::from_bytes(&buf);
    let tile = Tile::from_reader(&mut reader, &buf).expect("Cannot read Tile");

    // preprocess tile into a thing with hashmaps for lookup

    for layer in &tile.layers {
        if layer.name == "landuse" {
            for feature in &layer.features {
                if feature.type_pb != GeomType::POLYGON {
                    continue
                }
                rc.fill(draw::path(&feature.geometry,layer.extent), &park);
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
                rc.fill(draw::path(&feature.geometry,layer.extent), &water);
            }
        }
    }

    // draw stroked
    let mut rds = Vec::new();

    // get the road features in order
    for layer in &tile.layers {
        if layer.name != "roads" {
            continue;
        }
        for feature in &layer.features {
            if feature.type_pb != GeomType::LINESTRING {
                continue
            }
            let kind_val = taggetstr(layer,feature,"kind");
            let sort_rank = taggetint(layer,feature,"sort_rank");
            if kind_val.is_some() && kind_val.unwrap() == "ferry" {
                continue
            }

            if kind_val.is_some() && kind_val.unwrap() == "highway" {
                rds.push((&feature.geometry,sort_rank.unwrap()));
            } else {
                rc.stroke(draw::path(&feature.geometry,layer.extent), &mid_gray, 1.0);
            }
        }
    };

    rds.sort_by_key(|r| r.1);

    for rd in rds {
        let path = draw::path(&rd.0,8192);
        let size = highway_size(zoom);
        rc.stroke(&path, &road_0_buf, size.1);
        rc.stroke(&path, &road_0, size.0);
    }

    for layer in &tile.layers {
        if layer.name == "buildings" {
            for feature in &layer.features {
                if feature.type_pb != GeomType::POLYGON {
                    continue
                }
                rc.fill(draw::path(&feature.geometry,layer.extent), &buildings);
            }
        }
    }

    let mut collider = label::Collider{bboxes:Vec::new()};
    let font_size_big = 48.0;
    let font_big = rc.text().new_font_by_name("Helvetica", font_size_big).build().unwrap();

    let font_size_small = small_size(zoom);
    let font_small = rc.text().new_font_by_name("Helvetica", font_size_small).build().unwrap();

    for layer in &tile.layers {
        if layer.name != "places" {
            continue
        }
        for feature in &layer.features {
            let cursor_x = draw::de_zig_zag(layer.extent,feature.geometry[1]);
            let cursor_y = draw::de_zig_zag(layer.extent,feature.geometry[2]);

            let nam = taggetstr(layer,feature,"name");

            let kind_val = taggetstr(layer,feature,"kind");
            if nam.is_some() {
                if kind_val.is_some() && kind_val.unwrap() == "country" {
                    let layout = rc.text().new_text_layout(&font_big, &nam.unwrap()).build().unwrap();
                    if (cursor_y-font_size_big < 0.0) || (cursor_x - layout.width()/2.0 < 0.0) || (cursor_x + layout.width()/2.0 > 2048.0) || (cursor_y > 2048.0) {
                        continue;
                    }
                    if !collider.add((cursor_x-layout.width()/2.0,cursor_y-font_size_big),(cursor_x+layout.width()/2.0,cursor_y)) {
                        continue;
                    }
                    rc.stroke_text(&layout, (cursor_x-layout.width()/2.0,cursor_y), &text_halo,8.0);
                    rc.draw_text(&layout, (cursor_x-layout.width()/2.0,cursor_y), &text);
                } else if kind_val.is_some() && kind_val.unwrap() == "locality" {
                    let layout = rc.text().new_text_layout(&font_small, &nam.unwrap()).build().unwrap();
                    if (cursor_y-font_size_small < 0.0) || (cursor_x + layout.width()/2.0 > 2048.0) || (cursor_y > 2048.0) {
                        continue;
                    }
                    if !collider.add((cursor_x,cursor_y-font_size_small),(cursor_x+layout.width(),cursor_y)) {
                        continue;
                    }
                    rc.stroke_text(&layout, (cursor_x,cursor_y), &text_halo,8.0);
                    rc.draw_text(&layout, (cursor_x,cursor_y), &text);
                }
            }
        }
    } 
}