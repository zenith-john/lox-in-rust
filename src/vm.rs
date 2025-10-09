use crate::chunk;
use crate::chunk::Value;
use crate::object::{Class, Closure, Function, Instance, LoxType, Upvalue};
use crate::{BACKTRACE, DEBUG, USIZE};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct RuntimeError {
    pub reason: String,
    pub line: i32,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Line {}] in script, Runtime Error: {}",
            self.line, self.reason
        )
    }
}
impl std::error::Error for RuntimeError {}

pub struct VM {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    frames: Vec<Rc<RefCell<CallFrame>>>,
    captures: HashMap<usize, Rc<RefCell<Upvalue>>>,
}

macro_rules! binary_op {
    ($stack:expr, $op:tt, $frame: expr) => {{
        if let (Some(a), Some(b)) = ($stack.peek(0).as_number(), $stack.peek(1).as_number()) {
            $stack.pop();
            $stack.pop();
            $stack.push(LoxType::Number(b $op a));
        }
        else {
            return Err(RuntimeError {
                line: $frame.read_line()?,
                reason: "Operands must be numbers.".to_string()
            }
            )
        }
    }};
}

macro_rules! binary_op_bool {
    ($stack:expr, $op:tt, $frame: expr) => {{
        if let (Some(a), Some(b)) = ($stack.peek(0).as_number(), $stack.peek(1).as_number()) {
            $stack.pop();
            $stack.pop();
            $stack.push(LoxType::Bool(b $op a));
        }
        else {
            return Err(RuntimeError {
                line: $frame.read_line()?,
                reason: "Operands must be numbers.".to_string()
            }
            )
        }
    }};
}

impl VM {
    pub fn init() -> VM {
        VM {
            stack: Vec::new(),
            globals: HashMap::new(),
            frames: Vec::new(),
            captures: HashMap::new(),
        }
    }

    fn current(&self) -> Rc<RefCell<CallFrame>> {
        self.frames.last().expect("Frame is empty").clone()
    }

    pub fn reset_stack(&mut self) {
        self.stack.clear();
    }

    pub fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    pub fn pop(&mut self) -> Value {
        self.stack.pop().expect("Pop from empty stack")
    }

    pub fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - distance]
        // Hopefully remove clone in the future
    }

    pub fn interpret(&mut self, func: Rc<Function>) {
        let clos = Closure::new(func);
        self.push(LoxType::Closure(clos.clone()));
        let _ = self.call(clos, 0);
        if let Err(e) = self.run() {
            if BACKTRACE {
                eprintln!("Backtrace:");
                for frame in self.frames.iter().rev() {
                    let f = frame.borrow();
                    let line = f.read_line().unwrap();
                    eprintln!(
                        "[Line {}] in {}",
                        line,
                        if f.closure.function.name.is_empty() {
                            "Script"
                        } else {
                            &f.closure.function.name
                        }
                    );
                    eprintln!();
                }
            }
            self.reset_stack();
            eprintln!("{}", e);
        }
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        while !self.frames.is_empty() {
            let binding = self.current();
            let mut current = binding.borrow_mut();
            while current.ip < current.closure.function.chunk.len() {
                if DEBUG {
                    eprintln!();
                    for val in &self.stack {
                        eprint!("[ {} ]", val);
                    }
                    eprintln!();
                    current
                        .closure
                        .function
                        .chunk
                        .disassemble_instruction(current.ip);
                }
                if self.stack.len() > 16 * 256 {
                    return Err(RuntimeError {
                        reason: "Stack overflow".to_string(),
                        line: -1,
                    });
                }
                let op = current.read_chunk()?;
                match op {
                    chunk::OP_RETURN => {
                        let ret = self.pop();
                        for i in (current.slot..self.stack.len()).rev() {
                            self.close_upvalues(i); // Expected to optimize in the future
                        }
                        self.frames.pop();
                        if self.frames.is_empty() {
                            self.pop();
                            return Ok(());
                        }
                        let slot = current.slot;
                        self.stack.truncate(slot);
                        self.push(ret);
                        break;
                    }
                    chunk::OP_CONSTANT => {
                        let offset = current.read_chunk()?;
                        let constant = current.read_constant(offset as usize)?;
                        self.push(constant);
                    }
                    chunk::OP_NEGATE => {
                        if let Some(x) = self.peek(0).as_number() {
                            self.pop();
                            let val = LoxType::Number(-x);
                            self.push(val);
                        } else {
                            return Err(RuntimeError {
                                line: current.read_line()?,
                                reason: "Operand must be a number".to_string(),
                            });
                        }
                    }
                    chunk::OP_ADD => {
                        if let (Some(a), Some(b)) =
                            (self.peek(0).as_number(), self.peek(1).as_number())
                        {
                            self.pop();
                            self.pop();
                            self.push(LoxType::Number(b + a));
                        } else if let (Some(a), Some(b)) =
                            (self.peek(0).as_string(), self.peek(1).as_string())
                        {
                            self.pop();
                            self.pop();
                            self.push(LoxType::String(b + &a))
                        } else {
                            return Err(RuntimeError {
                                line: current.read_line()?,
                                reason: "Operands must be numbers.".to_string(),
                            });
                        }
                    }
                    chunk::OP_SUBTRACT => {
                        binary_op!(self, -, current);
                    }
                    chunk::OP_MULTIPLY => {
                        binary_op!(self, *, current);
                    }
                    chunk::OP_DIVIDE => {
                        binary_op!(self, /, current);
                    }
                    chunk::OP_NIL => {
                        self.push(LoxType::None);
                    }
                    chunk::OP_TRUE => {
                        self.push(LoxType::Bool(true));
                    }
                    chunk::OP_FALSE => {
                        self.push(LoxType::Bool(false));
                    }
                    chunk::OP_NOT => {
                        let logic = match self.pop() {
                            LoxType::None => true,
                            LoxType::Bool(x) => !x,
                            _ => false,
                        };
                        // permissive NOT
                        self.push(LoxType::Bool(logic))
                    }
                    chunk::OP_EQUAL => {
                        let left = self.pop();
                        let right = self.pop();
                        self.push(LoxType::Bool(left == right))
                    }
                    chunk::OP_GREATER => {
                        binary_op_bool!(self, >, current)
                    }
                    chunk::OP_LESS => {
                        binary_op_bool!(self, <, current)
                    }
                    chunk::OP_PRINT => {
                        println!("{}", self.pop());
                    }
                    chunk::OP_POP => {
                        self.pop();
                    }
                    chunk::OP_DEFINE_GLOBAL => {
                        let offset = current.read_chunk()?;
                        let constant = current.read_constant(offset as usize)?;
                        if let Some(name) = constant.as_string() {
                            let val = self.peek(0);
                            self.globals.insert(name, val.clone());
                            self.pop();
                        } else {
                            return Err(RuntimeError {
                                reason: format!("{} is not a variable name.", constant),
                                line: current.read_line()?,
                            });
                        }
                    }
                    chunk::OP_GET_GLOBAL => {
                        let offset = current.read_chunk()?;
                        let constant = current.read_constant(offset as usize)?;
                        if let Some(name) = constant.as_string() {
                            if let Some(val) = self.globals.get(&name) {
                                self.push(val.clone());
                            } else {
                                return Err(RuntimeError {
                                    reason: format!("Variable {} is not defined.", constant),
                                    line: current.read_line()?,
                                });
                            }
                        } else {
                            return Err(RuntimeError {
                                reason: format!("{} is not a variable name.", constant),
                                line: current.read_line()?,
                            });
                        }
                    }
                    chunk::OP_SET_GLOBAL => {
                        let offset = current.read_chunk()?;
                        let constant = current.read_constant(offset as usize)?;
                        if let Some(name) = constant.as_string() {
                            let val = self.peek(0);
                            if self.globals.insert(name.clone(), val.clone()).is_none() {
                                self.globals.remove(&name);
                                return Err(RuntimeError {
                                    reason: format!(
                                        "{} is not defined before assignment.",
                                        constant
                                    ),
                                    line: current.read_line()?,
                                });
                            }
                        } else {
                            return Err(RuntimeError {
                                reason: format!("{} is not a variable name.", constant),
                                line: current.read_line()?,
                            });
                        }
                    }
                    chunk::OP_GET_LOCAL => {
                        let offset = current.slot + current.read_chunk()? as usize;
                        self.push(self.stack[offset].clone());
                    }
                    chunk::OP_SET_LOCAL => {
                        let offset = current.slot + current.read_chunk()? as usize;
                        self.stack[offset] = self.peek(0).clone();
                    }
                    chunk::OP_JUMP_IF_FALSE => {
                        let offset = current.read_jump()?;
                        let is_false = match self.peek(0) {
                            LoxType::None => true,
                            LoxType::Bool(x) => !x,
                            _ => false,
                        };
                        if is_false {
                            current.ip += offset;
                        }
                    }
                    chunk::OP_JUMP => {
                        let offset = current.read_jump()?;
                        current.ip += offset;
                    }
                    chunk::OP_LOOP => {
                        let offset = current.read_jump()?;
                        current.ip -= offset;
                    }
                    chunk::OP_CALL => {
                        let cnt = current.read_chunk()?;
                        let function = self.peek(cnt as usize).clone(); // Hopefully, remove this clone in the future.
                        match function {
                            LoxType::Closure(cls) => {
                                if let Err(mut e) = self.call(cls, cnt) {
                                    e.line = current.read_line()?;
                                    return Err(e);
                                };
                            }
                            LoxType::Class(klass) => {
                                self.pop();
                                self.stack.push(LoxType::Instance(Rc::new(RefCell::new(
                                    Instance::new(klass.clone()),
                                ))));
                            }
                            _ => {
                                return Err(RuntimeError {
                                    reason: "Variable is not callable.".to_string(),
                                    line: current.read_line()?,
                                })
                            }
                        }
                        break;
                    }
                    chunk::OP_CLASS => {
                        let offset = current.read_chunk()?;
                        let constant = current.read_constant(offset as usize)?.as_string();
                        if let Some(name) = constant {
                            self.push(LoxType::Class(Rc::new(Class { name })));
                        } else {
                            return Err(RuntimeError {
                                reason: "Class name should be a string.".to_string(),
                                line: current.read_line()?,
                            });
                        }
                    }
                    chunk::OP_GET_PROPERTY => {
                        let instance = self.pop();
                        if let LoxType::Instance(ins) = instance {
                            let offset = current.read_chunk()?;
                            let constant = current.read_constant(offset as usize)?;

                            if let Some(name) = constant.as_string() {
                                let inst = ins.borrow();
                                if let Some(val) = inst.fields.get(&name) {
                                    self.push(val.clone());
                                } else {
                                    return Err(RuntimeError {
                                        reason: format!("Property {} is not defined.", constant),
                                        line: current.read_line()?,
                                    });
                                }
                            } else {
                                return Err(RuntimeError {
                                    reason: format!("{} is not a property name.", constant),
                                    line: current.read_line()?,
                                });
                            }
                        } else {
                            return Err(RuntimeError {
                                reason: format!("{} is not an instance.", instance),
                                line: current.read_line()?,
                            });
                        }
                    }
                    chunk::OP_SET_PROPERTY => {
                        let instance = self.peek(1);
                        if let LoxType::Instance(ins) = instance {
                            let offset = current.read_chunk()?;
                            let constant = current.read_constant(offset as usize)?;
                            if let Some(name) = constant.as_string() {
                                let val = self.peek(0).clone();
                                ins.borrow_mut().fields.insert(name.clone(), val.clone());
                                self.pop();
                                self.pop();
                                self.push(val);
                            } else {
                                return Err(RuntimeError {
                                    reason: format!("{} is not a property name.", constant),
                                    line: current.read_line()?,
                                });
                            }
                        } else {
                            return Err(RuntimeError {
                                reason: format!("{} is not an instance.", instance),
                                line: current.read_line()?,
                            });
                        }
                    }
                    chunk::OP_CLOSURE => {
                        let offset = current.read_chunk()?;
                        let constant = current.read_constant(offset as usize)?;
                        if let LoxType::Function(func) = constant {
                            let mut clos = Closure::new(func.clone());
                            for _ in 0..clos.function.upvalue {
                                let is_local = current.read_chunk()? == 1;
                                let index = current.read_chunk()?;
                                if is_local {
                                    let address = current.slot + index as usize;
                                    if let Some(upvalue) = self.captures.get(&address) {
                                        clos.upvalues.push(upvalue.clone());
                                    } else {
                                        let upvalue =
                                            Rc::new(RefCell::new(Upvalue::Stack(address)));
                                        self.captures.insert(address, upvalue.clone());
                                        clos.upvalues.push(upvalue.clone());
                                    }
                                } else {
                                    clos.upvalues
                                        .push(current.closure.upvalues[index as usize].clone());
                                }
                            }
                            self.push(LoxType::Closure(clos));
                        } else {
                            return Err(RuntimeError {
                                reason: format!("Expect a function but get {}", constant),
                                line: current.read_line()?,
                            });
                        }
                    }
                    chunk::OP_GET_UPVALUE => {
                        let offset = current.read_chunk()?;
                        let val = match &*current.closure.upvalues[offset as usize].borrow() {
                            Upvalue::Stack(location) => self.stack[*location].clone(),
                            Upvalue::Out(rc) => rc.clone(),
                        };
                        self.push(val);
                    }
                    chunk::OP_SET_UPVALUE => {
                        let val = self.peek(0).clone();
                        let offset = current.read_chunk()?;
                        let mut borrow_mut = current.closure.upvalues[offset as usize].borrow_mut();
                        let loc = match *borrow_mut {
                            Upvalue::Stack(location) => &mut self.stack[location],
                            Upvalue::Out(ref mut rc) => rc,
                        };
                        *loc = val.clone();
                    }
                    chunk::OP_CLOSE_UPVALUE => {
                        self.close_upvalues(self.stack.len() - 1);
                        self.pop();
                    }
                    _ => {
                        return Err(RuntimeError {
                            reason: "Unknown command.".to_string(),
                            line: current.read_line()?,
                        })
                    }
                }
            }
        }
        Ok(())
    }

    fn close_upvalues(&mut self, slot: usize) {
        if let Some(val) = self.captures.get(&slot) {
            *val.borrow_mut() = Upvalue::Out(self.peek(0).clone());
            self.captures.remove(&slot);
        }
    }

    fn call(&mut self, clos: Closure, arg_cnt: u8) -> Result<(), RuntimeError> {
        if arg_cnt != clos.function.arity {
            return Err(RuntimeError {
                reason: format!(
                    "Expect {} arguments but got {}.",
                    clos.function.arity, arg_cnt
                ),
                line: -1,
            });
        }
        self.frames.push(Rc::new(RefCell::new(CallFrame {
            closure: clos,
            ip: 0,
            slot: self.stack.len() - arg_cnt as usize - 1,
        })));
        Ok(())
    }
}

struct CallFrame {
    closure: Closure,
    ip: usize,
    slot: usize,
}

impl CallFrame {
    pub fn read_jump(&mut self) -> Result<usize, RuntimeError> {
        let ret = self.closure.function.chunk.read_jump(self.ip)?;
        self.ip += USIZE;
        Ok(ret)
    }

    pub fn read_chunk(&mut self) -> Result<u8, RuntimeError> {
        self.ip += 1;
        self.closure.function.chunk.read_chunk(self.ip - 1)
    }

    pub fn read_constant(&mut self, pos: usize) -> Result<Value, RuntimeError> {
        self.closure.function.chunk.read_constant(pos)
    }

    pub fn read_line(&self) -> Result<i32, RuntimeError> {
        self.closure.function.chunk.read_line(self.ip - 1)
    }
}
