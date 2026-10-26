# ==============================================================================
# Ind OS — Common Product Configuration
# Reference: Motorola Moto G60 (hanoip) / LineageOS 22.x / AOSP 15
# PRD Reference: v1.0.1-BIBLE (Sprint 16 — Zero-Bloat Product Makefile)
# ==============================================================================

# Ind OS Identity & Branding
PRODUCT_BRAND := IndOS
PRODUCT_SYSTEM_NAME := IndOS
PRODUCT_VERSION_MAJOR := 1
PRODUCT_VERSION_MINOR := 0
PRODUCT_VERSION_MAINTENANCE := 0
IND_VERSION := v$(PRODUCT_VERSION_MAJOR).$(PRODUCT_VERSION_MINOR).$(PRODUCT_VERSION_MAINTENANCE)-BIBLE

PRODUCT_SYSTEM_DEFAULT_PROPERTIES += \
    ro.ind.version=$(IND_VERSION) \
    ro.ind.founder=Sudhanshu_Raj_Singh \
    ro.ind.ai.native=true \
    ro.ind.airgap.enforced=true \
    ro.ind.release.type=OFFICIAL \
    ro.ind.target.device=hanoip \
    ro.build.display.id=Ind\ OS\ $(IND_VERSION) \
    ro.lineage.build.version=$(IND_VERSION) \
    ro.control_privapp_permissions=enforce

# Include Ind OS Core Privileged Subsystems in ROM build
PRODUCT_PACKAGES += \
    IndNeuralCore \
    IndZeroTouch \
    IndSurakshaShield \
    IndLauncher \
    IndDiagnosticClient

# Copy Whitelisted Privileged Permissions XML
PRODUCT_COPY_FILES += \
    vendor/ind/permissions/privapp-permissions-ind.xml:$(TARGET_COPY_OUT_SYSTEM_EXT)/etc/permissions/privapp-permissions-ind.xml

# Custom Ind OS Boot Animation (1080x2460 60fps)
PRODUCT_COPY_FILES += \
    vendor/ind/media/bootanimation.zip:$(TARGET_COPY_OUT_PRODUCT)/media/bootanimation.zip

# Device Package Overlays (About Phone & Settings branding)
DEVICE_PACKAGE_OVERLAYS += \
    vendor/ind/overlay/common

# Register Custom SELinux Policies
BOARD_VENDOR_SEPOLICY_DIRS += vendor/ind/sepolicy

# Purge Stock / Lineage Bloatware (PRD Invariant 1: Keep OS idle memory <= 1150 MB)
# Removes default browser, stock music, stock audio recorder, stock calendar, telemetry updater
PRODUCT_PACKAGES_REMOVE += \
    Jelly \
    Eleven \
    LineageUpdater \
    Etar \
    Recorder \
    Snap \
    PhotoTable \
    Exchange2
