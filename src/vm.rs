use crate::chunk;
use crate::chunk::{Chunk, Value};
use crate::error::RuntimeError;
use crate::token::BasicType;
use crate::DEBUG;

pub struct VM<'a> {
    vm: &'a Chunk,
    ip: usize,
    stack: Vec<Value>,
}

macro_rules! binary_op {
    ($stack:expr, $op:tt) => {{
        if let (Some(a), Some(b)) = ($stack.peek(0).as_number(), $stack.peek(1).as_number()) {
            $stack.pop();
            $stack.pop();
            $stack.push(BasicType::Number(b $op a));
        }
        else {
            return Err(RuntimeError::Reason {
                line: $stack.vm.read_line($stack.ip - 1)?,
                reason: "Operands must be numbers.".to_string()
            }
            )
        }
    }};
}

macro_rules! binary_op_bool {
    ($stack:expr, $op:tt) => {{
        if let (Some(a), Some(b)) = ($stack.peek(0).as_number(), $stack.peek(1).as_number()) {
            $stack.pop();
            $stack.pop();
            $stack.push(BasicType::Bool(b $op a));
        }
        else {
            return Err(RuntimeError::Reason {
                line: $stack.vm.read_line($stack.ip - 1)?,
                reason: "Operands must be numbers.".to_string()
            }
            )
        }
    }};
}

impl<'a> VM<'a> {
    pub fn init(chunk: &'a Chunk) -> VM<'a> {
        return VM {
            vm: chunk,
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

    pub fn peek(&mut self, distance: usize) -> Value {
        return self.stack[self.stack.len() - 1 - distance].clone();
        // Hopefully remove clone in the future
    }

    pub fn interpret(&mut self, chunk: &'a Chunk) {
        self.vm = chunk;
        self.ip = 0;
        self.reset_stack();
        if let Err(e) = self.run() {
            eprintln!("{:?}", e);
            self.reset_stack();
        }
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
                    if let Some(x) = self.peek(0).as_number() {
                        self.pop();
                        let val = BasicType::Number(-x);
                        self.push(val);
                    } else {
                        return Err(RuntimeError::Reason {
                            line: self.vm.read_line(self.ip - 1)?,
                            reason: "Operand must be a number".to_string(),
                        });
                    }
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
                chunk::OP_NIL => {
                    self.push(BasicType::None);
                }
                chunk::OP_TRUE => {
                    self.push(BasicType::Bool(true));
                }
                chunk::OP_FALSE => {
                    self.push(BasicType::Bool(false));
                }
                chunk::OP_NOT => {
                    let logic = match self.pop() {
                        BasicType::None => true,
                        BasicType::Bool(x) => !x,
                        _ => false,
                    };
                    // permissive NOT
                    self.push(BasicType::Bool(logic))
                }
                chunk::OP_EQUAL => {
                    let left = self.pop();
                    let right = self.pop();
                    self.push(BasicType::Bool(left == right))
                }
                chunk::OP_GREATER => {
                    binary_op_bool!(self, >)
                }
                chunk::OP_LESS => {
                    binary_op_bool!(self, <)
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
        chunk.add_constant(BasicType::Number(1.0));
        chunk.add_constant(BasicType::Number(3.0));
        chunk.write_chunk(1, 0);
        chunk.write_chunk(0, 0);
        chunk.write_chunk(1, 0);
        chunk.write_chunk(1, 0);
        chunk.write_chunk(2, 0);
        chunk.write_chunk(3, 0);
        chunk.write_chunk(0, 0);
        let mut vm = VM::init(&chunk);
        let _ = vm.interpret(&chunk);
        // should output -1
    }
}
