pub const MAX_XDR_LEN: usize = 8192; // Maximum expected XDR length

use super::StellarTransactionError;

pub struct Xdr<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Xdr<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    pub fn read_exact(&mut self, len: usize) -> Result<&'a [u8], StellarTransactionError> {
        if self.pos + len > self.buf.len() {
            return Err(StellarTransactionError::XdrError);
        }
        let out = &self.buf[self.pos..self.pos + len];
        self.pos += len;
        Ok(out)
    }

    pub fn align4(&mut self) {
        let r = self.pos & 3;
        if r != 0 {
            self.pos += 4 - r;
        }
    }

    pub fn read_u32(&mut self) -> Result<u32, StellarTransactionError> {
        let b = self.read_exact(4)?;
        Ok(u32::from_be_bytes(b.try_into().unwrap()))
    }

    pub fn read_i32(&mut self) -> Result<i32, StellarTransactionError> {
        Ok(self.read_u32()? as i32)
    }

    pub fn read_u64(&mut self) -> Result<u64, StellarTransactionError> {
        let hi = self.read_u32()? as u64;
        let lo = self.read_u32()? as u64;
        Ok((hi << 32) | lo)
    }

    pub fn read_i64(&mut self) -> Result<i64, StellarTransactionError> {
        Ok(self.read_u64()? as i64)
    }

    pub fn read_bool(&mut self) -> Result<bool, StellarTransactionError> {
        match self.read_u32()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(StellarTransactionError::XdrError),
        }
    }

    pub fn read_fixed_opaque(&mut self, len: usize) -> Result<&'a [u8], StellarTransactionError> {
        // fixed-length opaque is NOT length-prefixed in Stellar XDR, just raw bytes.
        self.read_exact(len)
    }

    pub fn read_opaque(&mut self, max_len: usize) -> Result<&'a [u8], StellarTransactionError> {
        let len = self.read_u32()? as usize;
        if len > max_len {
            return Err(StellarTransactionError::XdrError);
        }
        let data = self.read_exact(len)?;
        self.align4();
        Ok(data)
    }

    pub fn read_string(&mut self, max_len: usize) -> Result<&'a str, StellarTransactionError> {
        let bytes = self.read_opaque(max_len)?;
        core::str::from_utf8(bytes).map_err(|_| StellarTransactionError::XdrError)
    }
}