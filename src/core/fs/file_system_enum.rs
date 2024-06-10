// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use enum_iterator::Sequence;

// From standard library
use std::ffi::CString;
use std::fmt;
use std::str::FromStr;

// From this library
use crate::core::errors::ParserError;

/// Combination of file systems supported by the Linux Kernel, and `libblkid`.
#[derive(Debug, Eq, PartialEq, Sequence)]
#[non_exhaustive]
pub enum FileSystem {
    /// Name: `"adaptec_raid_member"`
    AdaptecRaid,
    /// Name: `"adfs"`
    Adfs,
    /// Name: `"afs"`
    Afs,
    /// Name: `"affs"`
    Affs,
    /// Name: `"apfs"`
    APFS,
    /// Name: `"aio"`
    Aio,
    /// Name: `"autofs"`
    Autofs,
    /// Name: `"bcache"`
    Bcache,
    /// Name: `"bcachefs"`
    BcacheFs,
    /// Name: `"bdev"`
    Bdev,
    /// Name: `"befs"`
    BeFS,
    /// Name: `"bfs"`
    Bfs,
    /// Name: `"binder"`
    Binder,
    /// Name: `"binfmt_misc"`
    BinfmtMisc,
    /// Name: `"BitLocker"`
    BitLocker,
    /// Name: `"ceph_bluestore"`
    BlueStore,
    /// Name: `"bpf"`
    Bpf,
    /// Name: `"btrfs"`
    BTRFS,
    /// Name: `"ceph"`
    Ceph,
    /// Name: `"cifs"`
    Cifs,
    /// Name: `"cgroup"`
    Cgroup,
    /// Name: `"cgroup2"`
    Cgroup2,
    /// Name: `"configfs"`
    Configfs,
    /// Name: `"cpuset"`
    Cpuset,
    /// Name: `"cramfs"`
    Cramfs,
    /// Name: `"debugfs"`
    Debugfs,
    /// Name: `"devpts"`
    Devpts,
    /// Name: `"devtmpfs"`
    Devtmpfs,
    /// Name: `"ddf_raid_member"`
    DDFRaid,
    /// Name: `"DM_integrity"`
    DmIntegrity,
    /// Name: `"DM_snapshot_cow"`
    DmSnapshot,
    /// Name: `"DM_verify_hash"`
    DmVerify,
    /// Name: `"drbd"`
    DRBD,
    /// Name: `"drbdmanage_control_volume"`
    DRBDManage,
    /// Name: `"drbdproxy_datalog"`
    DRBDProxyDatalog,
    /// Name: `"ecryptfs"`
    Ecryptfs,
    /// Name: `"efivarfs"`
    Efivarfs,
    /// Name: `"efs"`
    Efs,
    /// Name: `"erofs"`
    EROFS,
    /// Name: `"eventpollfs"`
    Eventpollfs,
    /// Name: `"exfat"`
    ExFAT,
    /// Name: `"exfs"`
    Exfs,
    /// Name: `"ext2"`
    Ext2,
    /// Name: `"ext3"`
    Ext3,
    /// Name: `"ext4"`
    Ext4,
    /// Name: `"ext4dev"`
    Ext4Dev,
    /// Name: `"f2fs"`
    F2FS,
    /// Name: `"cs_fvault2"`
    FileVault,
    /// Name: `"fuse"`
    Fuse,
    /// Name: `"fuse.portal"`
    FusePortal,
    /// Name: `"fuseblk"`
    Fuseblk,
    /// Name: `"fusectl"`
    Fusectl,
    /// Name: `"futexfs"`
    Futexfs,
    /// Name: `"gfs"`
    GFS,
    /// Name: `"gfs2"`
    GFS2,
    /// Name: `"hfs"`
    HFS,
    /// Name: `"hfsplus"`
    HFSPlus,
    /// Name: `"hpt37x_raid_member"`
    HighPoint37x,
    /// Name: `"hpt45x_raid_member"`
    HighPoint45x,
    /// Name: `"hostfs"`
    Hostfs,
    /// Name: `"hpfs"`
    HPFS,
    /// Name: `"hugetlbfs"`
    HugeTlbFs,
    /// Name: `"iso9660"`
    Iso9660,
    /// Name: `"isw_raid_member"`
    ISWRaid,
    /// Name: `"jbd"`
    JBD,
    /// Name: `"jffs2"`
    Jffs2,
    /// Name: `"jfs"`
    JFS,
    /// Name: `"jmicron_raid_member"`
    JmicronRaid,
    /// Name: `"linux_raid_member"`
    LinuxRaid,
    /// Name: `"lsi_mega_raid_member"`
    LSIRaid,
    /// Name: `"crypto_LUKS"`
    LUKS,
    /// Name: `"LVM1_member"`
    LVM1,
    /// Name: `"LVM2_member"`
    LVM2,
    /// Name: `"minix"`
    Minix,
    /// Name: `"none"`
    None,
    /// Name: `"mpool"`
    Mpool,
    /// Name: `"mqueue"`
    Mqueue,
    /// Name: `"nss"`
    Netware,
    /// Name: `"nfs"`
    NFS,
    /// Name: `"nilfs2"`
    Nilfs2,
    /// Name: `"nsfs"`
    NSFS,
    /// Name: `"ntfs"`
    NTFS,
    /// Name: `"ntfs3"`
    NTFS3,
    /// Name: `"nvidia_raid_member"`
    NvidiaRaid,
    /// Name: `"ocfs"`
    OCFS,
    /// Name: `"ocfs2"`
    OCFS2,
    /// Name: `"ocfs2_dlmfs"`
    OCFS2Dlmfs,
    /// Name: `"omfs"`
    Omfs,
    /// Name: `"openpromfs"`
    Openpromfs,
    /// Name: `"pidfs"`
    Pidfs,
    /// Name: `"pipefs"`
    Pipefs,
    /// Name: `"proc"`
    Proc,
    /// Name: `"promise_fasttrack_raid_member"`
    PromiseRaid,
    /// Name: `"pseudo_erofs"`
    PseudoEROFS,
    /// Name: `"pstore"`
    Pstore,
    /// Name: `"pvfs2"`
    Pvfs2,
    /// Name: `"qnx4"`
    QNX4,
    /// Name: `"qnx6"`
    QNX6,
    /// Name: `"ReFs"`
    ReFs,
    /// Name: `"reiserfs"`
    Reiserfs,
    /// Name: `"reiser4"`
    Reiser4,
    /// Name: `"ramfs"`
    Ramfs,
    /// Name: `"romfs"`
    Romfs,
    /// Name: `"rootfs"`
    Rootfs,
    /// Name: `"rpc_pipefs"`
    RpcPipefs,
    /// Name: `"securityfs"`
    SecurityFs,
    /// Name: `"selinuxfs"`
    SeLinuxFs,
    /// Name: `"silicon_medley_raid_member"`
    SiliconRaid,
    /// Name: `"sockfs"`
    Sockfs,
    /// Name: `"squashfs"`
    Squashfs,
    /// Name: `"squashfs3"`
    Squashfs3,
    /// Name: `"stratis"`
    Stratis,
    /// Name: `"swap"`
    Swap,
    /// Name: `"swsuspend"`
    SwapSuspend,
    /// Name: `"sysfs"`
    Sysfs,
    /// Name: `"sysv"`
    SYSV,
    /// Name: `"tmpfs"`
    Tmpfs,
    /// Name: `"tracefs"`
    Tracefs,
    /// Name: `"ubi"`
    UBI,
    /// Name: `"ubifs"`
    UBIFS,
    /// Name: `"udf"`
    UDF,
    /// Name: `"ufs"`
    UFS,
    /// Name: `"usbfs"`
    Usbfs,
    /// Name: `"usbdevfs"`
    Usbdevfs,
    /// Name: `"vboxsf"`
    Vboxsf,
    /// Name: `"vdo"`
    VDO,
    /// Name: `"vfat"`
    VFAT,
    /// Name: `"via_raid_member"`
    VIARaid,
    /// Name: `"virtiofs"`
    Virtiofs,
    /// Name: `"VMFS"`
    VMFS,
    /// Name: `"VMFS_volume_member"`
    VMFSVolume,
    /// Name: `"vxfs"`
    Vxfs,
    /// Name: `"xenix"`
    Xenix,
    /// Name: `"xfs"`
    XFS,
    /// Name: `"xfs_external_log"`
    XFSLog,
    /// Name: `"zfs_member"`
    ZFS,
    /// Name: `"zonefs"`
    Zonefs,
    Unknown,
}

impl FileSystem {
    // Each known filesystem is represented in `util-linux/libblkid/src/superblocks`
    // by a structure at the end of each file in the directory.
    //
    // For example in `util-linux/libblkid/src/superblock/svia_raid.c`
    //
    // const struct blkid_idinfo viaraid_idinfo = {
    //	.name		= "via_raid_member",
    //	.usage		= BLKID_USAGE_RAID,
    //	.probefunc	= probe_viaraid,
    //	.magics		= BLKID_NONE_MAGIC
    //};
    //
    // the attribute `name` is the ID used by `libblkid` to access the function `probe_viaraid`
    // to identify the type of superblock encountered during a probe.
    //
    //
    // `libblkid` supported file systems extracted with the following commands:
    // https://github.com/util-linux/util-linux.git
    //  rg --no-filename --after-context 3 "const struct blkid_idinfo" util-linux/libblkid/ | rg "\.name" | awk '{print $3}' | tr -d "," | sort -d | uniq
    //
    // Linux file systems extracted with the following commands:
    // git clone --depth 2 https://github.com/torvalds/linux.git
    //  rg --no-filename --after-context 3 "static struct file_system_type" linux/fs/ | rg "\.name" | awk '{print $3}' | tr -d "," | sort -d | uniq
    /// View this `FileSystem` as a UTF-8 `str`.
    pub fn as_str(&self) -> &str {
        match self {
            Self::AdaptecRaid => "adaptec_raid_member",
            Self::Adfs => "adfs",
            Self::Afs => "afs",
            Self::Affs => "affs",
            Self::APFS => "apfs",
            Self::Aio => "aio",
            Self::Autofs => "autofs",
            Self::Bcache => "bcache",
            Self::BcacheFs => "bcachefs",
            Self::Bdev => "bdev",
            Self::BeFS => "befs",
            Self::Bfs => "bfs",
            Self::Binder => "binder",
            Self::BinfmtMisc => "binfmt_misc",
            Self::BitLocker => "BitLocker",
            Self::BlueStore => "ceph_bluestore",
            Self::Bpf => "bpf",
            Self::BTRFS => "btrfs",
            Self::Ceph => "ceph",
            Self::Cifs => "cifs",
            Self::Cgroup => "cgroup",
            Self::Cgroup2 => "cgroup2",
            Self::Configfs => "configfs",
            Self::Cpuset => "cpuset",
            Self::Cramfs => "cramfs",
            Self::Debugfs => "debugfs",
            Self::Devpts => "devpts",
            Self::Devtmpfs => "devtmpfs",
            Self::DDFRaid => "ddf_raid_member",
            Self::DmIntegrity => "DM_integrity",
            Self::DmSnapshot => "DM_snapshot_cow",
            Self::DmVerify => "DM_verify_hash",
            Self::DRBD => "drbd",
            Self::DRBDManage => "drbdmanage_control_volume",
            Self::DRBDProxyDatalog => "drbdproxy_datalog",
            Self::Ecryptfs => "ecryptfs",
            Self::Efivarfs => "efivarfs",
            Self::Efs => "efs",
            Self::EROFS => "erofs",
            Self::Eventpollfs => "eventpollfs",
            Self::ExFAT => "exfat",
            Self::Exfs => "exfs",
            Self::Ext2 => "ext2",
            Self::Ext3 => "ext3",
            Self::Ext4 => "ext4",
            Self::Ext4Dev => "ext4dev",
            Self::F2FS => "f2fs",
            Self::FileVault => "cs_fvault2",
            Self::Fuse => "fuse",
            Self::FusePortal => "fuse.portal",
            Self::Fuseblk => "fuseblk",
            Self::Fusectl => "fusectl",
            Self::Futexfs => "futexfs",
            Self::GFS => "gfs",
            Self::GFS2 => "gfs2",
            Self::HFS => "hfs",
            Self::HFSPlus => "hfsplus",
            Self::HighPoint37x => "hpt37x_raid_member",
            Self::HighPoint45x => "hpt45x_raid_member",
            Self::HugeTlbFs => "hugetlbfs",
            Self::Hostfs => "hostfs",
            Self::HPFS => "hpfs",
            Self::Iso9660 => "iso9660",
            Self::ISWRaid => "isw_raid_member",
            Self::JBD => "jbd",
            Self::Jffs2 => "jffs2",
            Self::JFS => "jfs",
            Self::JmicronRaid => "jmicron_raid_member",
            Self::LinuxRaid => "linux_raid_member",
            Self::LSIRaid => "lsi_mega_raid_member",
            Self::LUKS => "crypto_LUKS",
            Self::LVM1 => "LVM1_member",
            Self::LVM2 => "LVM2_member",
            Self::Minix => "minix",
            Self::Mpool => "mpool",
            Self::Mqueue => "mqueue",
            Self::Netware => "nss",
            Self::NFS => "nfs",
            Self::Nilfs2 => "nilfs2",
            Self::None => "none",
            Self::NSFS => "nsfs",
            Self::NTFS => "ntfs",
            Self::NTFS3 => "ntfs3",
            Self::NvidiaRaid => "nvidia_raid_member",
            Self::OCFS => "ocfs",
            Self::OCFS2 => "ocfs2",
            Self::OCFS2Dlmfs => "ocfs2_dlmfs",
            Self::Omfs => "omfs",
            Self::Openpromfs => "openpromfs",
            Self::Pidfs => "pidfs",
            Self::Pipefs => "pipefs",
            Self::Proc => "proc",
            Self::PromiseRaid => "promise_fasttrack_raid_member",
            Self::PseudoEROFS => "pseudo_erofs",
            Self::Pstore => "pstore",
            Self::Pvfs2 => "pvfs2",
            Self::QNX4 => "qnx4",
            Self::QNX6 => "qnx6",
            Self::ReFs => "ReFs",
            Self::Reiserfs => "reiserfs",
            Self::Reiser4 => "reiser4",
            Self::Ramfs => "ramfs",
            Self::Romfs => "romfs",
            Self::Rootfs => "rootfs",
            Self::RpcPipefs => "rpc_pipefs",
            Self::SecurityFs => "securityfs",
            Self::SeLinuxFs => "selinuxfs",
            Self::SiliconRaid => "silicon_medley_raid_member",
            Self::Sockfs => "sockfs",
            Self::Squashfs => "squashfs",
            Self::Squashfs3 => "squashfs3",
            Self::Stratis => "stratis",
            Self::Swap => "swap",
            Self::SwapSuspend => "swsuspend",
            Self::Sysfs => "sysfs",
            Self::SYSV => "sysv",
            Self::Tracefs => "tracefs",
            Self::Tmpfs => "tmpfs",
            Self::UBI => "ubi",
            Self::UBIFS => "ubifs",
            Self::UDF => "udf",
            Self::UFS => "ufs",
            Self::Usbfs => "usbfs",
            Self::Usbdevfs => "usbdevfs",
            Self::Vboxsf => "vboxsf",
            Self::VDO => "vdo",
            Self::VFAT => "vfat",
            Self::VIARaid => "via_raid_member",
            Self::Virtiofs => "virtiofs",
            Self::VMFS => "VMFS",
            Self::VMFSVolume => "VMFS_volume_member",
            Self::Vxfs => "vxfs",
            Self::Xenix => "xenix",
            Self::XFS => "xfs",
            Self::XFSLog => "xfs_external_log",
            Self::ZFS => "zfs_member",
            Self::Zonefs => "zonefs",
            Self::Unknown => "",
        }
    }

    /// Converts this `Filesystem` to a [`CString`].
    pub fn to_c_string(&self) -> CString {
        // FileSystem's string representation does not contain NULL characters,  we can safely
        // unwrap the new CString.
        CString::new(self.as_str()).unwrap()
    }
}

impl AsRef<FileSystem> for FileSystem {
    #[inline]
    fn as_ref(&self) -> &FileSystem {
        self
    }
}

impl AsRef<str> for FileSystem {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for FileSystem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for FileSystem {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "adaptec_raid_member" => Ok(Self::AdaptecRaid),
            "adfs" => Ok(Self::Adfs),
            "afs" => Ok(Self::Afs),
            "affs" => Ok(Self::Affs),
            "apfs" => Ok(Self::APFS),
            "aio" => Ok(Self::Aio),
            "autofs" => Ok(Self::Autofs),
            "bcache" => Ok(Self::Bcache),
            "bcachefs" => Ok(Self::BcacheFs),
            "bdev" => Ok(Self::Bdev),
            "befs" => Ok(Self::BeFS),
            "bfs" => Ok(Self::Bfs),
            "binder" => Ok(Self::Binder),
            "binfmt_misc" => Ok(Self::BinfmtMisc),
            "BitLocker" => Ok(Self::BitLocker),
            "ceph_bluestore" => Ok(Self::BlueStore),
            "bpf" => Ok(Self::Bpf),
            "btrfs" => Ok(Self::BTRFS),
            "ceph" => Ok(Self::Ceph),
            "cifs" => Ok(Self::Cifs),
            "cgroup" => Ok(Self::Cgroup),
            "cgroup2" => Ok(Self::Cgroup2),
            "configfs" => Ok(Self::Configfs),
            "cpuset" => Ok(Self::Cpuset),
            "cramfs" => Ok(Self::Cramfs),
            "debugfs" => Ok(Self::Debugfs),
            "devpts" => Ok(Self::Devpts),
            "devtmpfs" => Ok(Self::Devtmpfs),
            "ddf_raid_member" => Ok(Self::DDFRaid),
            "DM_integrity" => Ok(Self::DmIntegrity),
            "DM_snapshot_cow" => Ok(Self::DmSnapshot),
            "DM_verify_hash" => Ok(Self::DmVerify),
            "drbd" => Ok(Self::DRBD),
            "drbdmanage_control_volume" => Ok(Self::DRBDManage),
            "drbdproxy_datalog" => Ok(Self::DRBDProxyDatalog),
            "ecryptfs" => Ok(Self::Ecryptfs),
            "efivarfs" => Ok(Self::Efivarfs),
            "efs" => Ok(Self::Efs),
            "erofs" => Ok(Self::EROFS),
            "eventpollfs" => Ok(Self::Eventpollfs),
            "exfat" => Ok(Self::ExFAT),
            "exfs" => Ok(Self::Exfs),
            "ext2" => Ok(Self::Ext2),
            "ext3" => Ok(Self::Ext3),
            "ext4" => Ok(Self::Ext4),
            "ext4dev" => Ok(Self::Ext4Dev),
            "f2fs" => Ok(Self::F2FS),
            "cs_fvault2" => Ok(Self::FileVault),
            "fuse" => Ok(Self::Fuse),
            "fuse.portal" => Ok(Self::FusePortal),
            "fuseblk" => Ok(Self::Fuseblk),
            "fusectl" => Ok(Self::Fusectl),
            "futexfs" => Ok(Self::Futexfs),
            "gfs" => Ok(Self::GFS),
            "gfs2" => Ok(Self::GFS2),
            "hfs" => Ok(Self::HFS),
            "hfsplus" => Ok(Self::HFSPlus),
            "hpt37x_raid_member" => Ok(Self::HighPoint37x),
            "hpt45x_raid_member" => Ok(Self::HighPoint45x),
            "hostfs" => Ok(Self::Hostfs),
            "hpfs" => Ok(Self::HPFS),
            "hugetlbfs" => Ok(Self::HugeTlbFs),
            "iso9660" => Ok(Self::Iso9660),
            "isw_raid_member" => Ok(Self::ISWRaid),
            "jbd" => Ok(Self::JBD),
            "jffs2" => Ok(Self::Jffs2),
            "jfs" => Ok(Self::JFS),
            "jmicron_raid_member" => Ok(Self::JmicronRaid),
            "linux_raid_member" => Ok(Self::LinuxRaid),
            "lsi_mega_raid_member" => Ok(Self::LSIRaid),
            "crypto_LUKS" => Ok(Self::LUKS),
            "LVM1_member" => Ok(Self::LVM1),
            "LVM2_member" => Ok(Self::LVM2),
            "minix" => Ok(Self::Minix),
            "mpool" => Ok(Self::Mpool),
            "mqueue" => Ok(Self::Mqueue),
            "nss" => Ok(Self::Netware),
            "nfs" => Ok(Self::NFS),
            "nilfs2" => Ok(Self::Nilfs2),
            "none" => Ok(Self::None),
            "nsfs" => Ok(Self::NSFS),
            "ntfs" => Ok(Self::NTFS),
            "ntfs3" => Ok(Self::NTFS3),
            "nvidia_raid_member" => Ok(Self::NvidiaRaid),
            "ocfs" => Ok(Self::OCFS),
            "ocfs2" => Ok(Self::OCFS2),
            "ocfs2_dlmfs" => Ok(Self::OCFS2Dlmfs),
            "omfs" => Ok(Self::Omfs),
            "openpromfs" => Ok(Self::Openpromfs),
            "pidfs" => Ok(Self::Pidfs),
            "pipefs" => Ok(Self::Pipefs),
            "proc" => Ok(Self::Proc),
            "promise_fasttrack_raid_member" => Ok(Self::PromiseRaid),
            "pseudo_erofs" => Ok(Self::PseudoEROFS),
            "pstore" => Ok(Self::Pstore),
            "pvfs2" => Ok(Self::Pvfs2),
            "qnx4" => Ok(Self::QNX4),
            "qnx6" => Ok(Self::QNX6),
            "ReFs" => Ok(Self::ReFs),
            "reiserfs" => Ok(Self::Reiserfs),
            "reiser4" => Ok(Self::Reiser4),
            "ramfs" => Ok(Self::Ramfs),
            "romfs" => Ok(Self::Romfs),
            "rootfs" => Ok(Self::Rootfs),
            "rpc_pipefs" => Ok(Self::RpcPipefs),
            "securityfs" => Ok(Self::SecurityFs),
            "selinuxfs" => Ok(Self::SeLinuxFs),
            "silicon_medley_raid_member" => Ok(Self::SiliconRaid),
            "sockfs" => Ok(Self::Sockfs),
            "squashfs" => Ok(Self::Squashfs),
            "squashfs3" => Ok(Self::Squashfs3),
            "stratis" => Ok(Self::Stratis),
            "swap" => Ok(Self::Swap),
            "swsuspend" => Ok(Self::SwapSuspend),
            "sysfs" => Ok(Self::Sysfs),
            "sysv" => Ok(Self::SYSV),
            "tmpfs" => Ok(Self::Tmpfs),
            "tracefs" => Ok(Self::Tracefs),
            "ubi" => Ok(Self::UBI),
            "ubifs" => Ok(Self::UBIFS),
            "udf" => Ok(Self::UDF),
            "ufs" => Ok(Self::UFS),
            "usbfs" => Ok(Self::Usbfs),
            "usbdevfs" => Ok(Self::Usbdevfs),
            "vboxsf" => Ok(Self::Vboxsf),
            "vdo" => Ok(Self::VDO),
            "vfat" => Ok(Self::VFAT),
            "via_raid_member" => Ok(Self::VIARaid),
            "virtiofs" => Ok(Self::Virtiofs),
            "VMFS" => Ok(Self::VMFS),
            "VMFS_volume_member" => Ok(Self::VMFSVolume),
            "vxfs" => Ok(Self::Vxfs),
            "xenix" => Ok(Self::Xenix),
            "xfs" => Ok(Self::XFS),
            "xfs_external_log" => Ok(Self::XFSLog),
            "zfs_member" => Ok(Self::ZFS),
            "zonefs" => Ok(Self::Zonefs),
            "" => Ok(Self::Unknown),
            unsupported => {
                let err_msg = format!("unsupported file system: {:?}", unsupported);
                Err(ParserError::FileSystem(err_msg))
            }
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    #[should_panic(expected = "unsupported file system")]
    fn file_system_can_not_parse_an_invalid_file_system_type() {
        let _: FileSystem = "DUMMY".parse().unwrap();
    }

    #[test]
    fn file_system_can_parse_a_valid_file_system_type() -> crate::Result<()> {
        let fs_str = "adaptec_raid_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::AdaptecRaid;
        assert_eq!(actual, expected);

        let fs_str = "adfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Adfs;
        assert_eq!(actual, expected);

        let fs_str = "afs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Afs;
        assert_eq!(actual, expected);

        let fs_str = "affs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Affs;
        assert_eq!(actual, expected);

        let fs_str = "apfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::APFS;
        assert_eq!(actual, expected);

        let fs_str = "aio";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Aio;
        assert_eq!(actual, expected);

        let fs_str = "autofs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Autofs;
        assert_eq!(actual, expected);

        let fs_str = "bcache";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Bcache;
        assert_eq!(actual, expected);

        let fs_str = "bcachefs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::BcacheFs;
        assert_eq!(actual, expected);

        let fs_str = "bdev";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Bdev;
        assert_eq!(actual, expected);

        let fs_str = "befs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::BeFS;
        assert_eq!(actual, expected);

        let fs_str = "bfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Bfs;
        assert_eq!(actual, expected);

        let fs_str = "binder";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Binder;
        assert_eq!(actual, expected);

        let fs_str = "binfmt_misc";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::BinfmtMisc;
        assert_eq!(actual, expected);

        let fs_str = "BitLocker";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::BitLocker;
        assert_eq!(actual, expected);

        let fs_str = "ceph_bluestore";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::BlueStore;
        assert_eq!(actual, expected);

        let fs_str = "bpf";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Bpf;
        assert_eq!(actual, expected);

        let fs_str = "btrfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::BTRFS;
        assert_eq!(actual, expected);

        let fs_str = "ceph";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Ceph;
        assert_eq!(actual, expected);

        let fs_str = "cifs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Cifs;
        assert_eq!(actual, expected);

        let fs_str = "cgroup";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Cgroup;
        assert_eq!(actual, expected);

        let fs_str = "cgroup2";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Cgroup2;
        assert_eq!(actual, expected);

        let fs_str = "configfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Configfs;
        assert_eq!(actual, expected);

        let fs_str = "cpuset";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Cpuset;
        assert_eq!(actual, expected);

        let fs_str = "cramfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Cramfs;
        assert_eq!(actual, expected);

        let fs_str = "debugfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Debugfs;
        assert_eq!(actual, expected);

        let fs_str = "devpts";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Devpts;
        assert_eq!(actual, expected);

        let fs_str = "devtmpfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Devtmpfs;
        assert_eq!(actual, expected);

        let fs_str = "ddf_raid_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::DDFRaid;
        assert_eq!(actual, expected);

        let fs_str = "DM_integrity";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::DmIntegrity;
        assert_eq!(actual, expected);

        let fs_str = "DM_snapshot_cow";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::DmSnapshot;
        assert_eq!(actual, expected);

        let fs_str = "DM_verify_hash";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::DmVerify;
        assert_eq!(actual, expected);

        let fs_str = "drbd";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::DRBD;
        assert_eq!(actual, expected);

        let fs_str = "drbdmanage_control_volume";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::DRBDManage;
        assert_eq!(actual, expected);

        let fs_str = "drbdproxy_datalog";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::DRBDProxyDatalog;
        assert_eq!(actual, expected);

        let fs_str = "ecryptfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Ecryptfs;
        assert_eq!(actual, expected);

        let fs_str = "efivarfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Efivarfs;
        assert_eq!(actual, expected);

        let fs_str = "efs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Efs;
        assert_eq!(actual, expected);

        let fs_str = "erofs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::EROFS;
        assert_eq!(actual, expected);

        let fs_str = "eventpollfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Eventpollfs;
        assert_eq!(actual, expected);

        let fs_str = "exfat";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::ExFAT;
        assert_eq!(actual, expected);

        let fs_str = "exfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Exfs;
        assert_eq!(actual, expected);

        let fs_str = "ext2";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Ext2;
        assert_eq!(actual, expected);

        let fs_str = "ext3";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Ext3;
        assert_eq!(actual, expected);

        let fs_str = "ext4";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Ext4;
        assert_eq!(actual, expected);

        let fs_str = "ext4dev";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Ext4Dev;
        assert_eq!(actual, expected);

        let fs_str = "f2fs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::F2FS;
        assert_eq!(actual, expected);

        let fs_str = "cs_fvault2";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::FileVault;
        assert_eq!(actual, expected);

        let fs_str = "fuse";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Fuse;
        assert_eq!(actual, expected);

        let fs_str = "fuse.portal";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::FusePortal;
        assert_eq!(actual, expected);

        let fs_str = "fuseblk";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Fuseblk;
        assert_eq!(actual, expected);

        let fs_str = "fusectl";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Fusectl;
        assert_eq!(actual, expected);

        let fs_str = "futexfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Futexfs;
        assert_eq!(actual, expected);

        let fs_str = "gfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::GFS;
        assert_eq!(actual, expected);

        let fs_str = "gfs2";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::GFS2;
        assert_eq!(actual, expected);

        let fs_str = "hfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::HFS;
        assert_eq!(actual, expected);

        let fs_str = "hfsplus";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::HFSPlus;
        assert_eq!(actual, expected);

        let fs_str = "hpt37x_raid_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::HighPoint37x;
        assert_eq!(actual, expected);

        let fs_str = "hpt45x_raid_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::HighPoint45x;
        assert_eq!(actual, expected);

        let fs_str = "hostfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Hostfs;
        assert_eq!(actual, expected);

        let fs_str = "hpfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::HPFS;
        assert_eq!(actual, expected);

        let fs_str = "hugetlbfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::HugeTlbFs;
        assert_eq!(actual, expected);

        let fs_str = "iso9660";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Iso9660;
        assert_eq!(actual, expected);

        let fs_str = "isw_raid_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::ISWRaid;
        assert_eq!(actual, expected);

        let fs_str = "jbd";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::JBD;
        assert_eq!(actual, expected);

        let fs_str = "jffs2";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Jffs2;
        assert_eq!(actual, expected);

        let fs_str = "jfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::JFS;
        assert_eq!(actual, expected);

        let fs_str = "jmicron_raid_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::JmicronRaid;
        assert_eq!(actual, expected);

        let fs_str = "linux_raid_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::LinuxRaid;
        assert_eq!(actual, expected);

        let fs_str = "lsi_mega_raid_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::LSIRaid;
        assert_eq!(actual, expected);

        let fs_str = "crypto_LUKS";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::LUKS;
        assert_eq!(actual, expected);

        let fs_str = "LVM1_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::LVM1;
        assert_eq!(actual, expected);

        let fs_str = "LVM2_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::LVM2;
        assert_eq!(actual, expected);

        let fs_str = "minix";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Minix;
        assert_eq!(actual, expected);

        let fs_str = "mpool";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Mpool;
        assert_eq!(actual, expected);

        let fs_str = "mqueue";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Mqueue;
        assert_eq!(actual, expected);

        let fs_str = "nss";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Netware;
        assert_eq!(actual, expected);

        let fs_str = "nfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::NFS;
        assert_eq!(actual, expected);

        let fs_str = "nilfs2";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Nilfs2;
        assert_eq!(actual, expected);

        let fs_str = "none";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::None;
        assert_eq!(actual, expected);

        let fs_str = "nsfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::NSFS;
        assert_eq!(actual, expected);

        let fs_str = "ntfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::NTFS;
        assert_eq!(actual, expected);

        let fs_str = "ntfs3";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::NTFS3;
        assert_eq!(actual, expected);

        let fs_str = "nvidia_raid_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::NvidiaRaid;
        assert_eq!(actual, expected);

        let fs_str = "ocfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::OCFS;
        assert_eq!(actual, expected);

        let fs_str = "ocfs2";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::OCFS2;
        assert_eq!(actual, expected);

        let fs_str = "ocfs2_dlmfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::OCFS2Dlmfs;
        assert_eq!(actual, expected);

        let fs_str = "omfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Omfs;
        assert_eq!(actual, expected);

        let fs_str = "openpromfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Openpromfs;
        assert_eq!(actual, expected);

        let fs_str = "pidfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Pidfs;
        assert_eq!(actual, expected);

        let fs_str = "pipefs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Pipefs;
        assert_eq!(actual, expected);

        let fs_str = "proc";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Proc;
        assert_eq!(actual, expected);

        let fs_str = "promise_fasttrack_raid_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::PromiseRaid;
        assert_eq!(actual, expected);

        let fs_str = "pseudo_erofs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::PseudoEROFS;
        assert_eq!(actual, expected);

        let fs_str = "pstore";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Pstore;
        assert_eq!(actual, expected);

        let fs_str = "pvfs2";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Pvfs2;
        assert_eq!(actual, expected);

        let fs_str = "qnx4";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::QNX4;
        assert_eq!(actual, expected);

        let fs_str = "qnx6";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::QNX6;
        assert_eq!(actual, expected);

        let fs_str = "ReFs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::ReFs;
        assert_eq!(actual, expected);

        let fs_str = "reiserfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Reiserfs;
        assert_eq!(actual, expected);

        let fs_str = "reiser4";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Reiser4;
        assert_eq!(actual, expected);

        let fs_str = "ramfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Ramfs;
        assert_eq!(actual, expected);

        let fs_str = "romfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Romfs;
        assert_eq!(actual, expected);

        let fs_str = "rootfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Rootfs;
        assert_eq!(actual, expected);

        let fs_str = "rpc_pipefs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::RpcPipefs;
        assert_eq!(actual, expected);

        let fs_str = "securityfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::SecurityFs;
        assert_eq!(actual, expected);

        let fs_str = "selinuxfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::SeLinuxFs;
        assert_eq!(actual, expected);

        let fs_str = "silicon_medley_raid_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::SiliconRaid;
        assert_eq!(actual, expected);

        let fs_str = "sockfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Sockfs;
        assert_eq!(actual, expected);

        let fs_str = "squashfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Squashfs;
        assert_eq!(actual, expected);

        let fs_str = "squashfs3";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Squashfs3;
        assert_eq!(actual, expected);

        let fs_str = "stratis";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Stratis;
        assert_eq!(actual, expected);

        let fs_str = "swap";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Swap;
        assert_eq!(actual, expected);

        let fs_str = "swsuspend";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::SwapSuspend;
        assert_eq!(actual, expected);

        let fs_str = "sysfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Sysfs;
        assert_eq!(actual, expected);

        let fs_str = "sysv";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::SYSV;
        assert_eq!(actual, expected);

        let fs_str = "tmpfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Tmpfs;
        assert_eq!(actual, expected);

        let fs_str = "tracefs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Tracefs;
        assert_eq!(actual, expected);

        let fs_str = "ubi";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::UBI;
        assert_eq!(actual, expected);

        let fs_str = "ubifs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::UBIFS;
        assert_eq!(actual, expected);

        let fs_str = "udf";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::UDF;
        assert_eq!(actual, expected);

        let fs_str = "ufs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::UFS;
        assert_eq!(actual, expected);

        let fs_str = "usbfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Usbfs;
        assert_eq!(actual, expected);

        let fs_str = "usbdevfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Usbdevfs;
        assert_eq!(actual, expected);

        let fs_str = "vboxsf";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Vboxsf;
        assert_eq!(actual, expected);

        let fs_str = "vdo";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::VDO;
        assert_eq!(actual, expected);

        let fs_str = "vfat";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::VFAT;
        assert_eq!(actual, expected);

        let fs_str = "via_raid_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::VIARaid;
        assert_eq!(actual, expected);

        let fs_str = "virtiofs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Virtiofs;
        assert_eq!(actual, expected);

        let fs_str = "VMFS";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::VMFS;
        assert_eq!(actual, expected);

        let fs_str = "VMFS_volume_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::VMFSVolume;
        assert_eq!(actual, expected);

        let fs_str = "vxfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Vxfs;
        assert_eq!(actual, expected);

        let fs_str = "xenix";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Xenix;
        assert_eq!(actual, expected);

        let fs_str = "xfs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::XFS;
        assert_eq!(actual, expected);

        let fs_str = "xfs_external_log";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::XFSLog;
        assert_eq!(actual, expected);

        let fs_str = "zfs_member";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::ZFS;
        assert_eq!(actual, expected);

        let fs_str = "zonefs";
        let actual: FileSystem = fs_str.parse()?;
        let expected = FileSystem::Zonefs;
        assert_eq!(actual, expected);

        Ok(())
    }
}
