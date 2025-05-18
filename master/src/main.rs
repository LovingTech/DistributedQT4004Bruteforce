
// === master.rs ===
use std::{io::{Read, Write}, env,net::{TcpListener, TcpStream}, sync::{Arc, Mutex}, thread, time::Instant};
use serde::{Deserialize, Serialize, Serializer};
use warp::Filter;

#[derive(Clone, Serialize, Deserialize)]
struct WorkBatch {
    start: u128,
    end: u128,
}

#[derive(Clone, Serialize, Deserialize)]
struct WorkResult {
    best_program: [i8; 16],
    steps: u64,
}

struct ServerState {
    current_seed: u128,
    batch_size: u128,
    metrics: Metrics,
}

#[derive(Clone)]
struct Metrics {
    total_batches: u64,
    best_steps: u64,
    best_program: [i8; 16],
    start_time: Instant,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            total_batches: 0,
            best_steps: 0,
            best_program: [0; 16],
            start_time: Instant::now(),
        }
    }
}

impl Serialize for Metrics {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize a struct manually with selected fields
        let elapsed_minutes = self.start_time.elapsed().as_secs_f64() / 60.0;

        // Use a helper struct for serialization
        #[derive(Serialize)]
        struct MetricsHelper {
            total_batches: u64,
            best_steps: u64,
            best_program: [i8; 16],
            elapsed_minutes: f64,
        }

        let helper = MetricsHelper {
            total_batches: self.total_batches,
            best_steps: self.best_steps,
            best_program: self.best_program,
            elapsed_minutes,
        };

        helper.serialize(serializer)
    }
}

fn handle_client(mut stream: TcpStream, state: Arc<Mutex<ServerState>>) {
    // 1. Send a batch to the slave
    let batch = {
        let mut state = state.lock().unwrap();
        let batch = WorkBatch {
            start: state.current_seed,
            end: state.current_seed + state.batch_size,
        };
        state.current_seed += state.batch_size;
        batch
    };
    let batch_json = serde_json::to_vec(&batch).unwrap();
    stream.write_all(&batch_json).unwrap();

    // 2. Wait for the slave to respond with WorkResult
    let mut buffer = vec![0u8; 1024];
    if let Ok(n) = stream.read(&mut buffer) {
        if let Ok(result) = serde_json::from_slice::<WorkResult>(&buffer[..n]) {
            let mut state = state.lock().unwrap();
            state.metrics.total_batches += 1;
            if result.steps > state.metrics.best_steps {
                state.metrics.best_steps = result.steps;
                state.metrics.best_program = result.best_program;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let batchsize: u128 = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1048576);

    let state = Arc::new(Mutex::new(ServerState {
        current_seed: 0,
        batch_size: batchsize,
        metrics: Metrics {
            start_time: Instant::now(),
            ..Default::default()
        },
    }));

    let tcp_state = Arc::clone(&state);
    thread::spawn(move || {
        let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
        println!("Master listening on port 7878");
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let s = Arc::clone(&tcp_state);
                thread::spawn(move || {
                    //let batch = {
                    //    let mut state = s.lock().unwrap();
                    //    let batch = WorkBatch {
                    //        start: state.current_seed,
                    //        end: state.current_seed + state.batch_size,
                    //    };
                    //    state.current_seed += state.batch_size;
                    //    batch
                    //};
                    //let json = serde_json::to_vec(&batch).unwrap();
                    //let _ = stream.write_all(&json);
                    handle_client(stream, s);
                });
            }
        }
    });

    // Web dashboard using warp
    let metrics_route = warp::path("metrics").map({
        let state = Arc::clone(&state);
        move || {
            let s = state.lock().unwrap();
            warp::reply::json(&s.metrics)
        }
    });

    println!("Web dashboard on http://localhost:3030/metrics");
    warp::serve(metrics_route).run(([0, 0, 0, 0], 3030)).await;
}
