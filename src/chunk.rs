#[repr(u8)]
enum Operation {
    OpReturn = 0,
    OpConstant = 1,
    Default,
}
type Value = f64;

struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> ValueArray {
        ValueArray { values: Vec::new() }
    }

    pub fn write_value(&mut self, val: Value) {
        self.values.push(val)
    }

    pub fn get_value(&self, pos: usize) -> Value {
        self.values[pos]
    }
}
struct Chunk {
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
    pub fn write_chunk(&mut self, byte: u8, line: i32) {
        self.code.push(byte);
        self.lines.push(line);
    }
    pub fn capacity(&self) -> usize {
        self.code.capacity()
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

    fn disassemble_instruction(&self, offset: usize) -> usize {
        let instruction: u8 = self.code[offset];
        match instruction {
            0 => self.simple_instruction("OP_RETURN".to_string(), offset),
            1 => self.constant_instruction("OP_CONSTANT".to_string(), offset),
            _ => {
                panic!("Line {}: Unknown code {}", self.lines[offset], instruction);
            }
        }
    }

    fn add_constant(&mut self, val: Value) {
        self.constants.write_value(val);
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
    fn test_chunk_new() {
        let chunk = Chunk::new();
        assert_eq!(chunk.capacity(), 0);
    }

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
        chunk.write_chunk(2, 1);
        chunk.disassemble_chunk();
    }
}
