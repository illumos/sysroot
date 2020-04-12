#
# Makefile for the generation of illumos sysroot archives.
#

MACH =			i386
MACH64 =		amd64

OUTPUT =		output
TARBASE =		illumos-sysroot-$(MACH)
TARVERSION =		custom-v$(shell date +%Y%m%d-%H%M%S)
TARFILE =		$(OUTPUT)/$(TARBASE)-$(TARVERSION).tar

#
# When producing the official archive, override TARVERSION; e.g.
#	gmake archive TARVERSION=de6af22ae73b-20181213-v1
#

USRLIB =		usr/lib
USRLIB64 =		usr/lib/$(MACH64)

MF2TAR =		$(PWD)/mf2tar/target/release/mf2tar

#
# A list of IPS packages to include in the sysroot archive.  Note that no
# dependency resolution is done, so if you need the dependencies for an
# included package you must enumerate them explicitly here as well.
#
INCLUDE_PACKAGES =	system/header \
			system/library \
			system/library/math \
			system/library/c-runtime

#
# A list of paths to exclude, even if they appear in the packages listed above.
# This is useful in order to omit files from larger packages that contain
# things other than just headers and libraries, in order to keep the size of
# the sysroot archive down.
#
EXCLUDE_DIRS =		usr/share \
			etc \
			var \
			usr/bin \
			usr/sbin \
			usr/ccs \
			sbin \
			bin

#
# Shim libraries that we generate for artefacts that come from consolidations
# other than illumos-gate, but which are expected to appear in /usr/lib in
# every illumos distribution:
#
LIBGCC_32 =		shims/libgcc_s/$(MACH)/libgcc_s.so.1
LIBGCC_64 =		shims/libgcc_s/$(MACH64)/libgcc_s.so.1
LIBSSP_32 =		shims/libssp/$(MACH)/libssp.so.0.0.0
LIBSSP_64 =		shims/libssp/$(MACH64)/libssp.so.0.0.0

SHIM_TARGETS  =		$(LIBGCC_32) $(LIBGCC_64) $(LIBSSP_32) $(LIBSSP_64)

.PHONY: all
all: archive

.PHONY: shims
shims: $(SHIM_TARGETS)

$(LIBGCC_32) $(LIBGCC_64):
	$(MAKE) -C shims/libgcc_s

$(LIBSSP_32) $(LIBSSP_64):
	$(MAKE) -C shims/libssp

.PHONY: $(MF2TAR)
$(MF2TAR):
	cd mf2tar && cargo build --release

$(OUTPUT):
	mkdir -p $@

.PHONY: archive
archive: $(SHIM_TARGETS) | $(OUTPUT) $(MF2TAR)
	@if [[ -z "$(ILLUMOS_PKGREPO)" || \
		! -f "$(ILLUMOS_PKGREPO)/cfg_cache" ]]; then \
		printf 'ERROR: specify valid ILLUMOS_PKGREPO location\n' >&2; \
		exit 1; \
	fi
	$(MF2TAR) \
	    --repository $(ILLUMOS_PKGREPO) \
	    $(addprefix -P ,$(INCLUDE_PACKAGES)) \
	    $(addprefix -E ,$(EXCLUDE_DIRS)) \
	    \
	    --file $(USRLIB)/libgcc_s.so.1=$(LIBGCC_32) \
	    --file $(USRLIB64)/libgcc_s.so.1=$(LIBGCC_64) \
	    --link $(USRLIB)/libgcc_s.so=libgcc_s.so.1 \
	    --link $(USRLIB64)/libgcc_s.so=libgcc_s.so.1 \
	    \
	    --file $(USRLIB)/libssp.so.0.0.0=$(LIBSSP_32) \
	    --file $(USRLIB64)/libssp.so.0.0.0=$(LIBSSP_64) \
	    --link $(USRLIB)/libssp.so.0=libssp.so.0.0.0 \
	    --link $(USRLIB)/libssp.so=libssp.so.0.0.0 \
	    --link $(USRLIB64)/libssp.so.0=libssp.so.0.0.0 \
	    --link $(USRLIB64)/libssp.so=libssp.so.0.0.0 \
	    \
	    $(TARFILE)
	gzip < $(TARFILE) > $(TARFILE).gz

.PHONY: clean
clean:
	rm -rf $(PROTO) $(MAKE_STAMPS_DIR)
	$(MAKE) -C shims/libgcc_s clean
	$(MAKE) -C shims/libssp clean

.PHONY: clobber
clobber: clean
	cd mf2tar && cargo clean
	rm -rf $(OUTPUT)
