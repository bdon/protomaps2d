use piet::kurbo::{ BezPath};

use piet::{
    Color, FontBuilder, RenderContext, Text, TextLayoutBuilder, TextLayout
};

extern crate quick_protobuf;
use quick_protobuf::{MessageRead, BytesReader};

mod vector_tile;
use vector_tile::Tile;
use crate::vector_tile::mod_Tile::GeomType;

fn de_zig_zag(param_u: u32) -> f64 {
    let param = param_u as i32;
    return ((param >> 1) ^ (-1 * (param & 1))) as f64 / 4.0
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

pub fn render_tile(rc:&mut impl RenderContext, buf:&Vec<u8>) {
    rc.clear(Color::BLACK);
    let black = rc.solid_brush(Color::rgba8(0x00, 0x00, 0x00, 0xFF));
    let white = rc.solid_brush(Color::rgba8(0xFF, 0xFF, 0xFF, 0xFF));
    let dark_gray = rc.solid_brush(Color::rgba8(0x11, 0x11, 0x11, 0xFF));
    let near_white = rc.solid_brush(Color::rgba8(0x77, 0x77, 0x77, 0xFF));

    let mut reader = BytesReader::from_bytes(&buf);
    let tile = Tile::from_reader(&mut reader, &buf).expect("Cannot read Tile");

    for layer in &tile.layers {
        if layer.name == "places"  || layer.name == "earth" {
            continue
        }
        for feature in &layer.features {
            if feature.type_pb == GeomType::POINT {
                continue
            }
            let cmd_len = feature.geometry.len();
            let mut pos = 0;
            let mut cursor_x = 0.0;
            let mut cursor_y = 0.0;
            let mut path = BezPath::new();
            while pos < cmd_len {

                let cmd_integer = feature.geometry[pos];
                let id = cmd_integer & 0x7;
                let count = cmd_integer >> 3;

                if id == 1 {
                    // MoveTo
                    for _c in 0..count {
                        pos+=1;
                        let x = de_zig_zag(feature.geometry[pos]);
                        pos+=1;
                        let y = de_zig_zag(feature.geometry[pos]);
                        cursor_x += x;
                        cursor_y += y;
                        path.move_to((cursor_x,cursor_y));
                    }
                } else if id == 2 {
                    // LineTo
                    for _c in 0..count {
                        pos+=1;
                        let x = de_zig_zag(feature.geometry[pos]);
                        pos+=1;
                        let y = de_zig_zag(feature.geometry[pos]);
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

            if feature.type_pb == GeomType::POLYGON {
                rc.fill(path, &dark_gray);
            } else {
                rc.stroke(path, &near_white, 1.0);
            }
        }
    };

    let mut collider = Collider{bboxes:Vec::new()};
    let font_size = 32.0;
    let font = rc.text().new_font_by_name("Helvetica", font_size).build().unwrap();
    for layer in &tile.layers {
        if layer.name != "places" {
            continue
        }
        for feature in &layer.features {
            let cursor_x = de_zig_zag(feature.geometry[1]);
            let cursor_y = de_zig_zag(feature.geometry[2]);

            for x in (0..feature.tags.len()).step_by(2) {
                if layer.keys[feature.tags[x] as usize] == "name" {
                    let name = layer.values[feature.tags[x+1] as usize].string_value.as_ref().unwrap();
                    let layout = rc.text().new_text_layout(&font, &name).build().unwrap();
                    if collider.add((cursor_x,cursor_y-font_size),(cursor_x+layout.width(),cursor_y)) {
                        rc.stroke_text(&layout, (cursor_x,cursor_y), &black, 8.0);
                        rc.draw_text(&layout, (cursor_x,cursor_y), &white);
                    }
                }
            }
        }
    } 
}