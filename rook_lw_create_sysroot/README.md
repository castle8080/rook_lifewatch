# Sysroot Cross Compiler Setup

These scripts download and create a raspberry pi sysroot from an OS image.

1. It will download the os image.
2. Mount that image
3. Fix symlinks
4. Install opencv4 and libcamera-dev etc...
5. Tars up the sysroot.

A valid sysroot should be created under var/sysroots.

Download images will be under var/rpi_images. These can be removed once the sysroot has been created.

To run the process:

```./setup.sh```


**Dependencies**

- **Host (Debian/Ubuntu)**: packages required on the machine running these scripts:

```bash
sudo apt update
sudo apt install -y sudo curl xz-utils rsync tar zstd util-linux pkg-config qemu-user-static binfmt-support
```

- **Installed into the sysroot (inside chroot)**: packages the scripts install into the Raspberry Pi sysroot:

```text
libcamera-dev
libopencv-dev
pkg-config
cmake
build-essential
```

**Unpack Sysroot Tarball**

- **Created by:** `create-sysroot-archive.sh` (uses `--zstd`, `-p` and `--numeric-owner`).
- **Purpose:** extract a pre-made sysroot tarball while preserving symlinks, ownership and permissions.

To extract the archive and preserve numeric owners, permissions and symlinks, run:

```bash
# Extract into the sysroots directory (creates the sysroot directory named in the archive)
sudo tar --numeric-owner --zstd -xpf /path/to/<base>-sysroot.tar.zst -C /path/to/var/sysroots

```

Notes:
- `--numeric-owner` preserves UID/GID from the archive.
- `-p` preserves file permissions; `tar` preserves symlinks by default.
- Use `sudo` so owners and permissions can be restored correctly on extraction.
