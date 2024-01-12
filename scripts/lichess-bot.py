import asyncio
import logging
import sys
from random import sample
from time import sleep
from datetime import datetime
from requests.exceptions import ChunkedEncodingError
from urllib3.exceptions import InvalidChunkLength, ProtocolError

import berserk
import chess
from chess import WHITE, BLACK
import chess.polyglot

from wrapper import ChessEngineWrapper



ALLOWED_SPEEDS = ["bullet", "blitz", "rapid"]

TIME_DIVIDER = 20

root = logging.getLogger()
root.setLevel(logging.DEBUG)

handler = logging.StreamHandler(sys.stdout)
handler.setLevel(logging.DEBUG)
formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
handler.setFormatter(formatter)
root.addHandler(handler)

bot_logger = logging.Logger("lichess-bot")
bot_logger.addHandler(handler)

            
class Game():
    book_speed = {  "bullet" : 0.2,
                    "blitz" : 0.5,
                    "rapid" : 1.0,
                    "classical" : 1.0,
                    'correspondence' : 1.0}

    def __init__(self, client, event):
        bot_logger.debug("Init game: " + str(event))
        color = event["game"]['color']
        speed = event["game"]["speed"]
        current_fen = event["game"]["fen"]
        self.board = chess.Board()
        self.board.set_fen(current_fen)
        
        self.game_id = event["game"]["gameId"]
        self.is_my_turn = event["game"]["isMyTurn"]
        self.client = client
        
        self.color = WHITE if color == "white" else BLACK
        self.initial_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        
        self.book_move_time = 1.0
        if speed in Game.book_speed.keys():
            self.book_move_time = Game.book_speed[speed]

    async def start(self) -> None:
        bot_logger.info("Game started: " + self.game_id)
        engine = ChessEngineWrapper()
        await engine.start()

        stream = self.client.bots.stream_game_state(self.game_id)
        current_state = next(stream)

        if current_state['initialFen'] != 'startpos':
            self.initial_fen = current_state['initialFen']

        if self.color == WHITE:
            seconds = current_state["state"]["wtime"]/1000
        else:
            seconds = current_state["state"]["btime"]/1000

        time_limit = seconds / 30

        if self.is_my_turn:
            await self.bot_move(engine, time_limit)


        for event in stream:
            if event['type'] == 'gameState':
                if event['status'] != 'started':
                    break

                self.board = chess.Board()
                self.board.set_fen(self.initial_fen)

                moves_played = event['moves'].split(" ")
                for move in moves_played:
                    self.board.push(chess.Move.from_uci(move))

                if self.board.turn != self.color:
                    continue  
                
                if self.color == WHITE:
                    time_left = event["wtime"].time()
                else:
                    time_left = event["btime"].time()

                seconds = (time_left.hour * 60 + time_left.minute) * 60 + time_left.second
                time_limit = seconds / TIME_DIVIDER

                await self.bot_move(engine, time_limit)


        bot_logger.info(f"ID: {self.game_id} finished!")
        await engine.quit()
       
        

    async def bot_move(self, engine, time_limit):
        book_move = engine.choose_book_move(self.board)

        if book_move is not None:
            move = book_move
            result = "book move"
            sleep(self.book_move_time)
        else:
            move, result = await engine.analyze_position(self.board, time_limit)

        
        self.client.bots.make_move(self.game_id, str(move))

        bot_logger.info("ID: " + str(self.game_id) + " info: " + str(move) + " " + str(result) + " time: " + str(time_limit))


def should_accept(challenge_event) -> bool:
    rated = challenge_event["challenge"]["rated"]
    speed = challenge_event["challenge"]["speed"]
    variant = challenge_event["challenge"]["variant"]["key"]
    challenger = challenge_event["challenge"]["challenger"]["id"]
    game_id = challenge_event["challenge"]["id"]

    is_rated = "rated" if rated else "unrated"
    bot_logger.info(f"{datetime.now()} | challenge by {challenger}; ID: {game_id} - {variant} - {speed} - {is_rated}")
 
    with open("allowed.challengers") as file:

        # These challengers can submit any challenge and it will be accepted
        allowed_challengers = file.read().splitlines()
        bot_logger.debug(f"allowed_challengers:{allowed_challengers}")
        if challenger in allowed_challengers:
            return True

        if speed in ALLOWED_SPEEDS and variant == "standard":
            return True
        
        return False
    
def main_loop():

    token = ""
    with open('./lichess.token') as f:
        token = f.read()

    session = berserk.TokenSession(token)
    client = berserk.Client(session)

    asyncio.set_event_loop_policy(chess.engine.EventLoopPolicy())

    
    for event in client.bots.stream_incoming_events():

        if event['type'] == 'challenge':
            if should_accept(event):
                bot_logger.info("Accepted challenge of " + event["challenge"]["challenger"]["id"])
                client.bots.accept_challenge(event["challenge"]['id'])

            else:
                client.bots.decline_challenge(event["challenge"]['id'])

        elif event['type'] == 'gameStart':
            
            asyncio.set_event_loop_policy(chess.engine.EventLoopPolicy())
            with asyncio.Runner() as runner:
                runner.run(Game(client, event).start())


while True:
    bot_logger.info("lichess-bot booted up!")
    try:     
        main_loop()
    except berserk.exceptions.ApiError as exc:
        sleep(1.0)
        bot_logger.warning(f"Restarting afer berserk ApiError: {exc}")
        continue
    except RuntimeError as exc:
        sleep(1.0)
        bot_logger.warning(f"Restarting afer RuntimeError: {exc}")
        continue
    except (ChunkedEncodingError, ProtocolError, InvalidChunkLength) as exc:
        sleep(1.0)
        bot_logger.warning(f"Restarting afer connection problem: {exc}")
        continue
    quit()
    