# ==============================================================================
# Ind OS — Device Product Makefile for Motorola Moto G60 (hanoip)
# Target Silicon: Snapdragon 732G (SM7150-AC / Kryo 470 / Adreno 618)
# RAM: 6 GB LPDDR4X | Display: 120 Hz FHD+ (1080 x 2460)
# PRD Reference: v1.0.1-BIBLE (Sprint 16)
# ==============================================================================

# Inherit from Motorola Moto G60 device tree (if building within full tree)
$(call inherit-product-if-exists, device/motorola/hanoip/device.mk)

# Inherit from Ind OS common product configuration
$(call inherit-product, vendor/ind/config/common.mk)

# Product Metadata
PRODUCT_NAME := ind_hanoip
PRODUCT_DEVICE := hanoip
PRODUCT_BRAND := Motorola
PRODUCT_MODEL := moto g(60)
PRODUCT_MANUFACTURER := Motorola

PRODUCT_GMS_CLIENTID_BASE := android-motorola

# Fingerprint and Build Description overrides
PRODUCT_BUILD_PROP_OVERRIDES += \
    PRODUCT_NAME=hanoip \
    TARGET_DEVICE=hanoip
