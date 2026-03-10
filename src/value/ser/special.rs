use core::mem;

use serde::{Serializer, ser::SerializeStruct};

use crate::value::ser::Error;

pub struct SpecialValueSerializer<S: Serializer> {
    marker: &'static str,
    slot: State<S>,
}

impl<S> SpecialValueSerializer<S>
where
    S: Serializer,
{
    pub const fn new(marker: &'static str, se: S) -> Self {
        Self {
            marker,
            slot: State::Start(se),
        }
    }
}

impl<S> SerializeStruct for SpecialValueSerializer<S>
where
    S: serde::Serializer<Error = Error>,
{
    type Ok = S::Ok;
    type Error = Error;

    fn serialize_field<V>(&mut self, key: &'static str, value: &V) -> Result<(), Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        if key != self.marker {
            return Err(Error::InvalidValue(self.marker));
        }

        let State::Start(se) = mem::replace(&mut self.slot, State::Invalid) else {
            return Err(Error::InvalidValue(self.marker));
        };

        self.slot = State::End(value.serialize(se)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let State::End(written) = self.slot else {
            return Err(Error::InvalidValue(self.marker));
        };

        Ok(written)
    }
}

enum State<S: Serializer> {
    Start(S),
    End(S::Ok),
    Invalid,
}
