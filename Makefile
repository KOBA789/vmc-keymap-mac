TARGET=vmc-keymap

RELEASE_DIR = target/release

APP_NAME = VMCKeyMap.app
APP_TEMPLATE = osx/$(APP_NAME)
APP_DIR = $(RELEASE_DIR)/osx
APP_BINARY = $(RELEASE_DIR)/$(TARGET)
APP_BINARY_DIR = $(APP_DIR)/$(APP_NAME)/Contents/MacOS
APP_EXTRAS_DIR = $(APP_DIR)/$(APP_NAME)/Contents/Resources
APP_ICON=/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericApplicationIcon.icns

vpath $(TARGET) $(RELEASE_DIR)

$(TARGET):
	cargo build --release

app: $(TARGET)
	mkdir -p $(APP_BINARY_DIR)
	mkdir -p $(APP_EXTRAS_DIR)
	cp -fRp $(APP_TEMPLATE) $(APP_DIR)
	cp -fp $(APP_ICON) $(APP_EXTRAS_DIR)
	cp -fp $(APP_BINARY) $(APP_BINARY_DIR)
	touch -r "$(APP_BINARY)" "$(APP_DIR)/$(APP_NAME)"
	codesign --remove-signature "$(APP_DIR)/$(APP_NAME)"
	codesign --force --deep --sign - "$(APP_DIR)/$(APP_NAME)"
	@echo "Created '$(APP_NAME)' in '$(APP_DIR)'"

.PHONY: app $(TARGET)
