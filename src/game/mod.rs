use std::collections::HashMap;
use std::fmt;

use ggez::{
    event::{EventHandler, MouseButton},
    graphics::{self, Color, DrawMode, DrawParam, Image},
    input::mouse,
    Context, GameResult,
};

use std::vec::Vec;

use crate::WIN_HEIGHT;
use crate::WIN_WIDTH;

#[derive(Clone, Copy)]
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

enum Castling {
    QueenSide,
    KingSide,
}

struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

struct BoardState {
    board: [[char; 8]; 8],
    player: Player,
    wk_pos: (u8, u8),
    bk_pos: (u8, u8),
    enp_b: u8,
    enp_w: u8,
    castling: u8,
    b_check: bool,
    w_check: bool,
}

impl std::clone::Clone for BoardState {
    fn clone(&self) -> Self {
        Self {
            board: self.board.clone(),
            player: self.player,
            wk_pos: self.wk_pos,
            bk_pos: self.bk_pos,
            enp_b: self.enp_b,
            enp_w: self.enp_w,
            castling: self.castling,
            b_check: self.b_check,
            w_check: self.w_check,
        }
    }
}

impl fmt::Display for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = String::from("[\n");

        for row in self.board.iter() {
            out.push_str(&format!("\t{:?}\n", row));
        }

        out.push_str("]");

        write!(f, "{}", out)
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
    moving: bool,
    needs_draw: bool,
    enp_b: u8,
    enp_w: u8,
    w_king_pos: (u8, u8),
    b_king_pos: (u8, u8),
    castling: u8,
    w_check: bool,
    b_check: bool,
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
            moving: false,
            needs_draw: true,
            enp_b: 0,
            enp_w: 0,
            w_king_pos: (4, 7),
            b_king_pos: (4, 0),
            castling: 0b1111,
            w_check: false,
            b_check: false,
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

    /* Function that returns the current state of the board
     */
    fn get_board_state(&self) -> BoardState {
        BoardState {
            board: self.board_pcs.clone(),
            player: self.turn,
            wk_pos: self.w_king_pos,
            bk_pos: self.b_king_pos,
            castling: self.castling,
            enp_b: self.enp_b,
            enp_w: self.enp_w,
            w_check: self.w_check,
            b_check: self.b_check,
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
    fn is_opponent(plyr: Player, ch: char) -> bool {
        match plyr {
            Player::White => Self::is_black_piece(ch),
            Player::Black => Self::is_white_piece(ch),
        }
    }

    /* Checks if a given character is a piece
     */
    fn is_piece(ch: char) -> bool {
        ch != '-'
    }

    fn move_piece_to(from: Point<u8>, to: Point<u8>, state: &mut BoardState) {
        let x = from.x as usize;
        let y = from.y as usize;

        let ch = state.board[y][x];
        if ch == 'K' {
            state.wk_pos = (to.x, to.y);
        } else if ch == 'k' {
            state.bk_pos = (to.x, to.y);
        }

        state.enp_b = 0;
        state.enp_w = 0;

        match ch {
            'K' => {
                state.wk_pos = (to.x, to.y);
            }

            'k' => {
                state.bk_pos = (to.x, to.y);
            }

            'p' => {
                if from.y == 1 && to.y == 3 {
                    state.enp_b = 0x80 >> from.x;
                } else if from.y == 4 && from.x != to.x && state.board[5][to.x as usize] == '-' {
                    state.board[4][to.x as usize] = '-';
                }
            }

            'P' => {
                if from.y == 6 && to.y == 4 {
                    state.enp_w = 0x80 >> from.x;
                } else if from.y == 3 && from.x != to.x && state.board[2][to.x as usize] == '-' {
                    state.board[3][to.x as usize] = '-';
                }
            }

            _ => (),
        }

        state.board[to.y as usize][to.x as usize] = ch;
        state.board[y][x] = '-';

        if Self::is_white_piece(ch) && Self::check_for_checks(Player::Black, state) {
            state.b_check = true;
        } else if Self::is_black_piece(ch) && Self::check_for_checks(Player::White, state) {
            state.w_check = true;
        }
    }

    /* Takes a dx and dy that specifies a line of path.
     * All squares along this path that does not have a piece
     * are by default added to the list of moves. If a piece is encountered,
     * a check is performed on the type. If it's an opponent piece, the piece
     * square is added to the list of possible moves, else not
     */
    fn get_line_moves(pos: &Point<u8>, dpos: Point<i8>, state: &BoardState) -> Vec<(u8, u8)> {
        let mut m_x = pos.x as i8 + dpos.x;
        let mut m_y = pos.y as i8 + dpos.y;

        let mut moves = Vec::<(u8, u8)>::with_capacity(7);

        while m_x >= 0 && m_x < 8 && m_y >= 0 && m_y < 8 {
            let ch = state.board[m_y as usize][m_x as usize];

            if Self::is_piece(ch) {
                if Self::is_opponent(state.player, ch) {
                    moves.push((m_x as u8, m_y as u8));
                }

                break;
            }

            moves.push((m_x as u8, m_y as u8));
            m_x += dpos.x;
            m_y += dpos.y;
        }

        moves
    }

    /* Takes a position and and pushes into the move vector
     * all the moves that a pawn at that position can make
     */
    fn mv_pawn(pos: Point<u8>, state: &BoardState) -> Vec<(u8, u8)> {
        let x_i = pos.x as usize;
        let y_i = pos.y as usize;

        let mut moves = Vec::<(u8, u8)>::with_capacity(4);

        match state.player {
            Player::White => {
                if pos.y == 0 {
                    return moves;
                }

                if !Self::is_piece(state.board[y_i - 1][x_i]) {
                    moves.push((pos.x, pos.y - 1));
                }

                if pos.y == 6 && !Self::is_piece(state.board[y_i - 2][x_i]) {
                    moves.push((pos.x, pos.y - 2));
                }

                if pos.x < 7 && Self::is_opponent(state.player, state.board[y_i - 1][x_i + 1]) {
                    moves.push((pos.x + 1, pos.y - 1));
                }

                if pos.x > 0 && Self::is_opponent(state.player, state.board[y_i - 1][x_i - 1]) {
                    moves.push((pos.x - 1, pos.y - 1));
                }
            }

            Player::Black => {
                if pos.y == 7 {
                    return moves;
                }

                if !Self::is_piece(state.board[y_i + 1][x_i]) {
                    moves.push((pos.x, pos.y + 1));
                }

                if pos.y == 1 && !Self::is_piece(state.board[y_i + 2][x_i]) {
                    moves.push((pos.x, pos.y + 2));
                }

                if pos.x < 7 && Self::is_opponent(state.player, state.board[y_i + 1][x_i + 1]) {
                    moves.push((pos.x + 1, pos.y + 1));
                }

                if pos.x > 0 && Self::is_opponent(state.player, state.board[y_i + 1][x_i - 1]) {
                    moves.push((pos.x - 1, pos.y + 1));
                }
            }
        }

        moves
    }

    /* Used for moving a knight. Unique function cuz
     * knights make a 2.5 move.
     */
    fn mv_knight(pos: Point<u8>, state: &BoardState) -> Vec<(u8, u8)> {
        let x_m = pos.x as i8;
        let y_m = pos.y as i8;

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
                let ch = state.board[pos_y as usize][pos_x as usize];
                if !Self::is_piece(ch) || Self::is_opponent(state.player, ch) {
                    poss_moves.push((pos_x as u8, pos_y as u8));
                }
            }
        }

        poss_moves
    }

    /* Used for moving a bishop
     */
    fn mv_bishop(pos: Point<u8>, state: &BoardState) -> Vec<(u8, u8)> {
        let mut moves = Vec::<(u8, u8)>::with_capacity(13);
        moves.append(&mut Self::get_line_moves(&pos, Point::new(1, 1), state));
        moves.append(&mut Self::get_line_moves(&pos, Point::new(1, -1), state));
        moves.append(&mut Self::get_line_moves(&pos, Point::new(-1, -1), state));
        moves.append(&mut Self::get_line_moves(&pos, Point::new(-1, 1), state));
        moves
    }

    /* Used for moving a Rook
     */
    fn mv_rook(pos: Point<u8>, state: &BoardState) -> Vec<(u8, u8)> {
        let mut moves = Vec::<(u8, u8)>::with_capacity(14);
        for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
            moves.append(&mut Self::get_line_moves(&pos, Point::new(*dx, *dy), state));
        }

        moves
    }

    /* Used for moving a Queen
     */
    fn mv_queen(pos: Point<u8>, state: &BoardState) -> Vec<(u8, u8)> {
        let mut moves = Vec::<(u8, u8)>::with_capacity(28);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                moves.append(&mut Self::get_line_moves(&pos, Point::new(dx, dy), state));
            }
        }

        moves
    }

    /* Used for moving a King
     */
    fn mv_king(pos: Point<u8>, state: &BoardState) -> Vec<(u8, u8)> {
        let mut moves = Vec::<(u8, u8)>::with_capacity(8);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if pos.x == 0 && pos.y == 0 {
                    continue;
                }

                let x_m = pos.x as i8 + dx;
                let y_m = pos.y as i8 + dy;

                if x_m >= 0 && x_m < 8 && y_m >= 0 && y_m < 8 {
                    let ch = state.board[y_m as usize][x_m as usize];
                    if !Self::is_piece(ch) || Self::is_opponent(state.player, ch) {
                        moves.push((x_m as u8, y_m as u8));
                    }
                }
            }
        }

        moves
    }

    /* Takes a piece and a position and returns all possible moves for the piece.
     */
    fn get_piece_moves(ch: char, pos: Point<u8>, state: &BoardState) -> Vec<(u8, u8)> {
        match ch {
            'p' | 'P' => Self::mv_pawn(pos, state),
            'r' | 'R' => Self::mv_rook(pos, state),
            'n' | 'N' => Self::mv_knight(pos, state),
            'b' | 'B' => Self::mv_bishop(pos, state),
            'q' | 'Q' => Self::mv_queen(pos, state),
            'k' | 'K' => Self::mv_king(pos, state),
            _ => Vec::<(u8, u8)>::new(),
        }
    }

    /* This function is called when the player clicks the board and when
     * a move is currently not in progress. The position of the clicked
     * square is passed to the function. If the player has clicked on one
     * of their own pieces, the piece's moves are added to self.moves,
     * the colors of the move squares are changed, the color of the square
     * under the piece is changed, the game state is set to 'moving' and the
     * info about the current piece is stored.
     */
    fn select_piece(&mut self, x: u8, y: u8) {
        let ch = self.board_pcs[y as usize][x as usize];

        if !Self::is_piece(ch) || Self::is_opponent(self.turn, ch) {
            return;
        }

        let board_state = self.get_board_state();

        let moves = Self::get_piece_moves(ch, Point::new(x, y), &board_state);

        self.current = Some(ch);
        self.current_pos = Some((x, y));

        //println!("{:?}", moves);

        for (m_x, m_y) in &moves {
            let mut state = board_state.clone();
            Self::move_piece_to(Point::new(x, y), Point::new(*m_x, *m_y), &mut state);
            //println!("{}", state);
            let checked = Self::check_for_checks(self.turn, &mut state);
            //println!("{}", checked);
            if checked {
                //println!("checked");
                continue;
            }
            self.board[*m_y as usize][*m_x as usize] = Color::from_rgb(200, 200, 0);

            self.moves.push((*m_x, *m_y));
        }

        //println!("MOVE END");

        self.board[y as usize][x as usize] = Color::from_rgb(255, 85, 85);

        self.needs_draw = true;
        self.moving = true;
    }

    fn move_piece(&mut self, x: u8, y: u8) -> bool {
        if self.moves.contains(&(x, y)) {
            let piece = self.current.unwrap();
            match piece {
                'K' => self.w_king_pos = (x, y),
                'k' => self.b_king_pos = (x, y),
                _ => (),
            }
            self.board_pcs[y as usize][x as usize] = piece;
            let curr = self.current_pos.unwrap();
            self.board_pcs[curr.1 as usize][curr.0 as usize] = '-';
            self.current = None;
            self.current_pos = None;
            self.moving = false;
            self.turn = self.turn.switch();
            self.moves.clear();
            self.needs_draw = true;
            self.reset_board();

            if Self::check_for_checkmate(self.turn, &self.get_board_state()) {
                return true;
            } else {
                return false;
            }
        }

        let ch = self.board_pcs[y as usize][x as usize];

        if Self::is_piece(ch) && !Self::is_opponent(self.turn, ch) {
            self.moves.clear();
            self.reset_board();
            self.select_piece(x, y);
            self.needs_draw = true;
        }

        false
    }

    fn check_for_checks(plyr: Player, state: &mut BoardState) -> bool {
        state.player = plyr.switch();
        for y in 0..8 {
            for x in 0..8 {
                let ch = state.board[y as usize][x as usize];

                let is_valid_piece = match plyr {
                    Player::White => Self::is_black_piece(ch),
                    Player::Black => Self::is_white_piece(ch),
                };

                if !is_valid_piece {
                    continue;
                }

                let k_pos = match plyr {
                    Player::White => &state.wk_pos,
                    Player::Black => &state.bk_pos,
                };

                if Self::get_piece_moves(ch, Point::new(x, y), state).contains(k_pos) {
                    state.player = plyr;
                    return true;
                }
            }
        }

        state.player = plyr;
        false
    }

    fn check_for_checkmate(plyr: Player, state: &BoardState) -> bool {
        for y in 0..8 {
            for x in 0..8 {
                let ch = state.board[y as usize][x as usize];

                let is_valid_piece = match plyr {
                    Player::White => Self::is_white_piece(ch),
                    Player::Black => Self::is_black_piece(ch),
                };

                if !is_valid_piece {
                    continue;
                }

                for (m_x, m_y) in Self::get_piece_moves(ch, Point::new(x, y), state) {
                    let mut state_ = state.clone();
                    Self::move_piece_to(Point::new(x, y), Point::new(m_x, m_y), &mut state_);

                    if !Self::check_for_checks(plyr, &mut state_) {
                        return false;
                    }
                }
            }
        }
        true
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

                if Self::is_piece(ch) {
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
                    let mated = self.move_piece(x, y);

                    if mated {
                        ggez::event::quit(ctx);
                    }
                }
            }

            _ => (),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn check() {}
}
