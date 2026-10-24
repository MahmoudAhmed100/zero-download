// Zero Download - Video overlay & page grabber
// Shows a draggable download pill on EVERY video on the page.
(() => {
  if (window.top !== window && !window.location.href.includes("youtube.com/embed")) return;

  const VIDEO_PAGE = [
    /youtube\.com\/(watch|shorts\/|live\/)/i,
    /tiktok\.com\/@[^/]+\/(video|photo)\//i,
    /instagram\.com\/(reels?|p|tv)\//i,
    /facebook\.com\/(.+\/videos\/|watch|reel\/|share\/[vr]\/)/i,
    /fb\.watch\//i,
    /(twitter|x)\.com\/[^/]+\/status\//i,
    /vimeo\.com\/\d+/i,
    /dailymotion\.com\/video\//i,
    /reddit\.com\/r\/[^/]+\/comments\//i,
    /twitch\.tv\/(videos\/|[^/]+\/clip\/)/i,
    /pinterest\.[a-z.]+\/pin\//i,
    /snapchat\.com\/(spotlight|add)\//i,
    /linkedin\.com\/(posts|feed\/update)\//i,
  ];

  const isVideoPage = (u) => VIDEO_PAGE.some((r) => r.test(u));

  function linkFor(video) {
    if (isVideoPage(location.href)) return location.href;
    let el = video;
    for (let depth = 0; el && depth < 25; depth++, el = el.parentElement) {
      const links = el.querySelectorAll ? el.querySelectorAll("a[href]") : [];
      for (const a of links) {
        if (isVideoPage(a.href)) return a.href.split("#")[0];
      }
      if (el.tagName === "ARTICLE") break;
    }
    return video.currentSrc || video.src || location.href;
  }

  const cleanTitle = () =>
    (document.title || "video").replace(/[<>:"/\\|?*\u0000-\u001f]/g, "_").trim().slice(0, 120) || "video";

  // Configuration
  let config = {
    enabled: true,
    showVideoPill: true,
    draggablePill: true,
    pillPosition: "top-right",
    pillSize: "medium",
    skipSites: []
  };

  chrome.storage.local.get(config, (res) => {
    if (res) Object.assign(config, res);
  });

  const activeOverlays = new Map(); // video -> overlay data
  let globalItems = [];

  // Drag offsets saved by user
  let userOffset = {
    x: Number(localStorage.getItem("zd_pill_ox")) || 0,
    y: Number(localStorage.getItem("zd_pill_oy")) || 0
  };

  function createOverlay(video) {
    if (activeOverlays.has(video)) return;
    if (video.offsetWidth < 120 || video.offsetHeight < 70) return;

    const host = document.createElement("div");
    host.style.position = "fixed";
    host.style.zIndex = "2147483647";
    host.style.all = "initial";
    host.style.display = "none";
    host.style.pointerEvents = "auto";

    const root = host.attachShadow({ mode: "closed" });
    root.innerHTML = `
      <style>
        :host { all: initial; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Cairo", sans-serif; }
        .pill-wrap { position: relative; user-select: none; }
        .pill {
          display: inline-flex;
          align-items: center;
          gap: 6px;
          background: linear-gradient(135deg, #1d4ed8, #2563eb);
          color: #ffffff;
          padding: 6px 10px;
          border-radius: 8px;
          font-size: 12px;
          font-weight: 600;
          box-shadow: 0 4px 14px rgba(0, 0, 0, 0.4), inset 0 1px 0 rgba(255, 255, 255, 0.2);
          cursor: pointer;
          border: 1px solid rgba(255, 255, 255, 0.15);
          transition: transform 0.1s, box-shadow 0.15s, background 0.15s;
        }
        .pill:hover {
          background: linear-gradient(135deg, #2563eb, #3b82f6);
          box-shadow: 0 6px 18px rgba(37, 99, 235, 0.5);
        }
        .drag-handle {
          cursor: grab;
          display: flex;
          align-items: center;
          opacity: 0.7;
          padding: 0 2px;
        }
        .drag-handle:active { cursor: grabbing; }
        .ico { width: 14px; height: 14px; fill: currentColor; flex-shrink: 0; }
        .btn-text { font-size: 11px; white-space: nowrap; }
        .badge {
          background: rgba(0,0,0,0.3);
          padding: 1px 5px;
          border-radius: 4px;
          font-size: 10px;
          font-weight: 700;
          color: #93c5fd;
        }
        .close-btn {
          margin-left: 2px;
          font-size: 11px;
          opacity: 0.6;
          padding: 0 3px;
          cursor: pointer;
        }
        .close-btn:hover { opacity: 1; color: #fca5a5; }
        /* Dropdown menu */
        .menu {
          position: absolute;
          top: 100%;
          right: 0;
          margin-top: 6px;
          background: #0f172a;
          border: 1px solid #334155;
          border-radius: 10px;
          box-shadow: 0 10px 25px rgba(0,0,0,0.5);
          width: 220px;
          overflow: hidden;
          z-index: 1000;
          display: none;
        }
        .menu.show { display: block; }
        .menu-item {
          padding: 8px 12px;
          display: flex;
          align-items: center;
          justify-content: space-between;
          border-bottom: 1px solid rgba(255,255,255,0.06);
          cursor: pointer;
          color: #f8fafc;
          font-size: 12px;
          transition: background 0.12s;
        }
        .menu-item:hover { background: #1e293b; color: #60a5fa; }
        .menu-item:last-child { border-bottom: none; }
        .msg { padding: 8px 12px; font-size: 11px; color: #10b981; text-align: center; }
      </style>
      <div class="pill-wrap">
        <div class="pill" id="pill">
          <span class="drag-handle" id="drag" title="اسحب الزر لأي مكان">⠿</span>
          <svg class="ico" viewBox="0 0 24 24"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 14l-4-4 1.41-1.41L11 13.17V7h2v6.17l2.59-2.58L17 12l-5 4z"/></svg>
          <span class="btn-text">تحميل</span>
          <span class="badge" id="quality">HD</span>
          <span class="close-btn" id="close" title="إخفاء الزر">✕</span>
        </div>
        <div class="menu" id="menu"></div>
      </div>
    `;

    document.documentElement.appendChild(host);

    const pill = root.getElementById("pill");
    const dragHandle = root.getElementById("drag");
    const closeBtn = root.getElementById("close");
    const menu = root.getElementById("menu");
    const qualityEl = root.getElementById("quality");

    let isMenuOpen = false;
    let isDismissed = false;

    // Draggable Logic
    let dragging = false;
    let startX = 0, startY = 0;
    let origOx = userOffset.x, origOy = userOffset.y;

    function onPointerDown(e) {
      if (e.target === closeBtn) return;
      if (e.target === dragHandle || config.draggablePill) {
        dragging = true;
        startX = e.clientX;
        startY = e.clientY;
        origOx = userOffset.x;
        origOy = userOffset.y;
        e.preventDefault();
        window.addEventListener("pointermove", onPointerMove);
        window.addEventListener("pointerup", onPointerUp);
      }
    }

    function onPointerMove(e) {
      if (!dragging) return;
      const dx = e.clientX - startX;
      const dy = e.clientY - startY;
      userOffset.x = origOx + dx;
      userOffset.y = origOy + dy;
      updatePosition();
    }

    function onPointerUp() {
      if (dragging) {
        dragging = false;
        localStorage.setItem("zd_pill_ox", userOffset.x);
        localStorage.setItem("zd_pill_oy", userOffset.y);
        window.removeEventListener("pointermove", onPointerMove);
        window.removeEventListener("pointerup", onPointerUp);
      }
    }

    pill.addEventListener("pointerdown", onPointerDown);

    closeBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      isDismissed = true;
      host.style.display = "none";
    });

    pill.addEventListener("click", (e) => {
      if (e.target === closeBtn || dragging) return;
      // If menu has items, toggle dropdown
      if (globalItems.length > 1) {
        isMenuOpen = !isMenuOpen;
        menu.classList.toggle("show", isMenuOpen);
        renderMenu();
      } else {
        triggerDownload(video);
      }
    });

    function renderMenu() {
      menu.innerHTML = "";
      if (!globalItems.length) {
        const item = document.createElement("div");
        item.className = "menu-item";
        item.textContent = "تحميل الفيديو الأصلي";
        item.addEventListener("click", () => {
          triggerDownload(video);
          menu.classList.remove("show");
        });
        menu.appendChild(item);
        return;
      }

      globalItems.forEach((m) => {
        const row = document.createElement("div");
        row.className = "menu-item";
        row.innerHTML = `<span>${m.label || "فيديو"}</span><small style="opacity:0.6">${m.ext ? m.ext.toUpperCase() : ""}</small>`;
        row.addEventListener("click", () => {
          chrome.runtime.sendMessage({
            type: "swiftget-download",
            item: m,
            title: cleanTitle(),
            fileName: `${cleanTitle()}.${m.ext || "mp4"}`
          });
          flashMsg("تم الإرسال لـ Zero Download ✔");
          menu.classList.remove("show");
        });
        menu.appendChild(row);
      });
    }

    function flashMsg(msg) {
      menu.innerHTML = `<div class="msg">${msg}</div>`;
      menu.classList.add("show");
      setTimeout(() => menu.classList.remove("show"), 2000);
    }

    function updatePosition() {
      if (isDismissed || !config.showVideoPill) {
        host.style.display = "none";
        return;
      }

      const rect = video.getBoundingClientRect();
      // Only show if video is visible on screen
      if (rect.width < 120 || rect.height < 70 || rect.bottom <= 0 || rect.top >= window.innerHeight || rect.right <= 0 || rect.left >= window.innerWidth) {
        host.style.display = "none";
        return;
      }

      host.style.display = "block";

      let baseX = rect.right - 140;
      let baseY = rect.top + 10;

      if (config.pillPosition === "top-left") {
        baseX = rect.left + 10;
        baseY = rect.top + 10;
      } else if (config.pillPosition === "bottom-right") {
        baseX = rect.right - 140;
        baseY = rect.bottom - 44;
      } else if (config.pillPosition === "bottom-left") {
        baseX = rect.left + 10;
        baseY = rect.bottom - 44;
      }

      const finalX = Math.max(10, Math.min(window.innerWidth - 150, baseX + userOffset.x));
      const finalY = Math.max(10, Math.min(window.innerHeight - 50, baseY + userOffset.y));

      host.style.left = `${finalX}px`;
      host.style.top = `${finalY}px`;
    }

    activeOverlays.set(video, { host, updatePosition, setQuality: (q) => qualityEl.textContent = q });
    updatePosition();
  }

  function triggerDownload(video) {
    const pageUrl = linkFor(video);
    const title = cleanTitle();
    chrome.runtime.sendMessage({
      type: "swiftget-download-page",
      pageUrl,
      title,
      currentSrc: video.currentSrc || video.src
    }, (res) => {
      // Notification
      const data = activeOverlays.get(video);
      if (data) {
        const m = data.host.shadowRoot?.getElementById("menu");
        if (m) {
          m.innerHTML = `<div style="padding:10px; color:#10b981; font-size:12px; font-weight:600; text-align:center;">تم إرسال التحميل لـ Zero Download ✔</div>`;
          m.classList.add("show");
          setTimeout(() => m.classList.remove("show"), 2200);
        }
      }
    });
  }

  function scanVideos() {
    const videos = document.querySelectorAll("video");
    videos.forEach((v) => {
      if (!activeOverlays.has(v)) {
        createOverlay(v);
      }
    });

    // Cleanup detached videos
    for (const [v, data] of activeOverlays.entries()) {
      if (!document.contains(v)) {
        data.host.remove();
        activeOverlays.delete(v);
      } else {
        data.updatePosition();
      }
    }
  }

  // Periodic position updates & scroll listener
  window.addEventListener("scroll", () => {
    activeOverlays.forEach((d) => d.updatePosition());
  }, { passive: true });

  window.addEventListener("resize", () => {
    activeOverlays.forEach((d) => d.updatePosition());
  }, { passive: true });

  setInterval(scanVideos, 1200);

  // MutationObserver for dynamic infinite scrolls
  const observer = new MutationObserver(() => scanVideos());
  observer.observe(document.documentElement, { childList: true, subtree: true });

  // Handle messages from background worker
  chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
    if (msg.type === "swiftget-media") {
      globalItems = msg.items || [];
      if (globalItems.length > 0) {
        const topQuality = globalItems[0].quality || (globalItems[0].ext ? globalItems[0].ext.toUpperCase() : "HD");
        activeOverlays.forEach((d) => d.setQuality(topQuality));
      }
    }
    if (msg.type === "swiftget-collect-links") {
      sendResponse(collectLinks(!!msg.selection));
      return true;
    }
    return false;
  });

  // Collect links function for site grabber
  function collectLinks(selectionOnly) {
    const sel = selectionOnly ? getSelection() : null;
    const inSel = (el) => !sel || (sel.rangeCount > 0 && sel.containsNode(el, true));
    const out = [];
    const seen = new Set();
    const add = (url, text) => {
      if (!url || !/^https?:/i.test(url)) return;
      url = url.split("#")[0];
      if (seen.has(url)) return;
      seen.add(url);
      out.push({ url, text: (text || "").replace(/\s+/g, " ").trim().slice(0, 200) });
    };
    for (const a of document.querySelectorAll("a[href]")) if (inSel(a)) add(a.href, a.innerText || a.title);
    for (const el of document.querySelectorAll("img[src], video[src], audio[src], source[src], embed[src]")) {
      if (inSel(el)) add(el.src, el.alt || el.title || "");
    }
    return { page: location.href, links: out };
  }

  // Initial scan
  setTimeout(scanVideos, 500);
})();
