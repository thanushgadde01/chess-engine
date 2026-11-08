import subprocess
import sys

UNICODE_PIECES = {
    'P': '♟', 'N': '♞', 'B': '♝', 'R': '♜', 'Q': '♛', 'K': '♚',
    'p': '♙', 'n': '♘', 'b': '♗', 'r': '♖', 'q': '♕', 'k': '♔',
}

class SimpleBoard:
    def __init__(self):
        # 8x8 board: row 0 is rank 8, row 7 is rank 1
        self.grid = [
            list("rnbqkbnr"),
            list("pppppppp"),
            list("........"),
            list("........"),
            list("........"),
            list("........"),
            list("PPPPPPPP"),
            list("RNBQKBNR"),
        ]

    def render(self):
        print("\n    a   b   c   d   e   f   g   h")
        print("  +---+---+---+---+---+---+---+---+")
        for r_idx, row in enumerate(self.grid):
            rank = 8 - r_idx
            row_symbols = [UNICODE_PIECES.get(ch, '·') for ch in row]
            print(f"{rank} | " + " | ".join(row_symbols) + f" | {rank}")
            print("  +---+---+---+---+---+---+---+---+")
        print("    a   b   c   d   e   f   g   h\n")

    def apply_move(self, mv):
        if len(mv) < 4:
            return
        from_file = ord(mv[0].lower()) - ord('a')
        from_rank = 8 - int(mv[1])
        to_file = ord(mv[2].lower()) - ord('a')
        to_rank = 8 - int(mv[3])

        piece = self.grid[from_rank][from_file]
        self.grid[from_rank][from_file] = '.'

        # Promotion check
        if len(mv) == 5:
            promo = mv[4].upper() if piece.isupper() else mv[4].lower()
            self.grid[to_rank][to_file] = promo
        else:
            self.grid[to_rank][to_file] = piece

        # Handle simple castling visually
        if piece.lower() == 'k':
            if from_file == 4 and to_file == 6: # Kingside
                rook = self.grid[from_rank][7]
                self.grid[from_rank][7] = '.'
                self.grid[from_rank][5] = rook
            elif from_file == 4 and to_file == 2: # Queenside
                rook = self.grid[from_rank][0]
                self.grid[from_rank][0] = '.'
                self.grid[from_rank][3] = rook

def main():
    engine_path = r"D:\chess\tools\bin\engine-play.exe"
    print("==================================================")
    print("⚔️  Welcome to RustChess CLI Game!")
    print("==================================================")

    try:
        proc = subprocess.Popen(
            [engine_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )
    except Exception as e:
        print(f"Error launching engine: {e}")
        return

    # Handshake
    proc.stdin.write("uci\n")
    proc.stdin.flush()
    while True:
        line = proc.stdout.readline()
        if "uciok" in line:
            break

    proc.stdin.write("isready\n")
    proc.stdin.flush()
    while True:
        line = proc.stdout.readline()
        if "readyok" in line:
            break

    proc.stdin.write("ucinewgame\n")
    proc.stdin.flush()

    board = SimpleBoard()
    played_moves = []

    board.render()

    while True:
        try:
            user_move = input("Your move (White) [e.g. e2e4]: ").strip()
        except (KeyboardInterrupt, EOFError):
            break

        if user_move.lower() in ["quit", "exit"]:
            proc.stdin.write("quit\n")
            proc.stdin.flush()
            print("Game ended.")
            break
        if not user_move:
            continue

        played_moves.append(user_move)
        board.apply_move(user_move)
        board.render()

        moves_str = " ".join(played_moves)
        proc.stdin.write(f"position startpos moves {moves_str}\n")
        proc.stdin.write("go movetime 1000\n")
        proc.stdin.flush()

        best_move = None
        while True:
            line = proc.stdout.readline()
            if not line:
                break
            line_str = line.strip()
            if line_str.startswith("bestmove"):
                parts = line_str.split()
                if len(parts) >= 2:
                    best_move = parts[1]
                break

        if not best_move or best_move == "0000":
            print("Engine has no moves or checkmate!")
            break

        print(f"🤖 RustChess played: {best_move}")
        played_moves.append(best_move)
        board.apply_move(best_move)
        board.render()

    try:
        proc.terminate()
    except:
        pass

if __name__ == "__main__":
    main()
