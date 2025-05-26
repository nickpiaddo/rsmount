// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! High-level API to mount/unmount devices.
//!
#![cfg_attr(doc,
    cfg_attr(all(),
        doc = ::embed_doc_image::embed_image!( "fig-01", "assets/diagrams/svg/fig01-initial-namespace.svg"),
        doc = ::embed_doc_image::embed_image!("fig-02", "assets/diagrams/svg/fig02-new-namespace.svg"),
        doc = ::embed_doc_image::embed_image!("fig-03", "assets/diagrams/svg/fig03-no-propagation.svg"),
        doc = ::embed_doc_image::embed_image!("fig-04", "assets/diagrams/svg/fig04-ns1-private-mount-point.svg"),
        doc = ::embed_doc_image::embed_image!("fig-05", "assets/diagrams/svg/fig05-shared-mnt-peer-group.svg"),
        doc = ::embed_doc_image::embed_image!("fig-06", "assets/diagrams/svg/fig06-mnt-usbdisk-peer-group.svg"),
        doc = ::embed_doc_image::embed_image!("fig-07", "assets/diagrams/svg/fig07-shared-mnt-3-namespaces.svg"),
        doc = ::embed_doc_image::embed_image!("fig-08", "assets/diagrams/svg/fig08-ns2-slave-mnt.svg"),
        doc = ::embed_doc_image::embed_image!("fig-09", "assets/diagrams/svg/fig09-shared-usbdisk-ns2-slave-mnt.svg"),
        doc = ::embed_doc_image::embed_image!("fig-10", "assets/diagrams/svg/fig10-ns2-slave-mnt-private-usbdisk.svg"),
        doc = ::embed_doc_image::embed_image!("fig-11", "assets/diagrams/svg/fig11-ns1-ns2-shared-mnt.svg"),
        doc = ::embed_doc_image::embed_image!("fig-12", "assets/diagrams/svg/fig12-ns2-slave-shared-mnt.svg"),
        doc = ::embed_doc_image::embed_image!("fig-13", "assets/diagrams/svg/fig13-ns2-ns3-slave-shared.svg"),
        doc = ::embed_doc_image::embed_image!("fig-14", "assets/diagrams/svg/fig14-ns1-usbdisk-ns2-ns3-slave-shared.svg"),
        doc = ::embed_doc_image::embed_image!("fig-15", "assets/diagrams/svg/fig15-ns2-usbdisk-ns2-ns3-slave-shared.svg"),
        doc = ::embed_doc_image::embed_image!("fig-16", "assets/diagrams/svg/fig16-ns1-shared-ns2-private-mnt.svg"),
        ))]
//! ## Description
//!
//! On Unix systems, files are organised around a file tree with `/` as its root. Files may be
//! spread out over several local or network devices, requiring specialised functions to connect
//! them to the file tree.
//!
//! The `mount` module provides the necessary tools:
//! - to attach, or detach, file systems found on a device to a node of the file tree (usually
//!   called *mount point*),
//! - to make the contents of a device accessible at different nodes of the file tree (called *bind
//!   mounts*),
//! - to move the location of a mount point in the tree,
//! - to create mirrors of a mount point that will propagate mount/unmount events within any of
//!   its mirror group.
//!
//! The [`Mount`] struct is the main entry-point to perform the actions outlined above.
//!
//! ## Examples
//!
//! To mount/unmount a device, a user needs to have the necessary permissions by either being the
//! `root` user, or being authorized by the `root` user to perform the operations described below.
//!
//! **Note:** When configuring a [`Mount`] object, all the examples below use the
//! [`MountBuilder::dry_run`] function to simulate mount/unmount operations; remove it from code
//! listings to effectively perform the actions described.
//!
//! ### Mount a local device
//!
//! The following code shows how to attach a local device to the file tree. It tells the kernel to
//! mount the `ext4` file system found on `/dev/vda` at the `/mnt` directory.
//!
//! ```
//! use rsmount::core::device::BlockDevice;
//! use rsmount::core::flags::MountFlag;
//! use rsmount::core::fs::FileSystem;
//! use rsmount::mount::Mount;
//!
//! fn main() -> rsmount::Result<()> {
//!     // Configure the `Mount` struct.
//!     let block_device: BlockDevice = "/dev/vda".parse()?;
//!     let mut mount = Mount::builder()
//!         // Device to mount.
//!         .source(block_device)
//!         // Location of the mount point in the file tree.
//!         .target("/mnt")
//!         // Do not allow writing to the file system while it is mounted.
//!         .mount_flags(vec![MountFlag::ReadOnly])
//!         // Gives a hint about the file system used by the device (optional).
//!         .file_system(FileSystem::Ext4)
//!         // Skips all mount source preparation, mount option analysis, and the actual mounting
//!         // process.
//!         .dry_run()
//!         .build()?;
//!
//!     // Mount `/dev/vda` at `/mnt`.
//!     mount.mount_device()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Mount a network device
//!
//! You can also attach a network device.
//!
//! ```
//! use rsmount::core::device::NFS;
//! use rsmount::core::flags::MountFlag;
//! use rsmount::core::fs::FileSystem;
//! use rsmount::mount::Mount;
//!
//! fn main() -> rsmount::Result<()> {
//!     // Configure the `Mount` struct.
//!     let net_device: NFS = "knuth.cwi.nl:/nfs/share".parse()?;
//!     let mut mount = Mount::builder()
//!         // Device to mount.
//!         .source(net_device)
//!         // Location of the mount point in the file tree.
//!         .target("/net/share")
//!         // Do not allow writing to the file system while it is mounted.
//!         .mount_flags(vec![MountFlag::ReadOnly])
//!         // Gives a hint about the file system used by the device (optional).
//!         .file_system(FileSystem::NFS)
//!         // Skips all mount source preparation, mount option analysis, and the actual mounting
//!         // process.
//!         .dry_run()
//!         .build()?;
//!
//!     // Mount `knuth.cwi.nl:/nfs/share` at `/net/share`.
//!     mount.mount_device()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Mount a device in `/etc/fstab`
//!
//! In the examples above, we provided values for the `source` and `target` configuration methods.
//! If we however set only one of them, the resulting `Mount` will look for the missing parameter
//! in `/etc/fstab`.
//!
//! Assuming the following content in `/etc/fstab`...
//!
//! ```text
//! # /etc/fstab
//! # Alpine Linux 3.19 (installed from alpine-virt-3.19.1-x86_64.iso)
//! #
//!
//! UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f  /      ext4  rw,relatime        0 1
//! UUID=07aae9ba-12a1-4325-8b16-2ee214a6d7fd  /boot  ext4  noauto,rw,relatime 0 2
//! UUID=b9d72af2-f231-4cf8-9d0a-ba19e94a5087  swap   swap  defaults           0 0
//!
//! /dev/cdrom    /media/cdrom  iso9660 noauto,ro       0 0
//! /dev/usbdisk  /media/usb    vfat    noauto          0 0
//! none          /tmp          tmpfs   nosuid,nodev    0 0
//! ```
//!
//! ...we can manually mount the boot device mentioned in `/etc/fstab` by only configuring its target
//! mount point (i.e. `/boot`).
//!
//! The resulting [`Mount`] object will use the file system type (`ext4`), and mount options in the
//! file (`noauto,rw,relatime`) to complete the operation.
//!
//! ```
//! use rsmount::mount::Mount;
//!
//! fn main() -> rsmount::Result<()> {
//!     // Configure the `Mount` struct to mount the boot device.
//!     let mut mount = Mount::builder()
//!         // Location of the mount point in the file tree.
//!         .target("/boot")
//!         // Skips all mount source preparation, mount option analysis, and the actual mounting
//!         // process.
//!         .dry_run()
//!         .build()?;
//!
//!     // Mount the device with UUID=07aae9ba-12a1-4325-8b16-2ee214a6d7fd
//!     mount.mount_device()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Override the mount options, and mount a device in `/etc/fstab`
//!
//! You can override the mount options set in `/etc/fstab` when mounting a device. In the example
//! below, the original mount option `noauto` is replaced by `ro,exec` before `/dev/usbdisk` is
//! mounted at `/media/usb`.
//!
//! ```text
//! ...snip...
//! /dev/usbdisk  /media/usb    vfat    noauto          0 0
//! none          /tmp          tmpfs   nosuid,nodev    0 0
//! ```
//!
//! ```
//! use rsmount::core::device::BlockDevice;
//! use rsmount::mount::Mount;
//! use rsmount::mount::MountOptionsMode;
//!
//! fn main() -> rsmount::Result<()> {
//!     // Configure the `Mount` struct to mount the boot device.
//!     let block_device: BlockDevice = "/dev/usbdisk".parse()?;
//!     let mut mount = Mount::builder()
//!         // Device to mount.
//!         .source(block_device)
//!         // Comma-separated list of file system independent mount options.
//!         // `ro`: mount the filesystem as read-only.
//!         // `exec`: permit the execution of binaries and other executable files on the mounted
//!         // device.
//!         .mount_options("ro,exec")
//!         // Ignore options in `/etc/fstab` for `/dev/usbdisk`.
//!         .mount_options_mode(vec![MountOptionsMode::IgnoreOptions])
//!         // Skips all mount source preparation, mount option analysis, and the actual mounting
//!         // process.
//!         .dry_run()
//!         .build()?;
//!
//!     // Mount `/dev/usbdisk` at `/media/usb`.
//!     mount.mount_device()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Mount all devices with a specific file system
//!
//! Assuming the following content in `/etc/fstab`...
//!
//! ```text
//! # /etc/fstab
//!
//! UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f  /      ext4  rw,relatime        0 1
//! UUID=07aae9ba-12a1-4325-8b16-2ee214a6d7fd  /boot  ext4  noauto,rw,relatime 0 2
//! UUID=b9d72af2-f231-4cf8-9d0a-ba19e94a5087  swap   swap  defaults           0 0
//!
//! /dev/cdrom    /media/cdrom  iso9660 noauto,ro       0 0
//! /dev/usbdisk  /media/usb    vfat    noauto          0 0
//! none          /tmp          tmpfs   nosuid,nodev    0 0
//! ```
//!
//! ...we can manually mount all devices with an `ext4` file system with the code below.
//!
//! ```
//! use rsmount::mount::Mount;
//! use rsmount::mount::StepResult;
//!
//! fn main() -> rsmount::Result<()> {
//!     let mut mount = Mount::builder()
//!         .match_file_systems("ext4")
//!         // Skips all mount source preparation, mount option analysis, and the actual mounting
//!         // process.
//!         .dry_run()
//!         .build()?;
//!
//!     for result in mount.seq_mount() {
//!         match result {
//!             StepResult::MountAlreadyDone(entry) => {
//!                 let source = entry.source().unwrap();
//!                 let mount_point = entry.target().unwrap();
//!
//!                 eprintln!("Already mounted: {} at {:?}", source, mount_point);
//!             }
//!             StepResult::MountFail(entry) => {
//!                 let source = entry.source().unwrap();
//!                 let mount_point = entry.target().unwrap();
//!
//!                 eprintln!("Failed to mount: {} at {:?}", source, mount_point);
//!             }
//!             StepResult::MountSkipped(entry) => {
//!                 let mount_point = entry.target().unwrap();
//!                 eprintln!("Skipped: {:?}", mount_point);
//!             }
//!             StepResult::MountSuccess(entry) => {
//!                 let source = entry.source().unwrap();
//!                 let mount_point = entry.target().unwrap();
//!
//!                 eprintln!("Mounted: {} at {:?}", source, mount_point);
//!             }
//!             _ => unreachable!(),
//!         }
//!     }
//!
//!     // Example output
//!     //
//!     // Already mounted: UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f  at "/"
//!     // Mounted: UUID=07aae9ba-12a1-4325-8b16-2ee214a6d7fd at "/boot"
//!     // Skipped: "swap"
//!     // Skipped: "/media/cdrom"
//!     // Skipped: "/media/usb"
//!     // Skipped: "/tmp"
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Unmount a device
//!
//! This example shows how to unmount the network device mounted [above](#mount-a-network-device).
//!
//! ```
//! use rsmount::mount::Unmount;
//!
//! fn main() -> rsmount::Result<()> {
//!     // Configure the `Unmount` struct to umount a device.
//!     let mut unmount = Unmount::builder()
//!         // Location of the mount point in the file tree.
//!         .target("/net/share")
//!         // Force an unmount (in case of an unreachable NFS system).
//!         .force_unmount()
//!         // Skips all mount source preparation, mount option analysis, and the actual mounting
//!         // process.
//!         .dry_run()
//!         .build()?;
//!
//!     // Unmount the shared NFS device at knuth.cwi.nl:/nfs/share
//!     unmount.unmount_device()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Create a bind mount
//!
//! A *bind mount* is a way to remount a part of the file tree at another location, where the same
//! contents are accessible in addition to at the source mount point, after the bind operation
//! is complete.
//!
//! Note that in the example, `/bind/mnt/boot` will not give access to files from devices mounted
//! on a subdirectory of `/boot` unless the mount flag
//! [`MountFlag::Recursive`](crate::core::flags::MountFlag::Recursive) is set when configuring
//! a [`Mount`].
//!
//! ```
//! use rsmount::core::device::MountPoint;
//! use rsmount::core::flags::MountFlag;
//! use rsmount::mount::Mount;
//!
//! fn main() -> rsmount::Result<()> {
//!     // Configure the `Mount` struct to create a bind mount of the boot device.
//!     let mount_point: MountPoint = "/boot".parse()?;
//!     let mut mount = Mount::builder()
//!         .source(mount_point)
//!         // Location of the mount point in the file tree.
//!         .target("/bind/mnt/boot")
//!         // Create a bind mount.
//!         // Do not allow writing to the file system while it is mounted.
//!         .mount_flags(vec![MountFlag::Bind, MountFlag::ReadOnly])
//!         // Skips all mount source preparation, mount option analysis, and the actual mounting
//!         // process.
//!         .dry_run()
//!         .build()?;
//!
//!     // Create a read-only bind mount of `/boot` at `/bind/mnt/boot`.
//!     mount.mount_device()?;
//!
//!     // From now on, we can also access the files in `/boot` from the `/bind/mnt/boot`
//!     // directory...
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Mark a mount point as `shared`
//!
//! Since Linux 2.6.15, it is possible to mark a mount point and its submounts as `shared`, `slave`,
//! `slave-and-shared`, `private`, or `unbindable`. A shared mount point has the ability to
//! propagate any mount/unmount event occurring on it (or one of its submounts) to members of its
//! peer group. See the next section for an in-depth explanation of [mount
//! namespaces](#mount-namespaces).
//!
//! ```ignore
//! use rsmount::core::device::MountPoint;
//! use rsmount::core::flags::MountFlag;
//! use rsmount::mount::Mount;
//!
//! fn main() -> rsmount::Result<()> {
//!     let mut mount = Mount::builder()
//!         // For a device already mounted at `/mnt/usbdisk`
//!         .target("/mnt/usbdisk")
//!         // Set the propagation type of this mount point to shared. Mount and unmount events on
//!         // sub mount points will propagate to its peers.
//!         .mount_flags(vec![MountFlag::Remount, MountFlag::Shared])
//!         // Skips all mount source preparation, mount option analysis, and the actual mounting
//!         // process.
//!         .dry_run()
//!         .build()?;
//!
//!     // Mark `/bind/mnt/boot` as `shared`
//!     mount.mount_device()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Mount namespaces
//!
//! *This section borrows heavily from the excellent series of articles: __[Namespaces in
//! operation](https://lwn.net/Articles/531114/)__ by Michael Kerrisk.*
//!
//! When the system is first booted, there is a single mount namespace, the so-called ***initial
//! namespace***. This namespace holds a list of mount points representing all bind mounts, and
//! devices attached to the file system tree. Each mount point is identified by a unique integer,
//! or mount ID.
//!
//! The diagram below shows a simplified view of an initial namespace (`ns1`) containing one mount
//! point `/mnt` with mount ID `21`.
//!
//! ![Diagram of the initial namespace as a container (ns1) with a rectangular box inside. The box
//! represents a mount point named /mnt with mount ID 21.][fig-01]
//!
//! New mount namespaces are created by using the `CLONE_NEWNS` flag with either the
//! [`clone()`](http://man7.org/linux/man-pages/man2/clone.2.html) system call, to create a new child
//! process in the new namespace, or the
//! [`unshare()`](http://man7.org/linux/man-pages/man2/unshare.2.html) system call, to move the
//! caller into the new namespace.
//!
//! When a new mount namespace is created, it receives a copy of the list of mount points,
//! replicated from the namespace of the caller of `clone()` or `unshare()`. To preserve the
//! uniqueness of mount IDs, each copy of a mount point is assigned a new identifier.
//!
//! The diagram below shows the state of the system after creating a new namespace (`ns2`), where
//! the copy of `/mnt` is assigned a value of `42` as its new mount ID.
//!
//! ![Diagram of two namespace containers. On the left, the initial namespace container (ns1) with
//! a mount point named /mnt with mount ID 21, represented as a rectangular box. On the right, the
//! new namespace (ns2) with a copy of the same mount point but with mount ID
//! 42.][fig-02]
//!
//! By default, changes to the list of mount points in a namespace are only visible to processes
//! within the namespace. Up until the introduction of ***shared subtrees***, the only way to make
//! a new disk visible in all mount namespaces was to mount the disk, separately, in each namespace.
//!
//! For example, in the diagram below, although a USB disk is mounted at `/mnt/usb` in the
//! namespace `ns1`, the new mount point is not automatically replicated in `ns2.`
//!
//! ![This diagram has the same structure as in the preceding image, except for a new mount point
//! named `/usb` with mount ID 22, in the ns1 namespace, connected to the `/mnt` mount point by
//! a vertical line.][fig-03]
//!
//!
//! ### Shared subtrees
//!
//! Shared subtrees allow us to perform a single mount operation that makes a new disk visible in
//! all, or some subset, of the mount namespaces in a system. The key benefit of shared subtrees is
//! to allow automatic, controlled propagation of mount and unmount events between namespaces. This
//! means, for example, that mounting a disk in one mount namespace can trigger a mount of that
//! same disk in all other namespaces.
//!
//! Under the shared subtrees feature, each mount point is marked with a *propagation type*, which
//! determines whether operations creating and/or removing mount points below a given mount point,
//! in a specific namespace, are propagated to other mount points in other namespaces.
//!
//! Linux kernel developers defined four propagation types:
//!  - **shared**: A mount point marked `shared` propagates mount and unmount events to other mount
//!    points belonging to its *peer group* (peer groups are described in the [next
//!    section](#peer-groups)). When a mount point is added (or removed) below a `shared` mount
//!    point, changes will propagate to its peer group, so that a mount or unmount will also take
//!    place below each of the peer mount points. Event propagation works both ways, from the mount
//!    point to its peers and vice-versa.
//! - **private**: A mount point marked `private` is the converse of a `shared` mount point. A
//!   `private` mount point does neither propagate events to, nor receive propagation events from
//!   peers.
//! - **slave**: A mount point marked `slave` sits midway between `shared` and `private`. A slave
//!   mount has as master, a shared peer group whose members propagate mount and unmount events to
//!   the slave. However, the slave mount does not propagate events to the master peer group.
//! - **unbindable**: Like a `private` mount point, a mount point marked `unbindable` does neither
//!   propagate to, nor receive events from peers. In addition, an `unbindable` mount point can't be
//!   the source for a bind mount operation.
//!
//! By default, the kernel marks new mount points `private`, as long as their parent mount (if they
//! have one) is not marked `shared`. In that case, they will also be marked `shared`. We can now
//! redraw below the previous diagram, incorporating each mount point's propagation type.
//!
//! ![This diagram is a copy of the previous one where mount points a represented by blue squares.
//! A legend was added at the bottom of the image, showing a blue square which represents a private
//! mount point, a yellow circle represents a shared mount point, a red diamond represents a slave
//! mount point, an orange oval represents a slave-shared mount point, and finally a purple hexagon
//! stands for an unbindable mount point.][fig-04]
//!
//! It is worth expanding on a few points that were glossed over.
//!
//! First, setting propagation types is a **per-mount-point** business. Within a namespace, some
//! mount points might be marked `shared`, while others are marked `private`, `slave` or
//! `unbindable`.
//!
//! Second, the propagation type of a marked mount point determines the propagation of
//! mount/unmount events of mount points **immediately below** it.
//!
//! In passing, it is perhaps worth clarifying that the word *event* is used here as an abstract
//! term, in the sense of *something happened*. The notion of event propagation does not imply some
//! sort of message passing between mount points. It rather carries the idea that a mount/unmount
//! operation, on one mount point, triggers a matching operation on at least one other mount point.
//!
//! Finally, it is possible for a mount point to be both the slave of a master peer group, as well
//! as share events with a set of peers of its own, a so-called **slave-and-shared** mount. In this
//! case, the mount point might receive propagation events from its master, and then forward them
//! to its peers.
//!
//! ### Peer groups
//!
//! A peer group is a set of mount points that propagate mount and unmount events to one another. A
//! peer group acquires a new member when a mount point with a `shared` propagation type is, either
//! replicated during the creation of a new namespace, or used as the source for a bind mount.
//! For a bind mount, the details are more complex than we describe here; you can find more
//! information in the kernel source file
//! [Documentation/filesystems/sharedsubtree.txt](https://www.kernel.org/doc/Documentation/filesystems/sharedsubtree.txt).
//!
//! In both cases, the new mount point is made a member of the same peer group as the existing
//! mount point. Conversely, a mount point ceases to be a member of a peer group when it is
//! unmounted, either explicitly, or implicitly when a mount namespace is torn down because the
//! last member process terminates or moves to another namespace.
//!
//! For example, let's assume we have a namespace (ns1) with a mount point `/mnt` (mount ID 21). If
//! we mark this mount point as `shared`, and then create a new process in a new namespace (ns2),
//! the list of mount points will be replicated. As we have explained earlier, each copy of a
//! mount point will be assigned a new mount ID. In this case, the copy of `/mnt` will be assigned
//! the mount ID 42.
//!
//! Since `/mnt` is `shared`, a new peer group (peer group 1) will be created with `/mnt` and its
//! copy as members (see diagram below). They will share every mount/unmount event as long as they
//! keep their `shared` propagation type.
//!
//! ![][fig-05]
//!
//! Now if we mount a USB disk below  `/mnt` at `/mnt/usb` in ns1, an event will be propagated
//! to ns2; making the USB disk accessible from ns2 without manual intervention. The new mount
//! point will be automatically marked `shared`, and added, with its copy, to a new peer group
//! (peer group 2).
//!
//! ![][fig-06]
//!
//! ### Effects of propagation type transitions on peer groups
//!
//! In each namespace, processes equipped with the `CAP_SYS_ADMIN` capability can modify the propagation
//! type of their mount points. Here, we will study the effects a change of a mount point's
//! propagation type can have on peer groups.
//!
//! #### Effects of a `shared` mount point
//!
//! Let's start with `ns1` the initial namespace with a `shared` mount point `/mnt` (mount ID 21).
//! From `ns1`, we spawn two new child processes using the `CLONE_NEWNS` flag with the
//! [`clone()`](http://man7.org/linux/man-pages/man2/clone.2.html) system call. This will create
//! two new namespaces, `ns2`, and `ns3` triggering a replication of `/mnt` whose copies will be
//! assigned new mount IDs, respectively 22 and 23.
//!
//! Since `/mnt` is a `shared` mount point, a new peer group (peer group 1) is formed with all
//! replicas as its members. Each peer will share mount/unmount events with all its
//! counterparts, as shown by the fully connected graph of peers in the diagram below.
//!
//! ![][fig-07]
//!
//! #### Effects of a `slave` mount point
//!
//! Now, let's switch the propagation type of `/mnt` in `ns2` (mount ID 22) from `shared` to
//! `slave`, meaning from now on `/mnt` in `ns2` will receive mount/unmount events from its peers,
//! but won't send back new events occurring in `ns2`. This removes peer group arrows, in the
//! diagram above, pointing from `/mnt` with mount ID 22 to `/mnt` with mount ID 21, and `/mnt`
//! with mount ID 23. Bi-directional communication between the replicas with mount ID 21 and 23 is
//! preserved.
//!
//! ![][fig-08]
//!
//! If we mount a USB disk on `/mnt/usb` in `ns1`, the new mount point (`/usb`, mount ID
//! 24) will be propagated to its peers.  `/mnt` (mount ID 21) in `ns1` is marked as `shared`, so
//! `/usb` (mount Id 24) will be marked `shared` as well. A new peer group (peer group 2) will
//! be created with copies of the new mount point.
//!
//! However, we previously switched the propagation type of `/mnt` (mount ID 22) in `ns2` to
//! `slave`, the `/usb` copy in `ns2` will also get a `slave` marking to conform to the state
//! of its parent mount point (see next diagram).
//!
//! ![][fig-09]
//!
//! If instead of mounting the USB disk on a mount point in `ns1`, we mount it on `/mnt/usb` in
//! `ns2`. The `/mnt` mount point (mount ID 22) in `ns2` has a `slave` propagation type. So, as we
//! stated in the section [Shared subtrees](#shared-subtree), the new mount point `/usb` (mount
//! ID 24) will be set to `private` since its parent mount point is not marked `shared`.
//!
//! As such, it will neither receive nor send mount/unmount events, and will not be replicated in
//! any other namespace. Therefore, there is no need to create a new peer group.
//!
//! Here is diagram depicting this case.
//!
//! ![][fig-10]
//!
//! #### Effects of a `slave-and-shared` mount point
//!
//! Let's revert to a new initial state, a namespace `ns1` with a `shared` mount point `/mnt` (mount
//! ID 21).  From `ns1`, we spawn one child process using the `CLONE_NEWNS` flag with the
//! [`clone()`](http://man7.org/linux/man-pages/man2/clone.2.html) system call. This will create
//! a new namespace, `ns2`, triggering a replication of `/mnt` whose copy will be assigned a new
//! mount ID 22.
//!
//! They will both be members of peer group 1 with a bi-directional communication channel for
//! propagating mount/unmount events.
//!
//! ![][fig-11]
//!
//! Now, let's switch the propagation type of `/mnt` (mount ID 22) in `ns2` from `shared` to
//! `slave-and-shared`. This change has two consequences:
//! - from `ns1`'s point of view, the mount point in `ns2` (mount ID 22) is now `slave` to its
//!   `/mnt` mount point (mount ID 21). In peer group 1, the previous bi-directional channel between
//!   `/mnt` mount ID 21 and its copy `/mnt` mount ID 22 is now unidirectional, with `/mnt` mount ID 22
//!   set as `slave`.
//! - from `ns2`'s point of view, `/mnt` mount ID 22 is a `shared` mount point, thus a new peer
//!   group is created, adding it as a member (peer group 2). See the diagram below.
//!
//! ![][fig-12]
//!
//! For the purpose of our demonstration, we will call `ns2` the "child" namespace of `ns1`. From a
//! process in this "child" namespace, we spawn a new process in a new namespace, `ns3`.
//!
//! So, `ns3` is the "child" of `ns2`, and thus logically the "grandchild" of `ns1`.  As usual the
//! list of mount points in `ns2` is copied to `ns3`, updating the mount IDs to keep their
//! uniqueness. In this case, a copy of `/mnt` mount ID 22 is created and is assigned the mount ID
//! 23. This new mount point will have the same propagation type as its parent, i.e.
//! `slave-and-shared`.
//!
//! As a mount point in a "grandchild" namespace of `ns1`, it will gain membership to peer
//! group 1 as a `slave` mount point, receiving but not transmitting mount/unmount events to peers
//! in its peer group.
//!
//! At the same time, as a mount point in a "child" namespace of `ns2`, it will be added to peer
//! group 2 as a `shared` mount point.
//!
//! ![][fig-13]
//!
//! How would this configuration fare if we were to mount a USB disk on `/mnt/usb` in `ns1`? This
//! is where thing get interesting!
//!
//! `/mnt` in `ns1` is a `shared` mount point, thus any mount point created directly below it will
//! be `shared`. In the example below, a mount point named `/usb` with mount ID 24 is created. Since it is
//! a new `shared` mount point, it is added to a new peer group (i.e. peer group 3).
//!
//! Now, time to deal with the mount event propagation to the copy of `/mnt` in `ns2`.
//!
//! Looking at peer group 1, we have `/mnt` mount ID 22 set as a `slave` mount point in `ns2`; propagating
//! the mount event to it should create a new `slave` mount point in `ns2` (`/usb` mount ID 25), and add a `slave` copy
//! to peer group 3.
//!
//! However to peer group 2, `/mnt` mount ID 22 is a `shared` mount point, so `/usb` mount ID 25
//! should also be marked `shared`. Taking this into account, `/usb` mount ID 25 adopts a
//! `slave-and-shared` marking. Its `shared` status implies the creation of a new peer group to
//! which it will added (peer group 4).
//!
//! With the same reasoning, we add a `slave-and-shared` `/usb` mount point with mount ID 26 to `ns3`, a
//! corresponding `slave` copy to peer group 3, and a `shared` copy to peer group 4.
//!
//! ![][fig-14]
//!
//! Now if instead of mounting the USB disk on `/mnt/usb` in `ns1` we chose to mount it on `/mnt/usb`
//! in `ns3`, what would happen?
//!
//! Well, that would create a new mount point `/usb` (mount ID 25) connected to `/mnt` mount ID 23,
//! which as a `slave` according to its status in peer group 1 shouldn't propagate the event. So,
//! `/mnt` mount ID 21 and 22 shouldn't be notified.
//!
//! However, `/mnt` mount ID 23 is a `shared` mount point from peer group 2's point of view, which
//! implies a bi-directional communication channel with `/mnt` mount ID 22. The `shared` status of
//! `/mnt` mount ID 22 in peer group 2 supersedes its `slave` status in peer group 1. We thus need to
//! create new `/usb` mount point in `ns2`.
//!
//! What propagation type should the new `/usb` mount points have? `/mnt` mount ID 22 and 23 have a
//! `slave-and-shared` propagation type, in this situation their `shared` status takes precedence
//! over their `slave` status when choosing the propagation type of a new child mount point. As we
//! have seen before, a child of a `shared` mount point is also `shared`. Furthermore, a `shared`
//! mount point triggers the creation of a new peer group, in this example peer group 3, to which it
//! is added.
//!
//! To sum up, we have to create two new `shared` `/usb` mount points (mount ID 25 and 26), one in
//! `ns3` the other in `ns2` connected to their respective `/mnt` parent. They will also gain
//! membership to a new peer group, peer group 3 (see the diagram below).
//!
//! ![][fig-15]
//!
//! #### Effects of a `private` mount point
//!
//! Back to an initial state as in the previous section ([Effects of a `slave-and-shared` mount
//! point](#effects-of-a-slave-and-shared-mount-point)):
//! - two namespaces `ns1` and `ns2`, each with a `shared` mount point `/mnt` (respectively mount
//!   ID 21 and 22),
//! - one peer group (peer group 1) with the `shared` mount points as members.
//!
//! ![][fig-11]
//!
//! What effect would switching `/mnt` mount ID 22 in `ns2` from `shared` to `private`? We know
//! that a `private` mount point does neither propagate events to, nor receive propagation events
//! from peers. So, whatever happens to `/mnt` in `ns2` will not be propagated to its peer in `ns1`,
//! and vice-versa. A new child mount point of `/mnt` in `ns2` will inherit its `private` status,
//! not transmitting events.
//!
//! ![][fig-16]
//!
//! #### Effects of an `unbindable` mount point
//!
//! An `unbindable` mount point is first and foremost a `private` mount point, with the added
//! property of not accepting bind operations. As such, it has the same behaviour as a `private`
//! mount point (for more information see the [previous section](#effects-of-a-private-mount-point)).

pub use error_code_enum::ErrorCode;
pub use exit_code_enum::ExitCode;
pub use exit_status_struct::ExitStatus;
pub use mount_builder_error_enum::MountBuilderError;
use mount_builder_struct::MntBuilder;
pub use mount_builder_struct::MountBuilder;
pub use mount_error_enum::MountError;
pub use mount_iter_error_enum::MountIterError;
pub use mount_iter_struct::MountIter;
pub use mount_namespace_struct::MountNamespace;
pub use mount_options_mode_enum::MountOptionsMode;
pub use mount_source_enum::MountSource;
pub use mount_struct::Mount;
pub use process_exit_status_struct::ProcessExitStatus;
pub use remount_iter_error_enum::ReMountIterError;
pub use remount_iter_struct::ReMountIter;
pub use step_result_enum::StepResult;
pub use umount_iter_error_enum::UMountIterError;
pub use umount_iter_struct::UMountIter;
pub use umount_namespace_struct::UMountNamespace;
pub use unmount_builder_error_enum::UnmountBuilderError;
use unmount_builder_struct::UmntBuilder;
pub use unmount_builder_struct::UnmountBuilder;
pub use unmount_error_enum::UnmountError;
pub use unmount_struct::Unmount;

mod error_code_enum;
mod exit_code_enum;
mod exit_status_struct;
pub(crate) mod macros;
mod mount_builder_error_enum;
mod mount_builder_struct;
mod mount_error_enum;
mod mount_iter_error_enum;
mod mount_iter_struct;
mod mount_namespace_struct;
mod mount_options_mode_enum;
mod mount_source_enum;
mod mount_struct;
mod process_exit_status_struct;
mod remount_iter_error_enum;
mod remount_iter_struct;
mod step_result_enum;
mod umount_iter_error_enum;
mod umount_iter_struct;
mod umount_namespace_struct;
mod unmount_builder_error_enum;
mod unmount_builder_struct;
mod unmount_error_enum;
mod unmount_struct;
