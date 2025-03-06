## Debian Unstable
FROM debian:sid
ENV DEBIAN_FRONTEND=noninteractive

RUN rm /etc/apt/apt.conf.d/docker-clean
RUN echo 'APT::Keep-Downloaded-Packages "true";' >/etc/apt/apt.conf.d/keep-downloaded-packages

RUN \
	--mount=type=cache,sharing=locked,target=/var/cache/apt,id=debian-sid-var-cache-apt \
	--mount=type=cache,sharing=locked,target=/var/lib/apt,id=debian-sid-var-lib-apt \
	apt-get update && apt-get install -y \
		build-essential rustc cargo rust-clippy git \
		cmake nasm \
		libgtk-3-dev \
		libexiv2-dev \
		libgexiv2-dev \
		libturbojpeg0-dev

# Jenkins always runs commands in the container as its own UID/GID, which has no
# access to anything outside the container. It also does this for the entrypoint
# so a setuid executable is required for this hack.
#
# This file should not have a fixed UID/GID for Jenkins so determine the UID/GID
# at runtime when executing the entrypoint and use the setuid shell to set up
# a user and fix filesystem permissions on startup. We can't use sudo (which is
# just extra overhead) because that requires an entry in /etc/passwd.
RUN chmod u+s /bin/dash
RUN <<EOF
	cat >/entrypoint.sh <<EOT
#!/bin/dash
if [ -u /bin/dash ]; then
	/bin/dash -pc "groupadd -g \$(id -g) user" || exit 1
	/bin/dash -pc "useradd -u \$(id -u) -g \$(id -g) -d /home/user -s /bin/bash user" || exit 1
	/bin/dash -pc "install -d -m 0700 -o user -g user /home/user /home/user/.cargo" || exit 1
	/bin/dash -pc "chmod u-s /bin/dash" || exit 1
fi
exec "\$@"
EOT
	chmod +x /entrypoint.sh
EOF

HEALTHCHECK --start-period=1s --start-interval=1s --retries=1 CMD [ ! -u /bin/dash ]
ENTRYPOINT ["/entrypoint.sh"]
