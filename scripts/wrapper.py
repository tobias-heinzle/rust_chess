from random import sample
from time import sleep
import asyncio

from chess import engine, Board, Move
from chess.polyglot import open_reader


class ChessEngineWrapper:
    protocol: engine.Protocol = None
    path: str = "../target/release/rust_chess"
    polling_interval: float = 0.01
    retries: int = 3
    transport: asyncio.SubprocessTransport = None; 

    async def start(self):
        if self.transport is not None:
            self.transport.terminate()
        self.transport, self.protocol = await engine.popen_uci(self.path)


    async def analyze_position(self, position: Board, time_limit: float = 0.5) -> (str, dict):
        
        assert(self.protocol is not None)

        bestmove = ""
        result = {}
        # for _ in range(self.retries):
        #     try:
        #         async with asyncio.timeout(time_limit + 1.0):
        with await self.protocol.analysis(position) as analysis:

            sleep(time_limit)

            analysis.stop()

            bestmove  = await analysis.wait()
        
            result = analysis.info
            
            return (bestmove.move, result)
                
            # except TimeoutError:
            #     print("analysis timed out")
        
        # raise RuntimeError("unable to complete analysis")

        
    
    async def quit(self):
        try:
            async with asyncio.timeout(1.0):
                await self.protocol.quit()
        except TimeoutError as exc:
            print("engine.quit timed out, terminating SubprocessTransport object")
            self.transport.terminate()

        self.protocol = None
        self.transport = None


    def choose_book_move(self, board: Board, book: str = "books/titans.bin") -> Move | None:
        with open_reader("books/titans.bin") as reader:
            book_entries = list(reader.find_all(board))

            if len(book_entries) > 0:
                weights = [entry.weight for entry in book_entries]
                return sample(book_entries, 1, counts=weights)[0].move

        return None