use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, OutputCallbackInfo, SampleFormat, SizedSample, Stream, StreamConfig, StreamError,
    SupportedStreamConfig,
};
use std::sync::Arc;

use crate::component::audio::AudioComponent;

// TODO: Audio basically does nothing right now

pub struct CpalContext {
    device: Device,
    stream: Stream,
}

impl CpalContext {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();

        let config = device
            .supported_output_configs()
            .unwrap()
            // We will work with i16 samples in this here app
            .find(|config| config.sample_format() == cpal::SampleFormat::I16)
            .map(|config| {
                SupportedStreamConfig::new(
                    config.channels(),
                    config.max_sample_rate(),
                    *config.buffer_size(),
                    config.sample_format(),
                )
            })
            // If we can't find an ideal format try the default one
            .or_else(|| device.default_output_config().ok())
            .expect("Unable to select a audio output format");

        let sample_format = config.sample_format();
        let output_config: StreamConfig = config.into();

        let stream = match sample_format {
            SampleFormat::I8 => device
                .build_output_stream(
                    &output_config,
                    audio_callback::<i8>(output_config.clone()),
                    audio_error,
                    None,
                )
                .unwrap(),
            SampleFormat::I16 => device
                .build_output_stream(
                    &output_config,
                    audio_callback::<i16>(output_config.clone()),
                    audio_error,
                    None,
                )
                .unwrap(),
            SampleFormat::I32 => device
                .build_output_stream(
                    &output_config,
                    audio_callback::<i32>(output_config.clone()),
                    audio_error,
                    None,
                )
                .unwrap(),
            SampleFormat::I64 => device
                .build_output_stream(
                    &output_config,
                    audio_callback::<i64>(output_config.clone()),
                    audio_error,
                    None,
                )
                .unwrap(),
            SampleFormat::U8 => device
                .build_output_stream(
                    &output_config,
                    audio_callback::<u8>(output_config.clone()),
                    audio_error,
                    None,
                )
                .unwrap(),
            SampleFormat::U16 => device
                .build_output_stream(
                    &output_config,
                    audio_callback::<u16>(output_config.clone()),
                    audio_error,
                    None,
                )
                .unwrap(),
            SampleFormat::U32 => device
                .build_output_stream(
                    &output_config,
                    audio_callback::<u32>(output_config.clone()),
                    audio_error,
                    None,
                )
                .unwrap(),
            SampleFormat::U64 => device
                .build_output_stream(
                    &output_config,
                    audio_callback::<u64>(output_config.clone()),
                    audio_error,
                    None,
                )
                .unwrap(),
            SampleFormat::F32 => device
                .build_output_stream(
                    &output_config,
                    audio_callback::<f32>(output_config.clone()),
                    audio_error,
                    None,
                )
                .unwrap(),
            SampleFormat::F64 => device
                .build_output_stream(
                    &output_config,
                    audio_callback::<f64>(output_config.clone()),
                    audio_error,
                    None,
                )
                .unwrap(),
            _ => panic!("Unsupported sample format"),
        };

        Self { device, stream }
    }

    pub fn startup_stream(&mut self, audio_components: Vec<Arc<dyn AudioComponent>>) {}

    pub fn terminate_stream(&mut self) {}
}

pub fn audio_callback<S: SizedSample>(
    output_config: StreamConfig,
) -> impl FnMut(&mut [S], &OutputCallbackInfo) {
    move |output, _| {
        for channel_buffer in output.chunks_mut(output_config.channels as usize) {}
    }
}

pub fn audio_error(error: StreamError) {}
