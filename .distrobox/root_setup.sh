#! /usr/bin/bash

set -xe

APP_USER=${1?:"First argument must be the container user"}
SSH_PORT=${2?:"Second argument must be the SSH port"}
GROUP=$(id -g -n $APP_USER)

# Fix stupid Distrobox ownership caused by mounting these... >:C
# chown ${APP_USER}:${GROUP} $HOME/.gitconfig
# chown ${APP_USER}:${GROUP} $HOME/.ssh

# Create an sshd config
cat <<EOT >/etc/ssh/sshd_config.d/app.conf
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

# Silly bug workaround (https://askubuntu.com/questions/1110828/ssh-failed-to-start-missing-privilege-separation-directory-var-run-sshd)
mkdir -p /var/run/sshd
chmod 0755 /var/run/sshd
chown root:root /run/sshd

# Verify the config
/usr/sbin/sshd -t -e

# Start sshd
/usr/sbin/sshd &
