use crate::models;
use tokio_util::bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

pub struct PacketDecoder {}

const MAX: usize = 8 * 1024 * 1024;

impl Decoder for PacketDecoder {
    type Item = models::Packet;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_le_bytes(length_bytes) as usize;

        if length > MAX {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", length),
            ));
        }

        if src.len() < 4 + length {
            src.reserve(4 + length - src.len());

            return Ok(None);
        }

        let data = &src[4..4 + length];

        let frame = serde_json::from_slice(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        src.advance(4 + length);

        Ok(Some(frame))
    }
}

pub struct PacketEncoder {}

impl Encoder<models::Packet> for PacketEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: models::Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let data = serde_json::to_vec(&item)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        dst.reserve(4 + data.len());

        dst.extend_from_slice(&u32::to_le_bytes(data.len() as u32));
        dst.extend_from_slice(&data);
        Ok(())
    }
}
