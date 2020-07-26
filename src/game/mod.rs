pub mod piece;

use piece::Piece;
use std::collections::HashMap;

use ggez::{
    event::{EventHandler, MouseButton},
    graphics::{self, Color, DrawMode, DrawParam, Image},
    input::mouse,
    Context, GameResult,
};

use std::vec::Vec;

use crate::WIN_HEIGHT;
use crate::WIN_WIDTH;

enum Player {
    White,
    Black,
}

impl Player {
    fn switch(&self) -> Self {
        match *self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

pub struct RChess {
    board: [[Color; 8]; 8],
    board_pcs: [[char; 8]; 8],
    current: Option<char>,
    current_pos: Option<(u8, u8)>,
    moves: Vec<(u8, u8)>,
    pieces: HashMap<char, Image>,
    turn: Player,
    w_color: Color,
    b_color: Color,
    sq_size: i32,
    check: u8,
    moving: bool,
    needs_draw: bool,
    en_p_black: u8,
    en_p_white: u8,
    w_king_pos: (u8, u8),
    b_king_pos: (u8, u8),
}

impl RChess {
    // Create a new instance of RChess
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut pieces = HashMap::<char, Image>::new();

        let board_pcs = [
            ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
            ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
            ['-', '-', '-', '-', '-', '-', '-', '-'],
            ['-', '-', '-', '-', '-', '-', '-', '-'],
            ['-', '-', '-', '-', '-', '-', '-', '-'],
            ['-', '-', '-', '-', '-', '-', '-', '-'],
            ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
            ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'],
        ];

        for row in board_pcs.iter() {
            for piece in row.iter() {
                if pieces.contains_key(piece) {
                    continue;
                }

                if piece != &'-' {
                    let img = Image::new(ctx, &format!("/{}.png", piece))?;
                    pieces.insert(*piece, img);
                }
            }
        }

        let w_color = Color::from_rgb(200, 200, 200);
        let b_color = Color::from_rgb(50, 50, 50);

        let mut chess = Self {
            board: [[w_color.clone(); 8]; 8],
            board_pcs,
            current: None,
            current_pos: None,
            moves: Vec::new(),
            pieces: pieces,
            turn: Player::White,
            w_color,
            b_color,
            sq_size: (WIN_WIDTH / 8) as i32,
            check: 0,
            moving: false,
            needs_draw: true,
            en_p_white: 0,
            en_p_black: 0,
            w_king_pos: (5, 0),
            b_king_pos: (5, 7),
        };

        chess.reset_board();

        Ok(chess)
    }

    // Reset the board square colors
    fn reset_board(&mut self) {
        for y in 0..8 {
            let row_even = y % 2 == 0;

            for x in 0..8 {
                let col_even = x % 2 == 0;

                self.board[y][x] = if (col_even && row_even) || (!col_even && !row_even) {
                    self.w_color.clone()
                } else {
                    self.b_color.clone()
                }
            }
        }
    }

    /* Gets the piece under the mouse.
     */
    fn get_piece_at_mouse(&self, ctx: &Context) -> char {
        let mouse_pos = mouse::position(ctx);
        let x = (mouse_pos.x as i32 / self.sq_size) as usize;
        let y = (mouse_pos.y as i32 / self.sq_size) as usize;

        self.board_pcs[y][x]
    }

    /* Checks if a piece belongs to white.
     */
    fn is_white_piece(pc: char) -> bool {
        ['K', 'Q', 'R', 'N', 'B', 'P'].contains(&pc)
    }

    /* Checks if a piece belongs to black.
     */
    fn is_black_piece(pc: char) -> bool {
        ['k', 'q', 'r', 'n', 'b', 'p'].contains(&pc)
    }

    /* Checks whether the supplied piece belongs to
     * the opponent
     */
    fn is_opponent(&self, ch: char) -> bool {
        match self.turn {
            Player::White => Self::is_black_piece(ch),
            Player::Black => Self::is_white_piece(ch),
        }
    }

    /* Checks if a given character is a piece
     */
    fn is_piece(&self, ch: char) -> bool {
        !['-'].contains(&ch)
    }

    /* Takes a dx and dy that specifies a line of path.
     * All squares along this path that does not have a piece
     * are by default added to the list of moves. If a piece is encountered,
     * a check is performed on the type. If it's an opponent piece, the piece
     * square is added to the list of possible moves, else not
     */
    fn get_line_moves(&mut self, x: u8, y: u8, dx: i8, dy: i8) -> Vec<(u8, u8)> {
        let mut m_x = x as i8 + dx;
        let mut m_y = y as i8 + dy;

        let mut moves = Vec::<(u8, u8)>::with_capacity(7);

        while m_x >= 0 && m_x < 8 && m_y >= 0 && m_y < 8 {
            let ch = self.board_pcs[m_y as usize][m_x as usize];

            if self.is_piece(ch) {
                if self.is_opponent(ch) {
                    moves.push((m_x as u8, m_y as u8));
                }

                break;
            }

            moves.push((m_x as u8, m_y as u8));
            m_x += dx;
            m_y += dy;
        }

        moves
    }

    /* Takes a position and and pushes into the move vector
     * all the moves that a pawn at that position can make
     */
    fn mv_pawn(&self, x: u8, y: u8) -> Vec<(u8, u8)> {
        let x_i = x as usize;
        let y_i = y as usize;

        let mut moves = Vec::<(u8, u8)>::with_capacity(4);

        match self.turn {
            Player::White => {
                if y == 0 {
                    return moves;
                }

                if !self.is_piece(self.board_pcs[y_i - 1][x_i]) {
                    moves.push((x, y - 1));
                }

                if y == 6 && !self.is_piece(self.board_pcs[y_i - 2][x_i]) {
                    moves.push((x, y - 2));
                }

                if x < 7 && self.is_opponent(self.board_pcs[y_i - 1][x_i + 1]) {
                    moves.push((x + 1, y - 1));
                }

                if x > 0 && self.is_opponent(self.board_pcs[y_i - 1][x_i - 1]) {
                    moves.push((x - 1, y - 1));
                }
            }

            Player::Black => {
                if y == 7 {
                    return moves;
                }

                if !self.is_piece(self.board_pcs[y_i + 1][x_i]) {
                    moves.push((x, y + 1));
                }

                if y == 1 && !self.is_piece(self.board_pcs[y_i + 2][x_i]) {
                    moves.push((x, y + 2));
                }

                if x < 7 && self.is_opponent(self.board_pcs[y_i + 1][x_i + 1]) {
                    moves.push((x + 1, y + 1));
                }

                if x > 0 && self.is_opponent(self.board_pcs[y_i + 1][x_i - 1]) {
                    moves.push((x - 1, y + 1));
                }
            }
        }

        moves
    }

    /* Used for moving a knight. Unique function cuz
     * knights make a 2.5 move.
     */
    fn mv_knight(&self, x: u8, y: u8) -> Vec<(u8, u8)> {
        let x_m = x as i8;
        let y_m = y as i8;

        let moves: Vec<(i8, i8)> = vec![
            (-2, -1),
            (-1, -2),
            (1, -2),
            (2, -1),
            (-2, 1),
            (-1, 2),
            (1, 2),
            (2, 1),
        ];

        let mut poss_moves = Vec::<(u8, u8)>::with_capacity(8);

        for (dx, dy) in moves {
            let pos_x = x_m + dx;
            let pos_y = y_m + dy;
            if pos_x >= 0 && pos_x < 8 && pos_y >= 0 && pos_y < 8 {
                let ch = self.board_pcs[pos_y as usize][pos_x as usize];
                if !self.is_piece(ch) || self.is_opponent(ch) {
                    poss_moves.push((pos_x as u8, pos_y as u8));
                }
            }
        }

        poss_moves
    }

    /* Used for moving a bishop
     */
    fn mv_bishop(&mut self, x: u8, y: u8) -> Vec<(u8, u8)> {
        let mut moves = Vec::<(u8, u8)>::with_capacity(13);
        moves.append(&mut self.get_line_moves(x, y, 1, 1));
        moves.append(&mut self.get_line_moves(x, y, 1, -1));
        moves.append(&mut self.get_line_moves(x, y, -1, -1));
        moves.append(&mut self.get_line_moves(x, y, -1, 1));
        moves
    }

    /* Used for moving a Rook
     */
    fn mv_rook(&mut self, x: u8, y: u8) -> Vec<(u8, u8)> {
        let mut moves = Vec::<(u8, u8)>::with_capacity(14);
        for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
            moves.append(&mut self.get_line_moves(x, y, *dx, *dy));
        }

        moves
    }

    /* Used for moving a Queen
     */
    fn mv_queen(&mut self, x: u8, y: u8) -> Vec<(u8, u8)> {
        let mut moves = Vec::<(u8, u8)>::with_capacity(28);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if x == 0 && y == 0 {
                    continue;
                }

                moves.append(&mut self.get_line_moves(x, y, dx, dy));
            }
        }

        moves
    }

    /* Used for moving a King
     */
    fn mv_king(&mut self, x: u8, y: u8) -> Vec<(u8, u8)> {
        let mut moves = Vec::<(u8, u8)>::with_capacity(8);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if x == 0 && y == 0 {
                    continue;
                }

                let x_m = x as i8 + dx;
                let y_m = y as i8 + dy;

                if x_m >= 0 && x_m < 8 && y_m >= 0 && y_m < 8 {
                    let ch = self.board_pcs[y_m as usize][x_m as usize];
                    if !self.is_piece(ch) || self.is_opponent(ch) {
                        moves.push((x_m as u8, y_m as u8));
                    }
                }
            }
        }

        moves
    }

    fn get_piece_moves(&mut self, ch: char, x: u8, y: u8) -> Vec<(u8, u8)> {
        match ch {
            'p' | 'P' => self.mv_pawn(x, y),
            'r' | 'R' => self.mv_rook(x, y),
            'n' | 'N' => self.mv_knight(x, y),
            'b' | 'B' => self.mv_bishop(x, y),
            'q' | 'Q' => self.mv_queen(x, y),
            'k' | 'K' => self.mv_king(x, y),
            _ => Vec::<(u8, u8)>::new(),
        }
    }

    fn select_piece(&mut self, x: u8, y: u8) -> GameResult<()> {
        let ch = self.board_pcs[y as usize][x as usize];

        if !self.is_piece(ch) || self.is_opponent(ch) {
            return Ok(());
        }

        let moves = self.get_piece_moves(ch, x, y);

        self.current = Some(ch);
        self.current_pos = Some((x, y));

        for (m_x, m_y) in &moves {
            self.board[*m_y as usize][*m_x as usize] = Color::from_rgb(200, 200, 0);

            self.moves.push((*m_x, *m_y));
        }

        self.board[y as usize][x as usize] = Color::from_rgb(255, 85, 85);

        self.needs_draw = true;
        self.moving = true;

        Ok(())
    }

    fn move_piece(&mut self, x: u8, y: u8) -> GameResult<()> {
        if self.moves.contains(&(x, y)) {
            self.board_pcs[y as usize][x as usize] = self.current.unwrap();
            let curr = self.current_pos.unwrap();
            self.board_pcs[curr.1 as usize][curr.0 as usize] = '-';
            self.current = None;
            self.current_pos = None;
            self.moving = false;
            self.turn = self.turn.switch();
            self.moves.clear();
            self.needs_draw = true;
            self.reset_board();
            return Ok(());
        }

        let ch = self.board_pcs[y as usize][x as usize];

        if self.is_piece(ch) && !self.is_opponent(ch) {
            self.moves.clear();
            self.reset_board();
            self.select_piece(x, y);
            self.needs_draw = true;
        }

        Ok(())
    }

    fn check_for_checks(&mut self, board: [[char; 8]; 8], plyr: Player) -> bool {
        match plyr {
            Player::White => {
                for y in 0..8 {
                    for x in 0..8 {
                        let ch = board[y as usize][x as usize];

                        if !Self::is_black_piece(ch) {
                            continue;
                        }

                        if self.get_piece_moves(ch, x, y).contains(&self.w_king_pos) {
                            return true;
                        }
                    }
                }
            }

            Player::Black => {
                for y in 0..8 {
                    for x in 0..8 {
                        let ch = board[y as usize][x as usize];

                        if !Self::is_white_piece(ch) {
                            continue;
                        }

                        if self.get_piece_moves(ch, x, y).contains(&self.b_king_pos) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }
}

impl EventHandler for RChess {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.needs_draw {
            return Ok(());
        }
        graphics::clear(ctx, Color::from_rgb(0, 0, 0));

        for (y, row) in self.board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let x_sq = x as i32 * self.sq_size;
                let y_sq = y as i32 * self.sq_size;

                let r = graphics::Rect::new_i32(x_sq, y_sq, self.sq_size, self.sq_size);
                let mesh = graphics::Mesh::new_rectangle(ctx, DrawMode::fill(), r, *cell)?;

                graphics::draw(ctx, &mesh, DrawParam::default())?;

                let ch = self.board_pcs[y][x];

                if self.is_piece(ch) {
                    let img = match self.pieces.get(&ch) {
                        Some(i) => i,
                        None => continue,
                    };

                    let ddraw = (self.sq_size as f32 - img.width() as f32 * 1.5) / 2.;
                    let x_draw = x_sq as f32 + ddraw;
                    let y_draw = y_sq as f32 + ddraw;
                    let draw_param = DrawParam::new().dest([x_draw, y_draw]).scale([1.5, 1.5]);

                    graphics::draw(ctx, img, draw_param)?;
                }
            }
        }

        self.needs_draw = false;

        graphics::present(ctx)
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, btn: MouseButton, x: f32, y: f32) {
        let x = (x as i32 / self.sq_size) as u8;
        let y = (y as i32 / self.sq_size) as u8;

        match btn {
            MouseButton::Left => {
                if !self.moving {
                    self.select_piece(x, y);
                } else {
                    self.move_piece(x, y);
                }
            }

            _ => (),
        }
    }
}
