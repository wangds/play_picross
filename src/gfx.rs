// resource.rs

use std::collections::HashMap;
use std::env;
use std::path::Path;
use sdl2::rect::Rect;
use sdl2::render::Renderer;
use sdl2::render::Texture;
use sdl2_image::LoadTexture;

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
}

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
            None => panic!("Error loading sudoku.png"),
            Some(t) => t
        };

        let mut lib = HashMap::new();

        lib.insert(Res::ToolbarPicross,
                Rect::new_unwrap( 0,  0, TOOLBAR_BUTTON_WIDTH, TOOLBAR_BUTTON_HEIGHT));
        lib.insert(Res::ToolbarUndo,
                Rect::new_unwrap(54,  0, TOOLBAR_UNDO_REDO_WIDTH, TOOLBAR_BUTTON_HEIGHT));
        lib.insert(Res::ToolbarRedo,
                Rect::new_unwrap(64,  0, TOOLBAR_UNDO_REDO_WIDTH, TOOLBAR_BUTTON_HEIGHT));

        lib.insert(Res::ToolbarActiveEmpty,
                Rect::new_unwrap( 0, 30, TOOLBAR_PAINT_WIDTH, TOOLBAR_BUTTON_HEIGHT));
        lib.insert(Res::ToolbarInactiveEmpty,
                Rect::new_unwrap( 0, 40, TOOLBAR_PAINT_WIDTH, TOOLBAR_BUTTON_HEIGHT));
        lib.insert(Res::ToolbarActiveCrossedOut,
                Rect::new_unwrap(14, 30, TOOLBAR_PAINT_WIDTH, TOOLBAR_BUTTON_HEIGHT));
        lib.insert(Res::ToolbarInactiveCrossedOut,
                Rect::new_unwrap(14, 40, TOOLBAR_PAINT_WIDTH, TOOLBAR_BUTTON_HEIGHT));
        lib.insert(Res::ToolbarActiveFilled,
                Rect::new_unwrap(28, 30, TOOLBAR_PAINT_WIDTH, TOOLBAR_BUTTON_HEIGHT));
        lib.insert(Res::ToolbarInactiveFilled,
                Rect::new_unwrap(28, 40, TOOLBAR_PAINT_WIDTH, TOOLBAR_BUTTON_HEIGHT));

        lib.insert(Res::TileEmpty,
                Rect::new_unwrap( 0, 10, TILE_WIDTH, TILE_HEIGHT));
        lib.insert(Res::TileCrossedOut,
                Rect::new_unwrap(20, 10, TILE_WIDTH, TILE_HEIGHT));
        lib.insert(Res::TileFilled,
                Rect::new_unwrap(40, 10, TILE_WIDTH, TILE_HEIGHT));

        GfxLib {
            renderer: renderer,
            texture: texture,
            lib: lib
        }
    }

    fn load_texture(renderer: &Renderer<'a>) -> Option<Texture> {
        let bmp = Path::new("resource/picross.png");
        if let Ok(t) = renderer.load_texture(bmp) {
            return Some(t);
        }

        match env::current_exe() {
            Err(e) => println!("{}", e),

            Ok(mut exe_path) => {
                exe_path.set_file_name("picross.png");
                match renderer.load_texture(exe_path.as_path()) {
                    Err(e) => println!("{}", e),
                    Ok(t) => return Some(t)
                }
            }
        }

        None
    }

    pub fn draw(&mut self, res: Res, dst: Rect) {
        if let Some(&src) = self.lib.get(&res) {
            self.renderer.copy(&self.texture, Some(src), Some(dst));
        }
    }
}
