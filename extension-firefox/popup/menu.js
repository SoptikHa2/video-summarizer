enableButton = document.getElementById('activateButton');
indexDiv = document.getElementById('indexDiv');
indexButton = document.getElementById('indexButton');
numLoud = document.getElementById('numLoud');
numSilent = document.getElementById('numSilent');
rangeLoud = document.getElementById('rangeLoud');
rangeSilent = document.getElementById('rangeSilent');
let suppress = false;

var RATE_LOUD = 1.5;
var RATE_SILENT = 4;
var disabled = false;
var indexurl = null;
// Try to load non-default settings
function loadSettings() {
    let settings_rloud = browser.storage.local.get("RATE_LOUD");
    settings_rloud.then((a) => { RATE_LOUD = a.RATE_LOUD ?? RATE_LOUD; numLoud.value = a.RATE_LOUD ?? RATE_LOUD; rangeLoud.value = a.RATE_LOUD ?? RATE_LOUD; }, () => {});
    let settings_rsilent = browser.storage.local.get("RATE_SILENT");
    settings_rsilent.then((a) => { RATE_SILENT = a.RATE_SILENT ?? RATE_SILENT; numSilent.value = a.RATE_SILENT ?? RATE_SILENT; rangeSilent.value = a.RATE_SILENT ?? RATE_SILENT; }, () => {});
    let settings_disabled = browser.storage.local.get("DISABLED");
    settings_disabled.then((a) => { disabled = a.DISABLED ?? false; enableButton.innerText = (a.DISABLED ?? false) ? 'Enable' : 'Disable'; }, () => {});
}
loadSettings();

// When user changes slider or number, update the other one and save it
function numLoudChanged() {
    if (suppress) return;
    suppress = true;
    rangeLoud.value = numLoud.value
    browser.storage.local.set({
        RATE_LOUD: numLoud.value
    });
    suppress = false;
}
function rangeLoudChanged() {
    if (suppress) return;
    suppress = true;
    numLoud.value = rangeLoud.value
    browser.storage.local.set({
        RATE_LOUD: numLoud.value
    });
    suppress = false;
}
function numSilentChanged() {
    if (suppress) return;
    suppress = true;
    rangeSilent.value = numSilent.value
    browser.storage.local.set({
        RATE_SILENT: numSilent.value
    });
    suppress = false;
}
function rangeSilentChanged() {
    if (suppress) return;
    suppress = true;
    numSilent.value = rangeSilent.value
    browser.storage.local.set({
        RATE_SILENT: numSilent.value
    });
    suppress = false;
}

// Handle extension enable togglebutton
function enableToggle() {
    let nextState = !disabled;
    let nextText = disabled ? '(Reload page to take effect)' : 'Enable';
    browser.storage.local.set({
        DISABLED: nextState
    });
    enableButton.innerText = nextText;
    disabled = nextState;
}

// Ask content script of currently active tab
// if we can allow user to index this tab, and if so,
// which URL should be used.
function checkIfWeCanIndex(tabs) {
    for (let tab of tabs) {
        console.log(tab);
        console.log('sending');
        var message = browser.tabs.sendMessage(
            tab.id,
        {
            type: "can_we_index"
        });
        message.then((m) => {
            console.log('received response');
            indexurl = m.url;
            console.log(indexurl);
            if (indexurl != null) {
                console.log(indexurl);
                indexDiv.style = "";
                indexButton.style = "";
            }
        }, console.error);
    }
}

// Send request to server to index current video
function indexCurrentPage() {
    if (indexurl != null) {
        var request = new XMLHttpRequest();
        request.open('POST', 'https://videosummarizer.soptik.tech', true);
        request.send(indexurl);
        indexButton.innerText = "indexing...";
        indexButton.disabled = true;
        request.onload = function() {
            if (this.status >= 200 && this.status < 400) {
                indexButton.innerText = "Indexed! Reload page";
            } else {
                indexButton.innerText = "An error occured.";
                indexButton.disabled = false;
            }
        }
        request.onerror = function() {
            indexButton.innerText = "Couldn\'t connect to server.";
            indexButton.disabled = false;
        }
    }
}



numLoud.addEventListener("input", numLoudChanged);
rangeLoud.addEventListener("input", rangeLoudChanged);
numSilent.addEventListener("input", numSilentChanged);
rangeSilent.addEventListener("input", rangeSilentChanged);
enableButton.addEventListener("click", enableToggle);
indexButton.addEventListener("click", indexCurrentPage);

browser.tabs.query({currentWindow: true, active: true}).then(checkIfWeCanIndex);
