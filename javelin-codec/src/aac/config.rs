use {
    std::{
        io::Cursor,
        convert::TryFrom,
    },
    bytes::Buf,
    super::{
        common::{SamplingFrequencyIndex, ChannelConfiguration, AudioObjectType},
        AacError,
    },
};


// Bits | Description
// ---- | -----------
// 5    | Audio object type
// 4    | Sampling frequency index
// 4    | Channel configuration
// AOT specific section
// 1    | Frame length flag
// 1    | Depends on core coder
// 1    | Extension flag
///
#[derive(Debug, Clone)]
pub struct AudioSpecificConfiguration {
    pub object_type: AudioObjectType,
    pub sampling_frequency_index: SamplingFrequencyIndex,
    pub sampling_frequency: Option<u32>,
    pub channel_configuration: ChannelConfiguration,
    pub frame_length_flag: bool,
    pub depends_on_core_coder: bool,
    pub extension_flag: bool,
}

impl TryFrom<&[u8]> for AudioSpecificConfiguration {
    type Error = AacError;

    fn try_from(val: &[u8]) -> Result<Self, Self::Error> {
        if val.len() < 2 {
            return Err(AacError::NotEnoughData("AAC audio specific config"));
        }

        let mut buf = Cursor::new(val);

        let header_a = buf.get_u8();
        let header_b = buf.get_u8();

        let object_type = AudioObjectType::try_from((header_a & 0xF8) >> 3)?;

        let sf_idx = ((header_a & 0x07) << 1) | (header_b >> 7);
        let sampling_frequency_index = SamplingFrequencyIndex::try_from(sf_idx)?;

        let channel_configuration = ChannelConfiguration::try_from((header_b >> 3) & 0x0F)?;
        let frame_length_flag = (header_b & 0x04) == 0x04;
        let depends_on_core_coder = (header_b & 0x02) == 0x02;
        let extension_flag = (header_b & 0x01) == 0x01;

        Ok(Self {
            object_type,
            sampling_frequency_index,
            sampling_frequency: None,
            channel_configuration,
            frame_length_flag,
            depends_on_core_coder,
            extension_flag,
        })
    }
}
