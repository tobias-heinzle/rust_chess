import argparse
import asyncio
from datetime import datetime
import sys
import logging
import os

import chess.pgn
from chess import Board, Move, WHITE, BLACK
import chess.engine
from wrapper import ChessEngineWrapper

from display import print_board

parser = argparse.ArgumentParser(
                    prog='selfplay',
                    description='Lets two uci chess engines duke it out',
                    epilog='Look at the source code to figure out what is going on! (it is simple)')

parser.add_argument('-e', '--engine', default='../target/release/rust_chess', required=False)   # option that takes a value
parser.add_argument('-t', '--movetime', default="0.5")      # option that takes a value
parser.add_argument('-c', '--color', default='white')
parser.add_argument('--no_book',action='store_true')
parser.add_argument('rest', nargs=argparse.REMAINDER)


args = parser.parse_args()

engine = ChessEngineWrapper()
engine.path = args.engine

color = args.color

book = not args.no_book

time_min = 0.1
time_limit = float(args.movetime)
assert(color in ['white', 'black'], 'color must be white or black')
assert(time_limit > time_min, f'movetime must be greater than {time_min}')

fen = None
if args.rest is not None:
    print(args.rest)
    fen = " ".join(args.rest)


logging.getLogger().setLevel(logging.DEBUG)

handler = logging.StreamHandler(sys.stdout)
handler.setLevel(logging.DEBUG)
formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
handler.setFormatter(formatter)

logger = logging.getLogger("chess.engine")
logger.addHandler(handler)

colors = {True : "White", False : "Black"}
engine_color = BLACK if color == "white" else WHITE

async def play() -> Board:
    name = engine.path.split('/')[-1]

    await engine.start()

    board = Board()

    if fen is not None:
        board = Board(fen)

    print_board(board, engine_color)

    try:
        while board.outcome(claim_draw=True) is None:

            if board.turn == engine_color:
                move = engine.choose_book_move(board) if book else None
                if move is None: 
                    move, result = await engine.analyze_position(board, time_limit)
                    print("score: ", result)
            else:
                move_str = input("Input move:")
                try:
                    if move_str == "0000":
                        board.pop()
                        board.pop()
                        print_board(board, engine_color)
                        continue
                    move = Move.from_uci(move_str)
                    if not move in board.legal_moves:
                        continue
                except:
                    continue
                
            board.push(move)
            
            print(f"You are {colors[not engine_color]};", name, "is", colors[engine_color])
            
            print_board(board, engine_color)
        

        
        await engine.quit()
        
        outcome = board.outcome(claim_draw = True)
        if outcome.winner == engine_color:
            print(f"win for {name}")
            
        elif outcome.winner == None:
            print("draw")
        else:
            print("win for player")

    finally:
        game = chess.pgn.Game.from_board(board)
        try:
            os.makedirs(f"games/{name}_vs_player")
        except:
            pass
        finally:
            with open(f"games/{name}_vs_player/game_{'_'.join(str(datetime.now()).split(' '))}.pgn", 'w') as file:
                file.write(str(game))

        quit()
    



asyncio.set_event_loop_policy(chess.engine.EventLoopPolicy())
asyncio.run(play())