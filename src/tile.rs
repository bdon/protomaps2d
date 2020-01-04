use std::borrow::Cow;
use crate::vector_tile::vector_tile::mod_Tile::{Layer,Feature};

fn tagmatch(layer:&Layer,feature:&Feature,key:&str,value:&str) -> bool {
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

pub fn taggetstr<'l,'f>(layer:&'l Layer,feature:&'f Feature,key:&str) -> Option<&'l Cow<'l,str>> {
    for x in (0..feature.tags.len()).step_by(2) {
        if layer.keys[feature.tags[x] as usize] == key {
            return layer.values[feature.tags[x+1] as usize].string_value.as_ref();
        }
    }
    return None;
}

pub fn taggetint<'l,'f>(layer:&'l Layer,feature:&'f Feature,key:&str) -> Option<i64> {
    for x in (0..feature.tags.len()).step_by(2) {
        if layer.keys[feature.tags[x] as usize] == key {
            return layer.values[feature.tags[x+1] as usize].int_value;
        }
    }
    return None;
}

