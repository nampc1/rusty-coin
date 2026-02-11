use sha2::{Sha256, Digest};
use ripemd::Ripemd160;

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    Op(OpCode),
    Push(Vec<u8>)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Script {
    pub cmds: Vec<Cmd>
}

#[derive(Debug)]
pub enum ScriptError {
    InvalidScript,
    InvalidOpCode
}

impl Script {
    pub fn parse(raw: &[u8]) -> Result<Self, ScriptError> {
        let mut cmds = Vec::new();
        let mut index: usize = 0;
        
        if raw.len() == 0 {
            return Ok(Script { cmds });
        }
        
        loop {
            match raw[index] {
                0x01..=0x4b => {
                    let from = index + 1;
                    let to = from + raw[index] as usize;
                    
                    let next_bytes = raw[from..to].to_vec();
                    cmds.push(Cmd::Push(next_bytes));
                    
                    index += raw[index] as usize + 1;
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
    
    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized: Vec<u8> = Vec::new();
        
        if self.cmds.len() == 0 {
            return serialized;
        }
        
        for cmd in &self.cmds {
            match cmd {
                Cmd::Push(bytes) => {
                    serialized.push(bytes.len() as u8);
                    serialized.extend_from_slice(&bytes);
                },
                Cmd::Op(op) => {
                    serialized.push(*op as u8);
                }
            }
        }
        
        serialized
    }
    
    pub fn evaluate(&self) -> bool {
        let mut stack: Vec<Vec<u8>> = Vec::new();
        
        for cmd in &self.cmds {
            match cmd {
                Cmd::Push(bytes) => {
                    stack.push(bytes.clone());
                },
                Cmd::Op(op) => {
                    match op {
                        OpCode::OpDup => {
                            if let Some(top) = stack.last() {
                                stack.push(top.clone());
                            } else {
                                return false;
                            }
                        },
                        OpCode::OpEqual => {
                            let Some(a) = stack.pop() else { return false; };
                            let Some(b) = stack.pop() else { return false; };
                            
                            if a == b {
                                stack.push(vec![1]);
                            } else {
                                stack.push(vec![]);
                            }
                        },
                        OpCode::OpEqualVerify => {
                            let Some(a) = stack.pop() else { return false; };
                            let Some(b) = stack.pop() else { return false; };
                            
                            if a != b {
                                return false;
                            }
                        },
                        OpCode::OpCheckSig => {
                            
                        },
                        OpCode::OpHash160 => {
                            let Some(data) = stack.pop() else { return false; };
                            
                            let h1 = Sha256::digest(data);
                            let h2 = Ripemd160::digest(h1);
                            
                            stack.push(h2.to_vec());
                        }
                    }
                }
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_op_only() {
        // OpDup (0x76)
        let raw = vec![0x76];
        let script = Script::parse(&raw).unwrap();
        assert_eq!(script.cmds.len(), 1);
        assert_eq!(script.cmds[0], Cmd::Op(OpCode::OpDup));
    }

    #[test]
    fn test_parse_push_only() {
        // Push 2 bytes: [0xCA, 0xFE]
        // 0x02 is the length, followed by the data
        let raw = vec![0x02, 0xCA, 0xFE];
        let script = Script::parse(&raw).unwrap();
        
        assert_eq!(script.cmds.len(), 1);
        if let Cmd::Push(data) = &script.cmds[0] {
            assert_eq!(data, &vec![0xCA, 0xFE]);
        } else {
            panic!("Expected Push command");
        }
    }

    #[test]
    fn test_parse_mixed() {
        // OpDup (0x76)
        // Push 1 byte [0xFF] (0x01 0xFF)
        // OpEqual (0x87)
        let raw = vec![0x76, 0x01, 0xFF, 0x87];
        let script = Script::parse(&raw).unwrap();

        assert_eq!(script.cmds.len(), 3);
        assert_eq!(script.cmds[0], Cmd::Op(OpCode::OpDup));
        assert_eq!(script.cmds[1], Cmd::Push(vec![0xFF]));
        assert_eq!(script.cmds[2], Cmd::Op(OpCode::OpEqual));
    }

    #[test]
    fn test_parse_empty() {
        let raw = vec![];
        if let Ok(script) = Script::parse(&raw) {
             assert_eq!(script.cmds.len(), 0);
        }
    }

    #[test]
    fn test_serialize() {
        // Construct a script manually
        let script = Script {
            cmds: vec![
                Cmd::Op(OpCode::OpDup),           // 0x76
                Cmd::Push(vec![0xCA, 0xFE]),      // 0x02 0xCA 0xFE
                Cmd::Op(OpCode::OpEqual),         // 0x87
            ]
        };

        let serialized = script.serialize();
        let expected = vec![0x76, 0x02, 0xCA, 0xFE, 0x87];

        assert_eq!(serialized, expected);
        
        // Bonus: Round-trip test
        // Parse(Serialize(Script)) == Script
        let parsed_again = Script::parse(&serialized).expect("Should parse valid serialization");
        assert_eq!(script.cmds, parsed_again.cmds);
    }

    #[test]
    fn test_evaluate_simple_push() {
        // Script: Push([0x01])
        // Stack: [[0x01]] -> Top is not empty -> True
        let script = Script {
            cmds: vec![Cmd::Push(vec![0x01])]
        };
        assert!(script.evaluate());
    }

    #[test]
    fn test_evaluate_op_dup() {
        // Script: Push([0x01]), OpDup, OpEqual
        // 1. Push [0x01] -> Stack: [[0x01]]
        // 2. OpDup       -> Stack: [[0x01], [0x01]]
        // 3. OpEqual     -> Stack: [[0x01]] (True)
        let script = Script {
            cmds: vec![
                Cmd::Push(vec![0x01]),
                Cmd::Op(OpCode::OpDup),
                Cmd::Op(OpCode::OpEqual),
            ]
        };
        assert!(script.evaluate());
    }

    #[test]
    fn test_evaluate_op_equal_fail() {
        // Script: Push([0x01]), Push([0x02]), OpEqual
        // 1. Push [0x01] -> Stack: [[0x01]]
        // 2. Push [0x02] -> Stack: [[0x01], [0x02]]
        // 3. OpEqual     -> Stack: [[]] (False, because 1 != 2)
        // Note: In Bitcoin, false is usually represented by an empty vector or 0x00
        let script = Script {
            cmds: vec![
                Cmd::Push(vec![0x01]),
                Cmd::Push(vec![0x02]),
                Cmd::Op(OpCode::OpEqual),
            ]
        };
        assert!(!script.evaluate());
    }
    
    #[test]
    fn test_evaluate_empty_stack_fail() {
        // Script: Empty
        // Stack: []
        // Result: False
        let script = Script { cmds: vec![] };
        assert!(!script.evaluate());
    }
}