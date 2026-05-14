# AgntOS Dev VM (VirtualBox) — Legacy

> **Note:** VirtualBox is the previous approach. The recommended dev workflow now uses the **QEMU flake VM**:
> `nix build .#agntos-dev-vm` then `./result/bin/run-agntos-dev-vm`.
> See the README for details.

This doc is kept for reference if QEMU is unavailable.

### 1. Download NixOS ISO

```bash
# Download NixOS minimal ISO (GNOME live image works best for initial setup)
wget https://channels.nixos.org/nixos-24.11/latest-nixos-minimal-x86_64-linux.iso
```

### 2. Create VM

Use the `scripts/create-vbox-vm.sh` script, or manually:

- Name: `agntos-dev`
- Type: Linux / Other Linux (64-bit)
- Memory: 8192 MB
- CPU: 4 cores
- Disk: 40 GB (VDI, dynamically allocated)
- Network: NAT + port forwarding: host 2222 -> guest 22
- Display: 128 MB VRAM, VMSVGA controller, 3D acceleration enabled
- Shared folder: Name `agntos-src`, path = agntos repo root, auto-mount, permanent

### 3. Install NixOS

Boot the ISO, partition the disk, and install:

```bash
# Partition (UEFI recommended for modern systems)
sudo parted /dev/sda -- mklabel gpt
sudo parted /dev/sda -- mkpart ESP fat32 1MB 512MB
sudo parted /dev/sda -- mkpart primary 512MB 100%
sudo parted /dev/sda -- set 1 esp on

# Format
sudo mkfs.fat -F 32 -n boot /dev/sda1
sudo mkfs.ext4 -L nixos /dev/sda2

# Mount
sudo mount /dev/disk/by-label/nixos /mnt
sudo mkdir -p /mnt/boot
sudo mount /dev/disk/by-label/boot /mnt/boot

# Generate config
sudo nixos-generate-config --root /mnt

# Install
sudo nixos-install

# Set root password when prompted, then reboot
sudo reboot
```

### 4. Mount Shared Folder

After reboot, mount the shared folder:

```bash
sudo mkdir -p /mnt/agntos-src
sudo mount -t vboxsf agntos-src /mnt/agntos-src
```

Add to `/etc/nixos/configuration.nix` for persistence:

```nix
fileSystems."/mnt/agntos-src" = {
  device = "agntos-src";
  fsType = "vboxsf";
  options = [ "rw" "nofail" ];
};
```

Then `sudo nixos-rebuild switch`.

### 5. Enable Flakes

Edit `/etc/nixos/configuration.nix` and add:

```nix
nix.settings.experimental-features = [ "nix-command" "flakes" ];
```

Then `sudo nixos-rebuild switch`.

### 6. Build AgntOS Tools

```bash
# Inside the VM, with the shared folder mounted:
cd /mnt/agntos-src
cargo build
```

### 7. Test agntctl

```bash
cargo run --bin agntctl -- inspect
```
