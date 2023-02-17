run_efi/mount_disk.sh
TAG=$(find target/aarch64-unknown-uefi -maxdepth 1 -type d  | egrep -v '^target/aarch64-unknown-uefi$')
NATURE=$(basename $TAG)
echo cp target/aarch64-unknown-uefi/$NATURE/barekit.afx  run_efi/tmpmount
cp target/aarch64-unknown-uefi/$NATURE/barekit.afx  run_efi/tmpmount
cp target/aarch64-unknown-uefi/$NATURE/barekit.efi  run_efi/tmpmount
run_efi/umount_disk.sh
