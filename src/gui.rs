// gui.rs

use sdl2;
use sdl2::EventPump;
use sdl2::TimerSubsystem;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::Mouse;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

use action::PicrossAction;
use board::Board;
use board::Tile;

const DEFAULT_SCREEN_WIDTH: u32 = 320;
const DEFAULT_SCREEN_HEIGHT: u32 = 200;
const TILE_WIDTH: u32 = 15;
const TILE_HEIGHT: u32 = 15;

pub struct Gui<'a> {
    renderer: Renderer<'a>,
    timer: TimerSubsystem,
    event_pump: EventPump,
    state: GuiState,

    redraw: bool,
    last_redraw: u32
}

struct GuiState {
    selected_paint: Tile
}

impl<'a> Gui<'a> {
    pub fn new() -> Gui<'a> {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        let window
            = video.window("Picross", DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT)
            .position_centered()
            .opengl()
            .build().unwrap();

        let renderer = window.renderer().build().unwrap();
        let timer = sdl.timer().unwrap();
        let event_pump = sdl.event_pump().unwrap();

        Gui {
            renderer: renderer,
            timer: timer,
            event_pump: event_pump,
            state: GuiState::new(),
            redraw: true,
            last_redraw: 0
        }
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

                Event::MouseButtonDown { mouse_btn: Mouse::Left, x, y, .. } =>
                    return self.state.on_lmb(board, x, y),

                Event::MouseButtonDown { mouse_btn: Mouse::Right, x, y, .. } =>
                    return self.state.on_rmb(board, x, y),

                _ => {}
            }
        }
        PicrossAction::NoOp
    }

    pub fn draw_to_screen(&mut self, board: &Board) {
        if !self.redraw {
            return;
        }

        let colour_white = Color::RGB(0xD0, 0xD0, 0xD0);

        self.renderer.set_draw_color(colour_white);
        self.renderer.clear();

        Gui::draw_board(&mut self.renderer, board);

        self.renderer.present();
        self.redraw = false;
        self.last_redraw = self.timer.ticks();
    }

    fn draw_board(renderer: &mut Renderer<'a>, board: &Board) {
        let colour_dark_grey = Color::RGB(0x58, 0x58, 0x58);
        let x_spacing = TILE_WIDTH + 2;
        let y_spacing = TILE_HEIGHT + 2;
        renderer.set_draw_color(colour_dark_grey);

        for y in 0..(board.height as u32) {
            for x in 0..(board.width as u32) {
                if let Some(Tile::Filled) = board.get(x, y) {
                    let x0 = (x_spacing * x) as i32;
                    let y0 = (y_spacing * y) as i32;
                    let rect = Rect::new_unwrap(x0, y0, TILE_WIDTH, TILE_HEIGHT);

                    renderer.fill_rect(rect);
                }
            }
        }
    }
}

impl GuiState {
    fn new() -> GuiState {
        GuiState {
            selected_paint: Tile::Filled
        }
    }

    fn on_key_down(&mut self, keycode: Keycode) -> PicrossAction {
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

    fn on_lmb(&self, board: &Board, mx: i32, my: i32) -> PicrossAction {
        if let Some((x,y)) = convert_mouse_coord_to_tile_coord(board, mx, my) {
            let mut b = board.clone();
            b.set(x, y, self.selected_paint);
            PicrossAction::Update(b)
        } else {
            PicrossAction::NoOp
        }
    }

    fn on_rmb(&self, board: &Board, mx: i32, my: i32) -> PicrossAction {
        if let Some((x,y)) = convert_mouse_coord_to_tile_coord(board, mx, my) {
            let mut b = board.clone();
            b.set(x, y, Tile::Empty);
            PicrossAction::Update(b)
        } else {
            PicrossAction::NoOp
        }
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
