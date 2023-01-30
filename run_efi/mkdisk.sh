dd if=/dev/zero of=disk.img bs=512 count=16389
parted --script disk.img mklabel gpt mkpart primary 1Mib 7Mib set 1 esp on
sudo losetup -Pf disk.img
LOOP=$(losetup --raw | grep disk.img | cut -d' ' -f1)
sudo mkfs -t vfat ${LOOP}p1
