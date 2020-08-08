var video_data = [];
var vid = null;

const RATE_LOUD = 1.5;
const RATE_SILENT = 4;

function change_video_rate() {
    next_rate = RATE_LOUD;
    let seconds = vid.currentTime;
    for (idx in video_data) {
        let data = video_data[idx];
        if (data[0] >= seconds) break;
        next_rate = data[1] ? RATE_LOUD : RATE_SILENT;
    }

    console.log(next_rate);
    vid.playbackRate = next_rate;

    window.requestAnimationFrame(change_video_rate);
}

async function setup() {
    videos = document.getElementsByTagName("video")
    url = document.location.toString().replace(/[#].*/g, "");
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
            for (idx in dataLines) {
                line = dataLines[idx].split(' ');
                video_data.push([parseFloat(line[0]), line[1] == '1']);
            }
            window.requestAnimationFrame(change_video_rate);
          } else {
            // We reached our target server, but it returned an error
            console.log('It looks like the target isnt cached yet. So far, users don\'t have an option to submit a video for caching. Sorry!');
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
