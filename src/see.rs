use cozy_chess::{BitBoard, Board, Color, Move, Piece, Square};

/// Piece values used for Static Exchange Evaluation (standard centipawn scale).
pub const SEE_VALUES: [i32; 6] = [
    100,    // Pawn
    320,    // Knight
    330,    // Bishop
    500,    // Rook
    900,    // Queen
    20_000, // King
];

#[inline(always)]
pub fn piece_value(piece: Piece) -> i32 {
    SEE_VALUES[piece as usize]
}

/// Computes the Static Exchange Evaluation (SEE) score for a move on a board.
///
/// Returns the material balance gain or loss (in centipawns) if the capture sequence
/// on `mv.to` is fully played out by both sides using least-valuable attackers.
pub fn see(board: &Board, mv: Move) -> i32 {
    let to = mv.to;
    let from = mv.from;

    let target_piece = if let Some(p) = board.piece_on(to) {
        p
    } else {
        // En-passant capture
        if board.pieces(Piece::Pawn).has(from) && Some(to.file()) == board.en_passant() {
            Piece::Pawn
        } else {
            return 0;
        }
    };

    let mut gain = [0i32; 32];
    let mut d = 0;

    gain[d] = piece_value(target_piece);
    if let Some(promo) = mv.promotion {
        gain[d] += piece_value(promo) - piece_value(Piece::Pawn);
    }

    let from_sq = from;
    let mut current_piece = board.piece_on(from).unwrap_or(Piece::Pawn);
    let side_to_move = board.side_to_move();

    // Occupied bitboard that gets updated as pieces are removed from the square
    let mut occupied = board.occupied();
    occupied &= !from_sq.bitboard();

    // Initial capture by side_to_move
    let mut side = !side_to_move;

    // Direct and sliding ray attacks on `to`
    let mut attackers = all_attackers_to(board, to, occupied);

    while !attackers.is_empty() {
        d += 1;
        gain[d] = piece_value(current_piece) - gain[d - 1];

        // Pruning: if even taking the piece cannot improve current side's position, stop
        if (-gain[d - 1]).max(gain[d]) < 0 {
            break;
        }

        // Find the least valuable attacker for `side`
        if let Some((least_sq, least_piece)) = find_least_valuable_attacker(board, attackers, side, occupied) {
            current_piece = least_piece;
            occupied &= !least_sq.bitboard();

            // Diagonal and orthogonal sliding pieces may be revealed behind `from_sq`
            if least_piece == Piece::Pawn || least_piece == Piece::Bishop || least_piece == Piece::Queen {
                attackers |= cozy_chess::get_bishop_rays(to) & (board.pieces(Piece::Bishop) | board.pieces(Piece::Queen));
            }
            if least_piece == Piece::Rook || least_piece == Piece::Queen {
                attackers |= cozy_chess::get_rook_rays(to) & (board.pieces(Piece::Rook) | board.pieces(Piece::Queen));
            }
            attackers &= occupied;

            side = !side;
        } else {
            break;
        }
    }

    // Minimax back up the gain sequence
    while d > 0 {
        gain[d - 1] = -( (-gain[d - 1]).max(gain[d]) );
        d -= 1;
    }

    gain[0]
}

/// Returns a bitboard of all attackers to `target` under current `occupied` mask.
fn all_attackers_to(board: &Board, target: Square, occupied: BitBoard) -> BitBoard {
    let mut attackers = BitBoard::EMPTY;

    // Pawns: A white pawn attacks `target` if `get_pawn_attacks(sq, Color::White).has(target)`,
    // which is mathematically equivalent to `get_pawn_attacks(target, Color::Black).has(sq)`.
    let white_pawns = board.pieces(Piece::Pawn) & board.colors(Color::White);
    let from_white_pawn = cozy_chess::get_pawn_attacks(target, Color::Black);
    attackers |= white_pawns & from_white_pawn;

    let black_pawns = board.pieces(Piece::Pawn) & board.colors(Color::Black);
    let from_black_pawn = cozy_chess::get_pawn_attacks(target, Color::White);
    attackers |= black_pawns & from_black_pawn;

    // Knights
    let knights = board.pieces(Piece::Knight);
    attackers |= knights & cozy_chess::get_knight_moves(target);

    // Kings
    let kings = board.pieces(Piece::King);
    attackers |= kings & cozy_chess::get_king_moves(target);

    // Bishops and Queens (diagonal)
    let bishops_queens = board.pieces(Piece::Bishop) | board.pieces(Piece::Queen);
    let bishop_attacks = cozy_chess::get_bishop_moves(target, occupied);
    attackers |= bishops_queens & bishop_attacks;

    // Rooks and Queens (orthogonal)
    let rooks_queens = board.pieces(Piece::Rook) | board.pieces(Piece::Queen);
    let rook_attacks = cozy_chess::get_rook_moves(target, occupied);
    attackers |= rooks_queens & rook_attacks;

    attackers & occupied
}

/// Finds the least valuable attacker belonging to `color` among `attackers`.
fn find_least_valuable_attacker(
    board: &Board,
    attackers: BitBoard,
    color: Color,
    occupied: BitBoard,
) -> Option<(Square, Piece)> {
    let color_attackers = attackers & board.colors(color) & occupied;
    if color_attackers.is_empty() {
        return None;
    }

    // Check in ascending order of piece value: Pawn, Knight, Bishop, Rook, Queen, King
    for &piece in &[Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King] {
        let piece_subset = color_attackers & board.pieces(piece);
        if !piece_subset.is_empty() {
            let sq = piece_subset.next_square().unwrap();
            return Some((sq, piece));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use cozy_chess::Move;

    #[test]
    fn test_simple_pawn_capture() {
        // e4 pawn defended by d3, black pawn on d5
        let board = "rnbqkbnr/ppp1pppp/8/3p4/4P3/3P4/PPP2PPP/RNBQKBNR b KQkq - 0 2".parse::<Board>().unwrap();
        let mv = Move {
            from: Square::D5,
            to: Square::E4,
            promotion: None,
        };
        // Black takes on e4, White retakes with dxe4: even trade (0)
        let score = see(&board, mv);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_hanging_piece_capture() {
        // Black queen hanging on d4
        let board = "rnb1kbnr/pppppppp/8/8/3q4/4P3/PPPP1PPP/RNBQKBNR w KQkq - 0 1".parse::<Board>().unwrap();
        let mv = Move {
            from: Square::E3,
            to: Square::D4,
            promotion: None,
        };
        // White takes queen for free
        let score = see(&board, mv);
        assert!(score >= 900);
    }

    #[test]
    fn test_losing_exchange() {
        // Rook on d1 attacking pawn on d7 defended by queen on d8, with kings on g1/g8
        let board = "3q2k1/3p4/8/8/8/8/8/3R2K1 w - - 0 1".parse::<Board>().unwrap();
        let mv = Move {
            from: Square::D1,
            to: Square::D7,
            promotion: None,
        };
        // Rook takes pawn (+100), queen takes rook (-500) -> net -400
        let score = see(&board, mv);
        assert_eq!(score, -400);
    }
}
