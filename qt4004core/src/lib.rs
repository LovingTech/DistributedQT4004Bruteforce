use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct QT4004 {
    pub r0: i8,
    pub r1: i8,
    pub is: i8,
    pub ip: i8,
    pub steps: u64,
    pub memory: [i8; 16],
    pub program: [i8; 16],
    pub running: bool,
    pub output: String,
}

impl Hash for QT4004 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is.hash(state);
        self.ip.hash(state);
        self.r0.hash(state);
        self.r1.hash(state);
        self.memory.hash(state);
    }
}

impl PartialEq for QT4004 {
    fn eq(&self, other: &Self) -> bool {
        self.is == other.is &&
        self.ip == other.ip &&
        self.r0 == other.r0 &&
        self.r1 == other.r1 &&
        self.memory == other.memory
    }
}

impl Eq for QT4004 {}

impl QT4004 {
    pub fn new() -> Self {
        Self {
            r0: 0,
            r1: 0,
            is: 0,
            ip: 0,
            steps: 0,
            memory: [0; 16],
            program: [0; 16],
            running: true,
            output: String::new(),
        }
    }

    pub fn load(&mut self, program: [i8; 16]) {
        self.program = program;
        self.reset();
    }

    pub fn reset(&mut self) {
        self.r0 = 0;
        self.r1 = 0;
        self.is = 0;
        self.ip = 0;
        self.memory = self.program;
        self.output = String::new();
        self.steps = 0;
        self.running = true;
    }

    pub fn run(&mut self) -> u64 {
        let mut states = HashSet::new();
        while self.running && self.ip < 16 {
            self.fetch();
            self.ip += 1;
            self.execute();
            if states.contains(self) {
                return 0;
            }
            states.insert(self.clone());
        }
        self.steps
    }

    fn fetch(&mut self) {
        self.is = self.memory[self.ip as usize];
    }

    fn execute(&mut self) {
        match self.is {
            0 => self.running = false,
            1 => self.r0 = modulo(self.r0 + 1, 16),
            2 => self.r0 = modulo(self.r0 - 1, 16),
            3 => self.r1 = modulo(self.r1 + 1, 16),
            4 => self.r1 = modulo(self.r1 - 1, 16),
            5 => self.r0 = modulo(self.r0 + self.r1, 16),
            6 => self.r0 = modulo(self.r0 - self.r1, 16),
            7 => self.output.push_str(&format!("{:X}", self.r0)),
            8..=15 => {
                if (self.memory.len() as i8 - self.ip) >= 2 {
                    let arg = self.memory[self.ip as usize];
                    match self.is {
                        8 => if self.r0 != 0 { self.ip = arg } else { self.ip += 1 },
                        9 => if self.r0 == 0 { self.ip = arg } else { self.ip += 1 },
                        10 => { self.r0 = arg; self.ip += 1; },
                        11 => { self.r1 = arg; self.ip += 1; },
                        12 => { self.memory[arg as usize] = self.r0; self.ip += 1; },
                        13 => { self.memory[arg as usize] = self.r1; self.ip += 1; },
                        14 => { (self.r0, self.memory[arg as usize]) = (self.memory[arg as usize], self.r0); self.ip += 1; },
                        15 => { (self.r1, self.memory[arg as usize]) = (self.memory[arg as usize], self.r1); self.ip += 1; },
                        _ => {}
                    }
                }
            },
            _ => {}
        }
        self.steps += 1;
    }
}

pub fn modulo(value: i8, modulus: i8) -> i8 {
    value & (modulus - 1)
}

pub fn program_from_seed(mut seed: u128) -> [i8; 16] {
    let mut program = [0; 16];
    for i in 0..16 {
        program[i] = (seed % 16) as i8;
        seed /= 16;
    }
    program
}

