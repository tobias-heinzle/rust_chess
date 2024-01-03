import asyncio
import berserk
import chess
from time import sleep
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
        fen = event["game"]["fen"]
        speed = event["game"]["speed"]
        self.is_my_turn = event["game"]["isMyTurn"]
        self.client = client
        self.color = 1 if color == "white" else 0
        self.board = chess.Board()
        self.board.set_fen(fen)
        self.time_limit = 1.0
        if speed in Game.speed_limit.keys():
            self.time_limit = Game.speed_limit[speed]

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

                for move in moves_played:
                    self.board.push(chess.Move.from_uci(move))

                await self.bot_move(engine)

        await engine.quit()

    async def bot_move(self, engine):
        move, result = await engine.analyze_position(self.board, time_limit=self.time_limit)
        self.client.bots.make_move(self.game_id, str(move))

        print("ID: " + str(self.game_id) + " info: " + str(move) + " " + str(result))



def should_accept(challenge_event) -> bool:
    print(challenge_event)
    with open("allowed.challengers") as file:

        # These challengers can submit any challenge and it will be accepted
        allowed_challengers = file.readlines()
        if challenge_event["challenge"]["challenger"]["id"] in allowed_challengers:
            return True
        
        # Other challengers may only challenge to rated games
        elif challenge_event["challenge"]["rated"]:
            if challenge_event["challenge"]["speed"] not in RATED_SPEEDS:
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
        sleep(2.0)
        print("Restarting afer api exception: ", exc)
        continue

    quit()
    