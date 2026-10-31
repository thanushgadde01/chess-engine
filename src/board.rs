use cozy_chess::{Board, Move};

#[derive(Clone)]
pub struct Position {
    board: Board,
}

impl Position {
    pub fn startpos() -> Self {
        Self {
            board: Board::default(),
        }
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        fen.parse::<Board>()
            .map(|board| Self { board })
            .map_err(|error| error.to_string())
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    pub fn play(&mut self, mv: Move) {
        self.board.play(mv);
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

        self.legal_moves()
            .into_iter()
            .map(|mv| {
                let mut next = self.clone();
                next.play(mv);
                next.perft(depth - 1)
            })
            .sum()
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
        let position =
            Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .unwrap();
        assert_eq!(position.perft(3), 97_862);
    }
}
