var video_data = null;
var vid = null;
var effective_url = null;

var RATE_LOUD = 1.5;
var RATE_SILENT = 4;
var enabled = true;
// Try to load non-default settings
{
    let settings_rloud = browser.storage.sync.get("RATE_LOUD");
    settings_rloud.then((a) => RATE_LOUD = a, () => {});
    let settings_rsilent = browser.storage.sync.get("RATE_SILENT");
    settings_rsilent.then((a) => RATE_SILENT = a, () => {});
    let settings_enabled = browser.storage.sync.get("ENABLED");
    settings_enabled.then((a) => enabled = a, () => {});
}

function change_video_rate() {
    if(!enabled) return;

    if (effective_url != document.location.toString()) {
        video_data = null;
        setup();
        return;
    }

    // We need to run this change_video_rate() even if we cannot alter video rate -
    // as above, we are listening to URL changes to be able to switch video without
    // page reload
    if(video_data != null) {
        next_rate = RATE_LOUD;
        let seconds = vid.currentTime;
        for (idx in video_data) {
            let data = video_data[idx];
            if (data[0] >= seconds) break;
            next_rate = data[1] ? RATE_LOUD : RATE_SILENT;
        }

        console.log(next_rate);
        vid.playbackRate = next_rate;
    }

    window.requestAnimationFrame(change_video_rate);
}

async function setup() {
    if(!enabled) return;
    videos = document.getElementsByTagName("video")
    effective_url = document.location.toString();
    url = document.location.toString().replace(/[#].*/g, "");
    // Remove all query params except for ?v= (on youtube)
    url = url.replace(/(\?[^v&]+=[^&]+)/, ""); // Remove all ?non-v
    url = url.replace(/(&[^v&]+=[^&\n]+)/g, ""); // Remove all &non-v
    url = url.replace(/\/&v/, "/?v"); // Transform /&v to /?v
    hash = await sha1(url);
    console.log('Url: ' + url);
    console.log('Will be testing ' + hash);

    console.log('it works, found videos length: ' + videos.length)
    if (videos.length > 0) {
        vid = videos[0]
        console.log('found video')
        console.log(vid)

        // Try to load settings from server
        var request = new XMLHttpRequest();
        request.open('GET', 'https://videosummarizer.soptik.tech/' + hash, true);
    
        request.onload = function() {
          if (this.status >= 200 && this.status < 400) {
            // Success!
            var data = this.response;
            var dataLines = data.split('\n');
            video_data = []
            for (idx in dataLines) {
                line = dataLines[idx].split(' ');
                video_data.push([parseFloat(line[0]), line[1] == '1']);
            }
            window.requestAnimationFrame(change_video_rate);
          } else {
            // We reached our target server, but it returned an error
            console.log('It looks like the target isnt cached yet. So far, users don\'t have an option to submit a video for caching. Sorry!');
            window.requestAnimationFrame(change_video_rate);
          }
        };
    
        request.onerror = function() {
            console.log('Couldn\'t connect to server.');
            console.log(this);
          // There was a connection error of some sort
        };
    
        request.send();
    }
}


async function sha1( str ) {
  const buffer = new TextEncoder( 'utf-8' ).encode( str );
  const digest = await crypto.subtle.digest('SHA-1', buffer);

  // Convert digest to hex string
  const result = Array.from(new Uint8Array(digest)).map( x => x.toString(16).padStart(2,'0') ).join('');

    return result;
}

setup();
