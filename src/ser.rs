use super::*;

pub use rustson::ser::*;
use rustson::spec::*;

pub struct RSerializer {}

impl RSerializer {
    pub fn new( ) -> Self {
        RSerializer{ }
    }

    pub fn encoded_size(&self, value: &SEXP) -> RTsonResult<usize> {
        let mut buf = CountWriter::new();
        self.add_string(&mut buf, VERSION)?;

        match self.add_object(value, &mut buf) {
            Ok(_) => Ok(buf.size),
            Err(e) => Err(e),
        }
    }

    pub fn encode(&self, value: &SEXP) -> RTsonResult<RawVec> {
        let size = self.encoded_size(value)?;

        let mut buf = Vec::with_capacity(size);
        self.add_string(&mut buf, VERSION)?;

        match self.add_object(value, &mut buf) {
            Ok(_) => {
                let mut raw_vec = RawVec::alloc(buf.len());

                unsafe {
                    for i in 0..buf.len() {
                        raw_vec.uset(i, buf[i]);
                    }
                }
                Ok(raw_vec)
            }
            Err(e) => Err(e),
        }
    }

    pub fn write(&self, value: &SEXP, writer: &mut dyn Writer) -> RTsonResult<()> {
        self.add_string(writer, VERSION)?;
        self.add_object(value, writer)
    }

    fn add_object(&self, object: &SEXP, buf: &mut dyn Writer) -> RTsonResult<()> {
        match object.rtype() {
            NILSXP => {
                buf.add_u8(NULL_TYPE)?;
            }
            RAWSXP => {
                let object_ = RawVec::rnew(*object)?;
                buf.add_u8(LIST_UINT8_TYPE)?;
                let len = object_.rsize() as usize;
                self.add_len(buf, len)?;
                let len_in_bytes = len;
                unsafe {
                    buf.put_slice(std::slice::from_raw_parts(RAW(object_.s()), len_in_bytes) );
                }
            }
            REALSXP => {
                let object_ = NumVec::new(*object)?;
                if inherits(*object, "scalar")? {
                    if object_.rsize() as usize != 1 {
                        return http_raise(format!("real : scalar bad length : {}", object_.rsize()).to_string() );
                    } else {
                        buf.add_u8(DOUBLE_TYPE)?;
                        buf.add_f64(object_.at(0).unwrap())?;
                    }
                } else if inherits(*object, "uint64")? {
                    buf.add_u8(LIST_UINT64_TYPE)?;
                    self.add_len(buf, object_.rsize() as usize)?;
                    for x in object_ {
                        buf.add_u64(x as u64)?;
                    }
                } else if inherits(*object, "int64")? {
                    buf.add_u8(LIST_INT64_TYPE)?;
                    self.add_len(buf, object_.rsize() as usize)?;
                    for x in object_ {
                        buf.add_i64(x as i64)?;
                    }
                } else {
                    buf.add_u8(LIST_FLOAT64_TYPE)?;
                    let len = object_.rsize() as usize;
                    self.add_len(buf, len)?;
                    let len_in_bytes = len * 8;
                    unsafe {
                        buf.put_slice(std::slice::from_raw_parts(RAW(object_.s()), len_in_bytes) );
                    }

                    // self.add_len(buf, object_.rsize() as usize)?;
                    // for x in object_ {
                    //     buf.add_f64(x as f64)?;
                    // }
                }
            }
            INTSXP => {
                let object_ = IntVec::rnew(*object)?;
                if inherits(*object, "scalar")? {
                    if object_.rsize() as usize != 1 {
                        return http_raise(format!("int : scalar bad length : {}", object_.rsize()).to_string());
                    } else {
                        buf.add_u8(INTEGER_TYPE)?;
                        buf.add_i32(object_.at(0).unwrap())?;
                    }
                } else {
                    if inherits(*object, "int8")? {
                        buf.add_u8(LIST_INT8_TYPE)?;
                        let len = object_.rsize() as usize;
                        self.add_len(buf, len)?;
                        let len_in_bytes = len;
                        unsafe {
                            buf.put_slice(std::slice::from_raw_parts(RAW(object_.s()), len_in_bytes) );
                        }
                    } else if inherits(*object, "int16")? {
                        buf.add_u8(LIST_INT16_TYPE)?;
                        let len = object_.rsize() as usize;
                        self.add_len(buf, len)?;
                        let len_in_bytes = len * 2;
                        unsafe {
                            buf.put_slice(std::slice::from_raw_parts(RAW(object_.s()), len_in_bytes) );
                        }
                    } else if inherits(*object, "int64")? {
                        buf.add_u8(LIST_INT64_TYPE)?;
                        let len = object_.rsize() as usize;
                        self.add_len(buf, len)?;
                        let len_in_bytes = len * 8;
                        unsafe {
                            buf.put_slice(std::slice::from_raw_parts(RAW(object_.s()), len_in_bytes) );
                        }
                    } else if inherits(*object, "uint8")? {
                        buf.add_u8(LIST_UINT8_TYPE)?;
                        let len = object_.rsize() as usize;
                        self.add_len(buf, len)?;
                        let len_in_bytes = len  ;
                        unsafe {
                            buf.put_slice(std::slice::from_raw_parts(RAW(object_.s()), len_in_bytes) );
                        }
                    } else if inherits(*object, "uint16")? {
                        buf.add_u8(LIST_UINT16_TYPE)?;
                        let len = object_.rsize() as usize;
                        self.add_len(buf, len)?;
                        let len_in_bytes = len * 2;
                        unsafe {
                            buf.put_slice(std::slice::from_raw_parts(RAW(object_.s()), len_in_bytes) );
                        }
                    } else if inherits(*object, "uint64")? {
                        buf.add_u8(LIST_UINT64_TYPE)?;
                        let len = object_.rsize() as usize;
                        self.add_len(buf, len)?;
                        let len_in_bytes = len * 8;
                        unsafe {
                            buf.put_slice(std::slice::from_raw_parts(RAW(object_.s()), len_in_bytes) );
                        }
                    } else if inherits(*object, "uint32")? {
                        buf.add_u8(LIST_UINT32_TYPE)?;
                        let len = object_.rsize() as usize;
                        self.add_len(buf, len)?;
                        let len_in_bytes = len * 4;
                        unsafe {
                            buf.put_slice(std::slice::from_raw_parts(RAW(object_.s()), len_in_bytes) );
                        }
                    } else {
                        buf.add_u8(LIST_INT32_TYPE)?;
                        let len = object_.rsize() as usize;
                        self.add_len(buf, len)?;
                        let len_in_bytes = len * 4;
                        unsafe {
                            buf.put_slice(std::slice::from_raw_parts(RAW(object_.s()), len_in_bytes) );
                        }
                    }
                }
            }
            LGLSXP => {
                let object_ = BoolVec::rnew(*object)?;
                if object_.rsize() as usize != 1 {
                    return http_raise(format!("bool : bad length : {}", object_.rsize()).to_string());
                } else {
                    buf.add_u8(BOOL_TYPE)?;
                    let v = object_.at(0).unwrap();
                    if v {
                        buf.add_u8(1)?;
                    } else {
                        buf.add_u8(0)?;
                    }
                }
            }
            STRSXP => {
                let object_ = CharVec::rnew(*object)?;
                let len = object_.rsize() as usize;
                if inherits(*object, "scalar")? {
                    if len != 1 {
                        return http_raise(format!("str : scalar bad length : {}", object_.rsize()).to_string());

                    } else {
                        self.add_string(buf, &object_.at(0).map_err(|e| RError::other(e))?)?;
                    }
                } else {
                    buf.add_u8(LIST_STRING_TYPE)?;

                    let mut len_in_bytes = 0;

                    for value in object_ {
                        len_in_bytes += value.as_bytes().len() + 1;
                    }

                    self.add_len(buf, len_in_bytes)?;

                    let object_ = CharVec::rnew(*object)?;
                    for value in object_ {
                        self.add_cstring2(buf, &value)?;
                    }
                }
            }
            VECSXP => {
                /* generic vectors */
                // empty list
                let rlist = RList::new(*object)?;

                let names: CharVec = RName::name(&rlist);

                if names.rsize() > 0 || inherits(*object, "tsonmap")? {
                    buf.add_u8(MAP_TYPE)?;
                    self.add_len(buf, names.rsize() as usize)?;

                    let mut index = 0;

                    for x in rlist {
                        self.add_string(buf, &names.at(index)?)?;
                        self.add_object(&x, buf)?;
                        index = index + 1;
                    }
                } else {
                    buf.add_u8(LIST_TYPE)?;
                    self.add_len(buf, rlist.rsize() as usize)?;
                    for x in rlist {
                        self.add_object(&x, buf)?;
                    }
                }
            }
            _ => {
                return http_raise(format!("bad object type : {}", object.rtype()).to_string())
            }
        }

        Ok(())
    }

    fn add_len(&self, buf: &mut dyn Writer, len: usize) -> RTsonResult<()> {
        if len > MAX_LIST_LENGTH {
            return http_raise("list too large");
        }
        buf.add_u32(len as u32)?;
        Ok(())
    }

    fn add_string(&self, buf: &mut dyn Writer, value: &str) -> RTsonResult<()> {
        buf.add_u8(STRING_TYPE)?;
        self.add_cstring(buf, value)
    }

    fn add_cstring(&self, buf: &mut dyn Writer, value: &str) -> RTsonResult<()> {
        for byte in value.as_bytes().iter() {
            buf.add_u8(*byte)?;
        }
        buf.add_u8(0)?;
        Ok(())
    }

    fn add_cstring2(&self, buf: &mut dyn Writer, value: &CString) -> RTsonResult<()> {
        buf.put_slice(value.as_bytes())?;
        buf.add_u8(0)?;
        Ok(())
    }
}


