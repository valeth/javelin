use bytes::Buf;
use crate::Error;


/// See [MPEG-4 Audio Object Types][audio_object_types]
///
/// [audio_object_types]: https://en.wikipedia.org/wiki/MPEG-4_Part_3#MPEG-4_Audio_Object_Types
#[allow(clippy::enum_variant_names, dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AudioObjectType {
    AacMain = 1,
    AacLowComplexity = 2,
    AacScalableSampleRate = 3,
    AacLongTermPrediction = 4
}

impl AudioObjectType {
    pub fn try_from_u8(value: u8) -> Result<Self, Error> {
        let val = match  value {
            1 => AudioObjectType::AacMain,
            2 => AudioObjectType::AacLowComplexity,
            3 => AudioObjectType::AacScalableSampleRate,
            4 => AudioObjectType::AacLongTermPrediction,
            _ => return Err(Error::Custom("Unsupported audio object type".into())),
        };

        Ok(val)
    }
}


/// Bits | Description
/// ---- | -----------
/// 5    | Audio object type
/// 4    | Sampling frequency index
/// 4    | Channel configuration
/// 1    | Frame length flag
/// 1    | Depends on core coder
/// 1    | Extension flag
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioSpecificConfiguration {
    pub object_type: AudioObjectType,
    pub sampling_frequency_index: u8,
    pub channel_configuration: u8,
    pub frame_length_flag: bool,
    pub depends_on_core_coder: bool,
    pub extension_flag: bool,
}

impl AudioSpecificConfiguration {
    pub fn try_from_buf<B>(buf: &mut B) -> Result<Self, Error>
        where B: Buf
    {
        let x = buf.get_u8();
        let y = buf.get_u8();

        let object_type = AudioObjectType::try_from_u8((x & 0xF8) >> 3)?;
        let sampling_frequency_index = ((x & 0x07) << 1) | (y >> 7);
        let channel_configuration = (y >> 3) & 0x0F;

        let frame_length_flag = (y & 0x04) == 0x04;
        let depends_on_core_coder = (y & 0x02) == 0x02;
        let extension_flag = (y & 0x01) == 0x01;

        Ok(Self {
            object_type,
            sampling_frequency_index,
            channel_configuration,
            frame_length_flag,
            depends_on_core_coder,
            extension_flag,
        })
    }
}


#[cfg(test)]
mod tests {
    use bytes::{Bytes, IntoBuf};
    use super::*;

    #[test]
    fn can_parse_sequence_header() {
        let expected = AudioSpecificConfiguration {
            object_type: AudioObjectType::AacLowComplexity,
            sampling_frequency_index: 4,
            channel_configuration: 2,
            frame_length_flag: false,
            depends_on_core_coder: false,
            extension_flag: false,
        };

        let raw = Bytes::from_static(&[
            0b0001_0010, 0b0001_0000,
            0b0101_0110, 0b1110_0101, 0b0000_0000
        ]);

        let actual = AudioSpecificConfiguration::try_from_buf(&mut raw.clone().into_buf()).unwrap();

        assert_eq!(expected, actual);
    }
}
