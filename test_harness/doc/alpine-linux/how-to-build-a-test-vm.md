# How to build a test virtual machine (VM)

## ISO and VM setup

The following describes how to build an Alpine Linux VM compatible with [vmtest][1]:

- Download an Alpine Linux ISO from the [download page][2]. In this example we
  choose the `alpine-virt-3.19.1-x86_64.iso` image.
- Download the checksum file associated with the ISO `alpine-virt-3.19.1-x86_64.iso.sha256`,
- Check the ISO's integrity. From a terminal, in the directory containing the
  ISO and the checksum file, issue the following command:

```console
sha256sum -c alpine-virt-3.19.1-x86_64.iso.sha256

# Expected output
alpine-virt-3.19.1-x86_64.iso: OK
```

Time now to create a VM image. Assuming you have QEMU installed, issue the
following command to create a disk image.

```shell
qemu-img create -f qcow2 alpine-linux-3.19.1-x86_64-4GB.qcow2 4G
```

Once this step complete, we can start the VM for system installation.

```shell
 qemu-kvm -serial mon:stdio -nographic -enable-kvm -m 4G -cpu host -smp 2 \
 -device nec-usb-xhci,id=xhci -netdev user,id=net0,hostfwd="tcp::2222-:22" \
 -device e1000,netdev=net0  \
 -drive if=none,id=cdrom,format=raw,readonly=on,file="alpine-virt-3.19.1-x86_64.iso" \
 -device ide-cd,drive=cdrom,bootindex=2 \
 -drive if=none,id=maindrive,format=qcow2,file="alpine-linux-3.19.1-x86_64-4GB.qcow2" \
 -device ide-hd,drive=maindrive,bootindex=1

```

After a few seconds, you should be greeted with the following message.

```console
Welcome to Alpine Linux 3.19
Kernel 6.6.14-0-virt on an x86_64 (/dev/ttyS0)

localhost login:
```

Type `root` then hit `Enter`. You should now see the following prompt...

```console
The Alpine Wiki contains a large amount of how-to guides and general
information about administrating Alpine systems.
See <https://wiki.alpinelinux.org/>.

You can setup the system with the command: setup-alpine

You may change this message by editing /etc/motd.

localhost:~#
```

...meaning you are logged in.

## System installation

We will now start the installation process. Type `setup-alpine` then hit
`Enter`.

The program will ask you a series of questions reproduced (with example
answers) below.

```console
 ALPINE LINUX INSTALL
----------------------

 Hostname
----------
Enter system hostname (fully qualified form, e.g. 'foo.example.org') [localhost] alpine-cargo-vmtest

 Interface
-----------
Available interfaces are: eth0.
Enter '?' for help on bridges, bonding and vlans.
Which one do you want to initialize? (or '?' or 'done') [eth0] <hit Enter>
Ip address for eth0? (or 'dhcp', 'none', '?') [dhcp] <hit Enter>
Do you want to do any manual network configuration? (y/n) [n] <hit Enter>
udhcpc: started, v1.36.1
udhcpc: broadcasting discover
udhcpc: broadcasting select for 10.0.2.15, server 10.0.2.2
udhcpc: lease of 10.0.2.15 obtained from 10.0.2.2, lease time 86400

 Root Password
---------------
Changing password for root
New password: alpine cargo vmtest
Retype password: alpine cargo vmtest
passwd: password for root changed by root

 Timezone
----------
Africa/            Egypt              Iran               Poland
America/           Eire               Israel             Portugal
Antarctica/        Etc/               Jamaica            ROC
Arctic/            Europe/            Japan              ROK
Asia/              Factory            Kwajalein          Singapore
Atlantic/          GB                 Libya              Turkey
Australia/         GB-Eire            MET                UCT
Brazil/            GMT                MST                US/
CET                GMT+0              MST7MDT            UTC
CST6CDT            GMT-0              Mexico/            Universal
Canada/            GMT0               NZ                 W-SU
Chile/             Greenwich          NZ-CHAT            WET
Cuba               HST                Navajo             Zulu
EET                Hongkong           PRC                leap-seconds.list
EST                Iceland            PST8PDT            posixrules
EST5EDT            Indian/            Pacific/           right/

Which timezone are you in? [UTC] <hit Enter>

 * Seeding random number generator ...
 * Saving 256 bits of creditable seed for next boot
 [ ok ]
 * Starting busybox acpid ...
 [ ok ]
 * Starting busybox crond ...
 [ ok ]

 Proxy
-------
HTTP/FTP proxy URL? (e.g. 'http://proxy:8080', or 'none') [none] <hit Enter>

 APK Mirror
------------
 (f)    Find and use fastest mirror
 (s)    Show mirrorlist
 (r)    Use random mirror
 (e)    Edit /etc/apk/repositories with text editor
 (c)    Community repo enable
 (skip) Skip setting up apk repositories

Enter mirror number or URL: [1] <hit Enter>

Added mirror dl-cdn.alpinelinux.org
Updating repository indexes... done.

 User
------
Setup a user? (enter a lower-case loginname, or 'no') [no] <hit Enter>
Which ssh server? ('openssh', 'dropbear' or 'none') [openssh] <hit Enter>
Allow root ssh login? ('?' for help) [prohibit-password] yes
Enter ssh key or URL for root (or 'none') [none] <hit Enter>
 * service sshd added to runlevel default
 * Caching service dependencies ...
 [ ok ]
ssh-keygen: generating new host keys: RSA ECDSA ED25519
 * Starting sshd ...
 [ ok ]

 Disk & Install
----------------
Available disks are:
  fd0   (0.0 GB  )
  sda   (4.3 GB ATA      QEMU HARDDISK   )

Which disk(s) would you like to use? (or '?' for help or 'none') [none] sda

The following disk is selected:
  sda   (4.3 GB ATA      QEMU HARDDISK   )

How would you like to use it? ('sys', 'data', 'crypt', 'lvm' or '?' for help) [?] sys

WARNING: The following disk(s) will be erased:
  sda   (4.3 GB ATA      QEMU HARDDISK   )

WARNING: Erase the above disk(s) and continue? (y/n) [n] y
Creating file systems...
Installing system on /dev/sda3:
/mnt/boot is device /dev/sda1
100% ████████████████████████████████████████████
==> initramfs: creating /boot/initramfs-virt for 6.6.14-0-virt
/boot is device /dev/sda1

Installation is complete. Please reboot.
alpine-cargo-vmtest:~#
```

Enter `reboot` followed by the `Enter` key. Your virtual machine should
restart.

After a few seconds your screen should display the message below:

```console
Welcome to Alpine Linux 3.19
Kernel 6.6.14-0-virt on an x86_64 (/dev/ttyS0)

alpine-cargo-vmtest login:
```

Enter `root`, hit the `Enter` key, then input the password for `root` defined
at installation time (`alpine cargo vmtest` in this example).

## Package installation

We will now configure the repositories used by the `APK` package manager.

- Edit the `/etc/apk/repositories` file.

```shell
vi /etc/apk/repositories
```

- Add the community repository to the list of repositories to search for new
  packages, then save your changes, and exit.

```diff
 #/media/cdrom/apks
 http://dl-cdn.alpinelinux.org/alpine/v3.19/main
-#http://dl-cdn.alpinelinux.org/alpine/v3.19/community
+http://dl-cdn.alpinelinux.org/alpine/v3.19/community
```

- Update `APK`'s cache.

```shell
apk update
```

Expected output:

```console
fetch http://dl-cdn.alpinelinux.org/alpine/v3.19/community/x86_64/APKINDEX.tar.gz
v3.19.1-26-g2c7dab72cd7 [http://dl-cdn.alpinelinux.org/alpine/v3.19/main]
v3.19.1-28-g2f20f968460 [http://dl-cdn.alpinelinux.org/alpine/v3.19/community]
OK: 22982 distinct packages available
```

We can now install a few packages we will need later.

```shell
apk add bash bat cargo qemu-guest-agent qemu-guest-agent-openrc util-linux vim
```

We will also install [`cargo-nextest`][3], which will run our Rust development
test suite.

```shell
cargo install nextest
```

Expected output:

```console
    Updating crates.io index
 Downloading crates ...
  Downloaded nextest v0.0.0
  Installing nextest v0.0.0
    Updating crates.io index
   Compiling nextest v0.0.0
    Finished release [optimized] target(s) in 3.82s
  Installing /root/.cargo/bin/nextest
   Installed package `nextest v0.0.0` (executable `nextest`)
warning: be sure to add `/root/.cargo/bin` to your PATH to be able to run the installed binaries
```

To add `/root/.cargo/bin` to your `PATH` issue the following command.

```shell
echo 'export PATH="$PATH:/root/.cargo/bin"' >> /root/.profile
```

## System configuration

### vmtest

`vmtest` has a few dependencies, stated on its [description page][1], that must
be satisfied to run correctly. The virtual machine image must have the
`qemu-guest-agent` utility installed, and have kernel 9p file system support.

We already installed `qemu-guest-agent` in the previous section. We just have
to add `qemu-guest-agent` to the list of services to activate at boot.

```shell
rc-update add qemu-guest-agent
```

Output:

```console
 * service qemu-guest-agent added to runlevel default
```

Now, we need to check that the following kernel options are set:

- `CONFIG_VIRTIO=y`
- `CONFIG_VIRTIO_PCI=y`
- `CONFIG_VIRTIO_CONSOLE=y`
- `CONFIG_NET_9P=y`
- `CONFIG_NET_9P_VIRTIO=y`
- `CONFIG_9P_FS=y`

To that end, issue the following command:

```shell
cat /boot/config-virt | grep CONFIG_VIRTIO
```

Output:

```console
CONFIG_VIRTIO_VSOCKETS=m
CONFIG_VIRTIO_VSOCKETS_COMMON=m
CONFIG_VIRTIO_BLK=m
CONFIG_VIRTIO_NET=m
CONFIG_VIRTIO_CONSOLE=y
CONFIG_VIRTIO_ANCHOR=y
CONFIG_VIRTIO=y
CONFIG_VIRTIO_PCI_LIB=y
CONFIG_VIRTIO_PCI_LIB_LEGACY=y
CONFIG_VIRTIO_MENU=y
CONFIG_VIRTIO_PCI=y
CONFIG_VIRTIO_PCI_LEGACY=y
CONFIG_VIRTIO_VDPA=m
# CONFIG_VIRTIO_PMEM is not set
CONFIG_VIRTIO_BALLOON=m
# CONFIG_VIRTIO_MEM is not set
CONFIG_VIRTIO_INPUT=m
CONFIG_VIRTIO_MMIO=y
CONFIG_VIRTIO_MMIO_CMDLINE_DEVICES=y
CONFIG_VIRTIO_DMA_SHARED_BUFFER=m
CONFIG_VIRTIO_IOMMU=m
CONFIG_VIRTIO_FS=m
```

We see that `CONFIG_VIRTIO`, `CONFIG_VIRTIO_PCI`, `CONFIG_VIRTIO_CONSOLE` are set.

Let's check 9p file system support.

```shell
cat /boot/config-virt | grep _9P
```

Output:

```console
CONFIG_NET_9P=m
CONFIG_NET_9P_FD=m
CONFIG_NET_9P_VIRTIO=m
CONFIG_NET_9P_XEN=m
# CONFIG_NET_9P_DEBUG is not set
CONFIG_9P_FS=m
CONFIG_9P_FSCACHE=y
CONFIG_9P_FS_POSIX_ACL=y
# CONFIG_9P_FS_SECURITY is not set
```

9p file system support is set as modules, instead of being compiled into the
kernel. To check if the modules are already loaded, execute the command:

```shell
lsmod | grep 9p
```

We notice the output is empty, the modules were not loaded automatically. We
will have to setup a configuration file to load them at boot.

To find the modules' names, issue the following command.

```shell
find /lib/modules/$(uname -r) -type f -name '*.ko*' | grep 9p
```

Output:

```console
/lib/modules/6.6.14-0-virt/kernel/net/9p/9pnet.ko.gz
/lib/modules/6.6.14-0-virt/kernel/net/9p/9pnet_fd.ko.gz
/lib/modules/6.6.14-0-virt/kernel/net/9p/9pnet_virtio.ko.gz
/lib/modules/6.6.14-0-virt/kernel/net/9p/9pnet_xen.ko.gz
/lib/modules/6.6.14-0-virt/kernel/fs/9p/9p.ko.gz
```

Sometimes loading a module will bring along all its dependencies. Let's start
by manually loading the module `9p` (no need to add the `.ko.gz` suffix)...

```shell
modprobe 9p
```

...let's now look at the modules that it brought along...

```shell
lsmod | grep 9p
```

Output:

```console
9p                     90112  0
9pnet                 114688  1 9p
fscache               401408  1 9p
netfs                  69632  2 9p,fscache
```

...three other modules were loaded with it. We notice that
`/lib/modules/6.6.14-0-virt/kernel/net/9p/9pnet_virtio.ko.gz` was not loaded.
Since `9pnet_virtio` is required by `vmtest`, let's add it.

```shell
modprobe 9pnet_virtio
lsmod | grep 9p
```

Output:

```console
9pnet_virtio           20480  0
9p                     90112  0
9pnet                 114688  2 9p,9pnet_virtio
fscache               401408  1 9p
netfs                  69632  2 9p,fscache
```

With this, the `vmtest requirements are satisfied.

To ensure these modules are automatically loaded after each reboot, we add a
 `9pfs.conf` configuration file to the `/etc/modules-load.d` directory.

```shell
printf "%s\n%s\n" 9p 9pnet_virtio >> /etc/modules-load.d/9pfs.conf
```

### `root` autologin

To not have to log in after each VM reboot, we activate the autologin feature
provided by `agetty` for terminal consoles.

```shell
ln -s /etc/init.d/agetty /etc/init.d/agetty.tty1
ln -s /etc/init.d/agetty /etc/init.d/agetty.ttyS0

cp /etc/conf.d/agetty /etc/conf.d/agetty.tty1
```

Edit `/etc/conf.d/agetty.tty1`...

```shell
vim /etc/conf.d/agetty.tty1
```

...then add the following modifications.

```diff
 #baud=""

 # set the terminal type
-#term_type="linux"
+term_type="linux"

 # extra options to pass to agetty for this port
-#agetty_options=""
+agetty_options="--autologin root --noclear"
```

After saving the changes, copy the configuration file to apply it to `ttyS0`.

```shell
cp /etc/conf.d/agetty.tty1 /etc/conf.d/agetty.ttyS0
```

Now, edit `/etc/inittab`...

```shell
vim /etc/inittab
```

...then comment out the following lines.

```diff
 # Set up a couple of getty's
-tty1::respawn:/sbin/getty 38400 tty1
+#tty1::respawn:/sbin/getty 38400 tty1
 tty2::respawn:/sbin/getty 38400 tty2
 tty3::respawn:/sbin/getty 38400 tty3
 tty4::respawn:/sbin/getty 38400 tty4

 # enable login on alternative console
-ttyS0::respawn:/sbin/getty -L 0 ttyS0 vt100
+#ttyS0::respawn:/sbin/getty -L 0 ttyS0 vt100
```

Now activate the associated services.

```shell
rc-update add agetty.tty1
rc-update add agetty.ttyS0
```

Output:

```console
 * service agetty.tty1 added to runlevel default
 * service agetty.ttyS0 added to runlevel default
```

As the last step, we modify the console greetings to display the password for the
`root` user, so as not to forget it since we will not use it as much as usual.

Apply the following changes to `/etc/motd`:

```diff
-Welcome to Alpine!
-
-The Alpine Wiki contains a large amount of how-to guides and general
-information about administrating Alpine systems.
-See <https://wiki.alpinelinux.org/>.
-
-You can setup the system with the command: setup-alpine
-
-You may change this message by editing /etc/motd.
-
+root password: "alpine cargo vmtest"
```

With this, our work is done! We can now exit QEMU with the keyboard shortcut
`Ctrl-a x`.

[1]: https://crates.io/crates/vmtest
[2]: https://www.alpinelinux.org/downloads/
[3]: https://crates.io/crates/cargo-nextest
