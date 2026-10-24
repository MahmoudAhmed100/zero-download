// Standalone options page script for Zero Download extension
const DEFAULTS = {
  port: 9614,
  intercept: true,
  minSizeKB: 0,
  showVideoPill: true,
  draggablePill: true,
  pillPosition: "top-right",
  pillSize: "medium",
  preferOriginalAudio: true,
  fallbackAudioLang: "en"
};

async function loadOptions() {
  const opts = Object.assign({}, DEFAULTS, await chrome.storage.local.get(DEFAULTS));
  document.getElementById("showVideoPill").checked = !!opts.showVideoPill;
  document.getElementById("draggablePill").checked = !!opts.draggablePill;
  document.getElementById("pillPosition").value = opts.pillPosition || "top-right";
  document.getElementById("pillSize").value = opts.pillSize || "medium";
  document.getElementById("preferOriginalAudio").checked = !!opts.preferOriginalAudio;
  document.getElementById("fallbackAudioLang").value = opts.fallbackAudioLang || "en";
  document.getElementById("intercept").checked = !!opts.intercept;
  document.getElementById("minSizeKB").value = opts.minSizeKB || 0;
  document.getElementById("port").value = opts.port || 9614;

  checkStatus();
}

function checkStatus() {
  chrome.runtime.sendMessage({ type: "swiftget-status" }, (res) => {
    const badge = document.getElementById("appStatus");
    const text = document.getElementById("appStatusText");
    const ok = res && res.ok;
    if (ok) {
      badge.style.background = "rgba(16, 185, 129, 0.15)";
      badge.style.color = "#10b981";
      badge.style.borderColor = "rgba(16, 185, 129, 0.3)";
      text.textContent = "متصل بالبرنامج ✔";
    } else {
      badge.style.background = "rgba(239, 68, 68, 0.15)";
      badge.style.color = "#ef4444";
      badge.style.borderColor = "rgba(239, 68, 68, 0.3)";
      text.textContent = "البرنامج غير متصل";
    }
  });
}

async function saveOptions() {
  const newOpts = {
    showVideoPill: document.getElementById("showVideoPill").checked,
    draggablePill: document.getElementById("draggablePill").checked,
    pillPosition: document.getElementById("pillPosition").value,
    pillSize: document.getElementById("pillSize").value,
    preferOriginalAudio: document.getElementById("preferOriginalAudio").checked,
    fallbackAudioLang: document.getElementById("fallbackAudioLang").value,
    intercept: document.getElementById("intercept").checked,
    minSizeKB: Math.max(0, Number(document.getElementById("minSizeKB").value) || 0),
    port: Number(document.getElementById("port").value) || 9614
  };

  await chrome.storage.local.set(newOpts);

  // Show toast
  const toast = document.getElementById("toast");
  toast.classList.add("show");
  setTimeout(() => toast.classList.remove("show"), 2500);
}

document.addEventListener("DOMContentLoaded", () => {
  loadOptions();
  document.getElementById("saveBtn").addEventListener("click", saveOptions);
});
