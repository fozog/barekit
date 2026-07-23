APPNAME := barekit
#NATURE := release
NATURE ?= debug
FEATURES := --features early_print
#SETUP: choose ARM exception_level (default 1)
#FEATURES += --features compile-for-el3

BUILDDIR := target/aarch64-unknown-uefi-nofp
TARGET   := $(BUILDDIR)/$(NATURE)

CARGO_BUILD_CMD  := cargo rustc
STACK_RESERVE    := 8192
HEAP_RESERVE     := 4096
CARGO_BUILD_TAIL := -- -Clink-arg=/stack:$(STACK_RESERVE) -Clink-arg=/heap:$(HEAP_RESERVE) -Clink-arg=/map:$(TARGET)/barekit.map -Clink-arg=/PDB:$(TARGET)/barekit.pdb

ifeq ($(NATURE),release)
	BUILD_TAG := --release
endif

ifeq ($(NATURE),debug)
	STACK_RESERVE := 65536
	HEAP_RESERVE  := 65536
	CARGO_PROFILE_DEV_DEBUG     := CARGO_PROFILE_DEV_DEBUG=2
	CARGO_PROFILE_DEV_OPT_LEVEL := CARGO_PROFILE_DEV_OPT_LEVEL=0
	CARGO_PROFILE_DEV_STRIP     := CARGO_PROFILE_DEV_STRIP=none
	CARGO_BUILD_TAIL := -- -Cforce-frame-pointers=yes -Cdebuginfo=2 -Clink-arg=/stack:$(STACK_RESERVE) -Clink-arg=/heap:$(HEAP_RESERVE) -Clink-arg=/map:$(TARGET)/barekit.map -Clink-arg=/PDB:$(TARGET)/barekit.pdb
endif

all:	$(BUILDDIR)/stub.bin $(BUILDDIR)/copy_to_secmem.bin $(TARGET)/$(APPNAME).afx

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
	@as -x assembler-with-cpp src/stub.s -o $(BUILDDIR)/stub.o
	@./extract_text $(BUILDDIR)/stub.o $(BUILDDIR)/stub.bin

$(TARGET)/$(APPNAME).efi:	src/*.rs
	$(CARGO_PROFILE_DEV_DEBUG) $(CARGO_PROFILE_DEV_OPT_LEVEL) $(CARGO_PROFILE_DEV_STRIP) $(CARGO_BUILD_CMD) $(BUILD_TAG) $(FEATURES) --target=aarch64-unknown-uefi-nofp.json $(CARGO_BUILD_TAIL)
	./stage_map $(TARGET)/barekit.map > $(TARGET)/$(APPNAME).mapsym

run_efi/flash.bin: $(TARGET)/$(APPNAME).afx 
	@./stage_flash run_efi/flash.bin 0x0e000000
	
clean:
	@rm -f $(BUILDDIR)/stub.bin $(TARGET)/$(APPNAME).efi $(TARGET)/$(APPNAME).afx $(TARGET)/$(APPNAME).mapsym $(TARGET)/barekit.map $(TARGET)/barekit.pdb $(BUILDDIR)/copy_to_secmem.bin $(BUILDDIR)/stub.o baremetal_init.o
	@mkdir -p $(BUILDDIR)

