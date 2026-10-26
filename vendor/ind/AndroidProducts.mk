# ==============================================================================
# Ind OS — AndroidProducts.mk
# Registers Ind OS lunch targets for Motorola Moto G60 (hanoip)
# PRD Reference: v1.0.1-BIBLE (Sprint 16)
# ==============================================================================

PRODUCT_MAKEFILES := \
    $(LOCAL_DIR)/products/ind_hanoip.mk

COMMON_LUNCH_CHOICES := \
    ind_hanoip-userdebug \
    ind_hanoip-user
