from random import sample
from time import sleep

from chess import engine, Board, Move
from chess.polyglot import open_reader


class ChessEngineWrapper:
    protocol: engine.Protocol = None
    path: str = "../target/release/rust_chess"
    polling_interval: float = 0.05

    async def start(self):
        if self.protocol is not None:
            await self.protocol.quit()
        _, self.protocol = await engine.popen_uci(self.path)


    async def analyze_position(self, position: Board, time_limit: float = 0.5) -> (str, dict):
        
        assert(self.protocol is not None)

        bestmove = ""
        result = {}

        with await self.protocol.analysis(position) as analysis:

            sleep(time_limit)

            analysis.stop()

            bestmove  = await analysis.wait()
            result = analysis.info
        
        return (bestmove.move, result)
    
    async def quit(self):
        await self.protocol.quit()

    def choose_book_move(self, board: Board, book: str = "books/titans.bin") -> Move | None:
        with open_reader("books/titans.bin") as reader:
            book_entries = list(reader.find_all(board))

            if len(book_entries) > 0:
                weights = [entry.weight for entry in book_entries]
                return sample(book_entries, 1, counts=weights)[0].move

        return None