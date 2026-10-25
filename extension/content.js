// Zero Download - Video overlay & page grabber
// Shows a draggable download pill on video players on the page.
(() => {
  if (window.top !== window && !window.location.href.includes("youtube.com/embed")) return;

  // Suppress harmless extension disconnection / lifecycle errors in the page
  window.addEventListener("unhandledrejection", (event) => {
    const msg = (event && event.reason && (event.reason.message || String(event.reason))) || "";
    if (msg.includes("No SW") || msg.includes("Receiving end does not exist") || msg.includes("Extension context invalidated")) {
      if (typeof event.preventDefault === "function") event.preventDefault();
    }
  });

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

  // Clean up legacy viewport-level offsets that caused pills to snap to (10px, 10px)
  try {
    localStorage.removeItem("zd_pill_ox");
    localStorage.removeItem("zd_pill_oy");
  } catch (_) {}

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

  const ZOOM = { small: 0.85, normal: 1, large: 1.25 };
  let zoom = 1;
  const applyZoom = () => {
    for (const host of activeOverlays.values()) {
      if (host && host.style) host.style.setProperty("--z", String(zoom));
    }
  };
  try {
    if (typeof chrome !== "undefined" && chrome.storage && chrome.storage.local) {
      chrome.storage.local.get({ bubbleSize: "normal" }, (o) => {
        zoom = ZOOM[o && o.bubbleSize] || 1;
        applyZoom();
      });
      chrome.storage.onChanged.addListener((ch) => {
        if (ch && ch.bubbleSize) {
          zoom = ZOOM[ch.bubbleSize.newValue] || 1;
          applyZoom();
        }
      });
    }
  } catch (_) {}
  let globalItems = [];

  function isValidVideo(video) {
    if (!video || !video.isConnected) return false;

    // Filter out thumbnail hover previews and mini sidebars
    if (video.closest("ytd-thumbnail, ytd-moving-thumbnail-renderer, .ytd-thumbnail, [class*='thumbnail'], [class*='preview']")) {
      return false;
    }

    // Must have real dimensions
    const rect = video.getBoundingClientRect();
    if (rect.width < 240 || rect.height < 140) return false;

    // Must be visible
    if (video.offsetParent === null && video.style.position !== "fixed") return false;
    const cs = window.getComputedStyle(video);
    if (cs.display === "none" || cs.visibility === "hidden" || cs.opacity === "0") return false;

    // On YouTube specifically: must be part of the player or shorts
    if (location.hostname.includes("youtube.com")) {
      const isPlayer = video.classList.contains("video-stream") ||
                       video.closest("#movie_player, ytd-player, ytd-reel-video-renderer, #player");
      if (!isPlayer) return false;
    }

    return true;
  }

  function findVideoContainer(video) {
    // 1. YouTube desktop & embed players
    const yt = video.closest("#movie_player, .html5-video-player, ytd-player, #player-container, ytd-reel-video-renderer");
    if (yt) return yt;

    // 2. Standard video containers on modern web
    const player = video.closest(".video-js, [class*='player-container'], [class*='video-player'], [class*='Player_'], figure, .player");
    if (player) return player;

    // 3. Fallback to immediate parent
    return video.parentElement || document.body;
  }

  function createOverlay(video) {
    if (activeOverlays.has(video)) return;
    if (!isValidVideo(video)) return;

    const container = findVideoContainer(video);
    if (!container) return;

    const cs = window.getComputedStyle(container);
    if (cs.position === "static") {
      container.style.position = "relative";
    }

    const host = document.createElement("div");
    host.style.position = "absolute";
    host.style.zIndex = "2147483647";
    host.style.all = "initial";
    host.style.display = "block";
    host.style.pointerEvents = "auto";
    host.style.top = "14px";
    host.style.right = "14px";
    host.style.left = "auto";
    host.style.bottom = "auto";
    host.style.setProperty("--z", String(zoom));

    const root = host.attachShadow({ mode: "closed" });
    root.innerHTML = `
      <style>
        :host { all: initial; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Cairo", sans-serif; }
        .pill-wrap { position: relative; user-select: none; zoom: var(--z, 1); }
        .pill {
          display: inline-flex;
          align-items: center;
          gap: 6px;
          background: linear-gradient(135deg, #1d4ed8, #2563eb);
          color: #ffffff;
          padding: 6px 11px;
          border-radius: 8px;
          font-size: 12px;
          font-weight: 600;
          box-shadow: 0 4px 14px rgba(0, 0, 0, 0.45), inset 0 1px 0 rgba(255, 255, 255, 0.25);
          cursor: pointer;
          border: 1px solid rgba(255, 255, 255, 0.2);
          transition: transform 0.1s, box-shadow 0.15s, background 0.15s;
          pointer-events: auto;
        }
        .pill:hover {
          background: linear-gradient(135deg, #2563eb, #3b82f6);
          box-shadow: 0 6px 18px rgba(37, 99, 235, 0.55);
        }
        .drag-handle {
          cursor: grab;
          display: flex;
          align-items: center;
          opacity: 0.8;
          padding: 2px 4px;
          touch-action: none;
        }
        .drag-handle:hover { opacity: 1; }
        .drag-handle:active { cursor: grabbing; }
        .ico { width: 14px; height: 14px; fill: currentColor; flex-shrink: 0; }
        .btn-text { font-size: 11px; white-space: nowrap; }
        .badge {
          background: rgba(0,0,0,0.35);
          padding: 1px 6px;
          border-radius: 4px;
          font-size: 10px;
          font-weight: 700;
          color: #93c5fd;
        }
        .close-btn {
          margin-left: 2px;
          font-size: 11px;
          opacity: 0.6;
          padding: 0 4px;
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
          box-shadow: 0 10px 25px rgba(0,0,0,0.6);
          width: 230px;
          overflow: hidden;
          z-index: 2147483647;
          display: none;
        }
        .menu.show { display: block; }
        .menu-item {
          padding: 9px 12px;
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
        .msg { padding: 9px 12px; font-size: 11px; color: #10b981; text-align: center; font-weight: 600; }
      </style>
      <div class="pill-wrap">
        <div class="pill" id="pill">
          <span class="drag-handle" id="drag" title="اسحب الزر لأي مكان على الفيديو">⠿</span>
          <svg class="ico" viewBox="0 0 24 24"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 14l-4-4 1.41-1.41L11 13.17V7h2v6.17l2.59-2.58L17 12l-5 4z"/></svg>
          <span class="btn-text">تحميل</span>
          <span class="badge" id="quality">HD</span>
          <span class="close-btn" id="close" title="إخفاء الزر">✕</span>
        </div>
        <div class="menu" id="menu"></div>
      </div>
    `;

    container.appendChild(host);

    const pill = root.getElementById("pill");
    const dragHandle = root.getElementById("drag");
    const closeBtn = root.getElementById("close");
    const menu = root.getElementById("menu");
    const qualityEl = root.getElementById("quality");

    let isMenuOpen = false;
    let isDismissed = false;

    // Smooth Dragging with Pointer Capture
    let isDragging = false;
    let hasMoved = false;
    let startX = 0, startY = 0;
    let initialLeft = 0, initialTop = 0;

    function onPointerDown(e) {
      if (e.target === closeBtn) return;
      e.preventDefault();
      e.stopPropagation();
      e.stopImmediatePropagation();

      isDragging = true;
      hasMoved = false;
      startX = e.clientX;
      startY = e.clientY;

      try { dragHandle.setPointerCapture(e.pointerId); } catch (_) {}

      const hostRect = host.getBoundingClientRect();
      const contRect = container.getBoundingClientRect();
      initialLeft = hostRect.left - contRect.left;
      initialTop = hostRect.top - contRect.top;

      dragHandle.addEventListener("pointermove", onPointerMove);
      dragHandle.addEventListener("pointerup", onPointerUp);
      dragHandle.addEventListener("pointercancel", onPointerUp);
    }

    function onPointerMove(e) {
      if (!isDragging) return;
      e.preventDefault();
      e.stopPropagation();
      e.stopImmediatePropagation();

      const dx = e.clientX - startX;
      const dy = e.clientY - startY;
      if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
        hasMoved = true;
      }

      const contRect = container.getBoundingClientRect();
      const hostW = host.offsetWidth || 140;
      const hostH = host.offsetHeight || 36;

      const minX = 6;
      const maxX = Math.max(6, contRect.width - hostW - 6);
      const minY = 6;
      const maxY = Math.max(6, contRect.height - hostH - 6);

      const newX = Math.max(minX, Math.min(maxX, initialLeft + dx));
      const newY = Math.max(minY, Math.min(maxY, initialTop + dy));

      host.style.left = `${newX}px`;
      host.style.top = `${newY}px`;
      host.style.right = "auto";
      host.style.bottom = "auto";
    }

    function onPointerUp(e) {
      if (isDragging) {
        isDragging = false;
        try { dragHandle.releasePointerCapture(e.pointerId); } catch (_) {}
        dragHandle.removeEventListener("pointermove", onPointerMove);
        dragHandle.removeEventListener("pointerup", onPointerUp);
        dragHandle.removeEventListener("pointercancel", onPointerUp);
      }
    }

    dragHandle.addEventListener("pointerdown", onPointerDown);

    closeBtn.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      e.stopImmediatePropagation();
      isDismissed = true;
      host.style.display = "none";
    });

    pill.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      e.stopImmediatePropagation();

      if (e.target === closeBtn || hasMoved) {
        hasMoved = false;
        return;
      }

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
        item.addEventListener("click", (e) => {
          e.stopPropagation();
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
        row.addEventListener("click", (e) => {
          e.stopPropagation();
          try {
            chrome.runtime.sendMessage(
              {
                type: "swiftget-download",
                item: m,
                title: cleanTitle(),
                fileName: `${cleanTitle()}.${m.ext || "mp4"}`
              },
              () => {
                if (chrome.runtime.lastError) {}
              }
            );
          } catch (_) {}
          flashMsg("تم الإرسال لـ Zero Download ✔");
          menu.classList.remove("show");
        });
        menu.appendChild(row);
      });
    }

    function flashMsg(msg) {
      menu.innerHTML = `<div class="msg">${msg}</div>`;
      menu.classList.add("show");
      setTimeout(() => menu.classList.remove("show"), 2200);
    }

    activeOverlays.set(video, {
      host,
      container,
      setQuality: (q) => { if (qualityEl) qualityEl.textContent = q; }
    });
  }

  function triggerDownload(video) {
    const pageUrl = linkFor(video);
    const title = cleanTitle();
    try {
      chrome.runtime.sendMessage(
        {
          type: "swiftget-download-page",
          pageUrl,
          title,
          currentSrc: video.currentSrc || video.src
        },
        () => {
          if (chrome.runtime.lastError) {}
          const data = activeOverlays.get(video);
          if (data) {
            const m = data.host.shadowRoot?.getElementById("menu");
            if (m) {
              m.innerHTML = `<div style="padding:10px; color:#10b981; font-size:12px; font-weight:600; text-align:center;">تم إرسال التحميل لـ Zero Download ✔</div>`;
              m.classList.add("show");
              setTimeout(() => m.classList.remove("show"), 2200);
            }
          }
        }
      );
    } catch (_) {}
  }

  function scanVideos() {
    if (!config.showVideoPill || !config.enabled) {
      activeOverlays.forEach((d) => { d.host.style.display = "none"; });
      return;
    }

    const videos = document.querySelectorAll("video");
    videos.forEach((v) => {
      if (isValidVideo(v) && !activeOverlays.has(v)) {
        createOverlay(v);
      }
    });

    // Cleanup detached or invalid videos
    for (const [v, data] of activeOverlays.entries()) {
      if (!document.contains(v) || !isValidVideo(v)) {
        data.host.remove();
        activeOverlays.delete(v);
      }
    }
  }

  // Periodic video scanner
  setInterval(scanVideos, 1500);

  // MutationObserver for infinite scrolls & SPA navigation
  const observer = new MutationObserver(() => scanVideos());
  observer.observe(document.documentElement, { childList: true, subtree: true });

  // YouTube navigation event
  window.addEventListener("yt-navigate-finish", () => setTimeout(scanVideos, 400));

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
  setTimeout(scanVideos, 600);
})();
