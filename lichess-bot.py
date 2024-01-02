from requests_oauthlib import OAuth2Session
import asyncio
import berserk
import chess
from chess import engine
from time import time, sleep


limit = 10
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
        


# async def main() -> None:
#     transport, engine = await chess.engine.popen_uci("./target/release/rust_chess")

#     board = chess.Board()
#     while not board.is_game_over():
#          move, result = await analyze_position(board, engine, limit)
#          board.push(move)
#          print(move, result)
#          print(board)
        

#     print(chess.Board().variation_san(board.move_stack))

#     await engine.quit()

# asyncio.run(main())

# quit()


with open('./lichess.token') as f:
    token = f.read()


# session = berserk.TokenSession(token)

session = OAuth2Session(...)

client = berserk.Client(session)

client.account.upgrade_to_bot()

class Game(threading.Thread):

    def __init__(self, client, game_id, **kwargs):

        super().__init__(**kwargs)

        self.game_id = game_id

        self.client = client

        self.stream = client.bots.stream_game_state(game_id)

        self.current_state = next(self.stream)

        self.board = chess.Board()


    async def run(self):
        transport, engine = await chess.engine.popen_uci("./target/release/rust_chess")

        for event in self.stream:

            if event['type'] == 'gameState':

                await self.handle_state_change(event)

            elif event['type'] == 'chatLine':

                self.handle_chat_line(event)

        await engine.quit()


    async def handle_state_change(self, game_state):
        move, result = await analyze_position(self.board, self.engine, limit)
        self.board.push(move)
        print("ID: " + str(self.game_id) + " info: " + str(move) + " " + str(result))
        pass


    def handle_chat_line(self, chat_line):
        print("ID: " + str(self.game_id) + " Chat: " + str(chat_line))
        pass


def should_accept(challenge_event) -> bool:
    return True

def main_loop():
    is_polite = True

    for event in client.bots.stream_incoming_events():

        if event['type'] == 'challenge':

            if should_accept(event):

                client.bots.accept_challenge(event['id'])

            elif is_polite:

                client.bots.decline_challenge(event['id'])

        elif event['type'] == 'gameStart':

            game = Game(event['id'])

            game.start()
