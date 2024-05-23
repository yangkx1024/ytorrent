use serde::de::{
    DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor,
};
use serde::Deserializer;

use super::*;
use super::Error::*;

macro_rules! deserialize_integer {
    ($self:ident, $int_type:ty, $target_type:literal) => {{
        let cur_position = $self.offset;
        println!("deserialize_integer for {}", $target_type);
        match $self.parse()? {
            Some(Object::Int(value)) => value.parse::<$int_type>().map_err(|e| {
                SerdeCustom(format!(
                    "invalid integer when parse {} at {}, {:?}",
                    $target_type, cur_position, e
                ))
            }),
            Some(other) => Err(SerdeCustom(format!(
                "expect integer for {} but get {} at {}",
                $target_type, other, cur_position
            ))),
            None => Err(SerdeCustom(format!(
                "unexpect EOF when parse integer for {} at {}",
                $target_type, cur_position
            ))),
        }
    }};
}

macro_rules! deserialize_string {
    ($self:ident, $target_type:literal) => {{
        let cur_position = $self.offset;
        println!("deserialize_string for {}", $target_type);
        match $self.parse()? {
            Some(Object::Bytes(bytes)) => std::str::from_utf8(bytes).map_err(|e| {
                SerdeCustom(format!(
                    "UTF-8 error: {} when parse {} at {}",
                    e, $target_type, cur_position
                ))
            }),
            Some(other) => Err(SerdeCustom(format!(
                "expect string for {} but get {} at {}",
                $target_type, other, cur_position
            ))),
            None => Err(SerdeCustom(format!(
                "unexpect EOF when parse string for {} at {}",
                $target_type, cur_position
            ))),
        }
    }};
}

macro_rules! deserialize_bytes {
    ($self:ident, $target_type:literal) => {{
        let cur_position = $self.offset;
        println!("deserialize_bytes for {}", $target_type);
        match $self.parse()? {
            Some(Object::Bytes(bytes)) => Ok(bytes),
            Some(other) => Err(SerdeCustom(format!(
                "expect bytes for {} but get {} at {}",
                $target_type, other, cur_position
            ))),
            None => Err(SerdeCustom(format!(
                "unexpect EOF when parse bytes for {} at {}",
                $target_type, cur_position
            ))),
        }
    }};
}

impl<'de, 'a> Deserializer<'de> for &'a mut BencodeParser<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("deserialize_any");
        let cur_position = self.offset;
        match *self.peek_token()? {
            Token::Dict => self.deserialize_map(visitor),
            Token::List => self.deserialize_seq(visitor),
            Token::Num(_) => self.deserialize_i64(visitor),
            Token::String(_) => self.deserialize_bytes(visitor),
            Token::End => Err(SerdeCustom(format!(
                "unexpected EOF at {} deserialize_any",
                cur_position
            ))),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match deserialize_integer!(self, i64, "bool")? {
            positive if positive > 0 => visitor.visit_bool(true),
            _ => visitor.visit_bool(false),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(deserialize_integer!(self, i8, "i8")?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(deserialize_integer!(self, i16, "i16")?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(deserialize_integer!(self, i32, "i32")?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(deserialize_integer!(self, i64, "i64")?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(deserialize_integer!(self, u8, "u8")?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(deserialize_integer!(self, u16, "u16")?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(deserialize_integer!(self, u32, "u32")?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(deserialize_integer!(self, u64, "u64")?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(deserialize_integer!(self, f32, "f32")?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(deserialize_integer!(self, f64, "f64")?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let position = self.offset;
        let str = deserialize_string!(self, "char")?;
        if str.len() != 1 {
            Err(SerdeCustom(format!(
                "expect char but get {} at {}",
                str, position
            )))
        } else {
            visitor.visit_char(str.chars().next().unwrap())
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(deserialize_string!(self, "str")?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(deserialize_string!(self, "str")?.to_string())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(deserialize_bytes!(self, "bytes")?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(deserialize_bytes!(self, "byte_buf")?.to_vec())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Delegate to OptionVisitor to parse original T of Option<T>
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.expect_empty_list("unit/unit_struct")?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("deserialize_seq");
        self.expect_list_begin("seq/tuple/tuple_struct")?;
        let value = visitor.visit_seq(&mut *self)?;
        self.expect_end("seq/tuple/tuple_struct")?;
        Ok(value)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("deserialize_tuple");
        self.deserialize_any(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("deserialize_tuple_struct");
        self.deserialize_any(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("deserialize_map");
        self.expect_dict_begin("map/struct")?;
        let value = visitor.visit_map(&mut *self)?;
        self.expect_end("map/struct")?;
        println!("end deserialize_map");
        Ok(value)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let cur_position = self.offset;
        match &*self.peek_token()? {
            Token::Dict => {
                self.expect_dict_begin("enum")?;
                visitor.visit_map(self)
            }
            Token::String(bytes) => {
                // consume the peeked token
                self.next_token()?;
                let str = std::str::from_utf8(bytes).map_err(|e| {
                    SerdeCustom(format!(
                        "UTF-8 error: {} when parse enum at {}",
                        e, cur_position
                    ))
                })?;
                // Delegate to StrDeserializer
                visitor.visit_enum(str.into_deserializer())
            }
            other => Err(SerdeCustom(format!(
                "expect dict/bytes for enum but get {} at {}",
                other, cur_position
            ))),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(deserialize_string!(self, "identifier")?)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

/// Here we already consumed the start 'e' of the list, make sure this visitor consume 'e' as well.
impl<'de> MapAccess<'de> for BencodeParser<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        let token = self.peek_token()?;
        if *token == Token::End {
            return Ok(None);
        }
        println!("visit map key {}", token);
        seed.deserialize(self).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        println!("visit map value");
        seed.deserialize(self)
    }
}

impl<'de> SeqAccess<'de> for BencodeParser<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        let token = self.peek_token()?;
        if *token == Token::End {
            return Ok(None);
        }
        seed.deserialize(self).map(Some)
    }
}

impl<'de> VariantAccess<'de> for &'de mut BencodeParser<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut *self)?;
        self.expect_end("newtype_variant_seed")?;
        Ok(value)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = serde::de::Deserializer::deserialize_seq(&mut *self, visitor)?;
        self.expect_end("tuple_variant")?;
        Ok(value)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = serde::de::Deserializer::deserialize_map(&mut *self, visitor)?;
        self.expect_end("struct_variant")?;
        Ok(value)
    }
}

impl<'de> EnumAccess<'de> for &'de mut BencodeParser<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self)>
    where
        V: DeserializeSeed<'de>,
    {
        Ok((seed.deserialize(&mut *self)?, self))
    }
}

pub fn from_bytes<'de, T>(b: &'de [u8]) -> Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    serde::de::Deserialize::deserialize(&mut BencodeParser::new(b))
}
