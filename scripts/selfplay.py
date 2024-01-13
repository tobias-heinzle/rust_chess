import argparse
import asyncio
import datetime
import json
import sys
import logging

from chess import Board, WHITE, BLACK
import chess.engine
from wrapper import ChessEngineWrapper

parser = argparse.ArgumentParser(
                    prog='selfplay',
                    description='Lets two uci chess engines duke it out',
                    epilog='Look at the source code to figure out what is going on! (it is simple)')

parser.add_argument('-a', '--engine_a', default='../target/release/rust_chess', required=False)
parser.add_argument('-b', '--engine_b', default='../target/release/rust_chess', required=False)           # positional argument
parser.add_argument('-n', '--n_games', default="1")      # option that takes a value
parser.add_argument('-t', '--time_per_move', default="0.5")      # option that takes a value
parser.add_argument('-v', '--verbose',
                    action='store_true')
parser.add_argument('-vv', '--very_verbose',
                    action='store_true')

args = parser.parse_args()

engine_a = ChessEngineWrapper()
engine_a.path = args.engine_a

engine_b = ChessEngineWrapper()
engine_b.path = args.engine_b

very_verbose = args.very_verbose
verbose = args.verbose or very_verbose


time_limit = float(args.time_per_move)

n = int(args.n_games)

assert(n > 0)
assert(time_limit > 0)

root = logging.getLogger()
if very_verbose:
    root.setLevel(logging.DEBUG)

handler = logging.StreamHandler(sys.stdout)
handler.setLevel(logging.DEBUG)
formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
handler.setFormatter(formatter)

logger = logging.getLogger("chess.engine")
logger.addHandler(handler)


async def selfplay_loop():
    name_a = engine_a.path.split('/')[-1]
    name_b = engine_b.path.split('/')[-1]
    stats = {"time_limit" : time_limit,
             "date" : str(datetime.datetime.now()).split('.')[0],
             "engine_a" : name_a,
             "engine_b" : name_b,
             "statistics" : {
                 "wins_a" : 0,
                 "wins_b" : 0,
                 "draws" : 0,
                 "total" : 0,
             },
            }
    
    if verbose:
        print(stats['date'])

    game = 0
    while True:

        await engine_a.start()
        await engine_b.start()

        board = Board()

        engine_a_color = WHITE if game % 2 == 1 else BLACK

        if very_verbose:
            print(board)

        while board.outcome(claim_draw=True) is None:

            if board.turn == engine_a_color:
                move = engine_a.choose_book_move(board)
                result = "book_move"
                if move is None: 
                    move, result = await engine_a.analyze_position(board, time_limit)
            else:
                move = engine_b.choose_book_move(board)
                result = "book_move"
                if move is None: 
                    move, result = await engine_b.analyze_position(board, time_limit)
            
            board.push(move)
            
            if verbose:
                print(f"Game {game}: {move} -- {result}")
            
            if very_verbose:
                print(name_a, "is", engine_a_color, ", opponent is", name_b)
                print(board)
        
        outcome = board.outcome(claim_draw = True)
        winner = None
        if outcome.winner == engine_a_color:
            if verbose:
                print(f"Game {game}: win for {name_a}")
            winner = name_a
            stats['statistics']['wins_a'] += 1
        elif outcome.winner == None:
            if verbose:
                print(f"Game {game}: draw")
            stats['statistics']['draws'] += 1
        else:
            if verbose:
                print(f"Game {game}: win for {name_b}")
            winner = name_b
            stats['statistics']['wins_b'] += 1
        stats['statistics']['total'] += 1

        if verbose:
            print(f"Game {game}: {outcome}")
        
        await engine_a.quit()
        await engine_b.quit()

    
        game += 1
        if game > n:
            break
        
        print(f"Game {game + 1}/{n} done - outcome: {'win for ' + winner if winner is not None else 'draw' }")
        
        with open(f"results/{name_a}_vs_{name_b}_{'_'.join(stats['date'].split(' '))}.json", 'w') as file:
            json.dump(stats, file)

asyncio.set_event_loop_policy(chess.engine.EventLoopPolicy())
asyncio.run(selfplay_loop())