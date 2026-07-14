APPNAME := barekit
#NATURE := release
NATURE ?= debug
FEATURES := --features early_print
#SETUP: choose ARM exception_level (default 1)
#FEATURES += --features compile-for-el3
STACK_RESERVE := 8192
HEAP_RESERVE := 4096

CARGO_BUILD_CMD := cargo rustc
CARGO_BUILD_TAIL := -- -Clink-arg=/stack:$(STACK_RESERVE) -Clink-arg=/heap:$(HEAP_RESERVE)

ifeq ($(NATURE),release)
	BUILD_TAG := --release
endif

ifeq ($(NATURE),debug)
	# Improve GDB experience for debug builds.
	STACK_RESERVE := 65536
	HEAP_RESERVE := 65536
	CARGO_PROFILE_DEV_DEBUG := CARGO_PROFILE_DEV_DEBUG=2
	CARGO_PROFILE_DEV_OPT_LEVEL := CARGO_PROFILE_DEV_OPT_LEVEL=0
	CARGO_PROFILE_DEV_STRIP := CARGO_PROFILE_DEV_STRIP=none
	# /debug:dwarf asks lld-link to emit DWARF instead of PDB/CodeView.
	CARGO_BUILD_TAIL := -- -Cforce-frame-pointers=yes -Cdebuginfo=2 -Clink-arg=/debug:dwarf -Clink-arg=/stack:$(STACK_RESERVE) -Clink-arg=/heap:$(HEAP_RESERVE)
endif

BUILDDIR := target/aarch64-unknown-uefi

TARGET := $(BUILDDIR)/$(NATURE)

all:	$(BUILDDIR)/stub.exe $(BUILDDIR)/copy_to_secmem.bin $(TARGET)/$(APPNAME).afx

.PHONY:	clean
.PHONY:	$(TARGET)/$(APPNAME).efi

$(TARGET)/$(APPNAME).afx:	$(TARGET)/$(APPNAME).efi $(BUILDDIR)/stub.bin
	@./replace_stub $(TARGET)/$(APPNAME).efi $(BUILDDIR)/stub.bin
	@cp $(TARGET)/$(APPNAME).afx /private/tftpboot

$(BUILDDIR)/copy_to_secmem.bin:	src/copy_to_secmem.s
	@as src/copy_to_secmem.s -o $(BUILDDIR)/copy_to_secmem.elf
	@./extract_text $(BUILDDIR)/copy_to_secmem.elf $(BUILDDIR)/copy_to_secmem.bin

$(BUILDDIR)/stub.bin:	src/stub.s
	@echo building stub.bin
	@mkdir -p $(BUILDDIR)
	@gcc src/stub.s -c -o $(BUILDDIR)/stub.o
	@./extract_text $(BUILDDIR)/stub.o $(BUILDDIR)/stub.bin

$(TARGET)/$(APPNAME).efi:	src/*.rs
	$(CARGO_PROFILE_DEV_DEBUG) $(CARGO_PROFILE_DEV_OPT_LEVEL) $(CARGO_PROFILE_DEV_STRIP) $(CARGO_BUILD_CMD) $(BUILD_TAG) $(FEATURES) --target=aarch64-unknown-uefi $(CARGO_BUILD_TAIL)
	./stage_map $(APPNAME).map > $(APPNAME).mapsym

run_efi/flash.bin: $(TARGET)/$(APPNAME).afx 
	@./stage_flash run_efi/flash.bin 0x0e000000
	
clean:
	@rm -f $(BUILDDIR)/stub.bin $(TARGET)/$(APPNAME).efi $(TARGET)/$(APPNAME).afx $(APPNAME).mapsym $(APPNAME).map  $(BUILDDIR)/copy_to_secmem.bin $(BUILDDIR)/stub.o
	@mkdir -p $(BUILDDIR)

