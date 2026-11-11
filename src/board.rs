use cozy_chess::{Board, Move};

#[derive(Clone, Debug)]
pub struct Position {
    board: Board,
    history: Vec<u64>,
}

impl Position {
    pub fn startpos() -> Self {
        let board = Board::default();
        let hash = board.hash();
        Self {
            board,
            history: vec![hash],
        }
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let board = fen.parse::<Board>().map_err(|error| error.to_string())?;
        let hash = board.hash();
        Ok(Self {
            board,
            history: vec![hash],
        })
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    pub fn history(&self) -> &[u64] {
        &self.history
    }

    pub fn play(&mut self, mv: Move) {
        self.board.play(mv);
        self.history.push(self.board.hash());
    }

    pub fn is_repetition(&self) -> bool {
        if self.history.len() < 2 {
            return false;
        }
        let current_hash = *self.history.last().unwrap();
        // Look back over history (at matching parity of turns)
        let count = self.history.iter().filter(|&&h| h == current_hash).count();
        count >= 3
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        self.board.generate_moves(|piece_moves| {
            moves.extend(piece_moves);
            false
        });
        moves
    }

    pub fn perft(&self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut count = 0;
        self.board.generate_moves(|piece_moves| {
            for mv in piece_moves {
                let mut next = self.board.clone();
                next.play_unchecked(mv);
                let next_pos = Position {
                    board: next,
                    history: Vec::new(),
                };
                count += next_pos.perft(depth - 1);
            }
            false
        });
        count
    }

    pub fn perft_divide(&self, depth: u8) -> Vec<(Move, u64)> {
        if depth == 0 {
            return Vec::new();
        }

        let mut results = Vec::new();
        self.board.generate_moves(|piece_moves| {
            for mv in piece_moves {
                let mut next = self.board.clone();
                next.play_unchecked(mv);
                let next_pos = Position {
                    board: next,
                    history: Vec::new(),
                };
                let nodes = next_pos.perft(depth - 1);
                results.push((mv, nodes));
            }
            false
        });
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_position_perft_depth_four() {
        assert_eq!(Position::startpos().perft(4), 197_281);
    }

    #[test]
    fn kiwipete_perft_depth_three() {
        let position = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();
        assert_eq!(position.perft(3), 97_862);
    }
}
