use super::*;

pub use rustson::deser::Reader;

pub trait RDeserializer {
    fn read(&self, reader: &mut Reader) -> RTsonResult<SEXP>;
}

pub struct RTsonDeserializer {}
pub struct RJsonDeserializer {}
pub struct RBinaryDeserializer {}
pub struct RUTF8Deserializer {}

impl RDeserializer for RTsonDeserializer{
    fn read(&self, reader: &mut Reader) -> RTsonResult<SEXP>{
        RTsonDeserializer::read(self, reader)
    }
}

impl RDeserializer for RJsonDeserializer {
    fn read(&self, reader: &mut Reader) -> RTsonResult<SEXP>{
        let mut buf = Vec::new();
        reader.read_all(&mut buf)?;

        if buf.is_empty() {
            Ok(().intor()?)
        } else {
            match rustson::decode_json(&buf) {
                Ok(object) => {
                    Ok((value_to_r(&object)?).intor()?)
                }
                Err(e) => Err(RTsonError::new(e.to_string()))
            }
        }
    }
}

impl RDeserializer for RBinaryDeserializer {
    fn read(&self, reader: &mut Reader) -> RTsonResult<SEXP>{
        let mut buf = Vec::new();
        reader.read_all(&mut buf)?;

        let mut raw_vec = RawVec::alloc(buf.len());

        unsafe {
            for i in 0..buf.len() {
                raw_vec.uset(i, buf[i]);
            }
        }

        Ok(raw_vec.intor()?)
    }
}

impl RDeserializer for RUTF8Deserializer {
    fn read(&self, reader: &mut Reader) -> RTsonResult<SEXP>{
        let mut buf = Vec::new();
        reader.read_all(&mut buf)?;

        unsafe {
            let utf8str = String::from_utf8_unchecked(buf);
            Ok(utf8str.intor()?)
        }
    }
}

impl RTsonDeserializer {
    pub fn new() -> RTsonDeserializer {
        RTsonDeserializer {}
    }

    pub fn read(&self, reader: &mut Reader) -> RTsonResult<SEXP> {
        let itype = self.read_type(reader)?;

        if itype != STRING_TYPE {
            return Err(RTsonError::new("wrong format"));
        }

        let version = self.read_string(reader)?;

        if !version.eq(VERSION) {
            return Err(RTsonError::new("wrong version"));
        }

        self.read_object(reader)
    }

    fn read_type(&self, reader: &mut Reader) -> RTsonResult<u8> {
        Ok(reader.read_u8()?)
    }

    fn read_len(&self, reader: &mut Reader) -> RTsonResult<usize> {
        Ok(reader.read_u32()? as usize)
    }

    fn read_string(&self, reader: &mut Reader) -> RTsonResult<String> {
        let mut done = false;
        let mut vec = Vec::new();
        while !done {
            let byte = reader.read_u8()?;
            if byte == 0 {
                done = true;
            } else {
                vec.push(byte);
            }
        }

        if let Ok(value) = String::from_utf8(vec) {
            Ok(value)
        } else {
            Err(RTsonError::new("bad string"))
        }
    }

    fn read_object(&self, reader: &mut Reader) -> RTsonResult<SEXP> {
        let itype = self.read_type(reader)?;
        match itype {
            NULL_TYPE => Ok(().intor()?),
            STRING_TYPE => Ok(self.read_string(reader)?.intor()?),
            INTEGER_TYPE => Ok(reader.read_i32()?.intor()?),
            DOUBLE_TYPE => Ok(reader.read_f64()?.intor()?),
            BOOL_TYPE => {
                Ok((reader.read_u8()? > 0).intor()?)
            }
            LIST_TYPE => {
                let len = self.read_len(reader)?;
                let mut lst = RList::alloc(len);

                for i in 0..len {
                    let obj = self.read_object(reader)?;
                    lst.set(i, obj)?;
                }

                Ok(lst.intor()?)
            }
            MAP_TYPE => {
                let len = self.read_len(reader)?;

                let mut names = CharVec::alloc(len);
                let mut values = RList::alloc(len);

                for i in 0..len {
                    let _itype = self.read_type(reader)?;
                    if _itype != STRING_TYPE {
                        return Err(RTsonError::new("wrong format"));
                    }
                    let name = self.read_string(reader)?;

                    names.set(i, &name as &str)?;
                    values.set(i, self.read_object(reader)?)?;
                }

                unsafe {
                    Rf_setAttrib(values.s(), R_NamesSymbol, names.s());
                    Ok(values.s())
                }
            }
            LIST_UINT8_TYPE => {
                let len = self.read_len(reader)?;
                let mut values = RawVec::alloc(len);
                unsafe {
                    for i in 0..len {
                        values.uset(i, reader.read_u8()?);
                    }
                }

                Ok(values.intor()?)
            }
            LIST_INT8_TYPE => {
                let len = self.read_len(reader)?;

                let mut values = IntVec::alloc(len);
                unsafe {
                    for i in 0..len {
                        values.uset(i, reader.read_i8()? as i32);
                    }
                }

                Ok(values.intor()?)
            }
            LIST_UINT16_TYPE => {
                let len = self.read_len(reader)?;

                let mut values = IntVec::alloc(len);
                unsafe {
                    for i in 0..len {
                        values.uset(i, reader.read_u16()? as i32);
                    }
                }

                Ok(values.intor()?)
            }
            LIST_INT16_TYPE => {
                let len = self.read_len(reader)?;
                let mut values = IntVec::alloc(len);
                unsafe {
                    for i in 0..len {
                        values.uset(i, reader.read_i16()? as i32);
                    }
                }
                Ok(values.intor()?)
            }

            LIST_UINT32_TYPE => {
                let len = self.read_len(reader)?;
                let mut values = NumVec::alloc(len);
                unsafe {
                    for i in 0..len {
                        values.uset(i, reader.read_u32()? as f64);
                    }
                }
                Ok(values.intor()?)
            }
            LIST_INT32_TYPE => {
                let len = self.read_len(reader)?;
                let mut values = IntVec::alloc(len);
                unsafe {
                    for i in 0..len {
                        values.uset(i, reader.read_i32()?);
                    }
                }
                Ok(values.intor()?)
            }
            LIST_INT64_TYPE => {
                let len = self.read_len(reader)?;
                let mut values = NumVec::alloc(len);
                unsafe {
                    for i in 0..len {
                        values.uset(i, reader.read_i64()? as f64);
                    }
                }
                Ok(values.intor()?)
            }
            LIST_UINT64_TYPE => {
                let len = self.read_len(reader)?;
                let mut values = NumVec::alloc(len);
                unsafe {
                    for i in 0..len {
                        values.uset(i, reader.read_u64()? as f64);
                    }
                }
                Ok(values.intor()?)
            }
            LIST_FLOAT32_TYPE => {
                let len = self.read_len(reader)?;
                let mut values = NumVec::alloc(len);
                unsafe {
                    for i in 0..len {
                        values.uset(i, reader.read_f32()? as f64);
                    }
                }
                Ok(values.intor()?)
            }
            LIST_FLOAT64_TYPE => {
                let len = self.read_len(reader)?;
                let mut values = NumVec::alloc(len);
                unsafe {
                    for i in 0..len {
                        values.uset(i, reader.read_f64()?);
                    }
                }

                Ok(values.intor()?)
            }
            LIST_STRING_TYPE => {
                let mut len_in_bytes = self.read_len(reader)?;

                let mut vec = Vec::new();
                while len_in_bytes > 0 {
                    let v = self.read_string(reader)?;
                    len_in_bytes -= v.as_bytes().len() + 1;
                    vec.push(v);
                }

                if len_in_bytes > 0 {
                    return Err(RTsonError::new("wrong format"));
                }

                Ok(vec.intor()?)
            }

            _ => Err(RTsonError::new("wrong format")),
        }
    }
}

//struct RawVecReader {
//    data: RawVec,
//    _remaining: usize,
//}
//
//impl RawVecReader {
//    pub fn new(data: RawVec) -> RawVecReader {
//        let len = data.rsize() as usize;
//        RawVecReader {data, _remaining: len}
//    }
//
//    fn remaining(&self) -> usize {
//        self._remaining
//    }
//}
//
//impl Reader for RawVecReader {
//
//}

