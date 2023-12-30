from requests_oauthlib import OAuth2Session
import berserk
import chess
from chess import engine


board = chess.Board()

engine = engine.SimpleEngine.popen_uci(
            "./target/release/rust_chess")

while not board.is_game_over():
    result = engine.play(board, chess.engine.Limit(time=0.1))
    board.push(result.move)
    print(board)

quit()


with open('./lichess.token') as f:
    token = f.read()


session = berserk.TokenSession(token)
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

        self.engine = chess.engine.SimpleEngine.popen_uci(
            "target/release/rust_chess")


    def run(self):

        for event in self.stream:

            if event['type'] == 'gameState':

                self.handle_state_change(event)

            elif event['type'] == 'chatLine':

                self.handle_chat_line(event)


    def handle_state_change(self, game_state):

        pass


    def handle_chat_line(self, chat_line):

        pass

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
