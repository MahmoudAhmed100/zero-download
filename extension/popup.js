// Zero Download Popup Script
const DEFAULTS = { port: 9614, intercept: true, minSizeKB: 0 };

async function init() {
  const opts = Object.assign({}, DEFAULTS, await chrome.storage.local.get(DEFAULTS));
  const quickIntercept = document.getElementById("quickIntercept");
  quickIntercept.checked = !!opts.intercept;
  quickIntercept.addEventListener("change", () => {
    chrome.storage.local.set({ intercept: quickIntercept.checked });
  });

  const openOptions = () => {
    if (chrome.runtime.openOptionsPage) {
      chrome.runtime.openOptionsPage();
    } else {
      window.open(chrome.runtime.getURL("options.html"));
    }
  };

  document.getElementById("openOptionsBtn").addEventListener("click", openOptions);
  document.getElementById("footerOptionsLink").addEventListener("click", openOptions);

  // Status check with app
  chrome.runtime.sendMessage({ type: "swiftget-status" }, (res) => {
    const pill = document.getElementById("statusPill");
    const text = document.getElementById("statusText");
    const ok = res && res.ok;
    if (ok) {
      pill.style.background = "rgba(16, 185, 129, 0.15)";
      pill.style.color = "#10b981";
      pill.style.borderColor = "rgba(16, 185, 129, 0.3)";
      text.textContent = "متصل بالبرنامج ✔";
    } else {
      pill.style.background = "rgba(239, 68, 68, 0.15)";
      pill.style.color = "#ef4444";
      pill.style.borderColor = "rgba(239, 68, 68, 0.3)";
      text.textContent = "التطبيق غير متصل";
    }
  });

  // Query media for the active tab
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab) return;

  chrome.runtime.sendMessage({ type: "swiftget-get-media", tabId: tab.id }, (items) => {
    const listEl = document.getElementById("mediaList");
    const countEl = document.getElementById("mediaCount");

    if (!items || !items.length) {
      countEl.textContent = "0 عناصر";
      return;
    }

    countEl.textContent = `${items.length} عنصر`;
    listEl.innerHTML = "";

    const cleanTabTitle = (tab.title || "video").replace(/[<>:"/\\|?*\u0000-\u001f]/g, "_").slice(0, 120);

    items.forEach((m) => {
      const card = document.createElement("div");
      card.className = "media-card";

      const icon = document.createElement("div");
      icon.className = "media-icon";
      const isAudio = m.ext === "mp3" || m.ext === "m4a" || (m.sub && m.sub.includes("Audio"));
      icon.innerHTML = isAudio ? "🎵" : "🎬";

      const info = document.createElement("div");
      info.className = "media-info";

      const title = document.createElement("div");
      title.className = "media-title";
      title.textContent = m.label || cleanTabTitle;

      const meta = document.createElement("div");
      meta.className = "media-meta";

      if (m.quality) {
        const qBadge = document.createElement("span");
        qBadge.className = "quality-badge";
        qBadge.textContent = m.quality;
        meta.appendChild(qBadge);
      }

      if (m.ext) {
        const extBadge = document.createElement("span");
        extBadge.className = "quality-badge";
        extBadge.textContent = m.ext.toUpperCase();
        meta.appendChild(extBadge);
      }

      if (m.sub) {
        const subSpan = document.createElement("span");
        subSpan.textContent = m.sub;
        meta.appendChild(subSpan);
      }

      info.append(title, meta);

      const btn = document.createElement("button");
      btn.className = "btn-download";
      btn.innerHTML = "<span>تحميل</span> ⬇";
      btn.disabled = !!m.disabled;

      btn.addEventListener("click", () => {
        btn.textContent = "جاري الإرسال...";
        btn.disabled = true;
        chrome.runtime.sendMessage(
          {
            type: "swiftget-download",
            item: m,
            title: tab.title,
            referrer: tab.url,
            fileName: `${cleanTabTitle}.${m.ext || "mp4"}`
          },
          (res) => {
            if (res && res.ok) {
              btn.textContent = "تم الإرسال ✔";
              btn.style.background = "#10b981";
            } else {
              btn.textContent = "فشل الإرسال";
              btn.style.background = "#ef4444";
              btn.disabled = false;
            }
          }
        );
      });

      card.append(icon, info, btn);
      listEl.appendChild(card);
    });
  });
}

document.addEventListener("DOMContentLoaded", init);
