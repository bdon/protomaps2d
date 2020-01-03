use piet::kurbo::{ BezPath };

pub fn de_zig_zag(param_e: u32, param_u: u32) -> f64 {
    let param = param_u as i32;
    let extent = (param_e / 2048) as f64;
    return ((param >> 1) ^ (-1 * (param & 1))) as f64 / extent;
}

pub fn path(geometry:&Vec<u32>, extent:u32) -> BezPath {
    let mut path = BezPath::new();
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
    return path;
}