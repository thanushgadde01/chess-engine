use cozy_chess::{Board, Color, Piece};

const VALUES: [i32; 6] = [100, 320, 330, 500, 900, 20_000];

pub fn evaluate(board: &Board) -> i32 {
    let white = material(board, Color::White);
    let black = material(board, Color::Black);
    let score = white - black;
    if board.side_to_move() == Color::White {
        score
    } else {
        -score
    }
}

fn material(board: &Board, color: Color) -> i32 {
    [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ]
    .iter()
    .enumerate()
    .map(|(index, piece)| board.colored_pieces(color, *piece).len() as i32 * VALUES[index])
    .sum()
}
