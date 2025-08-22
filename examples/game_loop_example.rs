use async_flow::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;
use tokio::time::sleep;

struct GameState {
    game_over: AtomicBool,
    turn_count: AtomicU32,
    player_ready: AtomicBool,
}

impl GameState {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            game_over: AtomicBool::new(false),
            turn_count: AtomicU32::new(0),
            player_ready: AtomicBool::new(false),
        })
    }

    fn is_game_over(&self) -> bool {
        self.game_over.load(Ordering::Relaxed)
    }

    fn end_game(&self) {
        self.game_over.store(true, Ordering::Relaxed);
        println!("Game ended after {} turns", self.turn_count.load(Ordering::Relaxed));
    }

    fn next_turn(&self) {
        let turn = self.turn_count.fetch_add(1, Ordering::Relaxed) + 1;
        println!("Starting turn {}", turn);
        
        if turn >= 3 {
            self.end_game();
        }
    }

    fn set_player_ready(&self, ready: bool) {
        self.player_ready.store(ready, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    fn is_player_ready(&self) -> bool {
        self.player_ready.load(Ordering::Relaxed)
    }
}

async fn start_game(game_state: Arc<GameState>) -> Result<()> {
    println!("Initializing game...");
    sleep(Duration::from_millis(100)).await;
    
    println!("Players drawing initial cards...");
    sleep(Duration::from_millis(200)).await;
    
    println!("Players ready for game start");
    game_state.set_player_ready(true);
    
    Ok(())
}

async fn player_turn(game_state: Arc<GameState>) -> Result<()> {
    if game_state.is_game_over() {
        return Ok(());
    }

    println!("Processing player turn...");
    sleep(Duration::from_millis(300)).await;
    
    println!("Turn completed");
    game_state.next_turn();
    
    Ok(())
}

async fn end_game(_game_state: Arc<GameState>) -> Result<()> {
    println!("Cleaning up game state...");
    sleep(Duration::from_millis(100)).await;
    println!("Game cleanup complete");
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let kernel = AsyncKernel::new();
    let root = kernel.root();
    let game_state = GameState::new();

    let start_game_coroutine = Arc::new(AsyncCoroutine::new(
        start_game(game_state.clone())
    )).named("StartGame");

    let game_loop_condition = {
        let game_state = game_state.clone();
        move || !game_state.is_game_over()
    };

    let player_turn_coroutine = Arc::new(AsyncCoroutine::new(
        player_turn(game_state.clone())
    )).named("PlayerTurn");

    let end_game_coroutine = Arc::new(AsyncCoroutine::new(
        end_game(game_state.clone())
    )).named("EndGame");

    let game_sequence = Arc::new(Sequence::new()).named("GameLoop");
    
    game_sequence.add_child(start_game_coroutine).await;
    
    let while_loop = Arc::new(Trigger::new(game_loop_condition)).named("WhileNotGameOver");
    while_loop.set_triggered_callback({
        let player_turn_coroutine = player_turn_coroutine.clone();
        let root = root.clone();
        move || {
            let player_turn_coroutine = player_turn_coroutine.clone();
            let root = root.clone();
            tokio::spawn(async move {
                root.add_child(player_turn_coroutine).await;
            });
        }
    }).await;
    
    game_sequence.add_child(while_loop).await;
    game_sequence.add_child(end_game_coroutine).await;

    root.add_child(game_sequence).await;

    println!("Starting AsyncFlow game loop example...");
    kernel.run_for(Duration::from_secs(5)).await?;
    println!("Example completed!");

    Ok(())
}