## Debian 12
FROM debian:bookworm
ENV DEBIAN_FRONTEND=noninteractive

RUN rm /etc/apt/apt.conf.d/docker-clean
RUN echo 'APT::Keep-Downloaded-Packages "true";' >/etc/apt/apt.conf.d/keep-downloaded-packages

RUN \
	--mount=type=cache,sharing=locked,target=/var/cache/apt,id=debian-bookworm-var-cache-apt \
	--mount=type=cache,sharing=locked,target=/var/lib/apt,id=debian-bookworm-var-lib-apt \
	apt-get update && apt-get install -y \
		build-essential git \
		libcairomm-1.0-dev \
		libexiv2-dev \
		libgtkmm-3.0-dev \
		libmagic-dev \
		libturbojpeg0-dev
