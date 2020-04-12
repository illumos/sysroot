
MACH64 =		amd64

OUTPUT =		output
TARBASE =		sysroot-illumos-$(MACH64)-
TARFILE =		$(OUTPUT)/$(TARBASE)$(shell date +%Y%m%d-%H%M%S).tar

RSYNC =			rsync
GNUTAR =		gtar

USRLIB =		usr/lib
USRLIB64 =		usr/lib/$(MACH64)

SHIMS =			libssp.so.0.0.0 \
			libgcc_s.so.1

MF2TAR =		$(PWD)/mf2tar/target/release/mf2tar

INCLUDE_PACKAGES =	system/header \
			system/library \
			system/library/math \
			system/library/c-runtime

EXCLUDE_DIRS =		usr/share \
			etc \
			var \
			usr/bin \
			usr/sbin \
			usr/ccs \
			sbin \
			bin

LIBGCC_32 =		shims/libgcc_s/i386/libgcc_s.so.1
LIBGCC_64 =		shims/libgcc_s/$(MACH64)/libgcc_s.so.1
LIBSSP_32 =		shims/libssp/i386/libssp.so.0.0.0
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
	    $(OUTPUT)/$(TARBASE)$(shell date +%Y%m%d-%H%M%S).tar
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
