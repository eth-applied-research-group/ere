#!/bin/bash

set -e

sudo ubuntu-drivers install

if command -v nvcc &> /dev/null; then
    echo "Cuda toolkit is already installed."
else
    # From https://docs.nvidia.com/cuda/cuda-installation-guide-linux/#ubuntu.
    wget https://developer.download.nvidia.com/compute/cuda/repos/$(. /etc/os-release; echo "${ID}${VERSION_ID}" | tr -d '.' | tr '[:upper:]' '[:lower:]')/$(uname -m)/cuda-keyring_1.1-1_all.deb
    sudo dpkg -i cuda-keyring_1.1-1_all.deb
    rm cuda-keyring_1.1-1_all.deb
    sudo apt-get update
    sudo apt-get install -y cuda-toolkit

    # From https://docs.nvidia.com/cuda/cuda-installation-guide-linux/#environment-setup.
    # Add to path.
    cat >> ~/.bashrc <<EOF
export PATH="$PATH:/usr/local/cuda/bin"
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/usr/local/cuda/lib64"
EOF
fi

if command -v nvidia-container-runtime &> /dev/null; then
    echo "Nvidia container runtime is already installed."
else
    # From https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html#with-apt-ubuntu-debian.
    # Configure the production repository:
    curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | sudo gpg --dearmor -o /usr/share/keyrings/nvidia-container-toolkit-keyring.gpg \
        && curl -s -L https://nvidia.github.io/libnvidia-container/stable/deb/nvidia-container-toolkit.list \
            | sed 's#deb https://#deb [signed-by=/usr/share/keyrings/nvidia-container-toolkit-keyring.gpg] https://#g' \
            | sudo tee /etc/apt/sources.list.d/nvidia-container-toolkit.list

    # Install the NVIDIA Container Toolkit packages:
    export NVIDIA_CONTAINER_TOOLKIT_VERSION=1.17.8-1
    sudo apt-get install -y \
        nvidia-container-toolkit=${NVIDIA_CONTAINER_TOOLKIT_VERSION} \
        nvidia-container-toolkit-base=${NVIDIA_CONTAINER_TOOLKIT_VERSION} \
        libnvidia-container-tools=${NVIDIA_CONTAINER_TOOLKIT_VERSION} \
        libnvidia-container1=${NVIDIA_CONTAINER_TOOLKIT_VERSION}

    # From https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html#configuring-docker.
    # Configure the container runtime by using the `nvidia-ctk` command:
    sudo nvidia-ctk runtime configure --runtime=docker

    # Restart docker
    sudo systemctl restart docker
fi
