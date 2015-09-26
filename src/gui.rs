// gui.rs

use std::cmp::{max,min};
use sdl2;
use sdl2::EventPump;
use sdl2::TimerSubsystem;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::Mouse;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2_image;
use sdl2_image::INIT_PNG;

use action::PicrossAction;
use board::Board;
use board::Tile;
use gfx::*;

const DEFAULT_SCREEN_WIDTH: u32 = 320;
const DEFAULT_SCREEN_HEIGHT: u32 = 200;

#[derive(Clone,Copy,Eq,PartialEq)]
enum GuiMode {
    Neutral,
    HoldLMB,
    HoldRMB,
    Pan,
}

enum WidgetType {
    Label,
    Undo,
    Redo,

    // Paint(tile,active,inactive)
    Paint(Tile,Res,Res)
}

pub struct Gui<'a> {
    gfx: GfxLib<'a>,
    timer: TimerSubsystem,
    event_pump: EventPump,
    state: GuiState,
    widgets: Vec<Widget>,

    redraw: bool,
    last_redraw: u32
}

struct GuiState {
    mode: GuiMode,
    selected_paint: Tile,
    board: Option<Board>,
    new_changes: bool,

    offset_x: i32,
    offset_y: i32,
    last_mouse_x: i32,
    last_mouse_y: i32
}

struct Widget {
    mode: WidgetType,
    rect: Rect,
}

impl<'a> Gui<'a> {
    pub fn new() -> Gui<'a> {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        sdl2_image::init(INIT_PNG);

        let window
            = video.window("Picross", DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT)
            .position_centered()
            .opengl()
            .build().unwrap();

        let renderer = window.renderer().build().unwrap();
        let timer = sdl.timer().unwrap();
        let event_pump = sdl.event_pump().unwrap();

        let (offset_x, offset_y) = Gui::calc_initial_offset(
                DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT, 10, 10);

        Gui {
            gfx: GfxLib::new(renderer),
            timer: timer,
            event_pump: event_pump,
            state: GuiState::new(offset_x, offset_y),
            widgets: Gui::make_widgets(),
            redraw: true,
            last_redraw: 0
        }
    }

    fn calc_initial_offset(screen_w: u32, screen_h: u32, width: u32, height: u32)
            -> (i32, i32) {
        let toolbar_height = TOOLBAR_BUTTON_HEIGHT + 6;
        let x_spacing = TILE_WIDTH + 2;
        let y_spacing = TILE_HEIGHT + 2;
        let pivot_x = (x_spacing * width - 2) / 2;
        let pivot_y = (y_spacing * height - 2) / 2;
        let centre_x = (screen_w as i32) / 2 - (pivot_x as i32);
        let centre_y = ((screen_h - toolbar_height) as i32) / 2
                        - (pivot_y as i32);

        (centre_x, centre_y)
    }

    fn make_widgets() -> Vec<Widget> {
        let mut ws = Vec::new();
        let screen_w = DEFAULT_SCREEN_WIDTH;
        let screen_h = DEFAULT_SCREEN_HEIGHT;
        let toolbar_scale: u32 = 1;
        let y = (screen_h - toolbar_scale * (TOOLBAR_BUTTON_HEIGHT + 3)) as i32;

        let paint_spacing = TOOLBAR_PAINT_WIDTH - 1;
        let palette_width = TOOLBAR_PAINT_WIDTH + 2 + paint_spacing * 2 + 1;

        let x_undo = (toolbar_scale * (3 + TOOLBAR_BUTTON_WIDTH + 3)) as i32;
        let x_redo = x_undo + (toolbar_scale * (TOOLBAR_UNDO_REDO_WIDTH + 2)) as i32;
        let x_palette = (screen_w - palette_width) as i32 / 2;

        // label
        ws.push(Widget {
                mode: WidgetType::Label,
                rect: Rect::new_unwrap(
                        (toolbar_scale * 3) as i32,
                        y,
                        toolbar_scale * TOOLBAR_BUTTON_WIDTH,
                        toolbar_scale * TOOLBAR_BUTTON_HEIGHT)
                });

        // undo
        ws.push(Widget {
                mode: WidgetType::Undo,
                rect: Rect::new_unwrap(x_undo, y,
                        toolbar_scale * TOOLBAR_UNDO_REDO_WIDTH,
                        toolbar_scale * TOOLBAR_BUTTON_HEIGHT),
                });

        // redo
        ws.push(Widget {
                mode: WidgetType::Redo,
                rect: Rect::new_unwrap(x_redo, y,
                        toolbar_scale * TOOLBAR_UNDO_REDO_WIDTH,
                        toolbar_scale * TOOLBAR_BUTTON_HEIGHT),
                });

        // paints
        ws.push(Widget {
                mode: WidgetType::Paint(Tile::Empty, Res::ToolbarActiveEmpty, Res::ToolbarInactiveEmpty),
                rect: Rect::new_unwrap(x_palette, y,
                        toolbar_scale * TOOLBAR_PAINT_WIDTH,
                        toolbar_scale * TOOLBAR_BUTTON_HEIGHT),
                });

        ws.push(Widget {
                mode: WidgetType::Paint(Tile::CrossedOut, Res::ToolbarActiveCrossedOut, Res::ToolbarInactiveCrossedOut),
                rect: Rect::new_unwrap(x_palette + (TOOLBAR_PAINT_WIDTH + 2) as i32, y,
                        toolbar_scale * TOOLBAR_PAINT_WIDTH,
                        toolbar_scale * TOOLBAR_BUTTON_HEIGHT),
                });

        ws.push(Widget {
                mode: WidgetType::Paint(Tile::Filled, Res::ToolbarActiveFilled, Res::ToolbarInactiveFilled),
                rect: Rect::new_unwrap(x_palette + (TOOLBAR_PAINT_WIDTH + 2 + paint_spacing * 1) as i32, y,
                        toolbar_scale * TOOLBAR_PAINT_WIDTH,
                        toolbar_scale * TOOLBAR_BUTTON_HEIGHT),
                });

        ws
    }

    pub fn read_input(&mut self, board: &Board) -> PicrossAction {
        let curr_ticks = self.timer.ticks();
        if curr_ticks >= self.last_redraw + 1000 / 60 {
            self.redraw = true;
            return PicrossAction::NoOp;
        }

        let timeout = self.last_redraw + 1000 / 60 - curr_ticks;
        if let Some(e) = self.event_pump.wait_event_timeout(timeout) {
            match e {
                Event::Quit {..} =>
                    return PicrossAction::Quit,

                Event::KeyDown { keycode: Some(k), .. } =>
                    return self.state.on_key_down(k),

                Event::MouseMotion { x, y, .. } =>
                    return self.state.on_mouse_motion(x, y),

                Event::MouseButtonDown { mouse_btn: Mouse::Left, x, y, .. } =>
                    return self.state.on_lmb(board, x, y),

                Event::MouseButtonDown { mouse_btn: Mouse::Right, x, y, .. } =>
                    return self.state.on_rmb(board, x, y),

                Event::MouseButtonDown { mouse_btn: Mouse::Middle, x, y, .. } =>
                    return self.state.on_mmb(x, y),

                Event::MouseButtonUp { mouse_btn: Mouse::Left, .. } =>
                    return self.state.on_lmb_up(),

                Event::MouseButtonUp { mouse_btn: Mouse::Right, .. } =>
                    return self.state.on_rmb_up(),

                Event::MouseButtonUp { mouse_btn: Mouse::Middle, .. } =>
                    return self.state.on_mmb_up(),

                _ => {}
            }
        }
        PicrossAction::NoOp
    }

    pub fn draw_to_screen(&mut self, board: &Board) {
        if !self.redraw {
            return;
        }

        let screen_w = DEFAULT_SCREEN_WIDTH;
        let screen_h = DEFAULT_SCREEN_HEIGHT;
        let toolbar_scale = 1;
        let colour_white = Color::RGB(0xD0, 0xD0, 0xD0);
        let colour_light_grey = Color::RGB(0x98, 0x98, 0x98);
        let colour_dark_grey = Color::RGB(0x58, 0x58, 0x58);

        let toolbar_rect = Rect::new_unwrap(
                0,
                (screen_h - toolbar_scale * (TOOLBAR_BUTTON_HEIGHT + 6)) as i32,
                screen_w,
                toolbar_scale * (TOOLBAR_BUTTON_HEIGHT + 6));

        self.gfx.renderer.set_draw_color(colour_white);
        self.gfx.renderer.clear();

        // board
        self.gfx.renderer.set_draw_color(colour_light_grey);
        for y in 0..(board.height + 1) as u32 {
            Gui::draw_board_line(&mut self.gfx, &self.state,
                    0, y, board.width as u32, y);
        }

        for x in 0..(board.width + 1) as u32 {
            Gui::draw_board_line(&mut self.gfx, &self.state,
                    x, 0, x, board.height as u32);
        }

        self.gfx.renderer.set_draw_color(colour_dark_grey);
        for y in 0..(board.height + 1) as u32 {
            if y % 5 == 0 || y == board.height as u32 {
                Gui::draw_board_line(&mut self.gfx, &self.state,
                        0, y, board.width as u32, y);
            }
        }

        for x in 0..(board.width + 1) as u32 {
            if x % 5 == 0 || x == board.width as u32 {
                Gui::draw_board_line(&mut self.gfx, &self.state,
                        x, 0, x, board.height as u32);
            }
        }

        if let Some(ref b) = self.state.board {
            Gui::draw_board(&mut self.gfx, &self.state, b);
        } else {
            Gui::draw_board(&mut self.gfx, &self.state, board);
        }

        // toolbar
        self.gfx.renderer.set_draw_color(colour_light_grey);
        self.gfx.renderer.fill_rect(toolbar_rect);

        self.gfx.renderer.set_draw_color(colour_dark_grey);
        self.gfx.renderer.draw_rect(toolbar_rect);

        // widgets
        for w in self.widgets.iter() {
            Gui::draw_widget(&mut self.gfx, &self.state, w);
        }

        self.gfx.renderer.present();
        self.redraw = false;
        self.last_redraw = self.timer.ticks();
    }

    fn draw_board(gfx: &mut GfxLib<'a>, state: &GuiState, board: &Board) {
        let board_w = DEFAULT_SCREEN_WIDTH as i32;
        let board_h = (DEFAULT_SCREEN_HEIGHT - TOOLBAR_BUTTON_HEIGHT - 6) as i32;
        let x_spacing = TILE_WIDTH + 2;
        let y_spacing = TILE_HEIGHT + 2;

        let xmin = max(0, (2 - state.offset_x) / (x_spacing as i32)) as u32;
        let ymin = max(0, (2 - state.offset_y) / (y_spacing as i32)) as u32;
        let xmax = max(0, min(board.width as i32,
                                (board_w - state.offset_x) / (x_spacing as i32) + 1)) as u32;
        let ymax = max(0, min(board.height as i32,
                                (board_h - state.offset_y) / (y_spacing as i32) + 1)) as u32;

        for y in ymin..ymax {
            for x in xmin..xmax {
                let maybe_t = board.get(x, y);
                if maybe_t.is_none() {
                    continue;
                }

                let res = match maybe_t.unwrap() {
                    Tile::Empty => Res::TileEmpty,
                    Tile::Filled => Res::TileFilled,
                    Tile::CrossedOut => Res::TileCrossedOut
                };

                let x0 = state.offset_x + (x_spacing * x) as i32;
                let y0 = state.offset_y + (y_spacing * y) as i32;
                let rect = Rect::new_unwrap(x0, y0, TILE_WIDTH, TILE_HEIGHT);

                gfx.draw(res, rect);
            }
        }
    }

    fn draw_board_line(gfx: &mut GfxLib, state: &GuiState,
            x1: u32, y1: u32, x2: u32, y2: u32) {
        let board_x = state.offset_x as i32;
        let board_y = state.offset_y as i32;
        let board_x_spacing = TILE_WIDTH + 2;
        let board_y_spacing = TILE_HEIGHT + 2;

        let line = Rect::new_unwrap(
                board_x - 2 + (board_x_spacing * x1) as i32,
                board_y - 2 + (board_y_spacing * y1) as i32,
                2 + board_x_spacing * (x2 - x1) as u32,
                2 + board_y_spacing * (y2 - y1) as u32);

        gfx.renderer.fill_rect(line);
    }

    fn draw_widget(gfx: &mut GfxLib, state: &GuiState, widget: &Widget) {
        let res = match widget.mode {
            WidgetType::Label => Res::ToolbarPicross,
            WidgetType::Undo => Res::ToolbarUndo,
            WidgetType::Redo => Res::ToolbarRedo,

            WidgetType::Paint(p, active, inactive) =>
                if p == state.selected_paint {
                    active
                } else {
                    inactive
                }
        };

        gfx.draw(res, widget.rect);
    }
}

impl GuiState {
    fn new(offset_x: i32, offset_y: i32) -> GuiState {
        GuiState {
            mode: GuiMode::Neutral,
            selected_paint: Tile::Filled,
            board: None,
            new_changes: false,
            offset_x: offset_x,
            offset_y: offset_y,
            last_mouse_x: 0,
            last_mouse_y: 0
        }
    }

    fn on_key_down(&mut self, keycode: Keycode) -> PicrossAction {
        if self.mode != GuiMode::Neutral {
            return PicrossAction::NoOp
        }

        match keycode {
            Keycode::Z => return PicrossAction::Undo,
            Keycode::X => return PicrossAction::Redo,

            Keycode::Num1 => self.selected_paint = Tile::Empty,
            Keycode::Num2 => self.selected_paint = Tile::CrossedOut,
            Keycode::Num3 => self.selected_paint = Tile::Filled,

            _ => {}
        }
        PicrossAction::NoOp
    }

    fn on_mouse_motion(&mut self, mx: i32, my: i32) -> PicrossAction {
        if self.mode == GuiMode::HoldLMB {
            // lmb will only draw on empty tiles.
            if let Some(ref mut b) = self.board {
                if let Some((tx, ty)) = convert_mouse_coord_to_tile_coord(
                        b, mx - self.offset_x, my - self.offset_y) {
                    let old_tile = b.get(tx, ty).unwrap();
                    let new_tile = self.selected_paint;

                    if (old_tile == Tile::Empty && new_tile != Tile::Empty)
                        || (old_tile != Tile::Empty && new_tile == Tile::Empty) {
                        b.set(tx, ty, new_tile);
                        self.new_changes = true;
                    }
                }
            }
        } else if self.mode == GuiMode::HoldRMB {
            // rmb will clear any tile.
            if let Some(ref mut b) = self.board {
                if let Some((tx, ty)) = convert_mouse_coord_to_tile_coord(
                        b, mx - self.offset_x, my - self.offset_y) {
                    let old_tile = b.get(tx, ty).unwrap();
                    let new_tile = Tile::Empty;

                    if old_tile != Tile::Empty {
                        b.set(tx, ty, new_tile);
                        self.new_changes = true;
                    }
                }
            }
        } else if self.mode == GuiMode::Pan {
            self.offset_x = self.offset_x + mx - self.last_mouse_x;
            self.offset_y = self.offset_y + my - self.last_mouse_y;

            self.last_mouse_x = mx;
            self.last_mouse_y = my;
        }

        PicrossAction::NoOp
    }

    fn on_lmb(&mut self, board: &Board, mx: i32, my: i32) -> PicrossAction {
        if self.mode != GuiMode::Neutral {
            return PicrossAction::NoOp
        }

        self.mode = GuiMode::HoldLMB;

        if self.board.is_none() {
            self.board = Some(board.clone());
            self.new_changes = false;
            self.on_mouse_motion(mx, my)
        } else {
            PicrossAction::NoOp
        }
    }

    fn on_lmb_up(&mut self) -> PicrossAction {
        if self.mode != GuiMode::HoldLMB {
            return PicrossAction::NoOp
        }

        self.mode = GuiMode::Neutral;

        if self.board.is_some() && self.new_changes {
            return PicrossAction::Update(self.board.take().unwrap())
        } else {
            self.board = None;
        }

        PicrossAction::NoOp
    }

    fn on_rmb(&mut self, board: &Board, mx: i32, my: i32) -> PicrossAction {
        if self.mode != GuiMode::Neutral {
            return PicrossAction::NoOp
        }

        self.mode = GuiMode::HoldRMB;

        if self.board.is_none() {
            self.board = Some(board.clone());
            self.new_changes = false;
            self.on_mouse_motion(mx, my)
        } else {
            PicrossAction::NoOp
        }
    }

    fn on_rmb_up(&mut self) -> PicrossAction {
        if self.mode != GuiMode::HoldRMB {
            return PicrossAction::NoOp
        }

        self.mode = GuiMode::Neutral;

        if self.board.is_some() && self.new_changes {
            return PicrossAction::Update(self.board.take().unwrap())
        } else {
            self.board = None;
        }

        PicrossAction::NoOp
    }

    fn on_mmb(&mut self, mx: i32, my: i32) -> PicrossAction {
        if self.mode != GuiMode::Neutral {
            return PicrossAction::NoOp
        }

        self.mode = GuiMode::Pan;
        self.last_mouse_x = mx;
        self.last_mouse_y = my;

        PicrossAction::NoOp
    }

    fn on_mmb_up(&mut self) -> PicrossAction {
        self.mode = GuiMode::Neutral;
        PicrossAction::NoOp
    }
}

fn convert_mouse_coord_to_tile_coord(board: &Board, mx: i32, my: i32)
        -> Option<(u32, u32)> {
    if mx >= 0 || my >= 0 {
        let x_spacing = TILE_WIDTH + 2;
        let y_spacing = TILE_HEIGHT + 2;
        let tx = (mx as u32) / x_spacing;
        let ty = (my as u32) / y_spacing;

        if tx < board.width as u32 && ty < board.height as u32 {
            return Some((tx, ty))
        }
    }

    None
}
