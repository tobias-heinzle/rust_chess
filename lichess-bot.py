from requests_oauthlib import OAuth2Session
import asyncio
import berserk
import threading
import chess
from chess import engine
from time import time, sleep


limit = 1
board = chess.Board()

async def analyze_position(position: chess.Board, engine: engine.Protocol, time_limit_seconds: float = 0.1) -> (str, dict):
    start = time()
    bestmove = ""
    result = {}
    with await engine.analysis(position) as analysis:

        while (time() - start) < limit:
            sleep(0.05)

        analysis.stop()

        bestmove  = await analysis.wait()
        result = analysis.info
    
    return (bestmove.move, result)

token = ""
with open('./lichess.token') as f:
    token = f.read()

session = berserk.TokenSession(token)
client = berserk.Client(session)

def should_accept(challenge_event) -> bool:
    with open("allowed.challengers") as file:

        allowed_challengers = file.readlines()
        if challenge_event["challenge"]["challenger"]["id"] in allowed_challengers:
            return True
        return False
    
async def main_loop():
    is_polite = True

    for event in client.bots.stream_incoming_events():

        if event['type'] == 'challenge':
            if should_accept(event):
                print("Accepted challenge of " + event["challenge"]["challenger"]["id"])
                client.bots.accept_challenge(event["challenge"]['id'])

            elif is_polite:

                client.bots.decline_challenge(event["challenge"]['id'])

        elif event['type'] == 'gameStart':
            print(event)

            game_id = event["game"]["gameId"]
            color = event["game"]['color']
            fen = event["game"]["fen"]

            await play_game(client,  game_id, color, fen)

async def bot_move(board, engine, client, game_id):
    move, result = await analyze_position(board, engine, limit)

    client.bots.make_move(game_id, str(move))

    print("ID: " + str(game_id) + " info: " + str(move) + " " + str(result))


async def play_game(client, game_id, color, fen):
    stream = client.bots.stream_game_state(game_id)
    current_state = next(stream)

    print(current_state)

    board = chess.Board()
    board.set_fen(fen)
    print("game started!")

    skip_value = 0
    transport, engine = await chess.engine.popen_uci("./target/release/rust_chess")

    if color == "white":
        skip_value = 1
        await bot_move(board, engine, client, game_id)


    for event in stream:
        print(event)

        if event['type'] == 'gameState':
            if event['status'] != 'started':
                break

            board = chess.Board()

            moves_played = event['moves'].split(" ")

            if len(moves_played) % 2 == skip_value:
                continue

            for move in moves_played:
                board.push(chess.Move.from_uci(move))

            await bot_move(board, engine, client, game_id)

    await engine.quit()


            

asyncio.run(main_loop())

quit()