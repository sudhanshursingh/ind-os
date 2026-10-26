/*
 * Copyright (C) 2026 Ind OS Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package indos.suraksha.screenshare;

import android.content.Context;
import android.hardware.display.DisplayManager;
import android.media.AudioManager;
import android.util.Log;
import android.view.Display;

import java.util.Arrays;
import java.util.Collections;
import java.util.HashSet;
import java.util.Set;
import java.util.concurrent.CopyOnWriteArraySet;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * Privileged Screen-Share and Active Call Detector (PRD v1.0.1 §4.3-C & Sprint 11).
 *
 * Monitors:
 * 1. Virtual / Mirror Displays via DisplayManager.DisplayListener (e.g. VIRTUAL_DISPLAY_FLAG_AUTO_MIRROR)
 * 2. Active Telecom / VoIP Call state via AudioManager (MODE_IN_COMMUNICATION / MODE_IN_CALL)
 * 3. Known Screen-Sharing remote assistance tools (AnyDesk, TeamViewer, RustDesk, WhatsApp)
 */
public class ScreenShareDetector implements DisplayManager.DisplayListener {

    private static final String TAG = "ScreenShareDetector";

    // Known Screen-Sharing & Remote Access Packages used in Indian BFSI social-engineering scams
    public static final Set<String> SCREEN_SHARING_PACKAGES = Collections.unmodifiableSet(new HashSet<>(Arrays.asList(
            "com.anydesk.anydeskandroid",
            "com.anydesk.adcontrol.ad1",
            "com.teamviewer.host.market",
            "com.teamviewer.quicksupport.market",
            "com.carriez.rustdesk",
            "com.whatsapp",                         // During video screen sharing
            "com.google.android.apps.tachyon",      // Google Meet
            "us.zoom.videomeetings",                // Zoom
            "com.microsoft.teams",                  // Teams
            "com.cisco.webex.meetings"              // Webex
    )));

    public interface StateListener {
        void onScreenShareStateChanged(boolean isScreenShared, boolean isInCall);
    }

    private static volatile ScreenShareDetector sInstance;

    public static ScreenShareDetector getInstance(Context context) {
        if (sInstance == null) {
            synchronized (ScreenShareDetector.class) {
                if (sInstance == null) {
                    sInstance = new ScreenShareDetector(context != null ? context.getApplicationContext() : null);
                }
            }
        }
        return sInstance;
    }

    private final Context mContext;
    private final DisplayManager mDisplayManager;
    private final AudioManager mAudioManager;

    private final AtomicBoolean mIsScreenShared = new AtomicBoolean(false);
    private final AtomicBoolean mIsInCall = new AtomicBoolean(false);
    private final AtomicBoolean mIsSimulated = new AtomicBoolean(false);

    private final Set<StateListener> mListeners = new CopyOnWriteArraySet<>();

    public ScreenShareDetector(Context context) {
        mContext = context;
        if (context != null) {
            mDisplayManager = (DisplayManager) context.getSystemService(Context.DISPLAY_SERVICE);
            mAudioManager = (AudioManager) context.getSystemService(Context.AUDIO_SERVICE);
        } else {
            mDisplayManager = null;
            mAudioManager = null;
        }
    }

    public void register() {
        if (mDisplayManager != null) {
            mDisplayManager.registerDisplayListener(this, null);
            checkDisplaysInternal();
        }
        checkAudioStateInternal();
        Log.i(TAG, "[REGISTER] ScreenShareDetector registered. ScreenShared=" + mIsScreenShared.get() + ", InCall=" + mIsInCall.get());
    }

    public void unregister() {
        if (mDisplayManager != null) {
            mDisplayManager.unregisterDisplayListener(this);
        }
        Log.i(TAG, "[UNREGISTER] ScreenShareDetector unregistered.");
    }

    public void addListener(StateListener listener) {
        if (listener != null) {
            mListeners.add(listener);
        }
    }

    public void removeListener(StateListener listener) {
        if (listener != null) {
            mListeners.remove(listener);
        }
    }

    /**
     * Checks if screen mirroring or casting is actively projecting the user's screen.
     */
    public boolean isScreenShared() {
        if (mIsSimulated.get()) {
            return mIsScreenShared.get();
        }
        checkDisplaysInternal();
        return mIsScreenShared.get();
    }

    /**
     * Checks if the device is in an active phone or VoIP call.
     */
    public boolean isInCall() {
        if (mIsSimulated.get()) {
            return mIsInCall.get();
        }
        checkAudioStateInternal();
        return mIsInCall.get();
    }

    /**
     * Critical threat condition: Screen is shared WHILE an active call is ongoing
     * (the textbook signature of AnyDesk / TeamViewer banking impersonation fraud).
     */
    public boolean isScreenSharedDuringCall() {
        return isScreenShared() && isInCall();
    }

    /**
     * Allows test fixtures and dev-harness to set simulated screen share and call states.
     */
    public void setSimulatedState(boolean isScreenShared, boolean isInCall) {
        mIsSimulated.set(true);
        boolean prevShared = mIsScreenShared.getAndSet(isScreenShared);
        boolean prevInCall = mIsInCall.getAndSet(isInCall);
        boolean changed = (prevShared != isScreenShared) || (prevInCall != isInCall);
        if (changed) {
            notifyListeners();
        }
        Log.i(TAG, "[SIMULATED_STATE] ScreenShared=" + isScreenShared + ", InCall=" + isInCall);
    }

    public void clearSimulation() {
        mIsSimulated.set(false);
        checkDisplaysInternal();
        checkAudioStateInternal();
    }

    @Override
    public void onDisplayAdded(int displayId) {
        Log.i(TAG, "[DISPLAY_ADDED] Display ID: " + displayId);
        checkDisplaysInternal();
    }

    @Override
    public void onDisplayRemoved(int displayId) {
        Log.i(TAG, "[DISPLAY_REMOVED] Display ID: " + displayId);
        checkDisplaysInternal();
    }

    @Override
    public void onDisplayChanged(int displayId) {
        checkDisplaysInternal();
    }

    private void checkDisplaysInternal() {
        if (mDisplayManager == null || mIsSimulated.get()) {
            return;
        }

        Display[] displays = mDisplayManager.getDisplays();
        boolean hasVirtualOrMirror = false;

        if (displays != null && displays.length > 1) {
            for (Display display : displays) {
                if (display.getDisplayId() == Display.DEFAULT_DISPLAY) {
                    continue;
                }
                // Check if display is virtual or presentation
                int flags = display.getFlags();
                boolean isPresentation = (flags & Display.FLAG_PRESENTATION) != 0;
                boolean isPrivate = (flags & Display.FLAG_PRIVATE) != 0;

                // Virtual mirror display projection indicator
                if (!isPrivate || isPresentation) {
                    hasVirtualOrMirror = true;
                    Log.w(TAG, "[MIRROR_DETECTED] Secondary display projecting: ID=" + display.getDisplayId() + ", Flags=" + flags);
                    break;
                }
            }
        }

        boolean prev = mIsScreenShared.getAndSet(hasVirtualOrMirror);
        if (prev != hasVirtualOrMirror) {
            notifyListeners();
        }
    }

    private void checkAudioStateInternal() {
        if (mAudioManager == null || mIsSimulated.get()) {
            return;
        }

        int mode = mAudioManager.getMode();
        boolean inCall = (mode == AudioManager.MODE_IN_CALL ||
                          mode == AudioManager.MODE_IN_COMMUNICATION ||
                          mode == AudioManager.MODE_RINGTONE);

        boolean prev = mIsInCall.getAndSet(inCall);
        if (prev != inCall) {
            notifyListeners();
        }
    }

    private void notifyListeners() {
        boolean shared = mIsScreenShared.get();
        boolean inCall = mIsInCall.get();
        for (StateListener listener : mListeners) {
            try {
                listener.onScreenShareStateChanged(shared, inCall);
            } catch (Exception e) {
                Log.e(TAG, "Error in listener callback: " + e.getMessage());
            }
        }
    }
}
