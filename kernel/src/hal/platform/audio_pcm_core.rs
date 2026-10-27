//! Kernel Audio PCM Stream Buffer Core (SP-94, REQ-HW-111, REQ-AUD-102)
//!
//! DMA circular audio ring-buffer and real-time audio playback HAL.

pub const KERNEL_PCM_SAMPLE_RATE_48K: u32 = 48_000;
pub const KERNEL_PCM_CHANNELS_STEREO: u8 = 2;
pub const KERNEL_PCM_PERIODS: usize = 4;
pub const KERNEL_PCM_PERIOD_FRAMES: usize = 192; // 4ms @ 48kHz
pub const KERNEL_CAP_AUDIO_PLAYBACK: u32 = 0x0000_0008;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAudioPcmError {
    PermissionDenied,
    InvalidConfiguration,
    BufferOverrun,
    BufferUnderrun,
    HardwareNotReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAudioPcmState {
    Idle,
    Configured,
    Running,
    XRun,
}

pub struct KernelAudioPcmHarness {
    state: KernelAudioPcmState,
    sample_rate: u32,
    channels: u8,
    buffered_frames: usize,
    xrun_count: usize,
    latency_ms: u32,
}

impl KernelAudioPcmHarness {
    pub const fn new() -> Self {
        Self {
            state: KernelAudioPcmState::Idle,
            sample_rate: 0,
            channels: 0,
            buffered_frames: 0,
            xrun_count: 0,
            latency_ms: 0,
        }
    }

    pub fn configure(&mut self, cap_token: u32, sample_rate: u32, channels: u8) -> Result<(), KernelAudioPcmError> {
        if (cap_token & KERNEL_CAP_AUDIO_PLAYBACK) == 0 {
            return Err(KernelAudioPcmError::PermissionDenied);
        }
        if sample_rate != KERNEL_PCM_SAMPLE_RATE_48K || channels != KERNEL_PCM_CHANNELS_STEREO {
            return Err(KernelAudioPcmError::InvalidConfiguration);
        }
        self.sample_rate = sample_rate;
        self.channels = channels;
        self.state = KernelAudioPcmState::Configured;
        self.latency_ms = ((KERNEL_PCM_PERIODS * KERNEL_PCM_PERIOD_FRAMES * 1000) / sample_rate as usize) as u32;
        Ok(())
    }

    pub fn write_frames(&mut self, frames: usize) -> Result<usize, KernelAudioPcmError> {
        if self.state != KernelAudioPcmState::Configured && self.state != KernelAudioPcmState::Running && self.state != KernelAudioPcmState::XRun {
            return Err(KernelAudioPcmError::HardwareNotReady);
        }
        let max_capacity = KERNEL_PCM_PERIODS * KERNEL_PCM_PERIOD_FRAMES;
        if self.buffered_frames + frames > max_capacity {
            return Err(KernelAudioPcmError::BufferOverrun);
        }
        self.buffered_frames += frames;
        self.state = KernelAudioPcmState::Running;
        Ok(self.buffered_frames)
    }

    pub fn handle_underrun(&mut self) -> usize {
        self.xrun_count += 1;
        self.state = KernelAudioPcmState::XRun;
        // Inject zero-padding period to prevent pop/click and hardware stall
        self.buffered_frames = KERNEL_PCM_PERIOD_FRAMES;
        self.xrun_count
    }

    pub fn is_configured(&self) -> bool {
        self.state != KernelAudioPcmState::Idle
    }

    pub fn is_running(&self) -> bool {
        self.state == KernelAudioPcmState::Running
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

    pub fn state(&self) -> KernelAudioPcmState {
        self.state
    }
}
