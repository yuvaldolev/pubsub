use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::error;

#[derive(Debug)]
pub struct SubscriptionRequest {
    topics: Vec<String>,
}

impl SubscriptionRequest {
    pub fn new(topics: Vec<String>) -> Self {
        Self { topics }
    }

    pub fn read(reader: &mut impl Read) -> error::Result<Self> {
        // Read the number of topics.
        let topics_number = reader.read_u32::<BigEndian>()?;

        // Read the topics.
        let mut topics: Vec<String> = Vec::with_capacity(topics_number as usize);
        for _ in 0..topics_number {
            // Read the topics's size.
            let size = reader.read_u32::<BigEndian>()?;

            // Read the topic.
            let mut bytes: Vec<u8> = vec![0; size as usize];
            reader.read_exact(&mut bytes)?;

            // Add the topic to the topics vector.
            topics.push(String::from_utf8(bytes)?);
        }

        Ok(Self { topics })
    }

    pub fn write(&self, writer: &mut impl Write) -> error::Result<()> {
        // Write the number of topics.
        writer.write_u32::<BigEndian>(self.topics.len() as u32)?;

        // Write the topics.
        for topic in self.topics.iter() {
            writer.write_u32::<BigEndian>(topic.len() as u32)?;
            writer.write_all(topic.as_bytes())?;
        }

        Ok(())
    }
}
