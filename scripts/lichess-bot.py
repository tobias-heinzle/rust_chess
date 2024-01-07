import asyncio
from random import sample
from time import sleep
from datetime import datetime
from requests.exceptions import ChunkedEncodingError
from urllib3.exceptions import InvalidChunkLength, ProtocolError

import berserk
import chess
import chess.polyglot

from wrapper import ChessEngineWrapper

RATED_SPEEDS = ["bullet", "blitz", "rapid"]

            
class Game():
    speed_limit = { "bullet" : 0.2,
                    "blitz" : 1.0,
                    "rapid" : 5.0,
                    "classical" : 10.0,
                    'correspondence' : 60.0}
    
    def __init__(self, client, event):
        self.game_id = event["game"]["gameId"]
        color = event["game"]['color']
        speed = event["game"]["speed"]
        self.fen = event["game"]["fen"]
        self.is_my_turn = event["game"]["isMyTurn"]
        self.client = client
        self.color = 1 if color == "white" else 0
        self.board = chess.Board()
        self.board.set_fen(self.fen)
        self.book_move_time = 1.0
        if speed in Game.speed_limit.keys():
            self.book_move_time = Game.speed_limit[speed]

    async def start(self) -> None:
        print("Game started: " + self.game_id)
        engine = ChessEngineWrapper()
        await engine.start()

        stream = self.client.bots.stream_game_state(self.game_id)
        current_state = next(stream)

        if self.is_my_turn:
            await self.bot_move(engine)

        for event in stream:
            if event['type'] == 'gameState':
                if event['status'] != 'started':
                    break

                self.board = chess.Board()
                
                moves_played = event['moves'].split(" ")

                if len(moves_played) % 2 == self.color:
                    continue  

                if self.color == 1:
                    time_left = event["wtime"].time()
                else:
                    time_left = event["btime"].time()

                seconds = (time_left.hour * 60 + time_left.minute) * 60 + time_left.second
                time_limit = seconds / 30


                for move in moves_played:
                    self.board.push(chess.Move.from_uci(move))

                await self.bot_move(engine, time_limit)

        await engine.quit()

    async def bot_move(self, engine, time_limit = None):
        book_move = engine.choose_book_move(self.board)

        if book_move is not None:
            move = book_move
            result = "book move"
            time_limit = self.book_move_time
            sleep(time_limit)
        else:
            if time_limit is None:
                time_limit = self.book_move_time
            move, result = await engine.analyze_position(self.board, time_limit)
        
        self.client.bots.make_move(self.game_id, str(move))

        print("ID: " + str(self.game_id) + " info: " + str(move) + " " + str(result) + " time: " + str(time_limit))


def should_accept(challenge_event) -> bool:
    rated = challenge_event["challenge"]["rated"]
    speed = challenge_event["challenge"]["speed"]
    variant = challenge_event["challenge"]["variant"]["key"]
    challenger = challenge_event["challenge"]["challenger"]["id"]
    game_id = challenge_event["challenge"]["id"]

    is_rated = "rated" if rated else "unrated"
    print(f"{datetime.now()} | challenge by {challenger}; ID: {game_id} - {variant} - {speed} - {is_rated}")
 
    with open("allowed.challengers") as file:

        # These challengers can submit any challenge and it will be accepted
        allowed_challengers = file.read().splitlines()
        print("allowed_challengers:", allowed_challengers)
        if challenger in allowed_challengers:
            return True

        # Other challengers may only challenge to rated games
        elif rated:
            if speed not in RATED_SPEEDS or variant != "standard":
                return False
            return True
        
        return False
    
def main_loop():

    token = ""
    with open('./lichess.token') as f:
        token = f.read()

    session = berserk.TokenSession(token)
    client = berserk.Client(session)

    
    with asyncio.Runner() as runner:
        for event in client.bots.stream_incoming_events():

            if event['type'] == 'challenge':
                if should_accept(event):
                    print("Accepted challenge of " + event["challenge"]["challenger"]["id"])
                    client.bots.accept_challenge(event["challenge"]['id'])

                else:
                    client.bots.decline_challenge(event["challenge"]['id'])

            elif event['type'] == 'gameStart':
                print(event)

                runner.run(Game(client, event).start())

while True:
    try:     
        main_loop()
    except berserk.exceptions.ApiError as exc:
        sleep(1.0)
        print("Restarting afer berserk ApiError: ", exc)
        continue
    except RuntimeError as exc:
        sleep(1.0)
        print("Restarting afer RuntimeError: ", exc)
        continue
    except (ChunkedEncodingError, ProtocolError, InvalidChunkLength) as exc:
        sleep(1.0)
        print("Restarting afer connection problem: ", exc)
        continue
    quit()
    