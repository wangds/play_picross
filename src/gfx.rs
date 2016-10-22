// resource.rs

use std::collections::HashMap;
use std::env;
use std::path::Path;
use sdl2::rect::Rect;
use sdl2::render::{Renderer,Texture};

#[cfg(feature = "flic")]
use flic;
#[cfg(feature = "flic")]
use sdl2::pixels::PixelFormatEnum;
#[cfg(feature = "flic")]
use sdl2::render::BlendMode;

#[cfg(feature = "png")]
use sdl2_image::LoadTexture;

#[derive(Clone,Copy)]
pub enum Font {
    Unsolved,
    Solved,
    Conflict
}

#[derive(Clone,Copy,Eq,Hash,PartialEq)]
pub enum Res {
    ToolbarPicross,
    ToolbarUndo,
    ToolbarRedo,
    ToolbarActiveEmpty,
    ToolbarInactiveEmpty,
    ToolbarActiveCrossedOut,
    ToolbarInactiveCrossedOut,
    ToolbarActiveFilled,
    ToolbarInactiveFilled,
    TileEmpty,
    TileFilled,
    TileCrossedOut,

    // Font(0..9)
    FontUnsolved(u8),
    FontSolved(u8),
    FontConflict(u8)
}

pub const FONT_WIDTH: u32 = 7;
pub const FONT_HEIGHT: u32 = 7;
pub const TILE_WIDTH: u32 = 15;
pub const TILE_HEIGHT: u32 = 15;
pub const TOOLBAR_BUTTON_HEIGHT: u32 = 9;
pub const TOOLBAR_BUTTON_WIDTH: u32 = 53;
pub const TOOLBAR_PAINT_WIDTH: u32 = 13;
pub const TOOLBAR_UNDO_REDO_WIDTH: u32 = 8;

pub struct GfxLib<'a> {
    pub renderer: Renderer<'a>,
    texture: Texture,
    lib: HashMap<Res, Rect>,
}

impl<'a> GfxLib<'a> {
    pub fn new(renderer: Renderer<'a>) -> GfxLib<'a> {
        let texture = match GfxLib::load_texture(&renderer) {
            None => panic!("Error loading picross.flc or picross.png"),
            Some(t) => t
        };

        let mut lib = HashMap::new();

        lib.insert(Res::ToolbarPicross,
                Rect::new( 0,  0, TOOLBAR_BUTTON_WIDTH, TOOLBAR_BUTTON_HEIGHT));
        lib.insert(Res::ToolbarUndo,
                Rect::new(54,  0, TOOLBAR_UNDO_REDO_WIDTH, TOOLBAR_BUTTON_HEIGHT));
        lib.insert(Res::ToolbarRedo,
                Rect::new(64,  0, TOOLBAR_UNDO_REDO_WIDTH, TOOLBAR_BUTTON_HEIGHT));

        lib.insert(Res::ToolbarActiveEmpty,
                Rect::new( 0, 30, TOOLBAR_PAINT_WIDTH, TOOLBAR_BUTTON_HEIGHT));
        lib.insert(Res::ToolbarInactiveEmpty,
                Rect::new( 0, 40, TOOLBAR_PAINT_WIDTH, TOOLBAR_BUTTON_HEIGHT));
        lib.insert(Res::ToolbarActiveCrossedOut,
                Rect::new(14, 30, TOOLBAR_PAINT_WIDTH, TOOLBAR_BUTTON_HEIGHT));
        lib.insert(Res::ToolbarInactiveCrossedOut,
                Rect::new(14, 40, TOOLBAR_PAINT_WIDTH, TOOLBAR_BUTTON_HEIGHT));
        lib.insert(Res::ToolbarActiveFilled,
                Rect::new(28, 30, TOOLBAR_PAINT_WIDTH, TOOLBAR_BUTTON_HEIGHT));
        lib.insert(Res::ToolbarInactiveFilled,
                Rect::new(28, 40, TOOLBAR_PAINT_WIDTH, TOOLBAR_BUTTON_HEIGHT));

        lib.insert(Res::TileEmpty,
                Rect::new( 0, 10, TILE_WIDTH, TILE_HEIGHT));
        lib.insert(Res::TileCrossedOut,
                Rect::new(20, 10, TILE_WIDTH, TILE_HEIGHT));
        lib.insert(Res::TileFilled,
                Rect::new(40, 10, TILE_WIDTH, TILE_HEIGHT));

        for i in 0..10 {
            let font_spacing = FONT_WIDTH + 1;
            let x = (font_spacing * i) as i32;

            lib.insert(Res::FontSolved(i as u8),
                    Rect::new(x, 50, FONT_WIDTH, FONT_HEIGHT));
            lib.insert(Res::FontUnsolved(i as u8),
                    Rect::new(x, 60, FONT_WIDTH, FONT_HEIGHT));
            lib.insert(Res::FontConflict(i as u8),
                    Rect::new(x, 70, FONT_WIDTH, FONT_HEIGHT));
        }

        GfxLib {
            renderer: renderer,
            texture: texture,
            lib: lib
        }
    }

    fn load_texture(renderer: &Renderer<'a>) -> Option<Texture> {
        if let Some(t) = try_load_flic(renderer) {
            return Some(t);
        }
        if let Some(t) = try_load_png(renderer) {
            return Some(t);
        }

        None
    }

    pub fn draw(&mut self, res: Res, dst: Rect) {
        if let Some(&src) = self.lib.get(&res) {
            let _ = self.renderer.copy(&self.texture, Some(src), Some(dst));
        }
    }

    pub fn text_centre(&mut self, font: Font, text: u32,
            scale: u32, xcentre: i32, y: i32) {
        let text_width = text_pixel_width(text, scale) as i32;
        self.text_right(font, text, scale, xcentre + text_width / 2, y);
    }

    pub fn text_right(&mut self, font: Font, text: u32,
            scale: u32, xright: i32, y: i32) {
        let font_spacing = (scale * (FONT_WIDTH - 1)) as i32;
        let mut x = xright - (scale * FONT_WIDTH) as i32;
        let mut n = text;

        // don't draw anything for 0 (empty lines)
        while n > 0 {
            let dst = Rect::new(x, y, scale * FONT_WIDTH, scale * FONT_HEIGHT);
            let digit = (n % 10) as u8;

            let res = match font {
                Font::Unsolved => Res::FontUnsolved(digit),
                Font::Solved => Res::FontSolved(digit),
                Font::Conflict => Res::FontConflict(digit)
            };

            if let Some(&src) = self.lib.get(&res) {
                let _ = self.renderer.copy(&self.texture, Some(src), Some(dst));
            }

            x = x - font_spacing;
            n = n / 10;
        }
    }
}

pub fn text_pixel_width(text: u32, scale: u32) -> u32 {
    let digits = count_digits(text);
    let font_spacing = FONT_WIDTH - 1;
    scale * (FONT_WIDTH + font_spacing * (digits - 1))
}

fn count_digits(text: u32) -> u32 {
    let mut digits = 1;
    let mut n = text;
    while n >= 10 {
        n = n / 10;
        digits = digits + 1;
    }

    digits
}

/*--------------------------------------------------------------*/

#[cfg(not(feature = "flic"))]
fn try_load_flic(_: &Renderer) -> Option<Texture> {
    None
}

#[cfg(feature = "flic")]
fn try_load_flic(renderer: &Renderer) -> Option<Texture> {
    let path = Path::new("resource/picross.flc");
    if let Some(t) = try_load_flic2(renderer, &path) {
        return Some(t);
    }

    if let Ok(mut path) = env::current_exe() {
        path.set_file_name("picross.flc");
        if let Some(t) = try_load_flic2(renderer, &path) {
            return Some(t);
        }
    }

    None
}

#[cfg(feature = "flic")]
fn try_load_flic2(renderer: &Renderer, path: &Path) -> Option<Texture> {
    if let Ok(mut f) = flic::FlicFile::open(path) {
        let w = f.width() as usize;
        let h = f.height() as usize;
        let mut buf = vec![0; w * h];
        let mut pal = [0; 3 * 256];

        let res = f.read_next_frame(
                &mut flic::RasterMut::new(w, h, &mut buf, &mut pal));
        if res.is_err() {
            return None;
        }

        let texture = renderer.create_texture_streaming(
                PixelFormatEnum::ABGR8888, w as u32, h as u32);
        if texture.is_err() {
            return None;
        }

        let mut t = texture.unwrap();
        render_to_texture(&mut t, w, h, &buf, &pal);
        t.set_blend_mode(BlendMode::Blend);
        return Some(t);
    }

    None
}

#[cfg(feature = "flic")]
fn render_to_texture(
        texture: &mut Texture,
        w: usize, h: usize, buf: &[u8], pal: &[u8]) {
    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        for y in 0..h {
            for x in 0..w {
                let offset = pitch * y + 4 * x;
                let c = buf[w * y + x] as usize;

                buffer[offset + 0] = pal[3 * c + 0];
                buffer[offset + 1] = pal[3 * c + 1];
                buffer[offset + 2] = pal[3 * c + 2];
                buffer[offset + 3] = if c == 0 { 0 } else { 255 };
            }
        }
    }).unwrap();
}

/*--------------------------------------------------------------*/

#[cfg(not(feature = "png"))]
fn try_load_png(_: &Renderer) -> Option<Texture> {
    None
}

#[cfg(feature = "png")]
fn try_load_png(renderer: &Renderer) -> Option<Texture> {
    let path = Path::new("resource/picross.png");
    if let Ok(t) = renderer.load_texture(&path) {
        return Some(t);
    }

    if let Ok(mut path) = env::current_exe() {
        path.set_file_name("picross.png");
        if let Ok(t) = renderer.load_texture(&path) {
            return Some(t);
        }
    }

    None
}
