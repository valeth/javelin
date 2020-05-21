use std::{fs::File, path::Path};
use bytes::{Bytes, Buf, IntoBuf};
use mpeg2ts::{
    ts::{
        ContinuityCounter,
        TsPacket,
        TsHeader,
        TsPayload,
        Pid,
    },
    pes::PesHeader,
};
use javelin_codec::{avc, aac};
use crate::Result;


const PMT_PID: u16 = 256;
const VIDEO_ES_PID: u16 = 257;
const AUDIO_ES_PID: u16 = 258;
const PES_VIDEO_STREAM_ID: u8 = 224;
const PES_AUDIO_STREAM_ID: u8 = 192;


pub struct Buffer {
    video_continuity_counter: ContinuityCounter,
    audio_continuity_counter: ContinuityCounter,
    packets: Vec<TsPacket>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            video_continuity_counter: ContinuityCounter::new(),
            audio_continuity_counter: ContinuityCounter::new(),
            packets: Vec::new(),
        }
    }

    pub fn write_to_file<P>(&mut self, filename: P) -> Result<()>
        where P: AsRef<Path>
    {
        use mpeg2ts::ts::{TsPacketWriter, WriteTsPacket};

        let file = File::create(filename)?;
        let packets: Vec<_> = self.packets.drain(..).collect();
        let mut writer = TsPacketWriter::new(file);

        writer.write_ts_packet(&default_pat_packet())?;
        writer.write_ts_packet(&default_pmt_packet())?;

        for packet in &packets {
            writer.write_ts_packet(packet)?;
        }

        Ok(())
    }

    pub fn push_video(&mut self, video: &avc::Packet) -> Result<()> {
        use mpeg2ts::{
            ts::{AdaptationField, payload},
            es::StreamId,
            time::{ClockReference, Timestamp},
        };

        let mut header = default_ts_header(VIDEO_ES_PID)?;
        header.continuity_counter = self.video_continuity_counter;

        let mut buf = video.try_as_bytes()?.into_buf();
        let pes_data: Bytes = buf.by_ref().take(153).collect();
        let pcr = ClockReference::new(video.timestamp() * 90)?;

        let adaptation_field = if video.is_keyframe() {
            Some(AdaptationField {
                discontinuity_indicator: false,
                random_access_indicator: true,
                es_priority_indicator: false,
                pcr: Some(pcr),
                opcr: None,
                splice_countdown: None,
                transport_private_data: Vec::new(),
                extension: None,
            })
        } else {
            None
        };

        let pts = Timestamp::new(video.presentation_timestamp() * 90)?;
        let dts = Timestamp::new(video.timestamp() * 90)?;

        let packet = TsPacket {
            header: header.clone(),
            adaptation_field,
            payload: Some(TsPayload::Pes(payload::Pes {
                header: PesHeader {
                    stream_id: StreamId::new(PES_VIDEO_STREAM_ID),
                    priority: false,
                    data_alignment_indicator: false,
                    copyright: false,
                    original_or_copy: false,
                    pts: Some(pts),
                    dts: Some(dts),
                    escr: None,
                },
                pes_packet_len: 0,
                data: payload::Bytes::new(&pes_data)?,
            })),
        };

        self.packets.push(packet);

        while buf.has_remaining() {
            let pes_data: Bytes = buf.by_ref().take(payload::Bytes::MAX_SIZE).collect();
            header.continuity_counter.increment();

            let packet = TsPacket {
                header: header.clone(),
                adaptation_field: None,
                payload: Some(TsPayload::Raw(payload::Bytes::new(&pes_data)?)),
            };

            self.packets.push(packet);
        }

        header.continuity_counter.increment();
        self.video_continuity_counter = header.continuity_counter;

        Ok(())
    }

    pub fn push_audio(&mut self, audio: &aac::Packet) -> Result<()> {
        use mpeg2ts::{
            ts::payload,
            es::StreamId,
            time::Timestamp,
        };

        let mut buf = audio.to_bytes().into_buf();
        let pes_data: Bytes = buf.by_ref().take(153).collect();

        let mut header = default_ts_header(AUDIO_ES_PID)?;
        header.continuity_counter = self.audio_continuity_counter;

        let packet = TsPacket {
            header: header.clone(),
            adaptation_field: None,
            payload: Some(TsPayload::Pes(payload::Pes {
                header: PesHeader {
                    stream_id: StreamId::new(PES_AUDIO_STREAM_ID),
                    priority: false,
                    data_alignment_indicator: false,
                    copyright: false,
                    original_or_copy: false,
                    pts: Some(Timestamp::new(audio.presentation_timestamp() * 90)?),
                    dts: None,
                    escr: None,
                },
                pes_packet_len: 0,
                data: payload::Bytes::new(&pes_data)?,
            })),
        };

        self.packets.push(packet);

        while buf.has_remaining() {
            let pes_data: Bytes = buf.by_ref().take(payload::Bytes::MAX_SIZE).collect();
            header.continuity_counter.increment();

            let packet = TsPacket {
                header: header.clone(),
                adaptation_field: None,
                payload: Some(TsPayload::Raw(payload::Bytes::new(&pes_data)?)),
            };

            self.packets.push(packet);
        }

        header.continuity_counter.increment();
        self.audio_continuity_counter = header.continuity_counter;

        Ok(())
    }
}


fn default_ts_header(pid: u16) -> Result<TsHeader> {
    use mpeg2ts::ts::TransportScramblingControl;

    Ok(TsHeader {
        transport_error_indicator: false,
        transport_priority: false,
        pid: Pid::new(pid)?,
        transport_scrambling_control: TransportScramblingControl::NotScrambled,
        continuity_counter: ContinuityCounter::new(),
    })
}

fn default_pat_packet() -> TsPacket {
    use mpeg2ts::ts::{VersionNumber, payload::Pat, ProgramAssociation};

    TsPacket {
        header: default_ts_header(0).unwrap(),
        adaptation_field: None,
        payload: Some(
            TsPayload::Pat(Pat {
                transport_stream_id: 1,
                version_number: VersionNumber::default(),
                table: vec![
                    ProgramAssociation {
                        program_num: 1,
                        program_map_pid: Pid::new(PMT_PID).unwrap(),
                    }
                ]
            })),
    }
}

fn default_pmt_packet() -> TsPacket {
    use mpeg2ts::{
        ts::{VersionNumber, payload::Pmt, EsInfo},
        es::StreamType,
    };

    TsPacket {
        header: default_ts_header(PMT_PID).unwrap(),
        adaptation_field: None,
        payload: Some(
            TsPayload::Pmt(Pmt {
                program_num: 1,
                pcr_pid: Some(Pid::new(VIDEO_ES_PID).unwrap()),
                version_number: VersionNumber::default(),
                table: vec![
                    EsInfo {
                        stream_type: StreamType::H264,
                        elementary_pid: Pid::new(VIDEO_ES_PID).unwrap(),
                        descriptors: vec![],
                    },
                    EsInfo {
                        stream_type: StreamType::AdtsAac,
                        elementary_pid: Pid::new(AUDIO_ES_PID).unwrap(),
                        descriptors: vec![],
                    }
                ]
            })),
    }
}
