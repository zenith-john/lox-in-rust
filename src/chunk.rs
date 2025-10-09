use crate::object::LoxType;
use crate::vm::RuntimeError;
use crate::USIZE;

pub const OP_RETURN: u8 = 0;
pub const OP_CONSTANT: u8 = 1;
pub const OP_NEGATE: u8 = 2;
pub const OP_ADD: u8 = 3;
pub const OP_SUBTRACT: u8 = 4;
pub const OP_MULTIPLY: u8 = 5;
pub const OP_DIVIDE: u8 = 6;
pub const OP_NIL: u8 = 7;
pub const OP_TRUE: u8 = 8;
pub const OP_FALSE: u8 = 9;
pub const OP_NOT: u8 = 10;
pub const OP_EQUAL: u8 = 11;
pub const OP_GREATER: u8 = 12;
pub const OP_LESS: u8 = 13;
pub const OP_PRINT: u8 = 14;
pub const OP_POP: u8 = 15;
pub const OP_DEFINE_GLOBAL: u8 = 16;
pub const OP_GET_GLOBAL: u8 = 17;
pub const OP_SET_GLOBAL: u8 = 18;
pub const OP_GET_LOCAL: u8 = 19;
pub const OP_SET_LOCAL: u8 = 20;
pub const OP_JUMP_IF_FALSE: u8 = 21;
pub const OP_JUMP: u8 = 22;
pub const OP_LOOP: u8 = 23;
pub const OP_CALL: u8 = 24;
pub const OP_CLASS: u8 = 25;
pub const OP_GET_PROPERTY: u8 = 26;
pub const OP_SET_PROPERTY: u8 = 27;
pub const OP_CLOSURE: u8 = 28;
pub const OP_GET_UPVALUE: u8 = 29;
pub const OP_SET_UPVALUE: u8 = 30;
pub const OP_CLOSE_UPVALUE: u8 = 31;
pub const OP_METHOD: u8 = 32;
pub const OP_INHERIT: u8 = 33;
pub const OP_GET_SUPER: u8 = 34;

pub type Value = LoxType;

#[derive(Clone)]
struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> ValueArray {
        ValueArray { values: Vec::new() }
    }

    pub fn write_value(&mut self, val: Value) -> usize {
        self.values.push(val);
        self.values.len() - 1
    }

    pub fn get_value(&self, pos: usize) -> Value {
        self.values[pos].clone()
        // Hopefully remove clone in the future
    }
}

#[derive(Clone)]
pub struct Chunk {
    code: Vec<u8>,
    constants: ValueArray,
    lines: Vec<i32>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: ValueArray::new(),
            lines: Vec::new(),
        }
    }

    pub fn read_chunk(&self, pos: usize) -> Result<u8, RuntimeError> {
        if pos >= self.code.len() {
            Err(RuntimeError {
                line: -1,
                reason: "Index out of Chunk".to_string(),
            })
        } else {
            Ok(self.code[pos])
        }
    }

    pub fn read_jump(&self, pos: usize) -> Result<usize, RuntimeError> {
        let mut bytes: [u8; USIZE] = [0; USIZE];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = self.read_chunk(pos + i)?;
        }
        Ok(usize::from_ne_bytes(bytes))
    }

    pub fn modify_chunk(&mut self, pos: usize, byte: u8) {
        self.code[pos] = byte;
    }

    pub fn read_line(&self, pos: usize) -> Result<i32, RuntimeError> {
        if pos >= self.code.len() {
            Err(RuntimeError {
                line: -1,
                reason: "Index out of Chunk".to_string(),
            })
        } else {
            Ok(self.lines[pos])
        }
    }

    pub fn write_chunk(&mut self, byte: u8, line: i32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn disassemble_chunk(&self) {
        let mut offset: usize = 0;
        while offset < self.len() {
            offset = self.disassemble_instruction(offset);
        }
        eprintln!();
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        let instruction: u8 = self.code[offset];
        match instruction {
            OP_RETURN => self.simple_instruction("OP_RETURN".to_string(), offset),
            OP_CONSTANT => self.constant_instruction("OP_CONSTANT".to_string(), offset),
            OP_NEGATE => self.simple_instruction("OP_NEGATE".to_string(), offset),
            OP_ADD => self.simple_instruction("OP_ADD".to_string(), offset),
            OP_SUBTRACT => self.simple_instruction("OP_SUBTRACT".to_string(), offset),
            OP_MULTIPLY => self.simple_instruction("OP_MULTIPLY".to_string(), offset),
            OP_DIVIDE => self.simple_instruction("OP_DIVIDE".to_string(), offset),
            OP_NIL => self.simple_instruction("OP_NIL".to_string(), offset),
            OP_TRUE => self.simple_instruction("OP_TRUE".to_string(), offset),
            OP_FALSE => self.simple_instruction("OP_FALSE".to_string(), offset),
            OP_NOT => self.simple_instruction("OP_NOT".to_string(), offset),
            OP_EQUAL => self.simple_instruction("OP_EQUAL".to_string(), offset),
            OP_GREATER => self.simple_instruction("OP_GREATER".to_string(), offset),
            OP_LESS => self.simple_instruction("OP_LESS".to_string(), offset),
            OP_PRINT => self.simple_instruction("OP_PRINT".to_string(), offset),
            OP_POP => self.simple_instruction("OP_POP".to_string(), offset),
            OP_DEFINE_GLOBAL => self.constant_instruction("OP_DEFINE_GLOBAL".to_string(), offset),
            OP_GET_GLOBAL => self.constant_instruction("OP_GET_GLOBAL".to_string(), offset),
            OP_SET_GLOBAL => self.constant_instruction("OP_SET_GLOBAL".to_string(), offset),
            OP_GET_LOCAL => self.byte_instruction("OP_GET_LOCAL".to_string(), offset),
            OP_SET_LOCAL => self.byte_instruction("OP_SET_LOCAL".to_string(), offset),
            OP_JUMP_IF_FALSE => self.jump_instruction("OP_JUMP_IF_FALSE".to_string(), offset),
            OP_JUMP => self.jump_instruction("OP_JUMP".to_string(), offset),
            OP_LOOP => self.loop_instruction("OP_LOOP".to_string(), offset),
            OP_CALL => self.byte_instruction("OP_CALL".to_string(), offset),
            OP_CLASS => self.constant_instruction("OP_CLASS".to_string(), offset),
            OP_GET_PROPERTY => self.byte_instruction("OP_GET_PROPERTY".to_string(), offset),
            OP_SET_PROPERTY => self.byte_instruction("OP_SET_PROPERTY".to_string(), offset),
            OP_CLOSURE => {
                let pos = self.code[offset + 1];
                let val = self.constants.get_value(pos as usize);
                eprintln!("[{}] OP_CLOSURE {}", offset, val);
                let func = val.as_function().expect("Value is not a function");
                let upvalue = func.upvalue as usize;
                for i in 0..upvalue {
                    let is_local = self.code[offset + 2 + 2 * i];
                    let index = self.code[offset + 2 + 2 * i];
                    eprintln!(
                        "[{}] {}: {}",
                        offset + 2 + 2 * i,
                        if is_local == 1 { "Local" } else { "Upvalue" },
                        index
                    );
                }
                offset + 2 + 2 * upvalue
            }
            OP_GET_UPVALUE => self.byte_instruction("OP_GET_UPVALUE".to_string(), offset),
            OP_SET_UPVALUE => self.byte_instruction("OP_SET_UPVALUE".to_string(), offset),
            OP_CLOSE_UPVALUE => self.simple_instruction("OP_CLOSE_UPVALUE".to_string(), offset),
            OP_METHOD => self.constant_instruction("OP_SET_GLOBAL".to_string(), offset),
            OP_INHERIT => self.simple_instruction("OP_INHERIT".to_string(), offset),
            OP_GET_SUPER => self.constant_instruction("GET_SUPER".to_string(), offset),
            _ => {
                panic!("Line {}: Unknown code {}", self.lines[offset], instruction);
            }
        }
    }

    pub fn read_constant(&self, offset: usize) -> Result<Value, RuntimeError> {
        if offset >= self.code.len() {
            Err(RuntimeError {
                line: -1,
                reason: "Index out of Constant".to_string(),
            })
        } else {
            Ok(self.constants.get_value(offset))
        }
    }

    pub fn add_constant(&mut self, val: Value) -> usize {
        self.constants.write_value(val)
    }

    fn simple_instruction(&self, name: String, offset: usize) -> usize {
        eprintln!("[{}] {}", offset, name);
        offset + 1
    }

    fn constant_instruction(&self, name: String, offset: usize) -> usize {
        let pos = self.code[offset + 1];
        let val = self.constants.get_value(pos as usize);
        eprintln!("[{}] {} {}", offset, name, val);
        offset + 2
    }

    fn byte_instruction(&self, name: String, offset: usize) -> usize {
        eprintln!("[{}] {} {}", offset, name, self.code[offset + 1]);
        offset + 2
    }

    fn jump_instruction(&self, name: String, offset: usize) -> usize {
        let address = self.read_jump(offset + 1).expect("Can not get address");
        eprintln!("[{}] {} -> {}", offset, name, offset + USIZE + 1 + address);
        offset + 1 + USIZE
    }

    fn loop_instruction(&self, name: String, offset: usize) -> usize {
        let address = self.read_jump(offset + 1).expect("Can not get address");
        eprintln!("[{}] {} -> {}", offset, name, offset + USIZE + 1 - address);
        offset + 1 + USIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_write() {
        let mut chunk = Chunk::new();
        chunk.write_chunk(1, 0);
        chunk.write_chunk(2, 0);
        assert_eq!(chunk.len(), 2);
    }

    #[test]
    #[should_panic = "Unknown code"]
    fn test_chunk_disassemble() {
        let mut chunk = Chunk::new();
        chunk.add_constant(LoxType::Number(1.0));
        chunk.write_chunk(0, 1);
        chunk.write_chunk(1, 1);
        chunk.write_chunk(0, 1);
        chunk.write_chunk(100, 1);
        chunk.disassemble_chunk();
    }
}
