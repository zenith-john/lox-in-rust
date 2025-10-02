use crate::chunk;
use crate::chunk::{Chunk, Value};
use crate::error::RuntimeError;

const DEBUG: bool = true;

pub struct VM {
    vm: Box<Chunk>,
    ip: usize,
    stack: Vec<Value>,
}

macro_rules! binary_op {
    ($stack:expr, $op:tt) => {{
        let b = $stack.pop();
        let a = $stack.pop();
        $stack.push(a $op b);
    }};
}

impl VM {
    pub fn init() -> VM {
        return VM {
            vm: Box::new(Chunk::new()),
            ip: 0,
            stack: Vec::new(),
        };
    }

    pub fn reset_stack(&mut self) {
        self.stack = Vec::new();
    }

    pub fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    pub fn pop(&mut self) -> Value {
        return self.stack.pop().expect("Pop from empty stack");
    }

    pub fn interpret(&mut self, chunk: Box<Chunk>) -> Result<(), RuntimeError> {
        self.vm = chunk;
        self.ip = 0;
        self.reset_stack();
        return self.run();
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        while self.ip < self.vm.len() {
            if DEBUG {
                self.vm.disassemble_instruction(self.ip);
                println!("");
                for val in &self.stack {
                    print!("[ {} ]", val);
                }
                println!("");
            }
            let op = self.read_chunk()?;
            match op {
                chunk::OP_RETURN => {
                    println!("{}", self.pop());
                    return Ok(());
                }
                chunk::OP_CONSTANT => {
                    let offset = self.read_chunk()?;
                    let constant = self.vm.read_constant(offset as usize)?;
                    self.push(constant);
                }
                chunk::OP_NEGATE => {
                    let val = -self.pop();
                    self.push(val);
                }
                chunk::OP_ADD => {
                    binary_op!(self, +);
                }
                chunk::OP_SUBTRACT => {
                    binary_op!(self, -);
                }
                chunk::OP_MULTIPLY => {
                    binary_op!(self, *);
                }
                chunk::OP_DIVIDE => {
                    binary_op!(self, /);
                }
                _ => {
                    return Err(RuntimeError::Reason {
                        reason: "Unknown command".to_string(),
                        line: self.vm.read_line(self.ip - 1)?,
                    })
                }
            }
        }
        return Err(RuntimeError::Reason {
            reason: "Don't find return command".to_string(),
            line: -1,
        });
    }

    fn read_chunk(&mut self) -> Result<u8, RuntimeError> {
        self.ip += 1;
        self.vm.read_chunk(self.ip - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_negate() {
        let mut chunk = Chunk::new();
        chunk.add_constant(1.0);
        chunk.add_constant(3.0);
        chunk.write_chunk(1, 0);
        chunk.write_chunk(0, 0);
        chunk.write_chunk(1, 0);
        chunk.write_chunk(1, 0);
        chunk.write_chunk(2, 0);
        chunk.write_chunk(3, 0);
        chunk.write_chunk(0, 0);
        let mut vm = VM::init();
        let _ = vm.interpret(Box::new(chunk));
        // should output -1
    }
}
