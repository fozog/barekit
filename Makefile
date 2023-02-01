APPNAME := barekit
NATURE := debug
FEATURES := --features early_print

ifeq ($(NATURE),release)
	BUILD_TAG := --release
endif

BUILDDIR := target/aarch64-unknown-uefi

TARGET := $(BUILDDIR)/$(NATURE)

all:	$(BUILDDIR)/stub.exe $(TARGET)/$(APPNAME).afx

.PHONY:	clean
.PHONY:	$(TARGET)/$(APPNAME).efi

$(TARGET)/$(APPNAME).afx:	$(TARGET)/$(APPNAME).efi $(BUILDDIR)/stub.exe
	@./replace_stub $(TARGET)/$(APPNAME).efi $(BUILDDIR)/stub.exe


$(BUILDDIR)/stub.exe:	src/stub.s
	@mkdir -p $(BUILDDIR)
	@gcc src/stub.s -c -o $(BUILDDIR)/stub.o
	@./extract_text $(BUILDDIR)/stub.o $(BUILDDIR)/stub.exe

$(TARGET)/$(APPNAME).efi:	src/*.rs aarch64-unknown-uefi.json
	cargo xbuild $(BUILD_TAG) $(FEATURES) --target=aarch64-unknown-uefi.json 
	@./stage_map $(APPNAME).map > $(APPNAME).mapsym

clean:
	@rm $(BUILDDIR)/stub.exe $(TARGET)/$(APPNAME).efi $(TARGET)/$(APPNAME).afx $(APPNAME).mapsym $(APPNAME).map
	@mkdir -p $(BUILDDIR)

