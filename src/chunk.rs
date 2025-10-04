use crate::error::RuntimeError;

pub const OP_RETURN: u8 = 0;
pub const OP_CONSTANT: u8 = 1;
pub const OP_NEGATE: u8 = 2;
pub const OP_ADD: u8 = 3;
pub const OP_SUBTRACT: u8 = 4;
pub const OP_MULTIPLY: u8 = 5;
pub const OP_DIVIDE: u8 = 6;

pub type Value = f64;

struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> ValueArray {
        ValueArray { values: Vec::new() }
    }

    pub fn write_value(&mut self, val: Value) -> usize {
        self.values.push(val);
        return self.values.len() - 1;
    }

    pub fn get_value(&self, pos: usize) -> Value {
        self.values[pos]
    }
}

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
            Err(RuntimeError::Reason {
                line: -1,
                reason: "Index out of Chunk".to_string(),
            })
        } else {
            Ok(self.code[pos])
        }
    }

    pub fn read_line(&self, pos: usize) -> Result<i32, RuntimeError> {
        if pos >= self.code.len() {
            Err(RuntimeError::Reason {
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
            _ => {
                panic!("Line {}: Unknown code {}", self.lines[offset], instruction);
            }
        }
    }

    pub fn read_constant(&self, offset: usize) -> Result<Value, RuntimeError> {
        if offset >= self.code.len() {
            Err(RuntimeError::Reason {
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
        println!("{}", name);
        offset + 1
    }

    fn constant_instruction(&self, name: String, offset: usize) -> usize {
        let pos = self.code[offset + 1];
        let val = self.constants.get_value(pos as usize);
        println!("{} {}", name, val);
        offset + 2
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
        chunk.add_constant(1.0);
        chunk.write_chunk(0, 1);
        chunk.write_chunk(1, 1);
        chunk.write_chunk(0, 1);
        chunk.write_chunk(100, 1);
        chunk.disassemble_chunk();
    }
}
