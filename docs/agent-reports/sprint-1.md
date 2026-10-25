# Ind OS — Agent Execution Report: Sprint 1
**Sprint Window:** Weeks 1–2 | Phase 1 (Foundation, Privileged Testbed & IPC Skeleton)  
**Target Hardware:** Motorola Moto G60 (`hanoip` — Snapdragon 732G / SM7150-AC, 6 GB LPDDR4X)  
**Document Reference:** `v1.0.1-BIBLE`  
**Execution Date:** 2026-10-26  

---

## 1. Architectural Amendments Recorded

### [BIBLE-AMENDMENT A-001 — APPROVED by Lead Architect, BIBLE v1.0.1]
* **Context & Root Cause:**
  When compiling the AIDL interfaces with the Android AIDL compiler (`build-tools/34.0.0/aidl`), the build failed with:
  `syntax error, unexpected in, expecting identifier`.
  Investigation and lexer experiments confirmed that in AIDL grammar, `in` is a reserved directional keyword (token `IN`). AIDL parsers strictly reject any package declaration containing the segment `in` (e.g. `package in.indos.neural.ipc;`). Furthermore, Android Gradle Plugin prohibits direct `implementation(project(...))` between application modules (`com.android.application`), causing AAPT errors.
* **Locked Resolution Approved by Lead Architect:**
  1. **AIDL Interface Package:** Renamed to `package indos.neural.ipc;`. All AIDL files are strictly forbidden from containing any `in.` package component.
  2. **Source Code Packages:** Standardized to `indos.neural`, `indos.zerotouch`, `indos.suraksha`, and `indos.launcher`.
  3. **Application Identifiers:** Preserved as `in.indos.neural`, `in.indos.zerotouch`, `in.indos.suraksha`, and `in.indos.launcher` in `build.gradle.kts` `applicationId`, manifests, and `vendor/ind/permissions/privapp-permissions-ind.xml`.
  4. **Shared IPC Module:** Created `packages/IndIpc` as a `com.android.library` (AOSP `aidl_interface { name: "ind_ai_ipc" }` for Phase 6) housing `IIndAIManager.aidl` and `IIndStreamCallback.aidl`. All 4 app modules link `:packages:IndIpc`.
  5. **Daemon Process:** Named `indos.neural.daemon`.

---

## 2. Sprint 1 Deliverables Summary

| Deliverable | Location | Status | Verification |
| :--- | :--- | :--- | :--- |
| **Dual Build System** | `settings.gradle.kts`, `build.gradle.kts`, `packages/*/Android.bp` | Complete | Both Gradle and AOSP Soong definitions configured |
| **Whitelisted Privileged Permissions** | `vendor/ind/permissions/privapp-permissions-ind.xml` | Complete | XML validated; grants `WRITE_SECURE_SETTINGS`, `MODIFY_PHONE_STATE`, `BIND_ACCESSIBILITY_SERVICE`, `CAPTURE_AUDIO_OUTPUT`, `REAL_GET_TASKS` |
| **Moto G60 Dev Harness** | `tools/dev-harness/deploy_to_hanoip.sh` | Complete | Executable; Magisk/KernelSU systemless overlay packager + automated DoD toggle runner |
| **CI Workflow** | `.github/workflows/android-ci.yml` | Complete | GitHub Actions contract validation, build, and artifact packaging |
| **SELinux Air-Gap Policy** | `vendor/ind/sepolicy/ind_ai_daemon.te` | Complete | Strict `neverallow` for all network sockets |
| **LineageOS / AOSP Tree Manifest** | `manifest/default.xml` | Complete | Syncs `lineage-22.1`, `hanoip` device tree, kernel, and Ind packages |
| **Clean OS Config** | `vendor/ind/config/common.mk` | Complete | Purges bloatware (`Jelly`, `Eleven`, `LineageUpdater`), registers Ind priv-apps |
| **Security & Flashing Docs** | `docs/SECURITY_MODEL.md`, `docs/HANOIP_FLASHING_GUIDE.md` | Complete | Air-gap threat model & fast testbed flashing instructions |

---

## 3. Definition of Done (DoD) Verification

* **DoD Item 1:** `./tools/dev-harness/deploy_to_hanoip.sh` packages APKs + permission XML into systemless `/system_ext/priv-app` overlay on Moto G60.
* **DoD Item 2:** Privileged toggle verification:
  * `IndSystemPrivilegeTestService` executes `Settings.Global.putInt(contentResolver, Settings.Global.WIFI_ON, ...)` and `Settings.System.putInt(contentResolver, Settings.System.SCREEN_BRIGHTNESS, ...)` with zero runtime permission popups.
  * `adb shell settings get global wifi_on` reflects toggle state.
