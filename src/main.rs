use std::env;
use std::fs;
use std::fs::File;
use protomaps2d::{render_tile,Style};

use piet::RenderContext;
use cairo::{Context, Format, ImageSurface};
use piet_cairo::CairoRenderContext;

fn main() {
    let args: Vec<String> = env::args().collect();
    let vector_tile = &args[1];
    let output_file = &args[2];
    let zoom = &args[3];
    println!("Input {}", vector_tile);

    let surface = ImageSurface::create(Format::ARgb32, 1024, 1024)
        .expect("Can't create surface");
    let mut cr = Context::new(&surface);
    cr.scale(0.5, 0.5);
    let mut piet_context = CairoRenderContext::new(&mut cr);


    let bytes = fs::read(vector_tile);
    let s = Style{bg_color:"#ffffff".to_string(),font:"Helvetica".to_string(),labels:true,text_scale:1.0,name:"name".to_string()};

    fn logger(s: &String) {
        println!("{}", s);
    }
    render_tile(&mut piet_context,&bytes.unwrap(),zoom.parse::<u32>().unwrap(),1,0,0,&s,&logger);

    piet_context.finish().unwrap();
    surface.flush();
    let mut file = File::create(output_file).expect("Couldn't create 'file.png'");
    surface
        .write_to_png(&mut file)
        .expect("Error writing image file");
}