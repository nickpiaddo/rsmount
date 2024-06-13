// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use enum_iterator::Sequence;

// From standard library

// From this library

/// Userspace mount flags.
///
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Sequence)]
#[repr(u64)]
#[non_exhaustive]
pub enum UserspaceMountFlag {
    // From
    // https://github.com/util-linux/util-linux/blob/8aa25617467a1249669cff7240ca31973bf9a127/libmount/src/optmap.c#L148

    //---- BEGIN dm-verity support
    // https://man.archlinux.org/man/veritysetup.8.en
    // https://www.kernel.org/doc/html/latest/admin-guide/device-mapper/verity.html
    // https://www.man7.org/linux/man-pages/man8/mount.8.html#DM-VERITY_SUPPORT
    /// dm-verity: Use forward error correction (FEC) to recover from corruption if a hash verification fails.
    /// Use encoding data from the specified device.
    ForwardErrorCorrectionDevice = libmount::MNT_MS_FEC_DEVICE as u64,

    /// dm-verity: Offset, in bytes, from the start of the FEC device to the beginning of the encoding data.
    ForwardErrorCorrectionOffset = libmount::MNT_MS_FEC_OFFSET as u64,

    /// dm-verity: Number of generator roots. This equals to the number of parity bytes in the encoding data.
    /// In RS(M, N) encoding, the number of roots is M-N. M is 255 and M-N is between 2 and 24
    /// included.
    ForwardErrorCorrectionRoots = libmount::MNT_MS_FEC_ROOTS as u64,

    /// dm-verity: Mount the device that supplies the hash tree data associated with the source
    /// volume to pass to dm-verity (see the [Hash
    /// Tree](https://www.kernel.org/doc/html/latest/admin-guide/device-mapper/verity.html#hash-tree)
    /// section of the Linux kernel documentation for more information).
    HashDevice = libmount::MNT_MS_HASH_DEVICE as u64,

    /// dm-verity: Offset, in bytes, from the start of the hash device to the root block of the hash tree.
    HashOffset = libmount::MNT_MS_HASH_OFFSET as u64,

    /// dm-verity: Hex-encoded hash of the root of the device providing the hash data tree.
    /// Mutually exclusive with `RootHashFile`.
    RootHash = libmount::MNT_MS_ROOT_HASH as u64,

    /// dm-verity: Path to file containing the hex-encoded hash of the root of the device
    /// providing the hash data tree.
    /// Mutually exclusive with `RootHash`.
    RootHashFile = libmount::MNT_MS_ROOT_HASH_FILE as u64,

    /// dm-verity:  the description of the USER_KEY that the kernel will lookup to get the pkcs7
    /// signature of the roothash. The pkcs7 signature is used to validate the root hash during the
    /// creation of the device mapper block device. Verification of roothash depends on the config
    /// DM_VERITY_VERIFY_ROOTHASH_SIG being set in the kernel. The signatures are checked against
    /// the built-in trusted keyring by default, or the secondary trusted keyring if
    /// DM_VERITY_VERIFY_ROOTHASH_SIG_SECONDARY_KEYRING is set. The secondary trusted keyring
    /// includes by default the built-in trusted keyring, and it can also gain new certificates at
    /// run time if they are signed by a certificate already in the secondary trusted keyring.
    RootHashSignature = libmount::MNT_MS_ROOT_HASH_SIG as u64,

    #[cfg(v2_39)]
    /// dm-verity: Instruct the kernel to ignore, reboot or panic when corruption is detected. By
    /// default the I/O operation simply fails.
    VerityOnCorruption = libmount::MNT_MS_VERITY_ON_CORRUPTION as u64,
    //---- END dm-verity support
    /// Use the loop device.
    LoopDevice = libmount::MNT_MS_LOOP as u64,
    LoopDeviceEncryption = libmount::MNT_MS_ENCRYPTION as u64,
    LoopDeviceSizeLimit = libmount::MNT_MS_SIZELIMIT as u64,
    LoopDeviceOffset = libmount::MNT_MS_OFFSET as u64,

    /// Allow a user other than root to mount a device.
    User = libmount::MNT_MS_USER as u64,
    /// Allow any user to mount/unmount a device.
    Users = libmount::MNT_MS_USERS as u64,
    /// Allow any member of a user group to mount/unmount a device.
    Group = libmount::MNT_MS_GROUP as u64,
    /// Allow the owner of a special file to mount/unmount it.
    Owner = libmount::MNT_MS_OWNER as u64,

    /// Only allow explicit mounts (no automatic mount permitted).
    NoAuto = libmount::MNT_MS_NOAUTO as u64,
    /// Do not fail if trying to mount a device triggers an `ENOENT: No such file or directory`
    /// error.
    NoFail = libmount::MNT_MS_NOFAIL as u64,

    DeviceRequiresNetwork = libmount::MNT_MS_NETDEV as u64,

    MountHelper = libmount::MNT_MS_HELPER as u64,
    UmountHelper = libmount::MNT_MS_UHELPER as u64,

    Comment = libmount::MNT_MS_COMMENT as u64,
    /// Persistent `utab` comments.
    XUTabComment = libmount::MNT_MS_XCOMMENT as u64,
    /// `fstab` only comment.
    XFstabComment = libmount::MNT_MS_XFSTABCOMM as u64,
}
