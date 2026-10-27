//! Userspace DevMgr Audio PCM Stream Buffer Driver (SP-94, REQ-HW-111, REQ-AUD-102)
//!
//! DMA circular audio ring-buffer and real-time audio playback HAL.

pub const PCM_SAMPLE_RATE_48K: u32 = 48_000;
pub const PCM_CHANNELS_STEREO: u8 = 2;
pub const PCM_PERIODS: usize = 4;
pub const PCM_PERIOD_FRAMES: usize = 192; // 4ms @ 48kHz
pub const CAP_AUDIO_PLAYBACK: u32 = 0x0000_0008;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPcmError {
    PermissionDenied,
    InvalidConfiguration,
    BufferOverrun,
    BufferUnderrun,
    HardwareNotReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPcmState {
    Idle,
    Configured,
    Running,
    XRun,
}

pub struct AudioPcmHarness {
    state: AudioPcmState,
    sample_rate: u32,
    channels: u8,
    buffered_frames: usize,
    xrun_count: usize,
    latency_ms: u32,
}

impl AudioPcmHarness {
    pub const fn new() -> Self {
        Self {
            state: AudioPcmState::Idle,
            sample_rate: 0,
            channels: 0,
            buffered_frames: 0,
            xrun_count: 0,
            latency_ms: 0,
        }
    }

    pub fn configure(&mut self, cap_token: u32, sample_rate: u32, channels: u8) -> Result<(), AudioPcmError> {
        if (cap_token & CAP_AUDIO_PLAYBACK) == 0 {
            return Err(AudioPcmError::PermissionDenied);
        }
        if sample_rate != PCM_SAMPLE_RATE_48K || channels != PCM_CHANNELS_STEREO {
            return Err(AudioPcmError::InvalidConfiguration);
        }
        self.sample_rate = sample_rate;
        self.channels = channels;
        self.state = AudioPcmState::Configured;
        self.latency_ms = ((PCM_PERIODS * PCM_PERIOD_FRAMES * 1000) / sample_rate as usize) as u32;
        Ok(())
    }

    pub fn write_frames(&mut self, frames: usize) -> Result<usize, AudioPcmError> {
        if self.state != AudioPcmState::Configured && self.state != AudioPcmState::Running && self.state != AudioPcmState::XRun {
            return Err(AudioPcmError::HardwareNotReady);
        }
        let max_capacity = PCM_PERIODS * PCM_PERIOD_FRAMES;
        if self.buffered_frames + frames > max_capacity {
            return Err(AudioPcmError::BufferOverrun);
        }
        self.buffered_frames += frames;
        self.state = AudioPcmState::Running;
        Ok(self.buffered_frames)
    }

    pub fn handle_underrun(&mut self) -> usize {
        self.xrun_count += 1;
        self.state = AudioPcmState::XRun;
        self.buffered_frames = PCM_PERIOD_FRAMES;
        self.xrun_count
    }

    pub fn is_configured(&self) -> bool {
        self.state != AudioPcmState::Idle
    }

    pub fn is_running(&self) -> bool {
        self.state == AudioPcmState::Running
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u8 {
        self.channels
    }

    pub fn xrun_count(&self) -> usize {
        self.xrun_count
    }

    pub fn latency_ms(&self) -> u32 {
        self.latency_ms
    }

    pub fn state(&self) -> AudioPcmState {
        self.state
    }
}
