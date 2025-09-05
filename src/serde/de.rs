use std::mem::transmute;

use libquickjs_ng_sys::JSContext;
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::{forward_to_deserialize_any, Deserialize};

use crate::utils::deserialize_borrowed_str;
use crate::value::{JsTag, OwnedJsArray, OwnedJsObject, OwnedJsPropertyIterator, OwnedJsValue};

use super::error::{Error, Result};

/// A structure that deserializes JS values into Rust values.
pub struct Deserializer<'de> {
    context: *mut JSContext,
    root: &'de OwnedJsValue,
    paths: Vec<(OwnedJsValue, u32, Option<OwnedJsPropertyIterator>)>,
    current: Option<OwnedJsValue>,
}

impl<'de> Deserializer<'de> {
    fn from_js(context: *mut JSContext, root: &'de OwnedJsValue) -> Self {
        Deserializer {
            context,
            root,
            paths: Vec::new(),
            current: Some(root.clone()),
        }
    }
}

/// Deserialize an instance of type `T` from a JS value.
pub fn from_js<'a, T>(context: *mut JSContext, value: &'a OwnedJsValue) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_js(context, value);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

impl<'de> Deserializer<'de> {
    fn get_current(&self) -> &OwnedJsValue {
        if let Some(current) = self.current.as_ref() {
            current
        } else {
            self.root
        }
    }

    fn next(&mut self) -> Result<Option<()>> {
        let (current, index, obj_iter) = self.paths.last_mut().expect("current must be Some");

        let next = if current.is_array() {
            let current = OwnedJsArray::try_from_value(current.clone()).unwrap();
            let item = current.get_index(*index)?;
            if item.is_some() {
                self.current = item;
                *index += 1;
                Some(())
            } else {
                None
            }
        } else if current.is_object() {
            let obj_iter = obj_iter.as_mut().expect("obj_iter must be Some");
            if let Some(ret) = obj_iter.next() {
                self.current = Some(ret?);
                // index here is useless, but we just keep it for consistency
                *index += 1;
                Some(())
            } else {
                None
            }
        } else {
            return Err(Error::ExpectedArrayOrObject);
        };

        if next.is_some() {
            Ok(next)
        } else {
            Ok(None)
        }
    }

    fn guard_circular_reference(&self, current: &OwnedJsValue) -> Result<()> {
        if self.paths.iter().any(|(p, _, _)| p == current) {
            Err(Error::CircularReference)
        } else {
            Ok(())
        }
    }

    fn enter_array(&mut self) -> Result<()> {
        let mut current = self.get_current().clone();

        if current.is_proxy() {
            current = current.get_proxy_target(true)?;
        }

        if current.is_array() {
            self.guard_circular_reference(&current)?;
            self.paths.push((current, 0, None));
            Ok(())
        } else {
            Err(Error::ExpectedArray)
        }
    }

    fn enter_object(&mut self) -> Result<()> {
        let mut current = self.get_current().clone();

        if current.is_proxy() {
            current = current.get_proxy_target(true)?;
        }

        if current.is_object() {
            let obj = OwnedJsObject::try_from_value(current.clone()).unwrap();
            self.guard_circular_reference(&current)?;
            self.paths.push((current, 0, Some(obj.properties_iter()?)));
            Ok(())
        } else {
            Err(Error::ExpectedObject)
        }
    }

    fn leave(&mut self) {
        if let Some((current, _, _)) = self.paths.pop() {
            self.current = Some(current);
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        let current = self.get_current();
        if current.is_string() {
            current.to_string().map_err(|err| err.into())
        } else {
            Err(Error::ExpectedString)
        }
    }

    fn parse_borrowed_str(&mut self) -> Result<&'de str> {
        let current = self.get_current();
        if current.is_string() {
            let s = deserialize_borrowed_str(self.context, &current.value).unwrap();

            // in this case, 'de is equal to '1
            // so force transmute lifetime
            let s = unsafe { transmute(s) };

            Ok(s)
        } else {
            Err(Error::ExpectedString)
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let current = self.get_current();

        match current.tag() {
            JsTag::Undefined => visitor.visit_unit(),
            JsTag::Int => visitor.visit_i32(current.to_int()?),
            JsTag::Bool => visitor.visit_bool(current.to_bool()?),
            JsTag::Null => visitor.visit_unit(),
            JsTag::String => visitor.visit_string(current.to_string()?),
            JsTag::Float64 => visitor.visit_f64(current.to_float()?),
            JsTag::Object => {
                if current.is_array() {
                    self.deserialize_seq(visitor)
                } else {
                    self.deserialize_map(visitor)
                }
            }
            JsTag::Symbol => visitor.visit_unit(),
            JsTag::Module => visitor.visit_unit(),
            JsTag::Exception => self.deserialize_map(visitor),
            JsTag::CatchOffset => visitor.visit_unit(),
            JsTag::Uninitialized => visitor.visit_unit(),
            JsTag::FunctionBytecode => visitor.visit_unit(),
            #[cfg(feature = "bigint")]
            JsTag::ShortBigInt => {
                let bigint = current.to_bigint()?;
                visitor.visit_i64(bigint.as_i64().ok_or(Error::BigIntOverflow)?)
            }
            #[cfg(feature = "bigint")]
            JsTag::BigInt => {
                let bigint = current.to_bigint()?;
                visitor.visit_i64(bigint.as_i64().ok_or(Error::BigIntOverflow)?)
            } // _ => {
              //     #[cfg(debug_assertions)]
              //     {
              //         println!("current type: {:?}", current.tag());
              //         println!("current: {}", current.to_json_string(0).unwrap());
              //     }
              //     unreachable!("unreachable tag: {:?}", current.tag());
              // }
        }
    }

    forward_to_deserialize_any! {
        bool
        i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
        string char
        unit
        identifier ignored_any
    }

    fn deserialize_str<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_borrowed_str()?)
    }

    fn deserialize_seq<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.enter_array()?;
        let r = visitor.visit_seq(&mut *self);
        self.leave();
        r
    }

    fn deserialize_bytes<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    // for some type like &[u8]
    fn deserialize_byte_buf<V>(self, _: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("borrowed bytes not supported yet")
        // self.deserialize_seq(visitor)
    }

    fn deserialize_tuple<V>(
        self,
        _len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.get_current().is_null() || self.get_current().is_undefined() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_map<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.enter_object()?;
        let r = visitor.visit_map(&mut *self);
        self.leave();
        r
    }

    // Structs look just like maps in JSON.
    //
    // Notice the `fields` parameter - a "struct" in the Serde data model means
    // that the `Deserialize` implementation is required to know what the fields
    // are before even looking at the input data. Any key-value pairing in which
    // the fields cannot be known ahead of time is probably a map.
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
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
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.get_current().is_object() {
            // Visit a newtype variant, tuple variant, or struct variant.
            self.enter_object()?;
            self.next()?;
            let r = visitor.visit_enum(Enum::new(self));
            self.leave();
            r
        } else {
            // Visit a unit variant.
            visitor.visit_enum(self.parse_string()?.into_deserializer())
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }
}

impl<'de, 'a> SeqAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(_) = self.next()? {
            seed.deserialize(self).map(Some)
        } else {
            Ok(None)
        }
    }
}

impl<'de, 'a> MapAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some(_) = self.next()? {
            seed.deserialize(self).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // It doesn't make a difference whether the colon is parsed at the end
        // of `next_key_seed` or at the beginning of `next_value_seed`. In this
        // case the code is a bit simpler having it here.
        // if self.de.next_char()? != ':' {
        //     return Err(Error::ExpectedMapColon);
        // }
        // Deserialize a map value.
        self.next()?;
        seed.deserialize(self)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        // The `deserialize_enum` method parsed a `{` character so we are
        // currently inside of a map. The seed will be deserializing itself from
        // the key of the map.

        let val = seed.deserialize(&mut *self.de)?;
        self.de.next()?;
        // Parse the colon separating map key from value.
        // if self.de.next_char()? == ':' {
        Ok((val, self))
        // } else {
        //     Err(Error::ExpectedMapColon)
        // }
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<()> {
        Err(Error::ExpectedString)
    }

    // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}
