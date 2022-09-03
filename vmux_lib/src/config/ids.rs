use serde::de::Visitor;
use serde::*;

//pub type PlaylistId = u64;
#[derive(PartialEq, Default, Hash, Debug, Clone, Copy)]
pub struct PlaylistId {
    asdasd: u64,
}

impl PlaylistId {
    pub fn from_pis(a: u64) -> PlaylistId {
        PlaylistId { asdasd: a }
    }

    pub fn acual_title_pis(&self) -> u64 {
        self.asdasd
    }
    pub fn set_acual_title_pis(&mut self, a: u64) {
        self.asdasd = a;
    }
}

impl serde::Serialize for PlaylistId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.asdasd)
    }
}

impl<'de> Deserialize<'de> for PlaylistId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        //deserializer.deserialize_any(CustomVisitor)
        let asd = deserializer.deserialize_any(TitleVisitor)?;
        Ok(PlaylistId { asdasd: asd })
    }
}

//pub type TitleId = u64;
#[derive(PartialEq, Default, Hash, Debug, Clone, Copy)]
pub struct TitleId {
    asdasd: u64,
}

impl TitleId {
    pub fn from_title_id(a: u64) -> TitleId {
        TitleId { asdasd: a }
    }

    pub fn acual_title_id(&self) -> u64 {
        self.asdasd
    }
    pub fn set_acual_title_id(&mut self, a: u64) {
        self.asdasd = a;
    }
}

impl serde::Serialize for TitleId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.asdasd)
    }
}

struct TitleVisitor;
impl<'de> Visitor<'de> for TitleVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an integer between -2^64 and 2^64")
    }

    fn visit_u64<E>(self, s: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        return Ok(s);
    }
}
impl<'de> Deserialize<'de> for TitleId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        //deserializer.deserialize_any(CustomVisitor)
        let asd = deserializer.deserialize_any(TitleVisitor)?;
        Ok(TitleId { asdasd: asd })
    }
}
