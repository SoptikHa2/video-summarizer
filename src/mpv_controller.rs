use anyhow::Result;

struct MpvController {
    playback_speed: Vec<f64>
}
impl MpvController {
    /// Initialize MPV controller. This doesn't actually start
    /// the video.
    fn new(mpv_video_source: &str) -> MpvPlayback {}
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
    /// This consumes MpvController.
    /// 
    /// TODO: Should we consume MpvController? Is there use case where &mut self could
    /// make this more useful to end user?
    /// 
    /// TODO: Should we return at the first erorr? Shouldn't we just return iterator of
    /// errors and just keep trying to survive as long as possible?
    fn start_playing(self) -> Result<()> {

    }
}