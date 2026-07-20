#
# @see https://www.friedrichkurz.me/posts/2025-06-07-zed-devcontainer/
#

FROM docker.io/library/ubuntu:latest
RUN userdel -r ubuntu

#
# zed didn't ask for a password - leave app user password empty
#
ARG APP_USER="app"
ARG APP_PASSWORD=""
ARG APP_DIR="/workspace"
ARG APP_UID="1000"
ARG SSH_PORT="2222"

#
# install ssh server and some essential development commands
#
RUN apt update \
    && apt install -y --no-install-recommends \
    iproute2 \
    net-tools \
    procps \
    ssh \
    sudo \
    openssl \
    ca-certificates \
    && echo >/etc/sudoers.d/10-installer  "%sudo ALL=(ALL) NOPASSWD: ALL"

#
# basic sshd configuration
# - accept (empty) password authentication
# - listen on custom high port
# - accept some env variables
#
RUN cat <<EOT >/etc/ssh/sshd_config.d/app.conf
AcceptEnv LANG LC_* GIT_*
AllowUsers ${APP_USER}
AuthenticationMethods none
#AuthenticationMethods none,keyboard-interactive,password
KbdInteractiveAuthentication yes
LogLevel VERBOSE
PasswordAuthentication yes
PermitEmptyPasswords yes
Port ${SSH_PORT}
PrintMotd no
Subsystem sftp /usr/lib/openssh/sftp-server
UsePAM yes
EOT

# Install the rest of the packages that we require

# General development tools + Rust
# (though actual Rust setup is run as user later)
RUN apt install -y --no-install-recommends \
	git \
	cmake \
	build-essential \
	ninja-build \
	vim \
	gdb \
	rustup

# For building Raylib dependency.
# Beware: if the wrong Clang package is installed, bindgen can
# fail to find stdarg.h (or other system headers).
RUN apt install -y --no-install-recommends \
	mesa-utils \
	mesa-common-dev \
	libxrandr-dev \
	libxinerama-dev \
	libxcursor-dev \
	libxi-dev \
	libclang1-20 \
	librust-clang-sys-dev

# Clean up redundant things
RUN apt clean

#
# add development group and user
#
RUN groupadd -f -g ${APP_UID} ${APP_USER}
RUN useradd -u ${APP_UID} -g ${APP_UID} -G sudo -p "${APP_PASSWORD}" -m -s /bin/bash ${APP_USER}

#
# validate sshd config
#
RUN /usr/sbin/sshd -t -e

#
# prepare app
#
USER ${APP_USER}
WORKDIR ${APP_DIR}

COPY --chown=${APP_UID}:${APP_UID} . ${APP_DIR}

# Set up our target versions of Rust
RUN rustup default 1.90

# For correct formatting (see comments in rustfmt.toml)
RUN rustup toolchain install nightly
RUN rustup default stable

VOLUME [ "${APP_DIR}" ]

#
# start sshd
#
USER root
EXPOSE ${SSH_PORT}
CMD ["/usr/sbin/sshd", "-D", "-e"]
