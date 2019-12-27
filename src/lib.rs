use piet::kurbo::{ BezPath};

use piet::{
    Color, FontBuilder, RenderContext, Text, TextLayoutBuilder, TextLayout
};

extern crate quick_protobuf;
use quick_protobuf::{MessageRead, BytesReader};

mod vector_tile;
use vector_tile::Tile;
use crate::vector_tile::mod_Tile::GeomType;

use std::borrow::Cow;
#[macro_use]

extern crate log;

fn de_zig_zag(param_e: u32, param_u: u32) -> f64 {
    let param = param_u as i32;
    let extent = (param_e / 2048) as f64;
    return ((param >> 1) ^ (-1 * (param & 1))) as f64 / extent;
}

// a very primitive label collider,
// that does a linear search over drawn labels,
// rejecting any that intersect.
pub struct Collider {
    pub bboxes:Vec<((f64, f64),(f64, f64))>
}

impl Collider {
    pub fn add(&mut self, topleft: (f64, f64), bottomright: (f64, f64)) -> bool {
        for bbox in &self.bboxes {
            // x axis
            if bottomright.0 < (bbox.0).0 {
                continue
            }
            if topleft.0 > (bbox.1).0 {
                continue
            }

            // y axis
            if bottomright.1 < (bbox.0).1 {
                continue
            }
            if topleft.1 > (bbox.1).1 {
                continue
            }

            return false;
        } 
        self.bboxes.push((topleft,bottomright));
        return true;
    }
}

fn geom_to_path(geometry:&Vec<u32>, extent:u32, path:&mut BezPath) {
    let cmd_len = geometry.len();
    let mut pos = 0;
    let mut cursor_x = 0.0;
    let mut cursor_y = 0.0;
    while pos < cmd_len {

        let cmd_integer = geometry[pos];
        let id = cmd_integer & 0x7;
        let count = cmd_integer >> 3;

        if id == 1 {
            // MoveTo
            for _c in 0..count {
                pos+=1;
                let x = de_zig_zag(extent,geometry[pos]);
                pos+=1;
                let y = de_zig_zag(extent,geometry[pos]);
                cursor_x += x;
                cursor_y += y;
                path.move_to((cursor_x,cursor_y));
            }
        } else if id == 2 {
            // LineTo
            for _c in 0..count {
                pos+=1;
                let x = de_zig_zag(extent,geometry[pos]);
                pos+=1;
                let y = de_zig_zag(extent,geometry[pos]);
                cursor_x += x;
                cursor_y += y;
                path.line_to((cursor_x,cursor_y));
            }

        } else {
            // ClosePath
            path.close_path();
        }
        pos+=1;
    }
}

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
        14 => return (10.0,20.0),
        13 => return (8.0,16.0),
        12 => return (7.0,12.0),
        9..=11 => return (5.0,10.0),
        6..=8 => return (4.0,8.0),
        _ => return (1.0,0.0)
    }
}


pub fn render_tile(rc:&mut impl RenderContext, buf:&Vec<u8>, zoom:u32) {
    rc.clear(Color::BLACK);
    let white = rc.solid_brush(Color::rgba8(0xFF, 0xFF, 0xFF, 0xFF));
    let black = rc.solid_brush(Color::rgba8(0x00, 0x00, 0x00, 0xFF));
    let dark_gray = rc.solid_brush(Color::rgba8(0x11, 0x11, 0x11, 0xFF));
    let mid_gray = rc.solid_brush(Color::rgba8(0x55, 0x55, 0x55, 0xFF));

    let mut reader = BytesReader::from_bytes(&buf);
    let tile = Tile::from_reader(&mut reader, &buf).expect("Cannot read Tile");

    // preprocess tile into a thing with hashmaps for lookup

    // draw water polygons
    for layer in &tile.layers {
        if layer.name == "water" {
            for feature in &layer.features {
                if feature.type_pb != GeomType::POLYGON {
                    continue
                }
                let mut path = BezPath::new();
                geom_to_path(&feature.geometry,layer.extent, &mut path);
                rc.fill(path, &dark_gray);
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
            if kind_val.is_some() && kind_val.unwrap() == "highway" {
                rds.push((&feature.geometry,sort_rank.unwrap()));
            } else {
                let mut path = BezPath::new();
                geom_to_path(&feature.geometry,layer.extent,&mut path);
                rc.stroke(&path, &mid_gray, 1.0);
            }
        }
    };

    rds.sort_by_key(|r| r.1);

    for rd in rds {
        let mut path = BezPath::new();
        geom_to_path(&rd.0,8192,&mut path);
        let size = highway_size(zoom);
        rc.stroke(&path, &black, size.1);
        rc.stroke(&path, &mid_gray, size.0);
    }

    // draw buildings
    for layer in &tile.layers {
        if layer.name == "buildings" {
            for feature in &layer.features {
                if feature.type_pb != GeomType::POLYGON {
                    continue
                }
                let mut path = BezPath::new();
                geom_to_path(&feature.geometry,layer.extent,&mut path);
                rc.fill(path, &mid_gray);
            }
        }
    }

    let mut collider = Collider{bboxes:Vec::new()};
    let font_size_big = 48.0;
    let font_big = rc.text().new_font_by_name("Helvetica", font_size_big).build().unwrap();

    let font_size_small = small_size(zoom);
    let font_small = rc.text().new_font_by_name("Helvetica", font_size_small).build().unwrap();

    for layer in &tile.layers {
        if layer.name != "places" {
            continue
        }
        for feature in &layer.features {
            let cursor_x = de_zig_zag(layer.extent,feature.geometry[1]);
            let cursor_y = de_zig_zag(layer.extent,feature.geometry[2]);

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
                    rc.draw_text(&layout, (cursor_x-layout.width()/2.0,cursor_y), &white);
                } else if kind_val.is_some() && kind_val.unwrap() == "locality" {
                    let layout = rc.text().new_text_layout(&font_small, &nam.unwrap()).build().unwrap();
                    if (cursor_y-font_size_small < 0.0) || (cursor_x + layout.width()/2.0 > 2048.0) || (cursor_y > 2048.0) {
                        continue;
                    }
                    if !collider.add((cursor_x,cursor_y-font_size_small),(cursor_x+layout.width(),cursor_y)) {
                        continue;
                    }
                    rc.draw_text(&layout, (cursor_x,cursor_y), &white);
                }


            }
        }
    } 
}