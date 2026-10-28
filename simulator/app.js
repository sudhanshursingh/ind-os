/* ==========================================================================
   Ind OS — Virtual Device Simulator Controller
   Platform: Motorola Moto G60 (Qualcomm Snapdragon 732G / SM7150)
   ========================================================================== */

(function() {
  'use strict';

  // --- State Variables ---
  let stages = window.BOOT_STAGES || [];
  let currentStageIndex = 0;
  let isBooting = true;
  let isPaused = false;
  let bootSpeedMs = 28; // Milliseconds per stage step for smooth animation
  let bootTimer = null;
  let currentFilter = 'all';
  let is120Hz = true;
  let currentLanguage = 'en';

  // --- DOM Elements ---
  const elProgressBar = document.getElementById('boot-progress-bar');
  const elProgressPercent = document.getElementById('boot-progress-percent');
  const elCurrentStageTag = document.getElementById('current-stage-tag');
  const elCurrentStageDesc = document.getElementById('current-stage-desc');
  const elSerialLines = document.getElementById('serial-lines');
  const elSerialTerminal = document.getElementById('serial-terminal');
  const elLogCount = document.getElementById('log-count-indicator');
  const elChkAutoscroll = document.getElementById('chk-autoscroll');

  const layerBootloader = document.getElementById('layer-bootloader');
  const layerBootAnim = document.getElementById('layer-boot-animation');
  const layerG9 = document.getElementById('layer-g9-celebration');
  const layerShell = document.getElementById('layer-shell');
  const animCanvas = document.getElementById('boot-anim-canvas');
  const animHudBadge = document.getElementById('anim-stage-badge');

  const elStatusClock = document.getElementById('status-clock');
  const elAmbientDate = document.getElementById('ambient-date');
  const elDeviceModal = document.getElementById('device-modal');
  const elModalTitle = document.getElementById('device-modal-title');
  const elModalBody = document.getElementById('device-modal-body');

  const elTelemTemp = document.getElementById('telem-temp');
  const elTelemRefresh = document.getElementById('telem-refresh');
  const elSystemStatus = document.getElementById('system-status-indicator');

  // Multi-Language Strings
  const I18N = {
    en: {
      card1Title: "Optimize Refresh Rate",
      card1Desc: "Panel is currently at 120Hz. Switch to 60Hz Eco Mode to extend 6000mAh battery life by 3.2 hours?",
      card1Action: "Apply 60Hz",
      dateStr: "Tuesday, Oct 27",
      intentPlaceholder: "What would you like Ind OS to do?",
    },
    hi: {
      card1Title: "रिफ्रेश दर अनुकूलित करें",
      card1Desc: "स्क्रीन वर्तमान में 120Hz पर है। 6000mAh बैटरी 3.2 घंटे बढ़ाने के लिए 60Hz इको मोड लागू करें?",
      card1Action: "60Hz लागू करें",
      dateStr: "मंगलवार, 27 अक्टूबर",
      intentPlaceholder: "आप Ind OS से क्या करवाना चाहते हैं?",
    },
    ta: {
      card1Title: "புதுப்பிப்பு வீதத்தை உகப்பாக்கு",
      card1Desc: "திரை தற்போது 120Hz இல் உள்ளது. 6000mAh பேட்டரியை 3.2 மணிநேரம் நீட்டிக்க 60Hz சூழல் பயன்முறைக்கு மாறவா?",
      card1Action: "60Hz மாற்றவும்",
      dateStr: "செவ்வாய்க்கிழமை, அக்டோபர் 27",
      intentPlaceholder: "Ind OS என்ன செய்ய விரும்புகிறீர்கள்?",
    }
  };

  // --- Clock Sync ---
  function updateClock() {
    const now = new Date();
    const hours = String(now.getHours()).padStart(2, '0');
    const mins = String(now.getMinutes()).padStart(2, '0');
    if (elStatusClock) elStatusClock.textContent = `${hours}:${mins}`;
  }
  setInterval(updateClock, 1000);
  updateClock();

  // --- Subsystem Badges Map ---
  function updateBadges(tag) {
    if (tag.includes('GIC') || tag.includes('73')) document.getElementById('badge-gic')?.classList.add('active');
    if (tag.includes('UFS') || tag.includes('74')) document.getElementById('badge-ufs')?.classList.add('active');
    if (tag.includes('DISPLAY') || tag.includes('76')) document.getElementById('badge-dsi')?.classList.add('active');
    if (tag.includes('TOUCH') || tag.includes('78')) document.getElementById('badge-touch')?.classList.add('active');
    if (tag.includes('PMIC') || tag.includes('80')) document.getElementById('badge-pmic')?.classList.add('active');
    if (tag.includes('BATTERY') || tag.includes('81')) document.getElementById('badge-batt')?.classList.add('active');
    if (tag.includes('WIFI') || tag.includes('87')) document.getElementById('badge-wifi')?.classList.add('active');
    if (tag.includes('AUDIO') || tag.includes('93')) document.getElementById('badge-audio')?.classList.add('active');
    if (tag.includes('AIS') || tag.includes('96')) document.getElementById('badge-ais')?.classList.add('active');
    if (tag.includes('GATE_G9') || tag.includes('499')) document.getElementById('badge-g9')?.classList.add('active');
  }

  // --- Serial Logging Output ---
  function appendSerialLog(text, type) {
    if (!elSerialLines) return;
    const lineEl = document.createElement('div');
    lineEl.className = `log-line ${type}`;
    lineEl.dataset.type = type;
    lineEl.textContent = text;
    elSerialLines.appendChild(lineEl);

    if (currentFilter !== 'all' && currentFilter !== type) {
      lineEl.style.display = 'none';
    }

    if (elChkAutoscroll && elChkAutoscroll.checked && elSerialTerminal) {
      elSerialTerminal.scrollTop = elSerialTerminal.scrollHeight;
    }
  }

  // --- Bootloader Step Function ---
  function stepBoot() {
    if (!isBooting || isPaused) return;

    if (currentStageIndex >= stages.length) {
      finishBoot();
      return;
    }

    const st = stages[currentStageIndex];
    const total = stages.length;
    const progress = Math.min(100, Math.round(((currentStageIndex + 1) / total) * 100));

    // Update Progress UI
    if (elProgressBar) elProgressBar.style.width = `${progress}%`;
    if (elProgressPercent) elProgressPercent.textContent = `${progress}%`;
    if (elCurrentStageTag) elCurrentStageTag.textContent = `STAGE: ${st.tag}`;
    if (elCurrentStageDesc) elCurrentStageDesc.textContent = st.desc || 'Executing hardware assertion...';

    // Update Subsystem Badges
    updateBadges(st.tag);

    // Stream Serial Logs
    appendSerialLog(`[STAGE:${st.tag}] ${st.desc}`, 'stage');
    if (st.logs && st.logs.length) {
      for (const logLine of st.logs) {
        if (!logLine.startsWith('[STAGE:')) {
          appendSerialLog(logLine, logLine.includes('[INFO ]') ? 'info' : 'milestone');
        }
      }
    }

    if (elLogCount) {
      elLogCount.textContent = `Executed Stage ${currentStageIndex + 1} of ${total}`;
    }

    // Jitter simulated temperature slightly
    if (currentStageIndex % 35 === 0 && elTelemTemp) {
      const temp = (36.2 + Math.random() * 2.5).toFixed(1);
      elTelemTemp.textContent = `${temp}°C`;
    }

    currentStageIndex++;
    bootTimer = setTimeout(stepBoot, bootSpeedMs);
  }

  function finishBoot() {
    isBooting = false;
    if (elProgressBar) elProgressBar.style.width = '100%';
    if (elProgressPercent) elProgressPercent.textContent = '100%';
    if (elSystemStatus) elSystemStatus.textContent = 'Gate G10 Ratified • Defensible & Beta Ready';

    appendSerialLog("==================================================", 'milestone');
    appendSerialLog(" 🏆 GATE G10 HARD-STOP AUDIT PASSED & RATIFIED 🏆", 'milestone');
    appendSerialLog(" Milestone: HARDENING, VERIFICATION & BETA READINESS", 'milestone');
    appendSerialLog(" Verified Proofs in CI • 0 Criticals • Red-Team Immune", 'info');
    appendSerialLog("==================================================", 'milestone');

    setTimeout(() => {
      layerBootloader.classList.add('hidden');
      layerG9.classList.remove('hidden');
    }, 600);
  }

  function startBoot() {
    clearTimeout(bootTimer);
    currentStageIndex = 0;
    isBooting = true;
    isPaused = false;
    document.querySelectorAll('.hw-badge').forEach(b => b.classList.remove('active'));
    layerBootloader.classList.remove('hidden');
    layerG9.classList.add('hidden');
    layerShell.classList.add('hidden');
    closeDeviceModal();

    if (elSerialLines) elSerialLines.innerHTML = '';
    appendSerialLog("==================================================", 'milestone');
    appendSerialLog(" Starting Headless IndK QEMU Session (Qualcomm SM7150)", 'info');
    appendSerialLog(" Hardware: Motorola Moto G60 (Kryo 470, 6GB LPDDR4x, UFS 2.1)", 'info');
    appendSerialLog("==================================================", 'milestone');

    stepBoot();
  }

  // --- Hardware Key Event Listeners ---
  document.getElementById('hw-vol-up')?.addEventListener('click', () => {
    appendSerialLog("[BUTTON] Volume Up pressed (+5% DAC gain)", 'info');
  });

  document.getElementById('hw-vol-down')?.addEventListener('click', () => {
    appendSerialLog("[BUTTON] Volume Down pressed (-5% DAC gain)", 'info');
  });

  document.getElementById('hw-power')?.addEventListener('click', () => {
    appendSerialLog("[BUTTON] Power key pressed (Fingerprint sensor scanned OK)", 'info');
  });

  // --- Control Bar Listeners ---
  document.getElementById('btn-cold-boot')?.addEventListener('click', () => {
    startBoot();
  });

  document.getElementById('btn-fast-forward')?.addEventListener('click', () => {
    if (isBooting) {
      bootSpeedMs = 0; // instantly zip through remaining stages
    } else {
      layerG9.classList.add('hidden');
      layerShell.classList.remove('hidden');
    }
  });

  document.getElementById('btn-pause-boot')?.addEventListener('click', () => {
    isPaused = !isPaused;
    const pauseIcon = document.getElementById('pause-icon');
    if (pauseIcon) pauseIcon.textContent = isPaused ? '▶' : '⏸';
    if (!isPaused && isBooting) {
      stepBoot();
    }
  });

  document.getElementById('btn-enter-shell')?.addEventListener('click', () => {
    layerG9.classList.add('hidden');
    playBootAnimation(() => {
      layerShell.classList.remove('hidden');
    });
  });

  document.getElementById('btn-play-boot-anim')?.addEventListener('click', () => {
    appendSerialLog("[ANIMATION] Playing 120Hz Cold Boot Animation Sequence (2.8s)", 'milestone');
    playBootAnimation(() => {
      appendSerialLog("[ANIMATION] Cold Boot Animation finished: entered Ind Surface Shell", 'info');
    });
  });

  // --- Cold Boot Animation Canvas Engine (120Hz/60Hz Double-Buffered) ---
  let animFrameId = null;

  function playBootAnimation(onComplete) {
    if (!animCanvas) {
      if (typeof onComplete === 'function') onComplete();
      return;
    }
    clearTimeout(bootTimer);
    isPaused = true;
    if (animFrameId) cancelAnimationFrame(animFrameId);

    // Switch to Animation Layer
    layerBootloader?.classList.add('hidden');
    layerG9?.classList.add('hidden');
    layerShell?.classList.add('hidden');
    layerBootAnim?.classList.remove('hidden');

    const ctx = animCanvas.getContext('2d');
    const dpr = window.devicePixelRatio || 1;
    const w = 346;
    const h = 736;
    animCanvas.width = w * dpr;
    animCanvas.height = h * dpr;
    ctx.scale(dpr, dpr);

    const startTime = performance.now();
    const DURATION = 2800; // 2.8s total

    function renderFrame(now) {
      const elapsed = Math.min(now - startTime, DURATION);
      const progress = elapsed / DURATION;

      // Clear dark background (#0A0D12)
      ctx.fillStyle = '#0A0D12';
      ctx.fillRect(0, 0, w, h);

      const cx = w / 2;
      const cy = h / 2 - 20;

      if (elapsed < 800) {
        // ==========================================
        // STAGE 1: SILICON IGNITION (0.0s -> 0.8s)
        // ==========================================
        const p1 = elapsed / 800;
        if (animHudBadge) animHudBadge.textContent = `STAGE 1: SILICON IGNITION • ${(elapsed / 1000).toFixed(1)}s`;

        // Central Microchip Square
        const chipSize = 70;
        ctx.strokeStyle = '#00F0FF';
        ctx.lineWidth = 2;
        ctx.shadowColor = '#00F0FF';
        ctx.shadowBlur = 12 * p1;
        ctx.strokeRect(cx - chipSize / 2, cy - chipSize / 2, chipSize, chipSize);

        // Core Silicon Die
        ctx.fillStyle = 'rgba(30, 41, 59, 0.8)';
        ctx.fillRect(cx - 20, cy - 20, 40, 40);
        ctx.fillStyle = '#00F0FF';
        ctx.fillRect(cx - 10, cy - 10, 20, 20);

        // Radiating PCB Circuit Traces
        const maxTrace = (w / 2) * p1;
        ctx.beginPath();
        ctx.moveTo(cx + chipSize / 2, cy - 12);
        ctx.lineTo(cx + chipSize / 2 + maxTrace, cy - 12);
        ctx.moveTo(cx - chipSize / 2, cy + 12);
        ctx.lineTo(cx - chipSize / 2 - maxTrace, cy + 12);
        ctx.moveTo(cx - 15, cy - chipSize / 2);
        ctx.lineTo(cx - 15, cy - chipSize / 2 - maxTrace * 0.8);
        ctx.moveTo(cx + 15, cy + chipSize / 2);
        ctx.lineTo(cx + 15, cy + chipSize / 2 + maxTrace * 0.8);
        ctx.stroke();

        // Traveling Light Data Pulses
        const pulseOffset = maxTrace * 0.7;
        if (pulseOffset > 10) {
          ctx.fillStyle = '#FFFFFF';
          ctx.beginPath();
          ctx.arc(cx + chipSize / 2 + pulseOffset, cy - 12, 3, 0, Math.PI * 2);
          ctx.arc(cx - chipSize / 2 - pulseOffset, cy + 12, 3, 0, Math.PI * 2);
          ctx.fill();
        }

        // Subtitle
        ctx.shadowBlur = 0;
        ctx.fillStyle = '#00F0FF';
        ctx.font = '600 11px Inter, sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText('AARCH64-INDK SILICON IGNITION', cx, cy + 85);

      } else if (elapsed < 1800) {
        // ==========================================
        // STAGE 2: MASCOT ASSEMBLY (0.8s -> 1.8s)
        // ==========================================
        const p2 = (elapsed - 800) / 1000;
        if (animHudBadge) animHudBadge.textContent = `STAGE 2: MASCOT ASSEMBLY • ${(elapsed / 1000).toFixed(1)}s`;

        ctx.save();
        ctx.translate(cx, cy);

        // Glowing Line-Art of Gaja the Elephant
        ctx.strokeStyle = '#00F0FF';
        ctx.lineWidth = 2.5;
        ctx.shadowColor = '#00F0FF';
        ctx.shadowBlur = 16 * p2;

        // Brow & Crown Arches
        ctx.beginPath();
        ctx.arc(0, -35, 42, Math.PI * 0.9, Math.PI * 2.1);
        ctx.stroke();

        // Sweeping Curved Trunk
        ctx.beginPath();
        ctx.moveTo(-10, -5);
        ctx.bezierCurveTo(-15, 30, -30 * p2, 50 * p2, -22 * p2, 60 * p2);
        ctx.stroke();

        // Mandala Ear Geometry (Gold #FFB300)
        ctx.strokeStyle = '#FFB300';
        ctx.shadowColor = '#FFB300';
        ctx.shadowBlur = 14 * p2;

        // Left Mandala Ear
        ctx.beginPath();
        ctx.ellipse(-52, -18, 30 * p2, 45 * p2, -0.2, 0, Math.PI * 2);
        ctx.stroke();
        ctx.beginPath();
        ctx.ellipse(-52, -18, 16 * p2, 25 * p2, -0.2, 0, Math.PI * 2);
        ctx.stroke();

        // Right Mandala Ear
        ctx.beginPath();
        ctx.ellipse(52, -18, 30 * p2, 45 * p2, 0.2, 0, Math.PI * 2);
        ctx.stroke();
        ctx.beginPath();
        ctx.ellipse(52, -18, 16 * p2, 25 * p2, 0.2, 0, Math.PI * 2);
        ctx.stroke();

        // Forward Tusks
        ctx.strokeStyle = '#FFFFFF';
        ctx.lineWidth = 3;
        ctx.shadowColor = '#FFFFFF';
        ctx.shadowBlur = 8;
        ctx.beginPath();
        ctx.moveTo(-18, 10);
        ctx.lineTo(-24, 28);
        ctx.moveTo(18, 10);
        ctx.lineTo(24, 28);
        ctx.stroke();

        // Forehead Node (Ajna Chakra / Intelligence Node)
        ctx.fillStyle = '#FFB300';
        ctx.beginPath();
        ctx.arc(0, -18, 6, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = '#FFFFFF';
        ctx.beginPath();
        ctx.arc(0, -18, 2.5, 0, Math.PI * 2);
        ctx.fill();

        ctx.restore();

        // Subtitle
        ctx.shadowBlur = 0;
        ctx.fillStyle = '#FFB300';
        ctx.font = '600 11px Inter, sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText('GAJA MASCOT VECTOR ASSEMBLY', cx, cy + 95);

      } else if (elapsed < 2500) {
        // ==========================================
        // STAGE 3: THE SOVEREIGNTY PULSE (1.8s -> 2.5s)
        // ==========================================
        const p3 = (elapsed - 1800) / 700;
        if (animHudBadge) animHudBadge.textContent = `STAGE 3: SOVEREIGNTY PULSE • ${(elapsed / 1000).toFixed(1)}s`;

        // Draw Full Gaja Mascot
        ctx.save();
        ctx.translate(cx, cy);

        ctx.strokeStyle = '#00F0FF';
        ctx.lineWidth = 2.5;
        ctx.shadowColor = '#00F0FF';
        ctx.shadowBlur = 16;
        ctx.beginPath();
        ctx.arc(0, -35, 42, Math.PI * 0.9, Math.PI * 2.1);
        ctx.stroke();

        ctx.beginPath();
        ctx.moveTo(-10, -5);
        ctx.bezierCurveTo(-15, 30, -30, 50, -22, 60);
        ctx.stroke();

        ctx.strokeStyle = '#FFB300';
        ctx.shadowColor = '#FFB300';
        ctx.shadowBlur = 14;
        ctx.beginPath();
        ctx.ellipse(-52, -18, 30, 45, -0.2, 0, Math.PI * 2);
        ctx.stroke();
        ctx.beginPath();
        ctx.ellipse(-52, -18, 16, 25, -0.2, 0, Math.PI * 2);
        ctx.stroke();

        ctx.beginPath();
        ctx.ellipse(52, -18, 30, 45, 0.2, 0, Math.PI * 2);
        ctx.stroke();
        ctx.beginPath();
        ctx.ellipse(52, -18, 16, 25, 0.2, 0, Math.PI * 2);
        ctx.stroke();

        ctx.strokeStyle = '#FFFFFF';
        ctx.lineWidth = 3;
        ctx.beginPath();
        ctx.moveTo(-18, 10);
        ctx.lineTo(-24, 28);
        ctx.moveTo(18, 10);
        ctx.lineTo(24, 28);
        ctx.stroke();

        ctx.fillStyle = '#FFB300';
        ctx.beginPath();
        ctx.arc(0, -18, 6, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = '#FFFFFF';
        ctx.beginPath();
        ctx.arc(0, -18, 2.5, 0, Math.PI * 2);
        ctx.fill();

        ctx.restore();

        // Expanding Energy Ring Shockwave
        const pulseR = p3 * 180;
        ctx.strokeStyle = `rgba(0, 240, 255, ${1 - p3})`;
        ctx.lineWidth = 3;
        ctx.shadowColor = '#00F0FF';
        ctx.shadowBlur = 24 * (1 - p3);
        ctx.beginPath();
        ctx.arc(cx, cy - 18, pulseR, 0, Math.PI * 2);
        ctx.stroke();

        // Brand Typography
        ctx.shadowBlur = 12 * p3;
        ctx.shadowColor = 'rgba(255, 255, 255, 0.6)';
        ctx.fillStyle = '#FFFFFF';
        ctx.font = '800 24px Inter, sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText('Ind OS', cx, cy + 105);

        ctx.shadowBlur = 0;
        ctx.fillStyle = '#00F0FF';
        ctx.font = '600 9px Inter, monospace';
        ctx.letterSpacing = '2px';
        ctx.fillText('FAST • SECURE • PRIVATE • INTELLIGENT', cx, cy + 128);

      } else {
        // ==========================================
        // STAGE 4: TRANSITION TO SHELL (2.5s -> 2.8s)
        // ==========================================
        const p4 = (elapsed - 2500) / 300;
        if (animHudBadge) animHudBadge.textContent = `STAGE 4: SHELL TRANSITION • ${(elapsed / 1000).toFixed(1)}s`;

        // Mascot smoothly interpolates from center (cx, cy) to top-left status bar (28, 22)
        const targetX = 26;
        const targetY = 22;
        const curX = cx + (targetX - cx) * p4;
        const curY = cy + (targetY - cy) * p4;
        const scale = 1 - 0.75 * p4;

        ctx.save();
        ctx.translate(curX, curY);
        ctx.scale(scale, scale);

        // Draw simplified morphing icon
        ctx.strokeStyle = '#FF6D00';
        ctx.lineWidth = 3;
        ctx.strokeRect(-12, -12, 24, 24);
        ctx.strokeStyle = '#00F0FF';
        ctx.strokeRect(-6, -6, 12, 12);
        ctx.restore();

        // Reveal shell by fading in
        if (layerShell) {
          layerShell.classList.remove('hidden');
          layerShell.style.opacity = p4;
        }
      }

      if (elapsed < DURATION) {
        animFrameId = requestAnimationFrame(renderFrame);
      } else {
        // Animation Complete -> Lock into Shell
        if (animHudBadge) animHudBadge.textContent = 'COLD BOOT ANIMATION COMPLETE (2.8s)';
        setTimeout(() => {
          layerBootAnim?.classList.add('hidden');
          layerShell?.classList.remove('hidden');
          if (layerShell) layerShell.style.opacity = '1';
          if (typeof onComplete === 'function') onComplete();
        }, 150);
      }
    }

    animFrameId = requestAnimationFrame(renderFrame);
  }

  // --- Indic Language Selector (§11) ---
  document.querySelectorAll('.lang-pill-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      document.querySelectorAll('.lang-pill-btn').forEach(b => b.classList.remove('active'));
      e.target.classList.add('active');
      const lang = e.target.dataset.lang;
      currentLanguage = lang;
      const dict = I18N[lang] || I18N.en;

      const c1Title = document.getElementById('card-1-title');
      const c1Desc = document.getElementById('card-1-desc');
      const c1Action = document.getElementById('btn-card-action');
      const ambDate = document.getElementById('ambient-date');
      const intentInput = document.getElementById('intent-input');

      if (c1Title) c1Title.textContent = dict.card1Title;
      if (c1Desc) c1Desc.textContent = dict.card1Desc;
      if (c1Action) c1Action.textContent = dict.card1Action;
      if (ambDate) ambDate.textContent = dict.dateStr;
      if (intentInput) intentInput.placeholder = dict.intentPlaceholder;

      appendSerialLog(`[SHELL] Active UI language switched to: ${lang.toUpperCase()}`, 'info');
    });
  });

  // --- Central Intent Bar Dispatcher (§6.1) ---
  function dispatchIntent(query) {
    if (!query) return;
    appendSerialLog(`[INTENT] Processing natural language intent: "${query}"`, 'info');
    const q = query.toLowerCase();

    if (q.includes('60hz') || q.includes('battery') || q.includes('save') || q.includes('power')) {
      is120Hz = false;
      const telemRefresh = document.getElementById('telem-refresh');
      if (telemRefresh) telemRefresh.textContent = '60 Hz';
      const c1Action = document.getElementById('btn-card-action');
      if (c1Action) c1Action.textContent = 'Applied (60Hz)';
      appendSerialLog('[INTENT-ACTION] Executed DSI panel shift to 60Hz Eco Mode', 'milestone');
    } else if (q.includes('file') || q.includes('doc') || q.includes('indfs')) {
      openFilesApp();
    } else if (q.includes('note') || q.includes('write') || q.includes('ime') || q.includes('hindi')) {
      openNotesApp();
    } else if (q.includes('why') || q.includes('receipt') || q.includes('audit')) {
      openWhyExplorerApp();
    } else if (q.includes('setting') || q.includes('config') || q.includes('rate')) {
      openSettingsApp();
    } else if (q.includes('term') || q.includes('shell') || q.includes('cli')) {
      openTerminalApp();
    } else if (q.includes('consent') || q.includes('perm') || q.includes('mic')) {
      openConsentModal();
    } else {
      appendSerialLog(`[AIS-PARSER] Intent parsed into task graph: "${query}" -> Capability Check [PASS]`, 'info');
      openDeviceModal('Intent Proposal', `
        <div style="line-height: 1.6;">
          <div style="color: var(--tok-color-accent, #7C6CFF); font-weight: bold; margin-bottom: 8px;">
            Intent Recognized: "${query}"
          </div>
          <div><strong>Synthesized Action:</strong> Ambient Task Pipeline</div>
          <div><strong>On-Device Model:</strong> AIS 8-bit Quantized LM (ARM NEON)</div>
          <div><strong>Confidence:</strong> I'm fairly sure (91% match)</div>
          <div style="margin-top: 12px; display: flex; gap: 8px;">
            <button class="proposal-btn apply-btn" onclick="alert('Action Dispatched!')">Proceed</button>
            <button class="proposal-btn dismiss-btn" onclick="document.getElementById('btn-close-device-modal').click()">Cancel</button>
          </div>
        </div>
      `, '⚡');
    }
  }

  const intentInput = document.getElementById('intent-input');
  const btnIntentSend = document.getElementById('btn-intent-send');
  const btnIntentMic = document.getElementById('btn-intent-mic');

  btnIntentSend?.addEventListener('click', () => {
    if (intentInput) {
      dispatchIntent(intentInput.value.trim());
      intentInput.value = '';
    }
  });

  intentInput?.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      dispatchIntent(intentInput.value.trim());
      intentInput.value = '';
    }
  });

  btnIntentMic?.addEventListener('click', () => {
    appendSerialLog('[INTENT-MIC] Push-To-Talk Mic PTT activated. Audio capture sampling...', 'info');
    openConsentModal();
  });

  // --- Proactive Proposal Card Actions ---
  document.getElementById('btn-card-action')?.addEventListener('click', () => {
    is120Hz = !is120Hz;
    const label = is120Hz ? '120Hz' : '60Hz';
    if (elTelemRefresh) elTelemRefresh.textContent = label;
    const btn = document.getElementById('btn-card-action');
    if (btn) btn.textContent = is120Hz ? 'Apply 60Hz' : 'Revert to 120Hz';
    appendSerialLog(`[DSI] Dynamic display governor shifted panel to ${label}`, 'milestone');
  });

  document.getElementById('btn-card-dismiss')?.addEventListener('click', () => {
    const card = document.getElementById('proposal-card-container');
    if (card) {
      card.style.opacity = '0';
      card.style.transform = 'translateY(-10px)';
      setTimeout(() => card.style.display = 'none', 300);
      appendSerialLog('[PROPOSAL] Proactive proposal dismissed by user', 'info');
    }
  });

  document.getElementById('btn-card-why')?.addEventListener('click', () => {
    openWhyExplorerApp();
  });

  // --- In-Device Modal Helpers ---
  // =========================================================================
  // SP-127 App Framework Extensions: Lifecycle & Back-Stack Engine
  // Invariants: INV-APP-LIFE-001, INV-APP-STACK-001, INV-APP-CHROME-001
  // =========================================================================
  const AppLifecycleState = {
    UNLOADED: 'Unloaded',
    CREATED: 'Created',
    FOREGROUND: 'Foreground',
    BACKGROUND: 'Background',
    SUSPENDED: 'Suspended',
    TERMINATED: 'Terminated',
  };

  class SimulatorBackStackManager {
    constructor() {
      this.stack = [];
      this.maxDepth = 16;
    }

    push(appId, title, icon, htmlContent, badge = 'Phase 11') {
      if (this.stack.length >= this.maxDepth) {
        appendSerialLog('[ERR] AppBackStack overflow: depth limit 16 reached (INV-APP-STACK-001)', 'err');
        return false;
      }

      // Background existing active window if any (INV-APP-LIFE-001)
      if (this.stack.length > 0) {
        const prev = this.stack[this.stack.length - 1];
        prev.state = AppLifecycleState.BACKGROUND;
        appendSerialLog(`[LIFECYCLE] ${prev.appId} (win #${prev.windowId}): on_background() -> Pausing render`, 'info');
      }

      const windowId = Math.floor(100 + Math.random() * 900);
      const entry = {
        appId,
        title,
        icon,
        htmlContent,
        badge,
        windowId,
        state: AppLifecycleState.CREATED,
      };

      this.stack.push(entry);
      appendSerialLog(`[LIFECYCLE] ${appId} (win #${windowId}): on_launch() -> Created (PID: ${100 + this.stack.length})`, 'sys');
      
      entry.state = AppLifecycleState.FOREGROUND;
      appendSerialLog(`[LIFECYCLE] ${appId} (win #${windowId}): on_foreground() -> Active focus (depth: ${this.stack.length})`, 'sys');

      this.renderTop();
      return true;
    }

    pop() {
      if (this.stack.length === 0) {
        return null;
      }

      const closing = this.stack.pop();
      closing.state = AppLifecycleState.TERMINATED;
      appendSerialLog(`[LIFECYCLE] ${closing.appId} (win #${closing.windowId}): on_terminate() -> Resources freed`, 'warn');

      if (this.stack.length > 0) {
        const top = this.stack[this.stack.length - 1];
        top.state = AppLifecycleState.FOREGROUND;
        appendSerialLog(`[LIFECYCLE] ${top.appId} (win #${top.windowId}): on_foreground() -> Resumed from back-stack (depth: ${this.stack.length})`, 'sys');
        this.renderTop();
        return top;
      } else {
        // Stack is empty -> Transition focus back to Bento Grid root (INV-APP-STACK-001)
        appendSerialLog(`[NAV] Back-stack empty -> Focus returned to Ind Surface v2 Bento Grid (INV-APP-STACK-001)`, 'sys');
        if (elDeviceModal) elDeviceModal.classList.add('hidden');
        return null;
      }
    }

    clearToHome() {
      while (this.stack.length > 0) {
        const closing = this.stack.pop();
        closing.state = AppLifecycleState.TERMINATED;
        appendSerialLog(`[LIFECYCLE] ${closing.appId} (win #${closing.windowId}): on_terminate() -> Cleared for Home`, 'info');
      }
      appendSerialLog(`[NAV] Home gesture triggered -> Back-stack reset to root (Bento Grid active)`, 'sys');
      if (elDeviceModal) elDeviceModal.classList.add('hidden');
    }

    renderTop() {
      if (!elDeviceModal || this.stack.length === 0) return;
      const top = this.stack[this.stack.length - 1];

      elModalTitle.textContent = top.title;
      const modalIcon = document.getElementById('device-modal-icon');
      if (modalIcon) modalIcon.textContent = top.icon;

      const modalBadge = document.getElementById('device-modal-badge');
      if (modalBadge) {
        modalBadge.textContent = top.badge || `Stack [${this.stack.length}]`;
        modalBadge.classList.remove('hidden');
      }

      const backBtn = document.getElementById('btn-chrome-back');
      if (backBtn) {
        // Back affordance visible if stack depth > 1 or child app view
        if (this.stack.length > 1) {
          backBtn.classList.remove('hidden');
          backBtn.title = `Go Back to ${this.stack[this.stack.length - 2].title}`;
        } else {
          backBtn.classList.add('hidden');
        }
      }

      elModalBody.innerHTML = top.htmlContent;
      elDeviceModal.classList.remove('hidden');
    }

    current() {
      return this.stack.length > 0 ? this.stack[this.stack.length - 1] : null;
    }
  }

  const appBackStack = new SimulatorBackStackManager();

  // Backward-compatible bridge to open modal as a managed window
  function openDeviceModal(title, htmlContent, icon = '📱', appId = 'ind.sys.app', badge = 'Phase 11') {
    appBackStack.push(appId, title, icon, htmlContent, badge);
  }

  function closeDeviceModal() {
    appBackStack.pop();
  }

  // Wire AppChrome and Navigation Bar listeners (INV-APP-STACK-001)
  document.getElementById('btn-chrome-back')?.addEventListener('click', () => {
    appBackStack.pop();
  });
  document.getElementById('btn-close-device-modal')?.addEventListener('click', () => {
    appBackStack.pop();
  });
  document.getElementById('btn-nav-back')?.addEventListener('click', () => {
    appBackStack.pop();
  });
  document.getElementById('nav-home-wrap')?.addEventListener('click', () => {
    appBackStack.clearToHome();
  });

  document.getElementById('btn-chrome-action')?.addEventListener('click', () => {
    const cur = appBackStack.current();
    if (cur) {
      appendSerialLog(`[WINDOW] Info: App="${cur.appId}", WinID=#${cur.windowId}, State=${cur.state}, StackDepth=${appBackStack.stack.length}`, 'sys');
    }
  });

  // --- Bento Grid Tile Handlers ---
  document.getElementById('tile-battery')?.addEventListener('click', () => {
    openSettingsApp('battery');
  });

  document.getElementById('tile-security')?.addEventListener('click', () => {
    openWhyExplorerApp('sovereignty');
  });

  // --- CORE APP 1: FilesApp (§10) ---
  function openFilesApp() {
    openDeviceModal('FilesApp (IndFS Explorer)', `
      <div style="display: flex; flex-direction: column; gap: 10px;">
        <div style="background: rgba(255,255,255,0.05); padding: 8px 12px; border-radius: 8px; font-size: 10.5px;">
          <div style="color: #4CD9A6; font-weight: 700;">✓ IndFS Mounted: /dev/ufs0p6 (Crash-Resilient)</div>
          <div style="color: #9BA0AE; font-size: 9.5px; margin-top: 2px;">FUA Commit Verified • Provenance Tracking Active</div>
        </div>
        <div style="display: flex; flex-direction: column; gap: 6px;">
          <div style="display: flex; justify-content: space-between; padding: 6px 8px; background: rgba(255,255,255,0.03); border-radius: 6px;">
            <span>📁 /system/kernel.elf</span>
            <span style="color: #9BA0AE; font-family: var(--font-mono);">128 KB [r-x]</span>
          </div>
          <div style="display: flex; justify-content: space-between; padding: 6px 8px; background: rgba(255,255,255,0.03); border-radius: 6px;">
            <span>📁 /system/dtb/sm7150.dtb</span>
            <span style="color: #9BA0AE; font-family: var(--font-mono);">335 KB [r--]</span>
          </div>
          <div style="display: flex; justify-content: space-between; padding: 6px 8px; background: rgba(255,255,255,0.03); border-radius: 6px;">
            <span>📁 /data/notes/namaste.txt</span>
            <span style="color: #9BA0AE; font-family: var(--font-mono);">4 KB [rw-]</span>
          </div>
          <div style="display: flex; justify-content: space-between; padding: 6px 8px; background: rgba(255,255,255,0.03); border-radius: 6px;">
            <span>📁 /certs/sovereignty.ed25519</span>
            <span style="color: #4CD9A6; font-family: var(--font-mono);">64 B [r--]</span>
          </div>
        </div>
        <div style="margin-top: 6px; font-size: 9.5px; color: #9BA0AE;">
          Dogfood Rule INV-APP-001: Zero unearned capabilities. FS write grants strictly user-mediated.
        </div>
      </div>
    `, '📁');
  }

  // --- CORE APP 2: SettingsApp (§10) ---
  function openSettingsApp(defaultTab = 'all') {
    openDeviceModal('SettingsApp (Hardware & Consent)', `
      <div style="display: flex; flex-direction: column; gap: 10px;">
        <div style="background: rgba(255,255,255,0.05); padding: 10px; border-radius: 10px;">
          <div style="font-weight: 700; margin-bottom: 6px;">Display Refresh Governor</div>
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-size: 10.5px; color: #9BA0AE;">120Hz Fluid / 60Hz Eco Rate</span>
            <button id="modal-toggle-hz" class="proposal-btn apply-btn" style="padding: 4px 10px;">${is120Hz ? 'Shift to 60Hz' : 'Shift to 120Hz'}</button>
          </div>
        </div>

        <div style="background: rgba(255,255,255,0.05); padding: 10px; border-radius: 10px;">
          <div style="font-weight: 700; margin-bottom: 6px;">Proactive AIS Dial (REQ-AI-004)</div>
          <div style="font-size: 10px; color: #9BA0AE; margin-bottom: 6px;">Controls autonomy level of on-device proposals:</div>
          <div style="display: flex; gap: 6px;">
            <button class="proposal-btn dismiss-btn" style="flex: 1; font-size: 9.5px;">Conservative</button>
            <button class="proposal-btn apply-btn" style="flex: 1; font-size: 9.5px;">Moderate</button>
            <button class="proposal-btn dismiss-btn" style="flex: 1; font-size: 9.5px;">Aggressive</button>
          </div>
        </div>

        <div style="background: rgba(255,255,255,0.05); padding: 10px; border-radius: 10px;">
          <div style="font-weight: 700; margin-bottom: 4px;">PMIC Fuel Gauge Telemetry</div>
          <div style="font-family: var(--font-mono); font-size: 10px; color: #4CD9A6;">
            4050 mV • 88% State-of-Charge • -250 mA
          </div>
          <div style="font-size: 9.5px; color: #9BA0AE; margin-top: 4px;">
            TSENS Thermal Governor: 36.8°C (Mild threshold: 38°C)
          </div>
        </div>
      </div>
    `, '⚙️');

    document.getElementById('modal-toggle-hz')?.addEventListener('click', () => {
      document.getElementById('btn-card-action')?.click();
      openSettingsApp();
    });
  }

  // --- CORE APP 3: WhyExplorerApp (§10 & §9.2) ---
  function openWhyExplorerApp(filter = 'all') {
    openDeviceModal('WhyExplorerApp (Causal Timeline)', `
      <div style="display: flex; flex-direction: column; gap: 10px; font-family: var(--font-mono); font-size: 10.5px;">
        <div style="background: rgba(124, 108, 255, 0.12); padding: 8px 12px; border-radius: 8px; border: 1px solid rgba(124, 108, 255, 0.3);">
          <div style="color: #7C6CFF; font-weight: 700;">✓ Audit-Log Read-Only Verification (§9.2)</div>
          <div style="color: #9BA0AE; font-size: 9.5px; margin-top: 2px;">Causal Trace & Cryptographic ed25519 Signatures</div>
        </div>

        <div style="border-left: 2px solid #7C6CFF; padding-left: 10px; display: flex; flex-direction: column; gap: 8px;">
          <div>
            <div style="color: #4CD9A6; font-weight: 700;">Event #8491: AIS_DISP_60HZ_PROPOSAL</div>
            <div style="color: #9BA0AE; font-size: 9.5px;">Trigger: TSENS inactive session > 45s</div>
            <div style="color: #CBD5E1; font-size: 9px;">Receipt Hash: 0x7a89_f92c_b14e_338a [VALID]</div>
          </div>

          <div>
            <div style="color: #7C6CFF; font-weight: 700;">Event #8490: GATE_G10_RATIFICATION</div>
            <div style="color: #9BA0AE; font-size: 9.5px;">Trigger: 8/8 Acceptance Criteria Certified</div>
            <div style="color: #CBD5E1; font-size: 9px;">Receipt Hash: 0x5e8c_31a9_ff04_1120 [VALID]</div>
          </div>

          <div>
            <div style="color: #C084FC; font-weight: 700;">Event #8489: INDFS_JOURNAL_COMMIT</div>
            <div style="color: #9BA0AE; font-size: 9.5px;">Trigger: FUA flush to UFS 2.1 flash</div>
            <div style="color: #CBD5E1; font-size: 9px;">Receipt Hash: 0x992b_1402_cc77_4119 [VALID]</div>
          </div>
        </div>
      </div>
    `, '🔍');
  }

  // --- CORE APP 4: NotesApp with Indic IME Dogfood (§10 & §11) ---
  function openNotesApp() {
    openDeviceModal('NotesApp (Indic IME Dogfood)', `
      <div style="display: flex; flex-direction: column; gap: 8px;">
        <div style="font-size: 10px; color: #9BA0AE;">
          Live Transliteration-first IME (Hindi & Tamil). Zero-heap input path.
        </div>
        <div style="display: flex; gap: 6px; background: rgba(255,255,255,0.06); padding: 6px 10px; border-radius: 8px;">
          <span style="font-size: 9.5px; color: #7C6CFF; font-weight: 700;">SUGGESTIONS:</span>
          <span id="ime-suggestion-1" style="font-size: 11px; color: #4CD9A6; cursor: pointer; font-weight: 600;">नमस्ते</span>
          <span id="ime-suggestion-2" style="font-size: 11px; color: #FFFFFF; cursor: pointer;">வணக்கம்</span>
          <span id="ime-suggestion-3" style="font-size: 11px; color: #CBD5E1; cursor: pointer;">dhanyavad</span>
        </div>
        <textarea id="notes-textarea" placeholder="Type in English (e.g., 'namaste', 'shukriya', 'vanakkam') to transliterate..." style="width: 100%; height: 140px; background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.1); border-radius: 8px; color: #FFF; padding: 10px; font-family: var(--font-sans); font-size: 12px; resize: none; outline: none;"></textarea>
        <div style="display: flex; justify-content: space-between; align-items: center;">
          <span style="font-size: 9.5px; color: #4CD9A6;">✓ IndFS Auto-Saved</span>
          <button class="proposal-btn apply-btn" id="btn-save-note" style="padding: 4px 12px;">Save Note</button>
        </div>
      </div>
    `, '📝');

    setTimeout(() => {
      const textarea = document.getElementById('notes-textarea');
      const sug1 = document.getElementById('ime-suggestion-1');
      const sug2 = document.getElementById('ime-suggestion-2');
      const sug3 = document.getElementById('ime-suggestion-3');

      if (textarea) {
        textarea.focus();
        textarea.addEventListener('input', (e) => {
          const val = textarea.value.toLowerCase();
          if (val.includes('namaste')) {
            textarea.value = textarea.value.replace(/namaste/gi, 'नमस्ते ');
          } else if (val.includes('shukriya')) {
            textarea.value = textarea.value.replace(/shukriya/gi, 'शुक्रिया ');
          } else if (val.includes('vanakkam')) {
            textarea.value = textarea.value.replace(/vanakkam/gi, 'வணக்கம் ');
          } else if (val.includes('dhanyavad')) {
            textarea.value = textarea.value.replace(/dhanyavad/gi, 'धन्यवाद ');
          }
        });
      }

      sug1?.addEventListener('click', () => { if (textarea) textarea.value += 'नमस्ते '; });
      sug2?.addEventListener('click', () => { if (textarea) textarea.value += 'வணக்கம் '; });
      sug3?.addEventListener('click', () => { if (textarea) textarea.value += 'धन्यवाद '; });
    }, 100);
  }

  // --- CORE APP 5: TerminalApp (§10) ---
  function openTerminalApp() {
    openDeviceModal('Ind OS Microkernel Shell', `
      <div class="in-modal-terminal" style="font-family: var(--font-mono); font-size: 11px;">
        <div style="color: var(--tok-color-accent, #7C6CFF); margin-bottom: 6px;">Ind OS Sovereign Shell v2 (PID 4)</div>
        <div style="color: var(--text-dim, #64748B); margin-bottom: 10px;">Type 'help', 'ps', 'ls', 'version', 'caps', 'soak', or 'reboot'</div>
        <div id="modal-term-output" style="max-height: 200px; overflow-y: auto; white-space: pre-wrap; line-height: 1.4; color: #cbd5e1;"></div>
        <div style="display: flex; gap: 6px; margin-top: 10px; align-items: center;">
          <span style="color: #4CD9A6; font-weight: bold;">indos#</span>
          <input type="text" id="modal-term-input" style="flex: 1; background: rgba(255,255,255,0.08); border: 1px solid rgba(255,255,255,0.1); border-radius: 4px; color: #fff; padding: 4px 8px; font-family: inherit; font-size: 11px;" autofocus autocomplete="off">
        </div>
      </div>
    `, '💻');

    setTimeout(() => {
      const termInput = document.getElementById('modal-term-input');
      const termOutput = document.getElementById('modal-term-output');
      if (termInput) {
        termInput.focus();
        termInput.addEventListener('keydown', (e) => {
          if (e.key === 'Enter') {
            const cmd = termInput.value.trim().toLowerCase();
            termInput.value = '';
            handleTerminalCommand(cmd, termOutput);
          }
        });
      }
    }, 100);
  }

  function handleTerminalCommand(cmd, outputEl) {
    outputEl.textContent += `indos# ${cmd}\n`;
    switch(cmd) {
      case 'help':
        outputEl.textContent += "Builtins:\n  help      - Print available commands\n  version   - Show Ind OS and kernel build version\n  ps        - List running capability-isolated processes\n  ls        - Inspect mounted UFS IndFS namespaces\n  caps      - List active capability tokens\n  soak      - Run 100-cycle subsystem stress audit\n  reboot    - Cold reboot Moto G60 hardware\n  clear     - Clear terminal buffer\n\n";
        break;
      case 'version':
        outputEl.textContent += "Ind OS v0.1.0-hardware-v0.1 (aarch64-unknown-none)\nMilestone: MOTO G60 BRING-UP (Gate G10 Ratified)\nRule N-01: Zero external crates in TCB\nRule N-04: 100% Safe Rust in userspace\n\n";
        break;
      case 'ps':
        outputEl.textContent += "PID  COMM          STATUS   CAPS\n  1  indus-init    RUNNING  [SYS_ADMIN, STORAGE_MOUNT]\n  2  indus-logd    RUNNING  [LOG_RECORD]\n  3  indus-store   RUNNING  [SETTINGS_POWER, TAMPER_PROOF]\n  4  indus-shell   RUNNING  [DISPLAY_RENDER, TOUCH_INPUT]\n  5  indus-edit    RUNNING  [IPC_SYNC]\n\n";
        break;
      case 'ls':
        outputEl.textContent += "/sys    (Read-Only Kernel Interface)\n/dev    (Hardware Device Nodes: ufs0, dsi0, i2c1, spmi0)\n/cfg    (Integrity-Protected Settings)\n/data   (Persistent User Storage on UFS 2.1)\n\n";
        break;
      case 'caps':
        outputEl.textContent += "ACTIVE CAPABILITY TOKENS:\n  0x0000_1000 - CAP_DISPLAY_RENDER\n  0x0000_1001 - CAP_TOUCH_INPUT\n  0x0000_1010 - CAP_SETTINGS_POWER\n  0x0000_1080 - CAP_G9_PACK_ADMIN\n  0x0000_1090 - CAP_GATE_G10_RATIFY\n\n";
        break;
      case 'soak':
        outputEl.textContent += "Running 100-cycle concurrency soak...\n  [UFS, Display, Sensors, Audio, AIS]\n  100/100 cycles passed. Zero bus stalls, 0B heap leaks.\n\n";
        break;
      case 'clear':
        outputEl.textContent = "";
        break;
      case 'reboot':
        closeDeviceModal();
        startBoot();
        return;
      default:
        outputEl.textContent += `Unknown command: '${cmd}'. Type 'help' for options.\n\n`;
    }
    outputEl.scrollTop = outputEl.scrollHeight;
  }

  // --- TRUSTED CONSENT SURFACE (§8, PlaneId::HardwareIsolated) ---
  function openConsentModal() {
    openDeviceModal('Hardware-Isolated Consent Surface', `
      <div style="display: flex; flex-direction: column; gap: 12px;">
        <div style="background: rgba(76, 217, 166, 0.12); border: 1px solid rgba(76, 217, 166, 0.35); padding: 10px; border-radius: 10px;">
          <div style="display: flex; align-items: center; gap: 6px; color: #4CD9A6; font-weight: 700; font-size: 11px;">
            <span>🛡️</span>
            <span>PlaneId::HardwareIsolated (REQ-SEC-201)</span>
          </div>
          <div style="font-size: 9.5px; color: #9BA0AE; margin-top: 4px;">
            Unspoofable hardware overlay. No userspace app can paint or intercept this dialogue.
          </div>
        </div>

        <div>
          <div style="font-weight: 700; font-size: 12px; margin-bottom: 4px;">
            Audio Capture Permission Requested
          </div>
          <div style="color: #9BA0AE; font-size: 10.5px; line-height: 1.45;">
            The Indic Voice IME is requesting temporary access to the WCD9385 microphone for push-to-talk speech transcription.
          </div>
        </div>

        <div style="display: flex; gap: 8px; margin-top: 6px;">
          <button class="proposal-btn apply-btn" id="btn-grant-consent" style="flex: 1; background: #4CD9A6; color: #0E0F14; font-weight: 700;">Grant Access</button>
          <button class="proposal-btn dismiss-btn" id="btn-deny-consent" style="flex: 1; background: rgba(255, 107, 107, 0.15); color: #FF6B6B; border-color: rgba(255, 107, 107, 0.3);">Deny</button>
          <button class="proposal-btn why-btn" id="btn-why-consent">Why?</button>
        </div>
      </div>
    `, '🛡️');

    document.getElementById('btn-grant-consent')?.addEventListener('click', () => {
      appendSerialLog('[CONSENT] User GRANTED capability CAP_AUDIO_CAPTURE to Voice IME', 'milestone');
      closeDeviceModal();
    });

    document.getElementById('btn-deny-consent')?.addEventListener('click', () => {
      appendSerialLog('[CONSENT] User DENIED capability CAP_AUDIO_CAPTURE', 'info');
      closeDeviceModal();
    });

    document.getElementById('btn-why-consent')?.addEventListener('click', () => {
      openWhyExplorerApp();
    });
  }

  // --- TIER 1 APP: Phone & Emergency (§9.9.2, REQ-APP-PHONE-001..004) ---
  function openPhoneApp() {
    openDeviceModal('Phone & Dialer', `
      <div style="display: flex; flex-direction: column; gap: 10px; text-align: center;">
        <div style="background: rgba(76, 217, 166, 0.08); padding: 8px; border-radius: 8px; border: 1px solid rgba(76, 217, 166, 0.2);">
          <div style="color: #4CD9A6; font-size: 11px; font-weight: 700;">✓ Cellular Radio Active (SIM 1: Airtel 4G)</div>
          <div style="color: #9BA0AE; font-size: 9.5px; margin-top: 2px;">Hardware Plane Protection: Active • Anti-Tapjacking</div>
        </div>
        <div style="font-size: 18px; font-weight: 700; color: #FFF; font-family: var(--font-mono); padding: 6px 0; border-bottom: 1px solid rgba(255,255,255,0.1);" id="phone-number-display">+91 98765 43210</div>
        <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; padding: 2px 8px;">
          ${['1', '2', '3', '4', '5', '6', '7', '8', '9', '*', '0', '#'].map(k => `
            <button class="proposal-btn dismiss-btn" style="padding: 8px; font-size: 13px; font-weight: 700;" onclick="document.getElementById('phone-number-display').textContent += '${k}'">${k}</button>
          `).join('')}
        </div>
        <div style="display: flex; gap: 6px; margin-top: 2px;">
          <button class="proposal-btn apply-btn" style="flex: 2; background: #4CD9A6; color: #0E0F14; font-weight: 700; padding: 6px;" onclick="alert('Placing Cellular Call via SM7150 Modem HAL...')">📞 Voice Call</button>
          <button class="proposal-btn dismiss-btn" style="flex: 1; background: rgba(255,107,107,0.15); color: #FF6B6B; border-color: rgba(255,107,107,0.3); font-weight: 700;" onclick="alert('Calling 112 National Emergency Hotline (Unrestricted Lock-Bypass)')">🚨 112 SOS</button>
        </div>
      </div>
    `, '📞', 'ind.app.phone', 'Tier 1');
  }

  // --- TIER 1 APP: Messages (§9.9.2, REQ-APP-MSG-001..006) ---
  function openMessagesApp() {
    openDeviceModal('Messages & SMS', `
      <div style="display: flex; flex-direction: column; gap: 10px;">
        <div style="background: rgba(124, 108, 255, 0.1); padding: 8px 10px; border-radius: 8px; border: 1px solid rgba(124, 108, 255, 0.25); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="color: #7C6CFF; font-size: 11px; font-weight: 700;">Suraksha Scam & OTP Shield Active</div>
            <div style="color: #9BA0AE; font-size: 9px;">Zero Screen-Share Leakage • Encrypted Local Vault</div>
          </div>
          <span style="font-size: 14px;">🛡️</span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 6px;">
          <div style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; padding: 8px 10px; cursor: pointer;">
            <div style="display: flex; justify-content: space-between;">
              <span style="font-size: 11px; font-weight: 700; color: #FFF;">AX-HDFCBK</span>
              <span style="font-size: 9px; color: #9BA0AE;">10:42 AM</span>
            </div>
            <div style="font-size: 10px; color: #CBD5E1; margin-top: 3px;">Your OTP is [••••••]. Valid for 10 mins. (Auto-redacted from screen share).</div>
          </div>
          <div style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; padding: 8px 10px; cursor: pointer;">
            <div style="display: flex; justify-content: space-between;">
              <span style="font-size: 11px; font-weight: 700; color: #FFF;">Aadhaar Portal</span>
              <span style="font-size: 9px; color: #9BA0AE;">Yesterday</span>
            </div>
            <div style="font-size: 10px; color: #CBD5E1; margin-top: 3px;">Authentication successful at 14:22 hrs via Ind OS Sovereign Vault.</div>
          </div>
        </div>
        <button class="proposal-btn apply-btn" style="padding: 7px 12px; margin-top: 4px;" onclick="alert('Composing SMS...')">✉️ New Message</button>
      </div>
    `, '💬', 'ind.app.messages', 'Tier 1');
  }

  // --- Attach App Dock and Control Listeners ---
  document.getElementById('app-phone')?.addEventListener('click', openPhoneApp);
  document.getElementById('app-messages')?.addEventListener('click', openMessagesApp);
  document.getElementById('app-files')?.addEventListener('click', openFilesApp);
  document.getElementById('app-settings')?.addEventListener('click', () => openSettingsApp());
  document.getElementById('app-why')?.addEventListener('click', () => openWhyExplorerApp());
  document.getElementById('app-notes')?.addEventListener('click', openNotesApp);
  document.getElementById('app-terminal')?.addEventListener('click', openTerminalApp);
  document.getElementById('btn-nav-consent')?.addEventListener('click', openConsentModal);

  document.getElementById('btn-open-terminal-app')?.addEventListener('click', () => {
    if (layerShell.classList.contains('hidden')) {
      layerBootloader.classList.add('hidden');
      layerG9.classList.add('hidden');
      layerShell.classList.remove('hidden');
    }
    openTerminalApp();
  });

  // --- Serial Filter Controls ---
  document.querySelectorAll('.filter-pill[data-filter]').forEach(pill => {
    pill.addEventListener('click', (e) => {
      document.querySelectorAll('.filter-pill[data-filter]').forEach(p => p.classList.remove('active'));
      e.target.classList.add('active');
      const filter = e.target.dataset.filter;
      currentFilter = filter;
      document.querySelectorAll('.log-line').forEach(line => {
        if (filter === 'all' || line.dataset.type === filter) {
          line.style.display = 'block';
        } else {
          line.style.display = 'none';
        }
      });
    });
  });

  document.getElementById('btn-clear-serial')?.addEventListener('click', () => {
    if (elSerialLines) elSerialLines.innerHTML = '';
  });

  // Start the boot sequence upon page load
  window.addEventListener('DOMContentLoaded', () => {
    startBoot();
  });

})();
