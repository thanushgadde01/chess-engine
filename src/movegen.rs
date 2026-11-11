use cozy_chess::{Board, Color, Move, Piece, Square};

/// Converts an engine move (cozy-chess Move) to standard UCI move string.
/// In cozy-chess, castling moves have from=King square and to=Rook square (e.g. e1h1 for O-O, e1a1 for O-O-O).
/// Standard UCI expects e1g1 for White O-O, e1c1 for White O-O-O, e8g8 for Black O-O, e8c8 for Black O-O-O.
pub fn move_to_uci(board: &Board, mv: Move) -> String {
    let piece = board.piece_on(mv.from);
    if piece == Some(Piece::King) {
        match (board.side_to_move(), mv.from, mv.to) {
            // White kingside
            (Color::White, Square::E1, Square::H1) => return "e1g1".to_string(),
            // White queenside
            (Color::White, Square::E1, Square::A1) => return "e1c1".to_string(),
            // Black kingside
            (Color::Black, Square::E8, Square::H8) => return "e8g8".to_string(),
            // Black queenside
            (Color::Black, Square::E8, Square::A8) => return "e8c8".to_string(),
            _ => {}
        }
    }

    // Default formatting (e.g. e2e4, e7e8q)
    mv.to_string()
}

/// Parses a UCI move string (e.g. "e2e4", "e7e8q", "e1g1", "e1h1") into a legal cozy-chess Move.
pub fn parse_uci_move(board: &Board, uci_str: &str) -> Result<Move, String> {
    let trimmed = uci_str.trim();
    if trimmed.len() < 4 {
        return Err(format!("Move string too short: {trimmed}"));
    }

    // Check if user input is standard UCI castling notation
    let is_king = match (board.side_to_move(), trimmed) {
        (Color::White, "e1g1") => Some((Square::E1, Square::H1)),
        (Color::White, "e1c1") => Some((Square::E1, Square::A1)),
        (Color::Black, "e8g8") => Some((Square::E8, Square::H8)),
        (Color::Black, "e8c8") => Some((Square::E8, Square::A8)),
        _ => None,
    };

    if let Some((from, to)) = is_king {
        if board.piece_on(from) == Some(Piece::King) {
            let castling_mv = Move {
                from,
                to,
                promotion: None,
            };
            if board.is_legal(castling_mv) {
                return Ok(castling_mv);
            }
        }
    }

    // Otherwise, parse standard Move string (e.g. "e2e4", "e7e8q" or cozy-chess format "e1h1")
    let mv: Move = trimmed
        .parse::<Move>()
        .map_err(|_| format!("Invalid move notation: {trimmed}"))?;

    if board.is_legal(mv) {
        Ok(mv)
    } else {
        Err(format!("Illegal move in position: {trimmed}"))
    }
}

