.PHONY: all clean install

all:
	+$(MAKE) -C build all

clean:
	+$(MAKE) -C build clean

install: all
	install -D build/fiv $(DESTDIR)/usr/bin/fiv
