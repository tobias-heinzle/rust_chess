use chess::{ChessMove, Piece, Square};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::thread::JoinHandle;

use crate::search::{SearchContext, SearchInfo, SearchOutcome};
use crate::table::{ScoreBound, TableEntryData, TranspositionTable};
use crate::uci::Position;

pub enum SearchGroupError {
    AlreadyRunning,
    StartAgentError,
}

const STOP_SIGNAL: bool = true;
const N_STOP_SIGNALS: u32 = 10;

const ORDERINGS: [[Piece; 6]; 7] = [
    [
        Piece::Rook,
        Piece::Queen,
        Piece::Bishop,
        Piece::Knight,
        Piece::Pawn,
        Piece::King,
    ],
    [
        Piece::Bishop,
        Piece::Queen,
        Piece::Rook,
        Piece::Knight,
        Piece::Pawn,
        Piece::King,
    ],
    [
        Piece::Knight,
        Piece::Queen,
        Piece::Rook,
        Piece::Bishop,
        Piece::Pawn,
        Piece::King,
    ],
    [
        Piece::Pawn,
        Piece::Queen,
        Piece::Rook,
        Piece::Bishop,
        Piece::Knight,
        Piece::King,
    ],
    [
        Piece::King,
        Piece::Knight,
        Piece::Queen,
        Piece::Rook,
        Piece::Bishop,
        Piece::Pawn,
    ],
    [
        Piece::King,
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
    ],
    [
        Piece::Queen,
        Piece::Rook,
        Piece::Bishop,
        Piece::Knight,
        Piece::Pawn,
        Piece::King,
    ],
];
pub struct SearchGroup {
    principal: SearchAgent,
    agents: Vec<SearchAgent>,
}

impl SearchGroup {
    pub fn start(
        position: Position,
        num_threads: u8,
        info_sender: Sender<SearchInfo>,
        table_size: u32,
        max_depth: u8,
        time_limit: Option<f32>,
    ) -> SearchGroup {
        assert!(num_threads > 0);

        let hash_table = TranspositionTable::new(
            table_size as usize,
            TableEntryData {
                best_move: ChessMove::new(Square::A1, Square::A1, None),
                score: 0,
                depth: 0,
                score_bound: ScoreBound::LowerBound,
            },
        );

        let (context, stop_sender) =
            create_search_context(info_sender, &position, hash_table.clone());
        let principal = SearchAgent::start(context, stop_sender, max_depth, time_limit);

        let (dummy_sender, _) = channel();

        let mut agents: Vec<SearchAgent> = vec![];
        for n_thread in 0..num_threads - 1 {
            let (mut agent_context, agent_stop_sender) =
                create_search_context(dummy_sender.clone(), &position, hash_table.clone());

            agent_context.move_ordering = ORDERINGS[(n_thread % 7) as usize];
            agent_context.start_depth = n_thread + 1;

            let agent = SearchAgent::start(agent_context, agent_stop_sender, max_depth, time_limit);

            agents.push(agent);
        }

        SearchGroup { principal, agents }
    }

    pub fn stop(self) -> Result<SearchOutcome, ()> {
        for agent in self.agents {
            let _ = agent.stop(N_STOP_SIGNALS);
        }

        match self.principal.stop(N_STOP_SIGNALS) {
            Ok(outcome) => Ok(outcome),
            Err(_) => Err(()),
        }
    }

    pub fn await_principal(self) -> Result<SearchOutcome, ()> {
        let search_outcome: Result<SearchOutcome, ()>;

        match self.principal.stop(0) {
            Ok(outcome) => search_outcome = Ok(outcome),
            Err(_) => search_outcome = Err(()),
        }

        for agent in self.agents {
            let _ = agent.stop(N_STOP_SIGNALS);
        }

        search_outcome
    }
}

fn create_search_context(
    info_sender: Sender<SearchInfo>,
    position: &Position,
    hash_table: TranspositionTable,
) -> (SearchContext, Sender<bool>) {
    let (stop_sender, stop_receiver) = channel();

    // let hash_table = Arc::new(TranspositionTable::new(HASH_TABLE_SIZE, TableEntryData{best_move : ChessMove::new(Square::A1, Square::A1, None), score : 0, depth : 0, score_bound : ScoreBound::LowerBound}));

    let mut search_context = SearchContext::new(
        position.board,
        stop_receiver,
        info_sender.clone(),
        hash_table, //Arc::clone(&hash_table)
    );
    for hash in position.hash_history.iter() {
        search_context.set_visited(*hash);
    }

    (search_context, stop_sender)
}

struct SearchAgent {
    pub stop: Sender<bool>,
    pub handle: JoinHandle<SearchOutcome>,
}

impl SearchAgent {
    fn start(
        mut context: SearchContext,
        stop_sender: Sender<bool>,
        max_depth: u8,
        time_limit: Option<f32>,
    ) -> SearchAgent {
        assert!(time_limit.is_none(), "time limit not implemented yet");

        SearchAgent {
            stop: stop_sender,
            handle: thread::spawn(move || context.root_search(max_depth)),
        }
    }

    fn stop(self, n_signals: u32) -> Result<SearchOutcome, ()> {
        send_termination_signals(self.stop, n_signals);
        match self.handle.join() {
            Ok(outcome) => Ok(outcome),
            Err(_) => Err(()),
        }
    }
}

fn send_termination_signals(sender: Sender<bool>, n_signals: u32) {
    for _ in 0..n_signals {
        let _ = sender.send(STOP_SIGNAL);
    }
}
