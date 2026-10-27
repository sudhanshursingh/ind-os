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
  const layerG9 = document.getElementById('layer-g9-celebration');
  const layerShell = document.getElementById('layer-shell');

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
    if (elSystemStatus) elSystemStatus.textContent = 'Gate G9.5 Ratified • Interactive Shell Active';

    appendSerialLog("==================================================", 'milestone');
    appendSerialLog(" 🏆 GATE G9.5 HARD-STOP AUDIT PASSED & RATIFIED 🏆", 'milestone');
    appendSerialLog(" Milestone: PERFORMANCE & USABILITY PASS V0.1", 'milestone');
    appendSerialLog(" FocalTech FT3518 Touch Digitizer & 120Hz Pipelined", 'info');
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
    layerShell.classList.remove('hidden');
  });

  // --- Indic Language Selector ---
  document.querySelectorAll('.lang-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      document.querySelectorAll('.lang-btn').forEach(b => b.classList.remove('active'));
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

  // --- Quick Toggles ---
  document.getElementById('tile-refresh')?.addEventListener('click', function() {
    is120Hz = !is120Hz;
    this.classList.toggle('active', is120Hz);
    const label = is120Hz ? '120Hz' : '60Hz';
    this.querySelector('.tile-label').textContent = label;
    if (elTelemRefresh) elTelemRefresh.textContent = `${label}`;
    appendSerialLog(`[DSI] Dynamic display governor shifted panel to ${label}`, 'info');
  });

  document.getElementById('btn-card-action')?.addEventListener('click', () => {
    const tile = document.getElementById('tile-refresh');
    if (tile) tile.click();
  });

  document.getElementById('btn-card-dismiss')?.addEventListener('click', () => {
    const card = document.querySelector('.proposal-card');
    if (card) {
      card.style.opacity = '0';
      card.style.transform = 'translateY(-10px)';
      setTimeout(() => card.style.display = 'none', 300);
    }
  });

  document.getElementById('btn-card-why')?.addEventListener('click', () => {
    openDeviceModal('Why-Receipt Verification', `
      <div style="font-family: var(--font-mono); line-height: 1.6;">
        <div style="color: var(--accent-cyan); font-weight: bold; margin-bottom: 8px;">
          ✓ Cryptographic Why-Receipt #574800001075
        </div>
        <div><strong>Proposal:</strong> Switch 120Hz -> 60Hz Eco Rate</div>
        <div><strong>Causal Trigger:</strong> Inactive reading session > 45s</div>
        <div><strong>Battery Delta:</strong> -250mA -> -140mA (+3.2h)</div>
        <div><strong>Capability Token:</strong> CAP_SETTINGS_POWER (0x0000_1010)</div>
        <div><strong>Authority:</strong> Pure User-Consent (Ambient Escalation Blocked)</div>
        <div style="margin-top: 10px; color: var(--accent-emerald);">
          Signature: ed25519:7a89f92cb14... [MATCHED]
        </div>
      </div>
    `);
  });

  // --- In-Device Modal Helpers ---
  function openDeviceModal(title, htmlContent) {
    if (!elDeviceModal) return;
    elModalTitle.textContent = title;
    elModalBody.innerHTML = htmlContent;
    elDeviceModal.classList.remove('hidden');
  }

  function closeDeviceModal() {
    if (elDeviceModal) elDeviceModal.classList.add('hidden');
  }

  document.getElementById('btn-close-device-modal')?.addEventListener('click', closeDeviceModal);
  document.getElementById('nav-home-handle')?.addEventListener('click', closeDeviceModal);

  // --- Interactive Terminal App View ---
  function openTerminalApp() {
    openDeviceModal('Ind OS Microkernel Shell', `
      <div class="in-modal-terminal" style="font-family: var(--font-mono); font-size: 11px;">
        <div style="color: var(--accent-cyan); margin-bottom: 6px;">Ind OS Sovereign Shell v2 (PID 4)</div>
        <div style="color: var(--text-dim); margin-bottom: 10px;">Type 'help', 'ps', 'ls', 'version', 'caps', 'soak', or 'reboot'</div>
        <div id="modal-term-output" style="max-height: 220px; overflow-y: auto; white-space: pre-wrap; line-height: 1.4; color: #cbd5e1;"></div>
        <div style="display: flex; gap: 6px; margin-top: 10px; align-items: center;">
          <span style="color: var(--accent-emerald); font-weight: bold;">indos#</span>
          <input type="text" id="modal-term-input" style="flex: 1; background: rgba(255,255,255,0.08); border: 1px solid rgba(255,255,255,0.1); border-radius: 4px; color: #fff; padding: 4px 8px; font-family: inherit; font-size: 11px;" autofocus autocomplete="off">
        </div>
      </div>
    `);

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
        outputEl.textContent += "Ind OS v0.1.0-hardware-v0.1 (aarch64-unknown-none)\nMilestone: MOTO G60 BRING-UP (Gate G9 Ratified)\nRule N-01: Zero external crates in TCB\nRule N-04: 100% Safe Rust in userspace\n\n";
        break;
      case 'ps':
        outputEl.textContent += "PID  COMM          STATUS   CAPS\n  1  indus-init    RUNNING  [SYS_ADMIN, STORAGE_MOUNT]\n  2  indus-logd    RUNNING  [LOG_RECORD]\n  3  indus-store   RUNNING  [SETTINGS_POWER, TAMPER_PROOF]\n  4  indus-shell   RUNNING  [DISPLAY_RENDER, TOUCH_INPUT]\n  5  indus-edit    RUNNING  [IPC_SYNC]\n\n";
        break;
      case 'ls':
        outputEl.textContent += "/sys    (Read-Only Kernel Interface)\n/dev    (Hardware Device Nodes: ufs0, dsi0, i2c1, spmi0)\n/cfg    (Integrity-Protected Settings)\n/data   (Persistent User Storage on UFS 2.1)\n\n";
        break;
      case 'caps':
        outputEl.textContent += "ACTIVE CAPABILITY TOKENS:\n  0x0000_1000 - CAP_DISPLAY_RENDER\n  0x0000_1001 - CAP_TOUCH_INPUT\n  0x0000_1010 - CAP_SETTINGS_POWER\n  0x0000_1080 - CAP_G9_PACK_ADMIN\n  0x0000_1090 - CAP_GATE_G9_RATIFY\n\n";
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

  // --- App Dock Launchers ---
  document.getElementById('app-terminal')?.addEventListener('click', openTerminalApp);
  document.getElementById('btn-open-terminal-app')?.addEventListener('click', () => {
    if (layerShell.classList.contains('hidden')) {
      layerBootloader.classList.add('hidden');
      layerG9.classList.add('hidden');
      layerShell.classList.remove('hidden');
    }
    openTerminalApp();
  });

  document.getElementById('app-settings')?.addEventListener('click', () => {
    openDeviceModal('Hardware Settings', `
      <div style="display: flex; flex-direction: column; gap: 10px;">
        <div style="background: rgba(255,255,255,0.05); padding: 8px; border-radius: 8px;">
          <strong>Qualcomm PM7150 / PM7150L SPMI</strong>
          <div style="color: var(--text-muted); font-size: 10px; margin-top: 4px;">18 Regulators Active • VDD_IO 1.8V • VDD_CORE 0.9V</div>
        </div>
        <div style="background: rgba(255,255,255,0.05); padding: 8px; border-radius: 8px;">
          <strong>6000mAh Battery Fuel Gauge</strong>
          <div style="color: var(--accent-emerald); font-size: 10px; margin-top: 4px;">4050mV • 88% State-of-Charge • -250mA Discharging</div>
        </div>
        <div style="background: rgba(255,255,255,0.05); padding: 8px; border-radius: 8px;">
          <strong>Thermal Governors (TSENS)</strong>
          <div style="color: var(--text-muted); font-size: 10px; margin-top: 4px;">Current 36.8°C • Mild: 38°C • Throttle: 42°C • Cutoff: 46°C</div>
        </div>
      </div>
    `);
  });

  document.getElementById('app-sensors')?.addEventListener('click', () => {
    openDeviceModal('Unified Sensor Hub', `
      <div style="display: flex; flex-direction: column; gap: 8px; font-family: var(--font-mono);">
        <div><strong>Bosch BMI260 (6-Axis IMU):</strong></div>
        <div style="color: var(--accent-cyan); font-size: 10px;">Accel: [X: 0.02g, Y: -0.01g, Z: 0.99g]</div>
        <div style="color: var(--accent-cyan); font-size: 10px;">Gyro:  [X: 0.1°/s, Y: 0.0°/s, Z: -0.1°/s]</div>
        <div style="margin-top: 6px;"><strong>Optical ALS & Proximity:</strong></div>
        <div style="color: var(--accent-emerald); font-size: 10px;">Ambient Lux: 150 Lux • Distance: Far (>5cm)</div>
        <div style="margin-top: 6px;"><strong>AK09918 3-Axis Magnetometer:</strong></div>
        <div style="color: var(--accent-violet); font-size: 10px;">Compass Azimuth: 45° North-East</div>
      </div>
    `);
  });

  document.getElementById('app-why')?.addEventListener('click', () => {
    openDeviceModal('Why-Engine Causal Log', `
      <div style="display: flex; flex-direction: column; gap: 8px; font-size: 10px; font-family: var(--font-mono);">
        <div style="color: var(--accent-cyan);">[Event 5001] Display Compositor Flipped Frame Buffer (120Hz)</div>
        <div style="color: #94a3b8;">[Event 5002] Touch Event Dispatched: Tap at (540, 1230)</div>
        <div style="color: var(--accent-emerald);">[Event 5003] UFS Journal FUA Commit Synced to Disk</div>
        <div style="color: #94a3b8;">[Event 5004] Battery Fuel Gauge Polled: 4050mV, 88% SoC</div>
        <div style="color: var(--accent-violet);">[Event 5005] Intent Proposal Generated with Verified Why-Receipt</div>
      </div>
    `);
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
