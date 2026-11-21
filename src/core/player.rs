//! Media Player nativo usando FFmpeg bindings.

use crate::error::{RusdiError, Result};
use ffmpeg_next::{format, codec, media, frame, Packet, Rational};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct MediaPlayer {
    ctx: format::context::Input,
    video_stream: usize,
    audio_stream: usize,
    video_decoder: codec::decoder::Video,
    audio_decoder: codec::decoder::Audio,
    video_frame: frame::Video,
    audio_frame: frame::Audio,
    sws_ctx: Option<ffmpeg_next::software::scaling::Context>,
    swr_ctx: Option<ffmpeg_next::software::resampling::Context>,
    playing: Arc<Mutex<bool>>,
    paused: Arc<Mutex<bool>>,
}

impl MediaPlayer {
    pub fn open(path: &Path) -> Result<Self> {
        ffmpeg_next::init().ok();
        let mut ctx = format::context::Input::open(path)?;
        let video_stream = ctx.streams().best(media::Type::Video).ok_or(RusdiError::MediaPlayer("No video stream".into()))?.index();
        let audio_stream = ctx.streams().best(media::Type::Audio).map(|s| s.index());

        let video_decoder = codec::decoder::Video::open(codec::decoder::find(ctx.stream(video_stream).parameters().id())?)?;
        let audio_decoder = if let Some(a_idx) = audio_stream {
            Some(codec::decoder::Audio::open(codec::decoder::find(ctx.stream(a_idx).parameters().id())?)?)
        } else { None };

        Ok(Self {
            ctx, video_stream, audio_stream: audio_stream.unwrap_or(0),
            video_decoder, audio_decoder: audio_decoder.unwrap(),
            video_frame: frame::Video::empty(), audio_frame: frame::Audio::empty(),
            sws_ctx: None, swr_ctx: None,
            playing: Arc::new(Mutex::new(false)), paused: Arc::new(Mutex::new(false)),
        })
    }

    pub fn play(&self) -> Result<()> { *self.playing.lock().unwrap() = true; *self.paused.lock().unwrap() = false; Ok(()) }
    pub fn pause(&self) -> Result<()> { *self.paused.lock().unwrap() = true; Ok(()) }
    pub fn stop(&self) -> Result<()> { *self.playing.lock().unwrap() = false; Ok(()) }
    pub fn is_playing(&self) -> bool { *self.playing.lock().unwrap() && !*self.paused.lock().unwrap() }

    pub fn decode_video_frame(&mut self) -> Result<Option<frame::Video>> {
        while *self.playing.lock().unwrap() {
            if *self.paused.lock().unwrap() { std::thread::sleep(std::time::Duration::from_millis(10)); continue; }
            let mut packet = Packet::empty();
            if self.ctx.read(&mut packet).is_err() { return Ok(None); }
            if packet.stream() == self.video_stream {
                self.video_decoder.send_packet(&packet)?;
                if self.video_decoder.receive_frame(&mut self.video_frame).is_ok() {
                    if self.video_frame.format() != ffmpeg_next::format::Pixel::RGB24 {
                        self.ensure_sws_ctx()?;
                        let mut rgb_frame = frame::Video::empty();
                        unsafe { self.sws_ctx.as_mut().unwrap().run(&self.video_frame, &mut rgb_frame)?; }
                        return Ok(Some(rgb_frame));
                    }
                    return Ok(Some(self.video_frame.clone()));
                }
            }
        }
        Ok(None)
    }

    fn ensure_sws_ctx(&mut self) -> Result<()> {
        if self.sws_ctx.is_none() {
            let ctx = ffmpeg_next::software::scaling::Context::get(
                self.video_frame.format(), self.video_frame.width(), self.video_frame.height(),
                ffmpeg_next::format::Pixel::RGB24, self.video_frame.width(), self.video_frame.height(),
                ffmpeg_next::software::scaling::Flags::BILINEAR,
            )?;
            self.sws_ctx = Some(ctx);
        }
        Ok(())
    }

    pub fn get_duration(&self) -> i64 { self.ctx.duration() }
    pub fn get_position(&self) -> i64 { 0 }
    pub fn seek(&mut self, pos: i64) -> Result<()> {
        self.ctx.seek(pos, ..pos).map(|_| ()).map_err(|e| RusdiError::MediaPlayer(e.to_string()))
    }
}
