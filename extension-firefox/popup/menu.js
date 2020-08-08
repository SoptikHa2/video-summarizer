enableButton = document.getElementById('activateButton');
numLoud = document.getElementById('numLoud');
numSilent = document.getElementById('numSilent');
rangeLoud = document.getElementById('rangeLoud');
rangeSilent = document.getElementById('rangeSilent');
let suppress = false;

var RATE_LOUD = 1.5;
var RATE_SILENT = 4;
var disabled = false;
// Try to load non-default settings
function loadSettings() {
    let settings_rloud = browser.storage.local.get("RATE_LOUD");
    settings_rloud.then((a) => { RATE_LOUD = a.RATE_LOUD; numLoud.value = a.RATE_LOUD; rangeLoud.value = a.RATE_LOUD; }, () => {});
    let settings_rsilent = browser.storage.local.get("RATE_SILENT");
    settings_rsilent.then((a) => { RATE_SILENT = a.RATE_SILENT; numSilent.value = a.RATE_SILENT; rangeSilent.value = a.RATE_SILENT; }, () => {});
    let settings_disabled = browser.storage.local.get("DISABLED");
    settings_disabled.then((a) => { disabled = a.DISABLED; enableButton.innerText = a.DISABLED ? 'Enable' : 'Disable'; }, () => {});
}
loadSettings();


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
function enableToggle() {
    let nextState = !disabled;
    let nextText = disabled ? 'Disable' : 'Enable';
    browser.storage.local.set({
        DISABLED: nextState
    });
    enableButton.innerText = nextText;
    disabled = nextState;
}

numLoud.addEventListener("input", numLoudChanged);
rangeLoud.addEventListener("input", rangeLoudChanged);
numSilent.addEventListener("input", numSilentChanged);
rangeSilent.addEventListener("input", rangeSilentChanged);
enableButton.addEventListener("click", enableToggle);
