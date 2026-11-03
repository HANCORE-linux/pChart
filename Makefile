PREFIX ?= /usr/local
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)

all:
	cargo build --release --locked

install:
	install -Dm755 target/release/pchart "$(DESTDIR)$(PREFIX)/bin/pchart"
	install -Dm644 man/pchart.1 "$(DESTDIR)$(PREFIX)/share/man/man1/pchart.1"
	install -Dm644 LICENSE "$(DESTDIR)$(PREFIX)/share/licenses/pchart/LICENSE"
	install -Dm644 crates/pchart-draw/assets/OFL-JetBrainsMono.txt "$(DESTDIR)$(PREFIX)/share/licenses/pchart/OFL-JetBrainsMono.txt"
	install -Dm644 crates/pchart-draw/assets/OFL-NotoSans.txt "$(DESTDIR)$(PREFIX)/share/licenses/pchart/OFL-NotoSans.txt"

uninstall:
	rm -f "$(DESTDIR)$(PREFIX)/bin/pchart" "$(DESTDIR)$(PREFIX)/share/man/man1/pchart.1"
	rm -rf "$(DESTDIR)$(PREFIX)/share/licenses/pchart"

dist:
	mkdir -p dist
	git archive --format=tar.gz --prefix=pchart-$(VERSION)/ -o dist/pchart-$(VERSION).tar.gz HEAD

package: dist
	tools/package/arch.sh $(VERSION)

.PHONY: all install uninstall dist package
