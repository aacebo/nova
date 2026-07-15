mod float;
mod int;

pub use float::*;
pub use int::*;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub enum Number {
    Int(Int),
    Float(Float),
}

#[cfg(feature = "serde")]
impl serde::Serialize for Number {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Int(v) => v.serialize(s),
            Self::Float(v) => v.serialize(s),
        }
    }
}

impl Number {
    pub fn to_type(&self) -> crate::Type {
        match self {
            Self::Int(v) => v.to_type(),
            Self::Float(v) => v.to_type(),
        }
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    pub fn to_int(&self) -> Option<Int> {
        self.as_int().copied()
    }

    pub fn as_int(&self) -> Option<&Int> {
        match self {
            Self::Int(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_float(&self) -> Option<Float> {
        self.as_float().copied()
    }

    pub fn as_float(&self) -> Option<&Float> {
        match self {
            Self::Float(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_i8(&self) -> i8 {
        match self {
            Self::Int(v) => v.to_i128() as i8,
            Self::Float(v) => v.to_f64_raw() as i8,
        }
    }

    pub fn to_i16(&self) -> i16 {
        match self {
            Self::Int(v) => v.to_i128() as i16,
            Self::Float(v) => v.to_f64_raw() as i16,
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            Self::Int(v) => v.to_i128() as i32,
            Self::Float(v) => v.to_f64_raw() as i32,
        }
    }

    pub fn to_i64(&self) -> i64 {
        match self {
            Self::Int(v) => v.to_i128() as i64,
            Self::Float(v) => v.to_f64_raw() as i64,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Int(v) => v.to_i128() as u8,
            Self::Float(v) => v.to_f64_raw() as u8,
        }
    }

    pub fn to_u16(&self) -> u16 {
        match self {
            Self::Int(v) => v.to_i128() as u16,
            Self::Float(v) => v.to_f64_raw() as u16,
        }
    }

    pub fn to_u32(&self) -> u32 {
        match self {
            Self::Int(v) => v.to_i128() as u32,
            Self::Float(v) => v.to_f64_raw() as u32,
        }
    }

    pub fn to_u64(&self) -> u64 {
        match self {
            Self::Int(v) => v.to_i128() as u64,
            Self::Float(v) => v.to_f64_raw() as u64,
        }
    }

    pub fn to_f32(&self) -> f32 {
        match self {
            Self::Int(v) => v.to_i128() as f32,
            Self::Float(v) => v.to_f64_raw() as f32,
        }
    }

    pub fn to_f64(&self) -> f64 {
        match self {
            Self::Int(v) => v.to_i128() as f64,
            Self::Float(v) => v.to_f64_raw(),
        }
    }
}

impl crate::ValueRef<'_> {
    pub fn to_i8(&self) -> Option<i8> {
        self.as_number().map(|v| v.to_i8())
    }

    pub fn to_i16(&self) -> Option<i16> {
        self.as_number().map(|v| v.to_i16())
    }

    pub fn to_i32(&self) -> Option<i32> {
        self.as_number().map(|v| v.to_i32())
    }

    pub fn to_i64(&self) -> Option<i64> {
        self.as_number().map(|v| v.to_i64())
    }

    pub fn to_u8(&self) -> Option<u8> {
        self.as_number().map(|v| v.to_u8())
    }

    pub fn to_u16(&self) -> Option<u16> {
        self.as_number().map(|v| v.to_u16())
    }

    pub fn to_u32(&self) -> Option<u32> {
        self.as_number().map(|v| v.to_u32())
    }

    pub fn to_u64(&self) -> Option<u64> {
        self.as_number().map(|v| v.to_u64())
    }

    pub fn to_f32(&self) -> Option<f32> {
        self.as_number().map(|v| v.to_f32())
    }

    pub fn to_f64(&self) -> Option<f64> {
        self.as_number().map(|v| v.to_f64())
    }
}

impl crate::Value {
    pub fn to_i8(&self) -> Option<i8> {
        self.as_number().map(|v| v.to_i8())
    }

    pub fn to_i16(&self) -> Option<i16> {
        self.as_number().map(|v| v.to_i16())
    }

    pub fn to_i32(&self) -> Option<i32> {
        self.as_number().map(|v| v.to_i32())
    }

    pub fn to_i64(&self) -> Option<i64> {
        self.as_number().map(|v| v.to_i64())
    }

    pub fn to_u8(&self) -> Option<u8> {
        self.as_number().map(|v| v.to_u8())
    }

    pub fn to_u16(&self) -> Option<u16> {
        self.as_number().map(|v| v.to_u16())
    }

    pub fn to_u32(&self) -> Option<u32> {
        self.as_number().map(|v| v.to_u32())
    }

    pub fn to_u64(&self) -> Option<u64> {
        self.as_number().map(|v| v.to_u64())
    }

    pub fn to_f32(&self) -> Option<f32> {
        self.as_number().map(|v| v.to_f32())
    }

    pub fn to_f64(&self) -> Option<f64> {
        self.as_number().map(|v| v.to_f64())
    }
}

impl crate::ToValue for Number {
    fn to_value_ref(&self) -> crate::ValueRef<'static> {
        crate::ValueRef::Number(*self)
    }
}

impl From<Number> for crate::Value {
    fn from(value: Number) -> Self {
        crate::Value::Number(value)
    }
}

impl PartialEq<crate::ValueRef<'_>> for Number {
    fn eq(&self, other: &crate::ValueRef<'_>) -> bool {
        other.as_number() == Some(self)
    }
}

impl PartialEq<crate::Value> for Number {
    fn eq(&self, other: &crate::Value) -> bool {
        other.as_number() == Some(self)
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{}", v),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::value_of;

    #[test]
    pub fn int_widens_up() {
        let value = value_of!(1_i32);

        assert_eq!(value.to_i64(), Some(1));
        assert_eq!(value.to_u8(), Some(1));
        assert_eq!(value.to_f32(), Some(1.0));
        assert_eq!(value.to_f64(), Some(1.0));
    }

    #[test]
    pub fn float_narrows_to_int() {
        let value = value_of!(3.9_f64);

        assert_eq!(value.to_i32(), Some(3));
        assert_eq!(value.to_f32(), Some(3.9));
    }

    #[test]
    pub fn same_type_still_reads() {
        assert_eq!(value_of!(7_u64).to_u64(), Some(7));
        assert_eq!(value_of!(-5_i16).to_i16(), Some(-5));
    }
}
