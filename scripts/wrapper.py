from chess import engine, Board
from time import sleep


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