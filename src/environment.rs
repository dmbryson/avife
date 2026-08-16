#[derive(Debug, Clone, Copy)]
pub enum Task {
    Not,
    Nand,
    And,
    OrN,
    Or,
    AndN,
    Nor,
    Xor,
    Equ,
}

#[allow(dead_code)]
pub enum ReactionProcessType {
    Add,
    Mult,
    Pow,
}

pub struct ReactionProcess {
    pub process_type: ReactionProcessType,
    pub value: f64,
}

pub struct Reaction {
    pub task_idx: usize,
    pub processes: Vec<ReactionProcess>,

    // requisites
    pub max_count: u64,
}

pub struct ReactionResult {
    pub tasks_done: Vec<bool>,
    pub reactions_done: Vec<bool>,
    pub bonus_add: f64,
    pub bonus_mult: f64,
}

impl ReactionResult {
    fn new(num_tasks: usize, num_reactions: usize) -> Self {
        ReactionResult {
            tasks_done: vec![false; num_tasks],
            reactions_done: vec![false; num_reactions],
            bonus_add: 0.0,
            bonus_mult: 1.0,
        }
    }
}

pub struct Environment {
    pub tasks: Vec<Task>,
    pub reactions: Vec<Reaction>,
}



impl Environment {
    pub fn new(tasks: Vec<Task>, reactions: Vec<Reaction>) -> Self {
        Environment {
            tasks,
            reactions
        }
    }

    pub fn default() -> Self {
        // <keyword> <name> <task> <option> <option>...
        // REACTION  NOT  not   process:value=1.0:type=pow  requisite:max_count=1
        // REACTION  NAND nand  process:value=1.0:type=pow  requisite:max_count=1
        // REACTION  AND  and   process:value=2.0:type=pow  requisite:max_count=1
        // REACTION  ORN  orn   process:value=2.0:type=pow  requisite:max_count=1
        // REACTION  OR   or    process:value=3.0:type=pow  requisite:max_count=1
        // REACTION  ANDN andn  process:value=3.0:type=pow  requisite:max_count=1
        // REACTION  NOR  nor   process:value=4.0:type=pow  requisite:max_count=1
        // REACTION  XOR  xor   process:value=4.0:type=pow  requisite:max_count=1
        // REACTION  EQU  equ   process:value=5.0:type=pow  requisite:max_count=1

        let tasks = vec![
            Task::Not,
            Task::Nand,
            Task::And,
            Task::OrN,
            Task::Or,
            Task::AndN,
            Task::Nor,
            Task::Xor,
            Task::Equ,
        ];

        let reactions = vec![
            Reaction { task_idx: 0, processes: vec![ReactionProcess { process_type: ReactionProcessType::Pow, value: 1.0 }], max_count: 1 },
            Reaction { task_idx: 1, processes: vec![ReactionProcess { process_type: ReactionProcessType::Pow, value: 1.0 }], max_count: 1 },
            Reaction { task_idx: 2, processes: vec![ReactionProcess { process_type: ReactionProcessType::Pow, value: 2.0 }], max_count: 1 },
            Reaction { task_idx: 3, processes: vec![ReactionProcess { process_type: ReactionProcessType::Pow, value: 2.0 }], max_count: 1 },
            Reaction { task_idx: 4, processes: vec![ReactionProcess { process_type: ReactionProcessType::Pow, value: 3.0 }], max_count: 1 },
            Reaction { task_idx: 5, processes: vec![ReactionProcess { process_type: ReactionProcessType::Pow, value: 3.0 }], max_count: 1 },
            Reaction { task_idx: 6, processes: vec![ReactionProcess { process_type: ReactionProcessType::Pow, value: 4.0 }], max_count: 1 },
            Reaction { task_idx: 7, processes: vec![ReactionProcess { process_type: ReactionProcessType::Pow, value: 4.0 }], max_count: 1 },
            Reaction { task_idx: 8, processes: vec![ReactionProcess { process_type: ReactionProcessType::Pow, value: 5.0 }], max_count: 1 },
        ];

        Self::new(tasks, reactions)
    }


    pub fn process_output(&self, output: i64, inputs: &[i64], cur_reaction_count: &[u64]) -> ReactionResult {
        let mut result = ReactionResult::new(self.tasks.len(), self.reactions.len());

        assert!(inputs.len() == 3);
        let logic_id = calc_logic_id(inputs, output);

        if let Some(logic_id) = logic_id {
            for (idx, task) in self.tasks.iter().enumerate() {
                let task_done = match task {
                    Task::Not => matches!(logic_id, 15 | 51 | 85),
                    Task::Nand => matches!(logic_id, 63 | 95 | 119),
                    Task::And => matches!(logic_id, 136 | 160 | 192),
                    Task::OrN => matches!(logic_id, 175 | 187 | 207 | 221 | 243 | 245),
                    Task::Or => matches!(logic_id, 238 | 250 | 252),
                    Task::AndN => matches!(logic_id, 10 | 12 | 34 | 48 | 68 | 80),
                    Task::Nor => matches!(logic_id, 3 | 5 | 17),
                    Task::Xor => matches!(logic_id, 60 | 90 | 102),
                    Task::Equ => matches!(logic_id, 153 | 165 | 195),
                    // _ => false,  // default for when we have non-logic tasks
                };
                if task_done {
                    result.tasks_done[idx] = task_done;
                }
            }
        }

        for (idx, reaction) in self.reactions.iter().enumerate() {
            if cur_reaction_count[idx] >= reaction.max_count {
                continue
            }
            if !result.tasks_done[reaction.task_idx] {
                continue
            }
            for process in reaction.processes.iter() {
                match process.process_type {
                    ReactionProcessType::Add => { result.bonus_add += process.value },
                    ReactionProcessType::Mult => { result.bonus_mult *= process.value },
                    ReactionProcessType::Pow => { result.bonus_mult *= 2.0_f64.powf(process.value) },
                }
            }
            result.reactions_done[idx] = true;
        }

        result
    }

}

fn calc_logic_id(inputs: &[i64], output: i64) -> Option<u8> {
    let (mut a, mut b, mut c) = (inputs[0] as u64, inputs[1] as u64, inputs[2] as u64);
    let mut out = output as u64;
    let mut table: [Option<bool>; 8] = [None; 8];

    for _ in 0..64 {
        let pos = ((a & 1) | ((b & 1) << 1) | ((c & 1) << 2)) as usize;
        let bit = out & 1 == 1;
        match table[pos] {
            Some(prev) if prev != bit => return None,   // inconsistent -> not a logic fn
            _ => table[pos] = Some(bit),
        }
        a >>= 1; b >>= 1; c >>= 1; out >>= 1;
    }

    let mut id = 0u8;
    for (i, slot) in table.into_iter().enumerate() {
        if slot? { id |= 1 << i; }
    }
    Some(id)
}
