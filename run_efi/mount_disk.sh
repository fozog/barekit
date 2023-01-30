sudo losetup -Pf run_efi/disk.img
LOOP=$(losetup --raw | grep disk.img | cut -d' ' -f1)
sudo mount -t vfat -o uid=$(whoami) ${LOOP}p1 run_efi/tmpmount
