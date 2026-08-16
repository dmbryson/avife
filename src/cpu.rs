use rand::RngExt;

use crate::World;
use crate::genome::{Genome, Instruction};
use crate::organism::{Offspring, OrganismContext};
use std::ops::Deref;

#[derive(Clone)]

struct Head {
    position: isize,
}

impl Head {
    fn new() -> Head {
        Head{position: 0}
    }

    fn reset(&mut self) {
        self.position = 0;
    }

    fn adjust(&mut self, genome: &Genome) {
        if self.position >= 0 && (self.position as usize) < genome.len() {
            return;
        }

        if genome.is_empty() || self.position < 0 {
            self.position = 0;
            return;
        }

        if self.position < (2 * genome.len()) as isize {
            self.position -= genome.len() as isize;
        } else {
            self.position %= genome.len() as isize
        }
    }

    fn set(&mut self, position: isize, genome: &Genome) {
        self.position = position;
        self.adjust(genome);
    }

    fn advance(&mut self, genome: &Genome) {
        self.position += 1;
        self.adjust(genome);
    }

    fn jump(&mut self, dist: isize, genome: &Genome) {
        self.position += dist;
        self.adjust(genome);
    }

    fn get_inst(&self, genome: &Genome) -> Instruction {
        genome[self.position as usize]
    }

    fn get_next_inst(&self, genome: &Genome) -> Instruction {
        if (self.position + 1) as usize == genome.len() {
            return Instruction::Error;
        }
        genome[(self.position + 1) as usize]
    }

    fn set_inst(&self, inst: Instruction, genome: &mut Genome) {
        genome[self.position as usize] = inst;
    }
}

#[derive(Clone)]
struct Stack {
    stack: Vec<i64>,
    pointer: usize,
}

impl Stack {
    const SIZE: usize = 10;

    fn new() -> Stack {
        Stack { stack: vec![0; Self::SIZE], pointer: 0 }
    }

    fn reset(&mut self) {
        self.stack.fill(0);
        self.pointer = 0;
    }

    fn push(&mut self, v: i64) {
        if self.pointer == 0 {
            self.pointer = self.stack.len() - 1;
        } else {
            self.pointer -= 1;
        }
        self.stack[self.pointer] = v;
    }

    fn pop(&mut self) -> i64 {
        let v = self.stack[self.pointer];
        self.stack[self.pointer] = 0;
        self.pointer += 1;
        if self.pointer == self.stack.len() {
            self.pointer = 0;
        }
        v
    }
}

#[derive(PartialEq, Eq)]
struct CodeLabel(Vec<usize>);

impl CodeLabel {
    fn new() -> CodeLabel {
        CodeLabel(Vec::new())
    }

    fn push(&mut self, nop_mod: usize) {
        self.0.push(nop_mod)
    }

    fn clear(&mut self) {
        self.0.clear()
    }

    fn rotate(&mut self, rot: usize, base: usize) {
        self.0.iter_mut().for_each(|x| {
            *x += rot;
            if *x >= base {
                *x -= base
            }
        })
    }

    fn read_inst(&mut self, inst: Instruction) {
        if inst.is_nop() {
            self.push(inst.get_nop_mod());
        } else {
            self.clear();
        }
    }
}

impl Deref for CodeLabel {
    type Target = Vec<usize>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

const NUM_REGISTERS: usize = 3;
const NUM_HEADS: usize = 4;
const NUM_NOPS: usize = 3;
const NUM_STACKS: usize = 2;

const REG_AX: usize = 0;
const REG_BX: usize = 1;
const REG_CX: usize = 2;

const HEAD_IP: usize = 0;
const HEAD_READ: usize = 1;
const HEAD_WRITE: usize = 2;
const HEAD_FLOW: usize = 3;

const OFFSPRING_SIZE_RANGE: usize = 2; // FIXME: make configurable

pub struct Cpu {
    registers: Vec<i64>,
    heads: Vec<Head>,
    stacks: Vec<Stack>,

    read_label: CodeLabel,
    next_label: CodeLabel,

    genome: Genome,
    initial_genome_size: usize,

    cur_stack: usize,
    advance_ip: bool,
    mal_active: bool,
}

impl Cpu {
    pub fn new(genome: Genome) -> Cpu {
        let initial_genome_size = genome.len();

        Cpu{
            registers: vec![0; NUM_REGISTERS],
            heads: vec![Head::new(); NUM_HEADS],
            stacks: vec![Stack::new(); NUM_STACKS],
            
            read_label: CodeLabel::new(),
            next_label: CodeLabel::new(),

            genome,
            initial_genome_size,

            cur_stack: 0,
            advance_ip: true,
            mal_active: false,
        }
    }

    fn reset(&mut self) {
        self.registers.fill(0);
        self.heads.iter_mut().for_each(|head| head.reset());
        self.stacks.iter_mut().for_each(|stack| stack.reset());
        self.cur_stack = 0;
        self.read_label.clear();
        self.next_label.clear();
    }

    pub fn single_process(&mut self, world: &mut World, octx: &mut OrganismContext) -> Option<Offspring> {
        self.advance_ip = true;
        self.heads[HEAD_IP].adjust(&self.genome);

        octx.phenotype.time_used += 1;

        let instruction = self.heads[HEAD_IP].get_inst(&self.genome);
        let offspring = self.single_process_execinst(instruction, world, octx);

        if self.advance_ip {
            self.heads[HEAD_IP].advance(&self.genome);
        }

        offspring
    }

    fn single_process_execinst(&mut self, instruction: Instruction, world: &mut World, octx: &mut OrganismContext) -> Option<Offspring> {
        match instruction {
            Instruction::NopA => { None },
            Instruction::NopB => { None },
            Instruction::NopC => { None },

            Instruction::IfNEqu => { self.inst_ifneq(); None },
            Instruction::IfLess => { self.inst_ifless(); None },
            Instruction::IfLabel => { self.inst_iflabel(); None },
            Instruction::MovHead => { self.inst_movhead(); None },
            Instruction::JmpHead => { self.inst_jmphead(); None },
            Instruction::GetHead => { self.inst_gethead(); None },
            Instruction::SetFlow => { self.inst_setflow(); None },

            Instruction::ShiftR => { self.inst_shiftr(); None },
            Instruction::ShiftL => { self.inst_shiftl(); None },
            Instruction::Inc => { self.inst_inc(); None },
            Instruction::Dec => { self.inst_dec(); None },
            Instruction::Push => { self.inst_push(); None },
            Instruction::Pop => { self.inst_pop(); None },
            Instruction::SwapStk => { self.inst_swapstk(); None },
            Instruction::Swap => { self.inst_swap(); None },

            Instruction::Add => { self.inst_add(); None },
            Instruction::Sub => { self.inst_sub(); None },
            Instruction::Nand => { self.inst_nand(); None },

            Instruction::HCopy => { self.inst_hcopy(world); None },
            Instruction::HAlloc => { self.inst_halloc(); None },
            Instruction::HDivide => { self.inst_hdivide() },

            Instruction::IO => { self.inst_io(world, octx); None },
            Instruction::HSearch => { self.inst_hsearch(); None },

            Instruction::Error => { None },
        }
    }

    fn find_modified_head(&mut self, default_head: usize) -> usize {
        let ip = &mut self.heads[HEAD_IP];
        if ip.get_next_inst(&self.genome).is_nop() {
            ip.advance(&self.genome);
            return ip.get_inst(&self.genome).get_nop_mod();
        }
        default_head
    }

    fn find_modified_register(&mut self, default_register: usize) -> usize {
        let ip = &mut self.heads[HEAD_IP];
        if ip.get_next_inst(&self.genome).is_nop() {
            ip.advance(&self.genome);
            // flag executed?
            return ip.get_inst(&self.genome).get_nop_mod();
        }
        default_register
    }

    fn find_next_register(base_reg: usize) -> usize {
        (base_reg + 1) % NUM_REGISTERS
    }

    // fn find_modified_next_register(&mut self, default_register: usize) -> usize {
    //     let ip = &mut self.heads[HEAD_IP];
    //     if ip.get_next_inst(&self.genome).is_nop() {
    //         ip.advance(&self.genome);
    //         // flag executed?
    //         return ip.get_inst(&self.genome).get_nop_mod();
    //     }
    //     (default_register + 1) % NUM_REGISTERS
    // }

    fn allocate_main(&mut self, alloc_size: usize) -> bool {
        if self.mal_active {
            return false;
        }
        if alloc_size == 0 {
            return false;
        }

        let old_size = self.genome.len();
        let new_size = old_size + alloc_size;

        if new_size > Genome::MAX_SIZE {
            return false;
        }

        let max_alloc_size = old_size * OFFSPRING_SIZE_RANGE;
        if alloc_size > max_alloc_size {
            // too large
            return false;
        }

        let max_old_size = alloc_size * OFFSPRING_SIZE_RANGE;
        if old_size > max_old_size {
            // too small
            return false;
        }

        let new_size = self.genome.len() + alloc_size;
        self.genome.resize(new_size, Instruction::NopA);
        self.mal_active = true;

        true
    }

    fn read_next_label(&mut self) {
        const MAX_SIZE: usize = 10;
        let mut count = 0;
        let ip = &mut self.heads[HEAD_IP];

        self.next_label.clear();

        while count < MAX_SIZE && ip.get_next_inst(&self.genome).is_nop() {
            count += 1;
            ip.advance(&self.genome);
            let nop_mod = ip.get_inst(&self.genome).get_nop_mod();
            self.next_label.push(nop_mod)
        }
    }

    fn find_label(&self, direction: i64) -> isize {
        if self.next_label.is_empty() {
            return self.heads[HEAD_IP].position;
        }

        let mut found_pos;
        if direction < 0 {
            // FIXME: backward search unimplemented
            found_pos = 0;
        } else if direction > 0 {
            let start_pos = self.heads[HEAD_IP].position;
            found_pos = Self::find_label_forward(&self.next_label, &self.genome, start_pos);
        } else {
            found_pos = Self::find_label_forward(&self.next_label, &self.genome, 0);
        }

        // Return the last line of the found label, if it was found.
        if found_pos >= 0 {
            found_pos -= 1;
        }

        found_pos
    }

    fn find_label_forward(search_label: &CodeLabel, genome: &Genome, search_pos: isize) -> isize {
        let mut pos = search_pos;
        let label_size = search_label.len() as isize;
        let mut found = false;

        pos += label_size;

        while pos < genome.len() as isize {
            // if in a label, rewind to the beginning of it
            if genome[pos as usize].is_nop() {
                let mut start = pos;
                let mut end = pos + 1;
                while start > search_pos && genome[(start - 1) as usize].is_nop() {
                    start -= 1;
                }
                while end < genome.len() as isize && genome[end as usize].is_nop() {
                    end += 1;
                }

                let test_size = end - start;

                let max_offset = test_size - label_size + 1;
                let mut offset = start;
                while offset < start + max_offset {
                    let mut matches = 0;
                    while matches < label_size {
                        if search_label[matches as usize] != genome[(offset + matches) as usize].get_nop_mod() {
                            break;
                        }
                        matches += 1;
                    }

                    if matches == label_size {
                        found = true;
                        break;
                    }

                    offset += 1;
                }

                // if we've found the complement label, set the position
                if found {
                    pos = label_size + offset;
                    break;
                }

                pos = end;
            }

            pos += label_size;
        }

        if !found {
            return -1
        }
        pos
    }

    fn stack_push(&mut self, v: i64) {
        self.stacks[self.cur_stack].push(v);
    }

    fn stack_pop(&mut self) -> i64 {
        self.stacks[self.cur_stack].pop()
    }



    fn inst_ifneq(&mut self) {
        let op1 = self.find_modified_register(REG_BX);
        let op2 = Self::find_next_register(op1);
        if self.registers[op1] == self.registers[op2] {
            self.heads[HEAD_IP].advance(&self.genome);
        }
    }

    fn inst_ifless(&mut self) {
        let reg = self.find_modified_register(REG_BX);
        if self.registers[reg] >= 0 {
            self.heads[HEAD_IP].advance(&self.genome);
        }
    }

    fn inst_iflabel(&mut self) {
        self.read_next_label();
        self.next_label.rotate(1, NUM_NOPS);
        if self.next_label != self.read_label {
            self.heads[HEAD_IP].advance(&self.genome);
        }
    }

    fn inst_movhead(&mut self) {
        let head = self.find_modified_head(HEAD_IP);
        let new_pos = self.heads[HEAD_FLOW].position;
        self.heads[head].set(new_pos, &self.genome);
        if head == HEAD_IP {
            self.advance_ip = false;
        }
    }

    fn inst_jmphead(&mut self) {
        let head = self.find_modified_register(HEAD_IP);
        self.heads[head].jump(self.registers[REG_CX] as isize, &self.genome);
    }

    fn inst_gethead(&mut self) {
        let head = self.find_modified_register(HEAD_IP);
        self.registers[REG_CX] = self.heads[head].position as i64;
    }

    fn inst_setflow(&mut self) {
        let reg = self.find_modified_register(REG_CX);
        self.heads[HEAD_FLOW].set(self.registers[reg] as isize, &self.genome);
    }

    fn inst_shiftr(&mut self) {
        let reg = self.find_modified_register(REG_BX);
        self.registers[reg] >>= 1;
    }

    fn inst_shiftl(&mut self) {
        let reg = self.find_modified_register(REG_BX);
        self.registers[reg] <<= 1;
    }

    fn inst_inc(&mut self) {
        let reg = self.find_modified_register(REG_BX);
        self.registers[reg] += 1;
    }

    fn inst_dec(&mut self) {
        let reg = self.find_modified_register(REG_BX);
        self.registers[reg] -= 1;
    }

    fn inst_push(&mut self) {
        let reg = self.find_modified_register(REG_BX);
        self.stack_push(self.registers[reg]);
    }

    fn inst_pop(&mut self) {
        let reg = self.find_modified_register(REG_BX);
        self.registers[reg] = self.stack_pop();
    }

    fn inst_swapstk(&mut self) {
        self.cur_stack = (self.cur_stack + 1) % self.stacks.len()
    }

    fn inst_swap(&mut self) {
        let op1 = self.find_modified_register(REG_BX);
        let op2 = Self::find_next_register(op1);
        self.registers.swap(op1, op2);
    }

    fn inst_add(&mut self) {
        let dst = self.find_modified_register(REG_BX);
        self.registers[dst] = self.registers[REG_BX].wrapping_add(self.registers[REG_CX]);
    }

    fn inst_sub(&mut self) {
        let dst = self.find_modified_register(REG_BX);
        self.registers[dst] = self.registers[REG_BX].wrapping_sub(self.registers[REG_CX]);
    }

    fn inst_nand(&mut self) {
        let dst = self.find_modified_register(REG_BX);
        self.registers[dst] = !(self.registers[REG_BX] & self.registers[REG_CX]);
    }

    fn inst_hcopy(&mut self, world: &mut World) {
        let (read_head, write_head) = split_two_mut(&mut self.heads, HEAD_READ, HEAD_WRITE);

        read_head.adjust(&self.genome);
        write_head.adjust(&self.genome);

        let mut read_inst = read_head.get_inst(&self.genome);
        self.read_label.read_inst(read_inst);

        // substition mutations here
        if world.config.mutation_rate > 0.0 && world.rng.random_bool(world.config.mutation_rate) {
            read_inst = world.instruction_set.random(&mut world.rng);
        }

        write_head.set_inst(read_inst, &mut self.genome);

        // ins/del/slip mutations here

        read_head.advance(&self.genome);
        write_head.advance(&self.genome);
    }

    fn inst_halloc(&mut self) {
        let cur_size = self.genome.len();
        let alloc_size = (cur_size * OFFSPRING_SIZE_RANGE).min(Genome::MAX_SIZE - cur_size);

        if self.allocate_main(alloc_size) {
            self.registers[REG_AX] = cur_size as i64;
        }
    }

    fn inst_hdivide(&mut self) -> Option<Offspring> {
        self.heads.iter_mut().for_each(|head| head.adjust(&self.genome));

        let len = self.genome.len() as isize;
        let divide_pos = self.heads[HEAD_READ].position;
        let mut child_end = self.heads[HEAD_WRITE].position;
        if child_end == 0 {
            child_end = len;
        }
        let extra_lines = len - child_end;

        let child_size = len - divide_pos - extra_lines;
        let min_size = Genome::MIN_SIZE.max(self.initial_genome_size / OFFSPRING_SIZE_RANGE) as isize;
        let max_size = Genome::MAX_SIZE.min(self.initial_genome_size * OFFSPRING_SIZE_RANGE) as isize;

        // Check that both the child and parent are within size bounds
        if child_size < min_size || child_size > max_size {
            return None
        }
        if divide_pos < min_size || divide_pos > max_size {
            return None
        }

        let divide_pos = divide_pos as usize;
        let child_size = child_size as usize;

        let child_genome = Genome::new(self.genome[divide_pos..divide_pos + child_size].to_vec());

        // FIXME: divide mutations

        self.genome.resize(divide_pos, Instruction::Error);
        self.mal_active = false;
        self.heads.iter_mut().for_each(|head| head.adjust(&self.genome));

        if true { // DIVIDE_METHOD_SPLIT
            self.advance_ip = false; // DIVIDE_METHOD_SPLIT
            self.reset()
        }

        Some(Offspring::new(child_genome))
    }

    fn inst_io(&mut self, world: &mut World, octx: &mut OrganismContext) {
        let reg = self.find_modified_register(REG_BX);

        let value_out = self.registers[reg];
        octx.process_output(world, value_out);

        let value_in = octx.get_next_input();
        self.registers[reg] = value_in;
    }

    fn inst_hsearch(&mut self) {
        self.read_next_label();
        self.next_label.rotate(1, NUM_NOPS);
        let found_pos = self.find_label(0);
        let search_size = found_pos - self.heads[HEAD_IP].position;
        self.registers[REG_BX] = search_size as i64;
        self.registers[REG_CX] = self.next_label.len() as i64;
        self.heads[HEAD_FLOW].set(found_pos, &self.genome);
        self.heads[HEAD_FLOW].advance(&self.genome);
    }
}

fn split_two_mut<T>(slice: &mut [T], i: usize, j: usize) -> (&mut T, &mut T) {
    assert!(i != j, "split_two_mut requires distinct indices");
    if i < j {
        let (a, b) = slice.split_at_mut(j);
        (&mut a[i], &mut b[0])
    } else {
        let (a, b) = slice.split_at_mut(i);
        (&mut b[0], &mut a[j])
    }
}