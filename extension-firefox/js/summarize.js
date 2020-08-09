var video_data = null;
var vid = null;
// This remembers the effective url of the page. Url of page can change
// without triggering this extension code reload! So we need to do this
// ourselves.
var effective_url = null;
// If permiturl is not set to null (Which happens when there is video on this page
// but it is not yet indexed), the popup will know that it can present index button
// to the user
var permiturl = null;

var RATE_LOUD = 1.5;
var RATE_SILENT = 4;
var disabled = false;
// Try to load non-default settings
function loadSettings() {
    let settings_rloud = browser.storage.local.get("RATE_LOUD");
    settings_rloud.then((a) => RATE_LOUD = a.RATE_LOUD ?? RATE_LOUD, () => {});
    let settings_rsilent = browser.storage.local.get("RATE_SILENT");
    settings_rsilent.then((a) => RATE_SILENT = a.RATE_SILENT ?? RATE_SILENT, () => {});
    let settings_disabled = browser.storage.local.get("DISABLED");
    settings_disabled.then((a) => disabled = a.DISABLED ?? false, () => {});
}
loadSettings();

function change_video_rate() {
    loadSettings();
    if(disabled) return;

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

        vid.playbackRate = next_rate;
    }

    window.requestAnimationFrame(change_video_rate);
}

// Check if HTML video exists on this page. If so,
// take the first one and check if there exists server record for this.
// If it does, start handling speed rate change.
// If it does not, remember so, so if user opens popup, an index button appears.
// The index button can be used to send video URL to server which then indexes it.
async function setup() {
    if(disabled) return;
    videos = document.getElementsByTagName("video")
    effective_url = document.location.toString();
    url = document.location.toString().replace(/[#].*/g, "");
    // Remove all query params except for ?v= (on youtube)
    url = url.replace(/(\?[^v&]+=[^&]+)/, ""); // Remove all ?non-v
    url = url.replace(/(&[^v&]+=[^&\n]+)/g, ""); // Remove all &non-v
    url = url.replace(/&v/, "?v"); // Transform &v to ?v
    hash = await sha1(url);
    if (videos.length > 0) {
        vid = videos[0]

        // Try to load settings from server
        var request = new XMLHttpRequest();
        request.open('GET', 'https://videosummarizer.soptik.tech/' + hash, true);
    
        request.onload = function() {
          if (this.status >= 200 && this.status < 400) {
            // Success!
            // No longer permit indexation
            permiturl = null;
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
            permiturl = url
            window.requestAnimationFrame(change_video_rate);
          }
        };
    
        request.onerror = function() {
            console.log('Couldn\'t connect to server.');
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

function handleMessage(request, sender, sendResponse) {
    if(request.type == "can_we_index") {
        sendResponse({url: permiturl});
    } else if (request.type == "restart_setup") {
        setup();
    } else {
        console.log("Unknown request type: " + request.type);
    }
}

browser.runtime.onMessage.addListener(handleMessage);
