
MACH64 =		amd64

PROTO =			proto

OUTPUT =		output
TARBASE =		sysroot-illumos-$(MACH64)-

MAKE_STAMPS_DIR ?=	make_stamps

MAKE_STAMP_REMOVE =	mkdir -p $(@D); rm -f $(@)
MAKE_STAMP_CREATE =	mkdir -p $(@D); touch $(@)

STAMP_ILLUMOS =		$(MAKE_STAMPS_DIR)/illumos
STAMP_LIBLINKS =	$(MAKE_STAMPS_DIR)/liblinks

RSYNC =			rsync
GNUTAR =		gtar
LIBLINKS =		$(PWD)/scripts/liblinks.sh

PROTO_USR_LIB =		$(PROTO)/usr/lib
PROTO_USR_LIB64 =	$(PROTO)/usr/lib/$(MACH64)

SHIMS =			libssp.so.0.0.0 \
			libgcc_s.so.1

PROTO_SHIMS =		$(addprefix $(PROTO_USR_LIB)/,$(SHIMS)) \
			$(addprefix $(PROTO_USR_LIB64)/,$(SHIMS))


.PHONY: all
all: archive

.PHONY: shims
shims: $(PROTO_SHIMS) $(STAMP_LIBLINKS)

$(STAMP_ILLUMOS): | $(PROTO)
	$(MAKE_STAMP_REMOVE)
	@if [[ -z "$(ILLUMOS_PROTO)" || \
	    ! -d "$(ILLUMOS_PROTO)/usr/lib" ]]; then \
		printf 'ERROR: specify valid ILLUMOS_PROTO location\n' >&2; \
		exit 1; \
	fi
	$(RSYNC) -a $(ILLUMOS_PROTO)/usr/ "$(PROTO)/usr/"
	$(RSYNC) -a $(ILLUMOS_PROTO)/lib/ "$(PROTO)/lib/"
	$(MAKE_STAMP_CREATE)

$(STAMP_LIBLINKS): $(LIBLINKS) | $(PROTO)
	$(MAKE_STAMP_REMOVE)
	$(LIBLINKS) $(PROTO) libssp.so.0.0.0 libssp.so.0 libssp.so
	$(LIBLINKS) $(PROTO) libgcc_s.so.1 libgcc_s.so
	$(MAKE_STAMP_CREATE)

$(PROTO_USR_LIB)/libgcc_s.so.1: | $(PROTO_USR_LIB)
	$(MAKE) -C shims/libgcc_s
	rm -f $@
	cp shims/libgcc_s/i386/$(@F) $@

$(PROTO_USR_LIB64)/libgcc_s.so.1: | $(PROTO_USR_LIB64)
	$(MAKE) -C shims/libgcc_s
	rm -f $@
	cp shims/libgcc_s/$(MACH64)/$(@F) $@

$(PROTO_USR_LIB)/libssp.so.0.0.0: | $(PROTO_USR_LIB)
	$(MAKE) -C shims/libssp
	rm -f $@
	cp shims/libssp/i386/$(@F) $@

$(PROTO_USR_LIB64)/libssp.so.0.0.0: | $(PROTO_USR_LIB64)
	$(MAKE) -C shims/libssp
	rm -f $@
	cp shims/libssp/$(MACH64)/$(@F) $@

$(PROTO) $(OUTPUT) $(PROTO_USR_LIB) $(PROTO_USR_LIB64):
	mkdir -p $@

.PHONY: archive
archive: $(STAMP_ILLUMOS) $(STAMP_LIBLINKS) $(PROTO_SHIMS) | $(OUTPUT)
	gtar -cz --directory=$(PROTO) --owner=0 --group=0 \
	    --file=$(OUTPUT)/$(TARBASE)$(shell date +%Y%m%d-%H%M%S).tar.gz \
	    usr lib

.PHONY: stamp-%
stamp-%: $(MAKE_STAMPS_DIR)/%
	@:

.PHONY: clean
clean:
	rm -rf $(PROTO) $(MAKE_STAMPS_DIR)
	$(MAKE) -C shims/libgcc_s clean
	$(MAKE) -C shims/libssp clean

.PHONY: clobber
clobber: clean
	rm -rf $(OUTPUT)
