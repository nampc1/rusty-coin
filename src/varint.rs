//! Implementation of Bitcoin's "CompactSize" Variable Integer format.
//!
//! See `note/11-compact-size-varint.md` for details on the format, its origin,
//! and the historical transaction malleability bugs associated with it.

pub fn encode_varint(buffer: &mut Vec<u8>, i: u64) {
    if i < 0xfd {
       buffer.push(i as u8);
    } else if i < 0x10000 {
       buffer.push(0xfd); 
       buffer.extend_from_slice(&(i as u16).to_le_bytes());
    } else if i < 0x100000000 {
        buffer.push(0xfe); 
        buffer.extend_from_slice(&(i as u32).to_le_bytes());
    } else {
        buffer.push(0xff); 
        buffer.extend_from_slice(&i.to_le_bytes());
    }
}

pub fn read_varint(s: &Vec<u8>) -> Result<u64, &'static str> {
    if s.is_empty() {
        return Err("Invalid VarInt: Data too short");
    }

    let i = s[0];
    
    if i == 0xfd {
        if s.len() < 3 { return Err("Invalid VarInt: Data too short"); }
        let bytes = s[1..3].try_into().map_err(|_| "VarInt conversion failed")?;
        return Ok(u16::from_le_bytes(bytes) as u64);
    } else if i == 0xfe {
        if s.len() < 5 { return Err("Invalid VarInt: Data too short"); }
        let bytes = s[1..5].try_into().map_err(|_| "VarInt conversion failed")?;
        return Ok(u32::from_le_bytes(bytes) as u64);
    } else if i == 0xff {
        if s.len() < 9 { return Err("Invalid VarInt: Data too short"); }
        let bytes = s[1..9].try_into().map_err(|_| "VarInt conversion failed")?;
        return Ok(u64::from_le_bytes(bytes));
    } else {
        return Ok(i as u64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_boundaries() {
        // Case 1: < 0xfd (1 byte)
        let mut buf = Vec::new();
        encode_varint(&mut buf, 252);
        assert_eq!(buf, vec![0xfc]);

        // Case 2: >= 0xfd (3 bytes)
        let mut buf = Vec::new();
        encode_varint(&mut buf, 253);
        assert_eq!(buf, vec![0xfd, 0xfd, 0x00]); // 0xfd marker + 253 (0x00fd) LE

        // Case 3: u16 max (3 bytes)
        let mut buf = Vec::new();
        encode_varint(&mut buf, 65535);
        assert_eq!(buf, vec![0xfd, 0xff, 0xff]);

        // Case 4: > u16 (5 bytes)
        let mut buf = Vec::new();
        encode_varint(&mut buf, 65536);
        assert_eq!(buf, vec![0xfe, 0x00, 0x00, 0x01, 0x00]); // 0xfe + 65536 LE

        // Case 5: u32 max (5 bytes)
        let mut buf = Vec::new();
        encode_varint(&mut buf, 4294967295);
        assert_eq!(buf, vec![0xfe, 0xff, 0xff, 0xff, 0xff]);

        // Case 6: > u32 (9 bytes)
        let mut buf = Vec::new();
        encode_varint(&mut buf, 4294967296);
        assert_eq!(buf, vec![0xff, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]);
    }
}
