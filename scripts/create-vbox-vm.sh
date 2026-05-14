#!/usr/bin/env bash
set -euo pipefail

# AgntOS VirtualBox Dev VM creation script.
# Creates a VBox VM with shared folders for AgntOS development.

VM_NAME="agntos-dev"
ISO_URL="https://channels.nixos.org/nixos-24.11/latest-nixos-minimal-x86_64-linux.iso"
ISO_FILE="/home/nate/Downloads/nixos-minimal-24.11.iso"
VM_DIR="$HOME/VirtualBox VMs/$VM_NAME"
DISK_SIZE_MB=40960
MEMORY_MB=8192
CPU_COUNT=4

echo "=== AgntOS VirtualBox Dev VM Setup ==="

# Download ISO if missing
if [ ! -f "$ISO_FILE" ]; then
    echo "Downloading NixOS ISO..."
    wget -q --show-progress -O "$ISO_FILE" "$ISO_URL"
    echo "Download complete: $ISO_FILE"
else
    echo "ISO already exists: $ISO_FILE"
fi

# Create VM if it doesn't exist
if VBoxManage list vms | grep -q "\"$VM_NAME\""; then
    echo "VM '$VM_NAME' already exists. Stopping..."
else
    echo "Creating VM '$VM_NAME'..."
    VBoxManage createvm --name "$VM_NAME" --ostype Linux26_64 --register

    VBoxManage modifyvm "$VM_NAME" \
        --memory "$MEMORY_MB" \
        --cpus "$CPU_COUNT" \
        --graphicscontroller vmsvga \
        --vram 128 \
        --accelerate3d on \
        --nic1 nat \
        --natpf1 "ssh,tcp,,2222,,22" \
        --audio-enabled off \
        --clipboard-mode bidirectional \
        --draganddrop bidirectional

    echo "VM created."
fi

# Create and attach disk
DISK_PATH="$VM_DIR/agntos-dev.vdi"
if [ ! -f "$DISK_PATH" ]; then
    echo "Creating disk ($DISK_SIZE_MB MB)..."
    VBoxManage createhd --filename "$DISK_PATH" --size "$DISK_SIZE_MB"
    VBoxManage storagectl "$VM_NAME" --name "SATA Controller" --add sata --controller IntelAhci
    VBoxManage storageattach "$VM_NAME" --storagectl "SATA Controller" \
        --port 0 --device 0 --type hdd --medium "$DISK_PATH"
fi

# Attach ISO
VBoxManage storageattach "$VM_NAME" --storagectl "SATA Controller" \
    --port 1 --device 0 --type dvddrive --medium "$ISO_FILE"

# Shared folder: mount agntos repo into /mnt/agntos-src
AGNTOS_ROOT=$(cd "$(dirname "$0")/.." && pwd)
echo "Shared folder source: $AGNTOS_ROOT"
VBoxManage sharedfolder add "$VM_NAME" \
    --name "agntos-src" \
    --hostpath "$AGNTOS_ROOT" \
    --automount --auto-mount-point "/mnt/agntos-src" \
    2>/dev/null || echo "Shared folder already exists or failed to add"

# Start the VM (no GUI unless requested)
echo ""
echo "=== VM Ready ==="
echo "Name:       $VM_NAME"
echo "Memory:     $MEMORY_MB MB"
echo "CPU:        $CPU_COUNT cores"
echo "Disk:       $DISK_SIZE_MB MB"
echo "SSH:        localhost:2222"
echo "Shared:     $AGNTOS_ROOT -> /mnt/agntos-src"
echo ""
echo "Start with:  VBoxManage startvm $VM_NAME"
echo "Stop with:   VBoxManage controlvm $VM_NAME poweroff"
echo ""
