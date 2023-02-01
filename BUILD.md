eEMU=qemu-system-aarch64
Qration

## Install latest Rust
distro packages may not work as you need nightly builds

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
Choose default installation

    source "$HOME/.cargo/env"
	
Update your login script  with this

    cargo install cargo-xbuild
    rustup component add rust-src

## Prepare environement
Assuming in $HOME

    mkdir workspace 
	
## Build tools

    cd $HOME/workspace
    git clone https://github.com/fozog/demangle.git
    cd  demangle
    cargo build

------------
# Build and run barekit

## For Qemu (no BIOS)

We assume qemu is installed and operational

    cd $HOME/workspace/
    git clone https://github.com/fozog/barekit.git 
    cd barekit
    make
    ./runon qemu

## For kvmtool (no BIOS)
Assuming you already have built barekit for Qemu.

    cd $HOME/workspace/
    git clone https://github.com/kvmtool/kvmtool.git 
    cd kvmtool
    make
	

update $HOME/workspace/barekit/runon and modify the LKVM variable to reflect you lvm actual path, $HOME/workspace/kvmtool/lkvm if you followed the above procedure

    cd $HOME/workspace/barekit
    ./runon kvmtool

## for Qemu (U-Boot with EFI as BIOS)

	cd $HOME/workspace/
	git clone https://github.com/u-boot/u-boot.git 
	cd u-boot
there is a bug in master related to virtio so, let’s just use 2023.01
	git checkout -b v2023.01 v2023.01
	make qemu_arm64_defconfig
	make menuconfig
	make
	cp u-boot-nodtb.bin ../barekit

Then place u_boot into a flash to be used by Qemu

	cd $HOME/workspace/barekit
	truncate --size 64M run_efi/QEMU_UBOOT.fd
	 dd if=u-boot.bin of=run_efi/QEMU_UBOOT.fd bs=4096 conv=notrunc

Create a virtual disk contaiing the EFI partition

	./run_efi/mkdisk.sh 

Then each time you want to run a new barekit (after make)
	./run_efi/update_disk.sh
	./runon qemu-uboot

for the first time, interupt the boot process by pressing any key at the prompt to setup the boot script

	setenv bootcmd 'virtio scan; load virtio 0:1 $kernel_addr_r /barekit.afx; bootefi $kernel_addr_r $fdtcontroladdr'
	saveenv
	run bootcmd

From now on you can just use

	./run_efi/update_disk.sh
	./runon qemu-uboot

to test the new compiled version. To exit Qemu, type CTRL-A x.

## Build and install for BL33

    cd $HOME/workspace
    git clone https://github.com/ARM-software/arm-trusted-firmware.git
    cd arm-trusted-firmware
    git checkout -bv2.8.0 v2.8.0

Assumes that barekit.afx is already built to be integrated in the fip

    make ARM_LINUX_KERNEL_AS_BL33=1 BL33=../barekit/target/aarch64-unknown-uefi/release/barekit.afx PLAT=qemu all fip
    cd ../barekit
    ./runon qemu-tfa1

Then each time you update barekit:

    make
    cd ../arm-trusted-firmware
    make ARM_LINUX_KERNEL_AS_BL33=1 BL33=../barekit/target/aarch64-unknown-uefi/release/barekit.afx PLAT=qe
    cd -
    ./runon qemu-tfa1


## Build and run for BL32

same as for BL33 except you need to apply a patch:

    cd $HOME/workspace/arm-trusted-firmware
    git apply ../barekit/bl32.patch
    make distclean
    make SPD=tlkd ARM_LINUX_KERNEL_AS_BL33=1 BL33=../barekit/target/aarch64-unknown-uefi/release/barekit.afx BL32=../barekit/target/aarch64-unknown-uefi/release/barekit.afx PLAT=qemu all fip
    cd $HOME/workspace/barekit
    ./runon qemu-tfa1

alternatively you can use U-Boot as BL33 with barekit as BL32 and EFI payload:

    cd $HOME/workspace/arm-trusted-firmware
    make distclean
    make SPD=tlkd BL33=../u-boot/u-boot-nodtb.bin BL32=../barekit/target/aarch64-unknown-uefi/release/barekit.afx PLAT=qemu all fip
    cd $HOME/workspace/barekit
    ./runon qemu-tfa1

## Build and run for RPI4

Create a bootable RPI4 64 bits image (use Raspberry PI Imager tool).
copy barekit.afx in the FAT root directory
update config.txt:

    enable_uart=1
    kernel=barekit.afx

Boot
