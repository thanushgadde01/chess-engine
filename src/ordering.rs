use cozy_chess::{Board, Move, Piece};

pub const MAX_PLY: usize = 64;

pub struct MoveOrdering {
    pub killers: [[Option<Move>; 2]; MAX_PLY],
    pub history: [[[i32; 64]; 64]; 2], // [color][from][to]
    pub countermoves: [[[Option<Move>; 64]; 64]; 2], // [color][prev_from][prev_to]
}

impl MoveOrdering {
    pub fn new() -> Self {
        Self {
            killers: [[None; 2]; MAX_PLY],
            history: [[[0; 64]; 64]; 2],
            countermoves: [[[None; 64]; 64]; 2],
        }
    }

    pub fn clear(&mut self) {
        self.killers = [[None; 2]; MAX_PLY];
        self.history = [[[0; 64]; 64]; 2];
        self.countermoves = [[[None; 64]; 64]; 2];
    }

    pub fn record_countermove(&mut self, board: &Board, prev_move: Option<Move>, mv: Move) {
        if let Some(prev) = prev_move {
            let color = board.side_to_move() as usize;
            self.countermoves[color][prev.from as usize][prev.to as usize] = Some(mv);
        }
    }

    pub fn record_killer(&mut self, mv: Move, ply: usize) {
        if ply >= MAX_PLY {
            return;
        }
        if self.killers[ply][0] != Some(mv) {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = Some(mv);
        }
    }

    pub fn record_history(&mut self, board: &Board, mv: Move, depth: u8) {
        let color = board.side_to_move() as usize;
        let from = mv.from as usize;
        let to = mv.to as usize;
        let bonus = (depth as i32) * (depth as i32);
        // Cap history score to avoid overflow
        if self.history[color][from][to].abs() < 16_000 {
            self.history[color][from][to] += bonus;
        }
    }

    pub fn score_move(
        &self,
        board: &Board,
        mv: Move,
        tt_move: Option<Move>,
        prev_move: Option<Move>,
        ply: usize,
    ) -> i32 {
        // 1. TT move has the highest priority
        if Some(mv) == tt_move {
            return 1_000_000;
        }

        let enemy = !board.side_to_move();
        let is_capture = board.colors(enemy).has(mv.to);
        let target_piece = board.piece_on(mv.to);
        let attacker_piece = board.piece_on(mv.from);

        // 2. Captures scored by MVV-LVA
        if is_capture || mv.promotion.is_some() {
            let victim_val = if let Some(p) = target_piece {
                piece_val(p)
            } else {
                100 // En-passant pawn
            };
            let attacker_val = attacker_piece.map(piece_val).unwrap_or(100);
            let promo_val = if mv.promotion.is_some() { 800 } else { 0 };
            return 100_000 + (victim_val * 10 - attacker_val) + promo_val;
        }

        // 3. Killer moves
        if ply < MAX_PLY {
            if self.killers[ply][0] == Some(mv) {
                return 90_000;
            }
            if self.killers[ply][1] == Some(mv) {
                return 80_000;
            }
        }

        // 4. Counter-move heuristic
        if let Some(prev) = prev_move {
            let color = board.side_to_move() as usize;
            if self.countermoves[color][prev.from as usize][prev.to as usize] == Some(mv) {
                return 70_000;
            }
        }

        // 5. History heuristic
        let color = board.side_to_move() as usize;
        let hist = self.history[color][mv.from as usize][mv.to as usize];
        hist
    }

    pub fn order_moves(
        &self,
        board: &Board,
        moves: &mut [Move],
        tt_move: Option<Move>,
        prev_move: Option<Move>,
        ply: usize,
    ) {
        if moves.len() <= 1 {
            return;
        }

        // Use stack buffer for sorting up to 256 moves (max legal chess moves in any position is 218)
        let mut scores = [(0i32, Move { from: cozy_chess::Square::A1, to: cozy_chess::Square::A1, promotion: None }); 256];
        let n = moves.len().min(256);

        for i in 0..n {
            let m = moves[i];
            scores[i] = (self.score_move(board, m, tt_move, prev_move, ply), m);
        }

        scores[..n].sort_unstable_by(|a, b| b.0.cmp(&a.0));

        for i in 0..n {
            moves[i] = scores[i].1;
        }
    }
}

fn piece_val(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 20_000,
    }
}
