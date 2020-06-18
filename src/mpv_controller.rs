use anyhow::{Result, Context};
use mpv::{MpvHandler, MpvHandlerBuilder};

struct MpvController<'a> {
    playback_speed: Vec<f64>,
    mpv_handler: MpvHandler,
    video_source: &'a str,
}
impl<'a> MpvController<'a> {
    /// Initialize MPV controller. This doesn't actually start
    /// the video, but does start MPV.
    fn new(mpv_video_source: &'a str) -> Result<MpvController> {
        let mpv_builder: MpvHandlerBuilder = MpvHandlerBuilder::new().with_context(||"Failed creating MPV handler. Check for libmpv availability. This might also indicate OOM situation or LC_NUMERIC not being set to C.")?;
        // Enable on-screen-controller, which is disabled by default when using libmpv.
        mpv_builder.set_option("osc", true).with_context(||"Failed enabling MPV on screen controller.")?;
        let mut mpv = mpv_builder.build().with_context(||"Failed to create MPV window.")?;
        Ok(
            MpvController {
                playback_speed: Vec::new(),
                mpv_handler: mpv,
                video_source: mpv_video_source,
            }
        )
    }
    /// Try to push playback speed of next frame.
    /// 
    /// MpvController remembers at which speed to play the target frame.
    /// All data don't have to be present at startup in order to actually
    /// play the video. When we don't yet have the required playback speed data,
    /// we will play the video at default speed.
    /// 
    /// This is used to push new data to mpvcontroller. Those data HAVE to be
    /// in correct order. 
    fn push_playback_speed_data(&mut self, playback_speed: f64) {
        self.playback_speed.push(playback_speed);
    }
    /// Start playing the video. This is a blocking code,
    /// it might be worth starting this in standalone thread.
    /// 
    /// This method returns when the video stopped playing for whatever reason
    /// (in which case this returns Ok), or we encountered something that we
    /// couldn't handle, in which case this returns Err. Playback should terminate
    /// prematurely in that case.
    /// 
    /// TODO: Should we return at the first erorr? Shouldn't we just return iterator of
    /// errors and just keep trying to survive as long as possible?
    fn start_playing(&mut self) -> Result<()> {
        // First of all, load the video into MPV so it starts playing.
        self.mpv_handler.command(&["loadfile", self.video_source])
            .with_context(||"Failed to tell MPV about target video source.")?;

        'main: loop {
            while let Some(event) = self.mpv_handler.wait_event(0.0) {
                // even if you don't do anything with the events, it is still necessary to empty
                // the event loop
                println!("RECEIVED EVENT @ {}: {:?}", self.mpv_handler.get_property::<i64>("stream-pos").unwrap_or(-1), event);
                match event {
                    // Shutdown will be triggered when the window is explicitely closed,
                    // while Idle will be triggered when the queue will end
                    mpv::Event::Shutdown | mpv::Event::Idle => {
                        break 'main;
                    }
                    _ => {}
                };
            }
        }
        
        Ok(())
    }
}