APPNAME := barekit
NATURE := release
FEATURES := --features early_print
#SETUP: choose exception level (default 1)
#FEATURES += --features compile-for-el3

ifeq ($(NATURE),release)
	BUILD_TAG := --release
endif

BUILDDIR := target/aarch64-unknown-uefi

TARGET := $(BUILDDIR)/$(NATURE)

all:	$(BUILDDIR)/stub.exe $(BUILDDIR)/copy_to_secmem.bin $(TARGET)/$(APPNAME).afx

.PHONY:	clean
.PHONY:	$(TARGET)/$(APPNAME).efi

$(TARGET)/$(APPNAME).afx:	$(TARGET)/$(APPNAME).efi $(BUILDDIR)/stub.exe
	@./replace_stub $(TARGET)/$(APPNAME).efi $(BUILDDIR)/stub.exe
	@cp $(TARGET)/$(APPNAME).afx /private/tftpboot

$(BUILDDIR)/copy_to_secmem.bin:	src/copy_to_secmem.s
	@as src/copy_to_secmem.s -o $(BUILDDIR)/copy_to_secmem.elf
	@./extract_text $(BUILDDIR)/copy_to_secmem.elf $(BUILDDIR)/copy_to_secmem.bin

$(BUILDDIR)/stub.exe:	src/stub.s
	@echo building stub.exe
	@mkdir -p $(BUILDDIR)
	@gcc src/stub.s -c -o $(BUILDDIR)/stub.o
	@./extract_text $(BUILDDIR)/stub.o $(BUILDDIR)/stub.exe

$(TARGET)/$(APPNAME).efi:	src/*.rs
	cargo build $(BUILD_TAG) $(FEATURES) --target=aarch64-unknown-uefi
	./stage_map $(APPNAME).map > $(APPNAME).mapsym

run_efi/flash.bin: $(TARGET)/$(APPNAME).afx 
	@./stage_flash run_efi/flash.bin 0x0e000000
	
clean:
	@rm -f $(BUILDDIR)/stub.exe $(TARGET)/$(APPNAME).efi $(TARGET)/$(APPNAME).afx $(APPNAME).mapsym $(APPNAME).map  $(BUILDDIR)/copy_to_secmem.bin $(BUILDDIR)/stub.o
	@mkdir -p $(BUILDDIR)

