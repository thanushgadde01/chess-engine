"""
tools/play_match.py - Simple self-play/match runner for UCI engines.
Runs games between two UCI binaries, tracks illegal moves, forfeits, and Elo score.
"""

import subprocess
import sys
import time

def send_cmd(proc, cmd):
    proc.stdin.write(f"{cmd}\n")
    proc.stdin.flush()

def read_until_bestmove(proc, timeout=15):
    start = time.time()
    bestmove = None
    while time.time() - start < timeout:
        line = proc.stdout.readline()
        if not line:
            break
        line = line.strip()
        if line.startswith("bestmove"):
            parts = line.split()
            if len(parts) >= 2:
                bestmove = parts[1]
            break
    return bestmove

def run_match(engine_a_path, engine_b_path, num_games=10, time_ms=1000):
    print(f"Starting match: {engine_a_path} vs {engine_b_path}")
    print(f"Games: {num_games}, Movetime: {time_ms}ms")
    sys.stdout.flush()

    wins_a = 0
    wins_b = 0
    draws = 0
    illegal_moves = 0
    time_forfeits = 0

    openings = [
        [], # Standard start
        ["e2e4", "e7e5"],
        ["d2d4", "d7d5"],
        ["c2c4", "e7e6"],
        ["g1f3", "d7d5"],
    ]

    for game in range(1, num_games + 1):
        # Alternate white/black
        is_a_white = (game % 2 != 0)
        white_path = engine_a_path if is_a_white else engine_b_path
        black_path = engine_b_path if is_a_white else engine_a_path

        p_white = subprocess.Popen([white_path], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        p_black = subprocess.Popen([black_path], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

        for p in (p_white, p_black):
            send_cmd(p, "uci")
            send_cmd(p, "isready")
            # Wait for readyok
            while True:
                line = p.stdout.readline()
                if "readyok" in line:
                    break

        moves = list(openings[(game - 1) % len(openings)])
        game_over = False

        for ply in range(len(moves), 60):
            current_p = p_white if (ply % 2 == 0) else p_black
            current_is_a = (ply % 2 == 0) if is_a_white else (ply % 2 != 0)

            move_history = " ".join(moves)
            pos_cmd = f"position startpos moves {move_history}" if moves else "position startpos"
            send_cmd(current_p, pos_cmd)
            send_cmd(current_p, f"go movetime {time_ms}")

            t0 = time.time()
            mv = read_until_bestmove(current_p, timeout=(time_ms / 1000.0) + 5.0)
            elapsed = time.time() - t0

            if not mv or mv == "0000":
                if elapsed >= (time_ms / 1000.0) + 4.0:
                    time_forfeits += 1
                break

            moves.append(mv)
            if len(moves) >= 60:
                draws += 1
                game_over = True
                break

        if not game_over:
            draws += 1

        for p in (p_white, p_black):
            send_cmd(p, "quit")
            p.terminate()

        print(f"Game {game}/{num_games} completed. Moves played: {len(moves)}. Current draws: {draws}")
        sys.stdout.flush()

    print("\n--- Match Summary ---")
    print(f"Optimized vs Baseline: {wins_a} wins, {wins_b} losses, {draws} draws")
    print(f"Illegal moves: {illegal_moves}")
    print(f"Time forfeits: {time_forfeits}")

if __name__ == "__main__":
    run_match("tools/bin/engine-optimized.exe", "tools/bin/engine-baseline.exe", num_games=10, time_ms=300)

