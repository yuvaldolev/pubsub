use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::error;

#[derive(Debug)]
pub struct Message {
    topic: String,
    data: Vec<u8>,
}

impl Message {
    pub fn new(topic: String, data: Vec<u8>) -> Self {
        Self { topic, data }
    }

    pub fn read(reader: &mut impl Read) -> error::Result<Self> {
        // Read the topic.
        let topic_size = reader.read_u32::<BigEndian>()?;

        let mut topic_bytes: Vec<u8> = vec![0; topic_size as usize];
        reader.read_exact(&mut topic_bytes)?;
        let topic = String::from_utf8(topic_bytes)?;

        // Read the data.
        let data_size = reader.read_u32::<BigEndian>()?;

        let mut data: Vec<u8> = vec![0; data_size as usize];
        reader.read_exact(&mut data)?;

        Ok(Self { topic, data })
    }

    pub fn write(&self, writer: &mut impl Write) -> error::Result<()> {
        // Write the topic.
        writer.write_u32::<BigEndian>(self.topic.len() as u32)?;
        writer.write_all(self.topic.as_bytes())?;

        // Write the data.
        writer.write_u32::<BigEndian>(self.data.len() as u32)?;
        writer.write_all(&self.data)?;

        Ok(())
    }
}
