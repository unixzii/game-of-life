TARGET_DIR=target
PKG_DIR=pkg
SRC_DIR=src

DEPS=$(SRC_DIR)/lib.rs

$(PKG_DIR): $(DEPS)
	wasm-pack build -m no-install -t no-modules

.PHONY: clean

clean:
	rm -rf $(TARGET_DIR) $(PKG_DIR)