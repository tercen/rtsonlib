extern crate rustr;
extern crate rustson;

use std::error;
use std::fmt;
use std::convert::{From};

use rustr::*;
use rustson::*;

use std::io::Cursor;
use std::collections::HashMap;
use ::std::ffi::*;

pub mod ser;
pub mod deser;


use deser::RTsonDeserializer;
use ser::RSerializer;

#[derive(Debug, Clone, PartialEq)]
pub struct RTsonError {
    description: String
}

impl RTsonError {
    pub fn new<T: Into<String>>(description: T) -> RTsonError {
        RTsonError { description: description.into() }
    }
}

impl fmt::Display for RTsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.description)
    }
}

impl error::Error for RTsonError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> { None }

    fn description(&self) -> &str {
        &self.description
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

type RTsonResult<T> = std::result::Result<T, RTsonError>;

impl From<RTsonError> for RError {
    fn from(e: RTsonError) -> Self {
        RError::unknown(e.to_string())
    }
}

impl From<rustson::TsonError> for RTsonError {
    fn from(e: TsonError) -> Self {
        RTsonError::new(e.to_string())
    }
}

impl From<RError> for RTsonError {
    fn from(e: RError) -> Self {
        RTsonError::new(e.to_string())
    }
}


#[inline]
pub fn http_raise<T, E>(msg: E) -> RTsonResult<T>
    where E: Into<String> {
    Err(RTsonError::new(msg))
}

pub fn to_json(object: SEXP) -> RResult<String> {
    let value = r_to_value(object)?;
    match encode_json(&value) {
        Ok(v) => Ok(v),
        Err(e) => Err(RError::other(e)),
    }
}

pub fn from_json(data: &str) -> RResult<SEXP> {
    match decode_json(data.as_bytes()) {
        Ok(object) => value_to_r(&object),
        Err(e) => Err(RError::other(e)),
    }
}

pub fn to_tson(object: SEXP) -> RResult<RawVec> {
    let ser = RSerializer::new();
    Ok(ser.encode(&object)?)
}

pub fn from_tson(rbytes: RawVec) -> RResult<SEXP> {
    let len = rbytes.rsize() as usize;

    unsafe {
        let rptr = RAW(rbytes.s());
        let bytes: Vec<u8> = Vec::from_raw_parts(rptr, len, len);
        let deser = RTsonDeserializer::new();

        let result;

        {
            let mut reader = Cursor::new(&bytes);
            match deser.read(&mut reader) {
                Ok(v) => {
                    result = Ok(v);
                }
                Err(e) => {
                    result = Err(RError::other(e));
                }
            }
        }

        std::mem::forget(bytes);
        result
    }
}

pub fn from_tson_reader(reader: &mut Cursor<&[u8]>) -> RResult<SEXP> {
    let deser = RTsonDeserializer::new();
     Ok(deser.read(reader)?)
}


pub fn c_str(x: &str) -> RResult<CString> {
    match CString::new(x) {
        Ok(some) => Ok(some),
        Err(e) => {
            Err(RError::other(e))
        }
    }
}

fn inherits(object: SEXP, clazz: &str) -> RResult<bool> {
    unsafe {
        Ok(rbool(Rf_inherits(object, c_str(clazz)?.as_ptr())))
    }
}

pub fn r_to_value(object: SEXP) -> RResult<Value> {
    match object.rtype() {
        NILSXP => Ok(Value::NULL),
        RAWSXP => {
            let object_ = RawVec::rnew(object)?;
            let mut _vec = Vec::<u8>::with_capacity(object_.rsize() as usize);
            for x in object_ {
                _vec.push(x);
            }
            Ok(Value::LSTU8(_vec))
        }
        REALSXP => {
            let object_ = Vec::<f64>::rnew(object)?;
            if inherits(object, "scalar")? {
                if object_.len() != 1 {
                    Err(RError::unknown(format!("real : scalar bad length : {}", object_.len()).to_string()))
                } else {
                    Ok(Value::F64(object_[0]))
                }
            } else if inherits(object, "uint64")? {
                let mut _vec = Vec::<u64>::with_capacity(object_.len());
                for x in object_ {
                    _vec.push(x as u64);
                }
                Ok(Value::LSTU64(_vec))
            } else if inherits(object, "int64")? {
                let mut _vec = Vec::<i64>::with_capacity(object_.len());
                for x in object_ {
                    _vec.push(x as i64);
                }
                Ok(Value::LSTI64(_vec))
            } else {
                Ok(Value::LSTF64(object_))
            }
        }
        INTSXP => {
            let object_ = Vec::<i32>::rnew(object)?;
            if inherits(object, "scalar")? {
                if object_.len() != 1 {
                    Err(RError::unknown(format!("int : scalar bad length : {}", object_.len()).to_string()))
                } else {
                    Ok(Value::I32(object_[0]))
                }
            } else {
                if inherits(object, "int8")? {
                    let mut _vec = Vec::<i8>::with_capacity(object_.len());
                    for x in object_ {
                        _vec.push(x as i8);
                    }
                    Ok(Value::LSTI8(_vec))
                } else if inherits(object, "int16")? {
                    let mut _vec = Vec::<i16>::with_capacity(object_.len());
                    for x in object_ {
                        _vec.push(x as i16);
                    }
                    Ok(Value::LSTI16(_vec))
                } else if inherits(object, "int64")? {
                    let mut _vec = Vec::<i64>::with_capacity(object_.len());
                    for x in object_ {
                        _vec.push(x as i64);
                    }
                    Ok(Value::LSTI64(_vec))
                } else if inherits(object, "uint8")? {
                    let mut _vec = Vec::<u8>::with_capacity(object_.len());
                    for x in object_ {
                        _vec.push(x as u8);
                    }
                    Ok(Value::LSTU8(_vec))
                } else if inherits(object, "uint16")? {
                    let mut _vec = Vec::<u16>::with_capacity(object_.len());
                    for x in object_ {
                        _vec.push(x as u16);
                    }
                    Ok(Value::LSTU16(_vec))
                } else if inherits(object, "uint64")? {
                    let mut _vec = Vec::<u64>::with_capacity(object_.len());
                    for x in object_ {
                        _vec.push(x as u64);
                    }
                    Ok(Value::LSTU64(_vec))
                } else if inherits(object, "uint32")? {
                    let mut _vec = Vec::<u32>::with_capacity(object_.len());
                    for x in object_ {
                        _vec.push(x as u32);
                    }
                    Ok(Value::LSTU32(_vec))
                } else {
                    Ok(Value::LSTI32(object_))
                }
            }
        }
        LGLSXP => {
            let object_ = Vec::<bool>::rnew(object)?;
            if object_.len() != 1 {
                Err(RError::unknown(format!("bool : bad length : {}", object_.len()).to_string()))
            } else {
                Ok(Value::BOOL(object_[0]))
            }
        }
        VECSXP => {
            /* generic vectors */
            // empty list
            let rlist = RList::new(object)?;

            let names: CharVec = RName::name(&rlist);

            if names.rsize() > 0 || inherits(object, "tsonmap")? {
                let mut map = HashMap::with_capacity(names.rsize() as usize);
                let mut index = 0;

                for x in rlist {
                    map.insert(names.at(index)?, r_to_value(x)?);
                    index = index + 1;
                }

                Ok(Value::MAP(map))
            } else {
                let mut list: Vec<Value> = Vec::with_capacity(rlist.rsize() as usize);

                for x in rlist {
                    list.push(r_to_value(x)?);
                }
                Ok(Value::LST(list))
            }
        }
        STRSXP => {
            let object_: Vec<String> = Vec::<String>::rnew(object)?;
            if inherits(object, "scalar")? {
                if object_.len() != 1 {
                    Err(RError::unknown(format!("int : scalar bad length : {}", object_.len()).to_string()))
                } else {
                    Ok(Value::STR(object_[0].clone()))
                }
            } else {
                Ok(Value::LSTSTR(object_.into()))
            }
        }
        _ => Err(RError::unknown(format!("bad object type : {}", object.rtype()).to_string()))
    }
}

pub fn value_to_r(value: &Value) -> RResult<SEXP> {
    match *value {
        Value::NULL => ().intor(),
        Value::STR(ref v) => v.intor(),
        Value::I32(v) => v.intor(),
        Value::F64(v) => v.intor(),
        Value::BOOL(v) => v.intor(),
        Value::LST(ref v) => {
            let mut lst = RList::alloc(v.len());
            let mut i = 0;
            for obj in v.iter() {
                match value_to_r(obj) {
                    Ok(robj) => {
                        lst.set(i, robj)?;
                        i += 1;
                    }
                    Err(e) => {
                        return Err(RError::unknown(format!("value_to_r : {}", e).to_string()));
                    }
                }
            }
            lst.intor()
        }
        Value::MAP(ref v) => {
            let mut names = CharVec::alloc(v.len());
            let mut values = RList::alloc(v.len());
            let mut i = 0;
            for (k, obj) in v.iter() {
                names.set(i, k)?;
                match value_to_r(obj) {
                    Ok(robj) => {
                        values.set(i, robj)?;
                        i += 1;
                    }
                    Err(e) => {
                        return Err(RError::unknown(format!("value_to_r : {}", e).to_string()));
                    }
                }
            }

            unsafe {
                Rf_setAttrib(values.s(), R_NamesSymbol, names.s());
            }
            values.intor()
        }
        Value::LSTU8(ref v) => v.intor(),
        Value::LSTI8(ref v) => v.intor(),
        Value::LSTU16(ref v) => v.intor(),
        Value::LSTI16(ref v) => v.intor(),
        Value::LSTU32(ref v) => v.intor(),
        Value::LSTI32(ref v) => v.intor(),
        Value::LSTU64(ref v) => v.intor(),
        Value::LSTI64(ref v) => v.intor(),
        Value::LSTF32(ref v) => v.intor(),
        Value::LSTF64(ref v) => v.intor(),
        Value::LSTSTR(ref v) => v.try_to_vec()
            .map_err(|e| RError::unknown(e.to_string()))?.intor() ,
    }
}

