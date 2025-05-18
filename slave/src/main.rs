
// === slave.rs ===
use qt4004core::{QT4004, program_from_seed};
use serde::{Deserialize, Serialize};
use std::{
    env,
    io::{Read, Write},
    net::TcpStream,
    thread,
    time::Duration,
};

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

fn worker(id: usize, server_addr: String, sub_batch_size: u128) {
    let mut processor = QT4004::new();

    loop {
        match TcpStream::connect(&server_addr) {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];
                if let Ok(n) = stream.read(&mut buffer) {
                    if let Ok(batch) = serde_json::from_slice::<WorkBatch>(&buffer[..n]) {
                        println!("[Thread {id}] Received batch: {} -> {}", batch.start, batch.end);

                        let mut best_steps = 0;
                        let mut best_program = [0i8; 16];

                        let mut current = batch.start;
                        while current < batch.end {
                            let end = (current + sub_batch_size).min(batch.end);

                            for seed in current..end {
                                let program = program_from_seed(seed);
                                processor.load(program);
                                let steps = processor.run();
                                if steps > best_steps {
                                    best_steps = steps;
                                    best_program = program;
                                }
                            }

                            current += sub_batch_size;
                        }

                        let result = WorkResult {
                            best_program,
                            steps: best_steps,
                        };

                        let json = serde_json::to_vec(&result).unwrap();
                        let _ = stream.write_all(&json);
                    }
                }
            }
            Err(e) => {
                eprintln!("[Thread {id}] Connection error: {e}");
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let thread_count: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(4);
    let server_addr = args.get(2).map(|s| s.as_str()).unwrap_or("127.0.0.1:7878").to_string();
    let sub_batch_size: u128 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(100000);

    for id in 0..thread_count {
        let addr = server_addr.clone();
        thread::spawn(move || worker(id, addr, sub_batch_size));
    }

    // Block main thread forever
    loop {
        thread::park();
    }
}
