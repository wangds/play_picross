// gui.rs

use std::cmp::{max,min};
use sdl2;
use sdl2::EventPump;
use sdl2::TimerSubsystem;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::FullscreenType;

use action::PicrossAction;
use board::Board;
use board::Tile;
use gfx::*;
use puzzle::Puzzle;
use puzzle::Rule;
use puzzle::Rules;

// FIXME - not sure what to import.
const SDL_WINDOW_FULLSCREEN_DESKTOP: u32 = 0x1001;

const MIN_TOOLBAR_WIDTH: u32
    = 3
    + TOOLBAR_UNDO_REDO_WIDTH + 2 // undo
    + TOOLBAR_UNDO_REDO_WIDTH + 2 // redo
    + TOOLBAR_PAINT_WIDTH + 2 // palette
    + (TOOLBAR_PAINT_WIDTH - 1) * 2 + 1 // palette
    + 3;

const DEFAULT_SCREEN_WIDTH: u32 = 640;
const DEFAULT_SCREEN_HEIGHT: u32 = 400;
const MIN_SCREEN_WIDTH: u32 = MIN_TOOLBAR_WIDTH;
const MIN_SCREEN_HEIGHT: u32 = 128;

// (w, h, toolbar_scale)
type ScreenSize = (u32,u32,u32);

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
    last_redraw: u32,

    // Some(new screen size) if need to relayout the widgets
    resize: Option<(u32,u32)>
}

struct GuiState {
    mode: GuiMode,
    selected_paint: Tile,
    board: Option<Board>,
    new_changes: bool,

    screen_size: ScreenSize,
    board_scale: u32,
    offset_x: i32,
    offset_y: i32,
    board_pixel_width: u32,
    board_pixel_height: u32,
    row_rule_max_pixel_width: u32,
    col_rule_max_pixel_height: u32,
    last_mouse_x: i32,
    last_mouse_y: i32,

    // Some(x,y) to highlight a row and a column
    highlight: Option<(u32,u32)>
}

struct Widget {
    mode: WidgetType,
    rect: Rect,
}

impl<'a> Gui<'a> {
    pub fn new() -> Gui<'a> {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        init_png();

        let state = GuiState::new(DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT);
        let screen_size = state.screen_size;

        let mut window
            = video.window("Picross", DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT)
            .resizable()
            .position_centered()
            .opengl()
            .build().unwrap();

        let _ = window.set_minimum_size(MIN_SCREEN_WIDTH, MIN_SCREEN_HEIGHT);

        let renderer = window.renderer().build().unwrap();
        let timer = sdl.timer().unwrap();
        let event_pump = sdl.event_pump().unwrap();

        Gui {
            gfx: GfxLib::new(renderer),
            timer: timer,
            event_pump: event_pump,
            state: state,
            widgets: Gui::make_widgets(screen_size),
            redraw: true,
            last_redraw: 0,
            resize: None
        }
    }

    fn is_picross_label_visible(screen_size: ScreenSize) -> bool {
        let (screen_w, _, toolbar_scale) = screen_size;
        let toolbar_w = MIN_TOOLBAR_WIDTH + TOOLBAR_BUTTON_WIDTH + 3; // picross
        screen_w >= toolbar_scale * toolbar_w
    }

    fn make_widgets(screen_size: ScreenSize) -> Vec<Widget> {
        let mut ws = Vec::new();
        let (screen_w, screen_h, toolbar_scale) = screen_size;
        let y = (screen_h - toolbar_scale * (TOOLBAR_BUTTON_HEIGHT + 3)) as i32;

        let paint_spacing = TOOLBAR_PAINT_WIDTH - 1;
        let palette_width = toolbar_scale * (TOOLBAR_PAINT_WIDTH + 2 + paint_spacing * 2 + 1);

        let label_visible = Gui::is_picross_label_visible(screen_size);
        let x_undo =
            if label_visible {
                (toolbar_scale * (3 + TOOLBAR_BUTTON_WIDTH + 3)) as i32
            } else {
                (toolbar_scale * 3) as i32
            };
        let x_redo = x_undo + (toolbar_scale * (TOOLBAR_UNDO_REDO_WIDTH + 2)) as i32;
        let x_palette = max(x_redo + (toolbar_scale * (TOOLBAR_UNDO_REDO_WIDTH + 2)) as i32,
                            (screen_w - palette_width) as i32 / 2);

        // label
        if label_visible {
            ws.push(Widget {
                    mode: WidgetType::Label,
                    rect: Rect::new(
                            (toolbar_scale * 3) as i32,
                            y,
                            toolbar_scale * TOOLBAR_BUTTON_WIDTH,
                            toolbar_scale * TOOLBAR_BUTTON_HEIGHT)
                    });
        }

        // undo
        ws.push(Widget {
                mode: WidgetType::Undo,
                rect: Rect::new(x_undo, y,
                        toolbar_scale * TOOLBAR_UNDO_REDO_WIDTH,
                        toolbar_scale * TOOLBAR_BUTTON_HEIGHT),
                });

        // redo
        ws.push(Widget {
                mode: WidgetType::Redo,
                rect: Rect::new(x_redo, y,
                        toolbar_scale * TOOLBAR_UNDO_REDO_WIDTH,
                        toolbar_scale * TOOLBAR_BUTTON_HEIGHT),
                });

        // paints
        ws.push(Widget {
                mode: WidgetType::Paint(Tile::Empty, Res::ToolbarActiveEmpty, Res::ToolbarInactiveEmpty),
                rect: Rect::new(x_palette, y,
                        toolbar_scale * TOOLBAR_PAINT_WIDTH,
                        toolbar_scale * TOOLBAR_BUTTON_HEIGHT),
                });

        ws.push(Widget {
                mode: WidgetType::Paint(Tile::CrossedOut, Res::ToolbarActiveCrossedOut, Res::ToolbarInactiveCrossedOut),
                rect: Rect::new(
                        x_palette + (toolbar_scale * (TOOLBAR_PAINT_WIDTH + 2)) as i32,
                        y,
                        toolbar_scale * TOOLBAR_PAINT_WIDTH,
                        toolbar_scale * TOOLBAR_BUTTON_HEIGHT),
                });

        ws.push(Widget {
                mode: WidgetType::Paint(Tile::Filled, Res::ToolbarActiveFilled, Res::ToolbarInactiveFilled),
                rect: Rect::new(
                        x_palette + (toolbar_scale * (TOOLBAR_PAINT_WIDTH + 2 + paint_spacing * 1)) as i32,
                        y,
                        toolbar_scale * TOOLBAR_PAINT_WIDTH,
                        toolbar_scale * TOOLBAR_BUTTON_HEIGHT),
                });

        ws
    }

    pub fn on_new_puzzle(&mut self, puzzle: &Puzzle) {
        self.state.on_new_puzzle(puzzle);
    }

    pub fn read_input(&mut self, board: &Board) -> PicrossAction {
        let curr_ticks = self.timer.ticks();
        if curr_ticks >= self.last_redraw + 1000 / 60 {
            self.redraw = true;
            return PicrossAction::NoOp;
        }

        let (_, screen_h, toolbar_scale) = self.state.screen_size;
        let toolbar_y = (screen_h - toolbar_scale * (TOOLBAR_BUTTON_HEIGHT + 6)) as i32;

        let timeout = self.last_redraw + 1000 / 60 - curr_ticks;
        if let Some(e) = self.event_pump.wait_event_timeout(timeout) {
            match e {
                Event::Quit {..} =>
                    return PicrossAction::Quit,

                Event::Window { win_event: WindowEvent::Resized(data1, data2), .. } =>
                    self.resize = Some((data1 as u32, data2 as u32)),

                Event::KeyDown { keycode: Some(Keycode::F), .. }
                | Event::KeyDown { keycode: Some(Keycode::F11), .. } => {
                    self.toggle_fullscreen();
                    return PicrossAction::NoOp
                },

                Event::KeyDown { keycode: Some(k), .. } =>
                    return self.state.on_key_down(k),

                Event::MouseMotion { x, y, .. } =>
                    return self.state.on_mouse_motion(board, x, y),

                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    let w = Gui::find_widget(&self.widgets, x, y);
                    if y < toolbar_y || w.is_some() {
                        return self.state.on_lmb(board, w, x, y)
                    }
                },

                Event::MouseButtonDown { mouse_btn: MouseButton::Right, x, y, .. } =>
                    if y < toolbar_y {
                        return self.state.on_rmb(board, x, y)
                    },

                Event::MouseButtonDown { mouse_btn: MouseButton::Middle, x, y, .. } =>
                    return self.state.on_mmb(x, y),

                Event::MouseButtonDown { mouse_btn: MouseButton::X1, .. } =>
                    if self.state.mode == GuiMode::Neutral {
                        return PicrossAction::Undo
                    },

                Event::MouseButtonDown { mouse_btn: MouseButton::X2, .. } =>
                    if self.state.mode == GuiMode::Neutral {
                        return PicrossAction::Redo
                    },

                Event::MouseButtonUp { mouse_btn: MouseButton::Left, .. } =>
                    return self.state.on_lmb_up(),

                Event::MouseButtonUp { mouse_btn: MouseButton::Right, .. } =>
                    return self.state.on_rmb_up(),

                Event::MouseButtonUp { mouse_btn: MouseButton::Middle, .. } =>
                    return self.state.on_mmb_up(),

                Event::MouseWheel { y, .. } =>
                    return self.state.on_wheel(y),

                Event::DropFile { filename, .. } => {
                    self.state.mode = GuiMode::Neutral;
                    self.state.board = None;
                    self.state.highlight = None;

                    return PicrossAction::New(filename)
                },

                _ => {}
            }
        }
        PicrossAction::NoOp
    }

    fn find_widget(widgets: &Vec<Widget>, x: i32, y: i32) -> Option<&Widget> {
        widgets.iter().find(|w| {
                let r = &w.rect;
                r.x() <= x && x <= r.x() + (r.width() as i32)
                && r.y() <= y && y <= r.y() + (r.height() as i32) })
    }

    fn toggle_fullscreen(&mut self) {
        let mut window = self.gfx.renderer.window_mut().unwrap();

        if window.window_flags() & SDL_WINDOW_FULLSCREEN_DESKTOP != 0 {
            window.set_fullscreen(FullscreenType::Off).unwrap();
        } else {
            window.set_fullscreen(FullscreenType::Desktop).unwrap();
        }
    }

    pub fn draw_to_screen(&mut self, rules: Rules, board: &Board) {
        if !self.redraw {
            return;
        }

        if let Some((new_w, new_h)) = self.resize {
            self.state.on_resize_window(new_w, new_h);
            self.widgets = Gui::make_widgets(self.state.screen_size);
            self.resize = None;
        }

        let (screen_w, screen_h, toolbar_scale) = self.state.screen_size;
        let colour_white = Color::RGB(0xD0, 0xD0, 0xD0);
        let colour_light_grey = Color::RGB(0x98, 0x98, 0x98);
        let colour_dark_grey = Color::RGB(0x58, 0x58, 0x58);
        let colour_rose = Color::RGB(0xC2, 0xBC, 0xBC);

        let toolbar_rect = Rect::new(
                0,
                (screen_h - toolbar_scale * (TOOLBAR_BUTTON_HEIGHT + 6)) as i32,
                screen_w,
                toolbar_scale * (TOOLBAR_BUTTON_HEIGHT + 6));

        self.gfx.renderer.set_draw_color(colour_white);
        self.gfx.renderer.clear();

        // board
        if let Some((x, y)) = self.state.highlight {
            self.gfx.renderer.set_draw_color(colour_rose);
            Gui::draw_board_line(&mut self.gfx, &self.state,
                    x, 0, x + 1, board.height as u32);
            Gui::draw_board_line(&mut self.gfx, &self.state,
                    0, y, board.width as u32, y + 1);
        }

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
            Gui::draw_rules(&mut self.gfx, &self.state, rules, b);
            Gui::draw_board(&mut self.gfx, &self.state, b);
        } else {
            Gui::draw_rules(&mut self.gfx, &self.state, rules, board);
            Gui::draw_board(&mut self.gfx, &self.state, board);
        }

        // toolbar
        self.gfx.renderer.set_draw_color(colour_light_grey);
        let _ = self.gfx.renderer.fill_rect(toolbar_rect);

        self.gfx.renderer.set_draw_color(colour_dark_grey);
        let _ = self.gfx.renderer.draw_rect(toolbar_rect);

        // widgets
        for w in self.widgets.iter() {
            Gui::draw_widget(&mut self.gfx, &self.state, w);
        }

        self.gfx.renderer.present();
        self.redraw = false;
        self.last_redraw = self.timer.ticks();
    }

    fn draw_rules(gfx: &mut GfxLib<'a>, state: &GuiState,
            rules: Rules, board: &Board) {
        let scale = state.board_scale;
        let text_scale = min(2, scale);
        let (col_rules, row_rules) = rules;
        let x_spacing = (text_scale * 5) as i32;
        let y_spacing = (text_scale * (FONT_HEIGHT + 2)) as i32;

        let mut x = state.offset_x + (scale * TILE_WIDTH / 2 + 1) as i32;
        for (col, rule) in col_rules.iter().enumerate() {
            let len = rule.len();
            let head = board.get_completed_column_segments_from_head(col);
            let tail = board.get_completed_column_segments_from_tail(col);
            let mut y = state.offset_y - (scale * 4 + text_scale * FONT_HEIGHT) as i32;

            for i in 0..len {
                let revi = len - i - 1;
                let v = rule[revi];
                let font = Gui::pick_font(v, len, &head, revi, &tail, i);

                gfx.text_centre(font, v, text_scale, x, y);
                y = y - y_spacing;
            }

            x = x + (scale * (TILE_WIDTH + 2)) as i32;
        }

        let mut y = state.offset_y + (scale * TILE_HEIGHT - text_scale * FONT_HEIGHT) as i32 / 2;
        for (row, rule) in row_rules.iter().enumerate() {
            let len = rule.len();
            let head = board.get_completed_row_segments_from_head(row);
            let tail = board.get_completed_row_segments_from_tail(row);
            let mut x = state.offset_x - (scale * 4) as i32;

            for i in 0..len {
                let revi = len - i - 1;
                let v = rule[revi];
                let font = Gui::pick_font(v, len, &head, revi, &tail, i);

                gfx.text_right(font, v, text_scale, x, y);
                x = x - x_spacing - text_pixel_width(v, text_scale) as i32;
            }

            y = y + (scale * (TILE_HEIGHT + 2)) as i32;
        }
    }

    fn pick_font(v: u32, len: usize,
            head: &Vec<u32>, head_idx: usize,
            tail: &Vec<u32>, tail_idx: usize) -> Font {
        if head.len() > len || tail.len() > len {
            Font::Conflict
        } else if head_idx < head.len() {
            if v == head[head_idx] {
                assert!(tail_idx >= tail.len() || v == tail[tail_idx]);
                Font::Solved
            } else {
                Font::Conflict
            }
        } else if tail_idx < tail.len() {
            if v == tail[tail_idx] {
                assert!(head_idx >= head.len() || v == head[head_idx]);
                Font::Solved
            } else {
                Font::Conflict
            }
        } else {
            Font::Unsolved
        }
    }

    fn draw_board(gfx: &mut GfxLib<'a>, state: &GuiState, board: &Board) {
        let (screen_w, screen_h, toolbar_scale) = state.screen_size;
        let board_w = screen_w as i32;
        let board_h = (screen_h - toolbar_scale * (TOOLBAR_BUTTON_HEIGHT + 6)) as i32;
        let x_spacing = state.board_scale * (TILE_WIDTH + 2);
        let y_spacing = state.board_scale * (TILE_HEIGHT + 2);

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

                let rect = Rect::new(
                        state.offset_x + (x_spacing * x) as i32,
                        state.offset_y + (y_spacing * y) as i32,
                        state.board_scale * TILE_WIDTH,
                        state.board_scale * TILE_HEIGHT);

                gfx.draw(res, rect);
            }
        }
    }

    fn draw_board_line(gfx: &mut GfxLib, state: &GuiState,
            x1: u32, y1: u32, x2: u32, y2: u32) {
        let board_x = state.offset_x;
        let board_y = state.offset_y;
        let scale = state.board_scale;
        let board_x_spacing = TILE_WIDTH + 2;
        let board_y_spacing = TILE_HEIGHT + 2;

        let line = Rect::new(
                board_x + (scale as i32) * ((board_x_spacing * x1) as i32 - 2),
                board_y + (scale as i32) * ((board_y_spacing * y1) as i32 - 2),
                scale * (2 + board_x_spacing * (x2 - x1)),
                scale * (2 + board_y_spacing * (y2 - y1)));

        let _ = gfx.renderer.fill_rect(line);
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
    fn new(screen_w: u32, screen_h: u32) -> GuiState {
        let screen_size = GuiState::calc_screen_size_and_scale(
                screen_w, screen_h);

        GuiState {
            mode: GuiMode::Neutral,
            selected_paint: Tile::Filled,
            board: None,
            new_changes: false,
            screen_size: screen_size,
            board_scale: 1,
            offset_x: 0,
            offset_y: 0,
            board_pixel_width: 0,
            board_pixel_height: 0,
            row_rule_max_pixel_width: 0,
            col_rule_max_pixel_height: 0,
            last_mouse_x: 0,
            last_mouse_y: 0,
            highlight: None
        }
    }

    fn calc_screen_size_and_scale(screen_w: u32, screen_h: u32) -> ScreenSize {
        let toolbar_w = MIN_TOOLBAR_WIDTH + TOOLBAR_BUTTON_WIDTH + 3; // picross
        let toolbar_x_scale = screen_w / toolbar_w;
        let toolbar_y_scale = (screen_h + 400) / 400;
        let toolbar_scale = max(1, min(toolbar_x_scale, toolbar_y_scale));
        (screen_w, screen_h, toolbar_scale)
    }

    fn calc_default_offset(&self) -> (i32, i32) {
        let (screen_w, screen_h, toolbar_scale) = self.screen_size;
        let board_scale = self.board_scale as i32;
        let text_scale = min(2, board_scale);
        let toolbar_h = toolbar_scale * (TOOLBAR_BUTTON_HEIGHT + 6);
        let canvas_w = screen_w as i32;
        let canvas_h = (screen_h - toolbar_h) as i32;
        let scaled_board_w = board_scale * (self.board_pixel_width + 2) as i32;
        let scaled_text_w = board_scale * 4 + text_scale * self.row_rule_max_pixel_width as i32;

        let offset_x =
            if canvas_w < scaled_board_w {
                // centre board
                (canvas_w - scaled_board_w) / 2
            } else if canvas_w < scaled_board_w + scaled_text_w {
                // right of board at right of screen
                canvas_w - scaled_board_w
            } else if canvas_w < scaled_board_w + scaled_text_w * 2 {
                // left of text at left of screen
                scaled_text_w
            } else {
                // centre board
                (canvas_w - scaled_board_w) / 2
            };
        let offset_y = (canvas_h
                        - board_scale * self.board_pixel_height as i32
                        + board_scale * 4
                        + text_scale * self.col_rule_max_pixel_height as i32) / 2;

        (offset_x, offset_y)
    }

    fn on_new_puzzle(&mut self, puzzle: &Puzzle) {
        let b = puzzle.get_board();
        let (col_rules, row_rules) = puzzle.get_rules();
        let board_x_spacing = TILE_WIDTH + 2;
        let board_y_spacing = TILE_HEIGHT + 2;

        self.board_pixel_width = board_x_spacing * b.width as u32 - 2;
        self.board_pixel_height = board_y_spacing * b.height as u32 - 2;

        self.row_rule_max_pixel_width = row_rules.iter().fold(0,
                |n, r| max(n, calc_rule_width(r)));

        self.col_rule_max_pixel_height = col_rules.iter().fold(0,
                |n, r| max(n, calc_rule_height(r)));

        let (mut offset_x, mut offset_y) = self.calc_default_offset();

        // if board is too big, show top-left corner of board
        if offset_x < self.row_rule_max_pixel_width as i32 {
            offset_x = self.row_rule_max_pixel_width as i32;
        }
        if offset_y < self.col_rule_max_pixel_height as i32 {
            offset_y = self.col_rule_max_pixel_height as i32;
        }
        if offset_x < 2 {
            offset_x = 2;
        }
        if offset_y < 2 {
            offset_y = 2;
        }

        self.offset_x = offset_x;
        self.offset_y = offset_y;
    }

    fn on_resize_window(&mut self, new_screen_w: u32, new_screen_h: u32) {
        let (old_screen_w, old_screen_h, _) = self.screen_size;
        let old_offset_x = self.offset_x;
        let old_offset_y = self.offset_y;

        self.screen_size = GuiState::calc_screen_size_and_scale(
                new_screen_w, new_screen_h);

        let (desired_x, desired_y) = self.calc_default_offset();
        let diff_x = (new_screen_w as i32 - old_screen_w as i32).abs();
        let diff_y = (new_screen_h as i32 - old_screen_h as i32).abs();

        self.offset_x = max(old_offset_x - diff_x,
                            min(desired_x, old_offset_x + diff_x));
        self.offset_y = max(old_offset_y - diff_y,
                            min(desired_y, old_offset_y + diff_y));
    }

    fn on_key_down(&mut self, keycode: Keycode) -> PicrossAction {
        if self.mode != GuiMode::Neutral {
            return PicrossAction::NoOp
        }

        match keycode {
            Keycode::Z => return PicrossAction::Undo,
            Keycode::X => return PicrossAction::Redo,
            Keycode::A => return PicrossAction::AutoFill,

            Keycode::Num1 => self.selected_paint = Tile::Empty,
            Keycode::Num2 => self.selected_paint = Tile::CrossedOut,
            Keycode::Num3 => self.selected_paint = Tile::Filled,

            _ => {}
        }
        PicrossAction::NoOp
    }

    fn on_mouse_motion(&mut self, board: &Board, mx: i32, my: i32) -> PicrossAction {
        let maybe_tile_coord = convert_mouse_coord_to_tile_coord(
                board, self.board_scale, mx - self.offset_x, my - self.offset_y);
        self.highlight = maybe_tile_coord;

        if self.mode == GuiMode::HoldLMB {
            // lmb will only draw on empty tiles.
            if let Some(ref mut b) = self.board {
                if let Some((tx, ty)) = maybe_tile_coord {
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
                if let Some((tx, ty)) = maybe_tile_coord {
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

    fn on_lmb(&mut self, board: &Board, widget: Option<&Widget>, mx: i32, my: i32)
            -> PicrossAction {
        if self.mode != GuiMode::Neutral {
            return PicrossAction::NoOp
        }

        if let Some(w) = widget {
            match w.mode {
                WidgetType::Label => {},
                WidgetType::Undo => return PicrossAction::Undo,
                WidgetType::Redo => return PicrossAction::Redo,

                WidgetType::Paint(paint,_,_) =>
                    self.selected_paint = paint
            }
        } else {
            self.mode = GuiMode::HoldLMB;

            if self.board.is_none() {
                self.board = Some(board.clone());
                self.new_changes = false;
                return self.on_mouse_motion(board, mx, my)
            }
        }

        PicrossAction::NoOp
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
            self.on_mouse_motion(board, mx, my)
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

    fn on_wheel(&mut self, y: i32) -> PicrossAction {
        self.board_scale = max(1, min(self.board_scale as i32 + y, 5)) as u32;
        PicrossAction::NoOp
    }
}

fn convert_mouse_coord_to_tile_coord(board: &Board, scale: u32, mx: i32, my: i32)
        -> Option<(u32, u32)> {
    if mx >= 0 || my >= 0 {
        let x_spacing = scale * (TILE_WIDTH + 2);
        let y_spacing = scale * (TILE_HEIGHT + 2);
        let tx = (mx as u32) / x_spacing;
        let ty = (my as u32) / y_spacing;

        if tx < board.width as u32 && ty < board.height as u32 {
            return Some((tx, ty))
        }
    }

    None
}

fn calc_rule_width(rule: &Rule) -> u32 {
    let x_spacing = 5;
    let num_rules = rule.len() as u32;

    if num_rules > 0 {
        rule.iter().fold(0, |sum, &v|
                sum + text_pixel_width(v, 1)) + x_spacing * (num_rules - 1)
    } else {
        0
    }
}

fn calc_rule_height(rule: &Rule) -> u32 {
    let y_spacing = 2;
    let num_rules = rule.len() as u32;

    if num_rules > 0 {
        FONT_HEIGHT * num_rules + y_spacing * (num_rules - 1)
    } else {
        0
    }
}

/*--------------------------------------------------------------*/

#[cfg(not(feature = "png"))]
fn init_png() {
}

#[cfg(feature = "png")]
fn init_png() {
    sdl2::image::init(sdl2::image::INIT_PNG).unwrap();
}
