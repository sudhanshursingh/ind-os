# ==============================================================================
# Ind OS — Common Product Configuration
# Reference: Motorola Moto G60 (hanoip) / LineageOS 22.x / AOSP 15
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
    ro.control_privapp_permissions=enforce

# Include Ind OS Core Privileged Subsystems in ROM build
PRODUCT_PACKAGES += \
    IndNeuralCore \
    IndZeroTouch \
    IndSurakshaShield \
    IndLauncher

# Copy Whitelisted Privileged Permissions XML
PRODUCT_COPY_FILES += \
    vendor/ind/permissions/privapp-permissions-ind.xml:$(TARGET_COPY_OUT_SYSTEM_EXT)/etc/permissions/privapp-permissions-ind.xml

# Register Custom SELinux Policies
BOARD_VENDOR_SEPOLICY_DIRS += vendor/ind/sepolicy

# Purge Stock / Lineage Bloatware (D10 / Invariant 1: Keep OS memory <= 1150 MB)
PRODUCT_PACKAGES_REMOVE += \
    Jelly \
    Eleven \
    LineageUpdater \
    Etar \
    Recorder
