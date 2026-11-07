# The Linux natives' build: Debian 12 for a glibc old enough that the library loads on Debian 12
# and Ubuntu 22.04 (scripts/build_jars.py checks the newest symbol it asks for), with what
# llama.cpp's build wants (cmake, a C++ compiler, libclang for the bindings) and, for the Vulkan
# native, the Vulkan loader to link against, glslc for its shaders, and Khronos's own Vulkan and
# SPIR-V headers (Debian 12's are older than llama.cpp's Vulkan code). Headers only: at run time
# the library uses whatever Vulkan loader the player's driver installed.
FROM rust:1.98.1-bookworm
ARG VULKAN_HEADERS=v1.4.309
RUN apt-get update \
 && apt-get install -y --no-install-recommends cmake clang libclang-dev libvulkan-dev glslc spirv-headers \
 && rm -rf /var/lib/apt/lists/* \
 && rm -rf /usr/include/vulkan \
 && git clone --depth 1 --branch "$VULKAN_HEADERS" https://github.com/KhronosGroup/Vulkan-Headers.git /tmp/vh \
 && cmake -S /tmp/vh -B /tmp/vh/build -DCMAKE_INSTALL_PREFIX=/usr/local \
 && cmake --install /tmp/vh/build \
 && rm -rf /tmp/vh
