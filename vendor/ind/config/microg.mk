# ==============================================================================
# Ind OS — Sandboxed microG Optional Integration Makefile
# PRD Reference: v1.0.1-BIBLE (Phase 6, Sprint 17)
# ==============================================================================

# Enable Sandboxed microG by default for seamless Indian UPI & Banking app support
IND_ENABLE_MICROG ?= true

ifeq ($(IND_ENABLE_MICROG),true)
    # Enable signature spoofing property for restricted microG daemon
    PRODUCT_SYSTEM_DEFAULT_PROPERTIES += \
        ro.ind.microg.sandboxed=true \
        ro.ind.playintegrity.spoof=true \
        persist.sys.ind.microg=1

    # Copy restricted microg permissions
    PRODUCT_COPY_FILES += \
        vendor/ind/permissions/microg-permissions.xml:$(TARGET_COPY_OUT_SYSTEM_EXT)/etc/permissions/microg-permissions.xml

    # Include open-source microG core packages
    PRODUCT_PACKAGES += \
        GmsCore \
        GsfProxy \
        FakeStore
endif
