LOOP="$(losetup --raw | grep disk.img | cut -d' ' -f1)"
echo "$LOOP" | while read dev
do
	sudo losetup -d $dev
done 
