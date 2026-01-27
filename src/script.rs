#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    OpDup = 0x76,
    OpHash160 = 0xa9,
    OpEqualVerify = 0x88,
    OpCheckSig = 0xac,
    OpEqual = 0x87
}

impl OpCode {
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0x76 => Some(OpCode::OpDup),
            0xa9 => Some(OpCode::OpHash160),
            0x88 => Some(OpCode::OpEqualVerify),
            0xac => Some(OpCode::OpCheckSig),
            0x87 => Some(OpCode::OpEqual),
            _ => None
        }
    }
}

#[derive(Debug, Clone)]
pub enum Cmd {
    Op(OpCode),
    Push(Vec<u8>)
}

#[derive(Debug, Clone)]
pub struct Script {
    pub cmds: Vec<Cmd>
}

pub enum ScriptError {
    InvalidScript,
    InvalidOpCode
}

impl Script {
    pub fn parse(raw: &[u8]) -> Result<Self, ScriptError> {
        let mut cmds = Vec::new();
        let mut index: usize = 0;
        
        loop {
            match raw[index] {
                0x01..=0x4b => {
                    let to = index + raw[index] as usize;
                    let next_bytes = raw[index..to].to_vec();
                    cmds.push(Cmd::Push(next_bytes));
                    
                    index += to;
                },
                _ => {
                    let op_code = OpCode::from_u8(raw[index]).ok_or(ScriptError::InvalidOpCode)?;
                    cmds.push(Cmd::Op(op_code));
                    
                    index += 1;
                }
            }
            
            if index == raw.len() {
                break;
            }
        }
        
        Ok(Script { cmds })
    }
}