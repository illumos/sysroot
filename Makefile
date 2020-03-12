
MACH64 =		amd64

PROTO =			proto

OUTPUT =		output
TARBASE =		sysroot-illumos-$(MACH64)-

MAKE_STAMPS_DIR ?=	make_stamps

MAKE_STAMP_REMOVE =	mkdir -p $(@D); rm -f $(@)
MAKE_STAMP_CREATE =	mkdir -p $(@D); touch $(@)

STAMP_ILLUMOS =		$(MAKE_STAMPS_DIR)/illumos
STAMP_LIFT =		$(MAKE_STAMPS_DIR)/lift

RSYNC =			rsync
GNUTAR =		gtar
LIFT_LIBRARY =		$(PWD)/scripts/lift_library.sh


all: archive

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

#
# XXX This isn't great, but was at least a start.  We should obviously not just
# lift this from the build machine:
#
$(STAMP_LIFT): $(LIFT_LIBRARY) | $(PROTO)
	$(MAKE_STAMP_REMOVE)
	$(LIFT_LIBRARY) $(PROTO) libssp.so.0.0.0 libssp.so.0 libssp.so
	$(LIFT_LIBRARY) $(PROTO) libgcc_s.so.1 libgcc_s.so
	$(MAKE_STAMP_CREATE)

$(PROTO) $(OUTPUT):
	mkdir -p $@

.PHONY: archive
archive: $(STAMP_ILLUMOS) $(STAMP_LIFT) | $(OUTPUT)
	gtar -cz --directory=$(PROTO) --owner=0 --group=0 \
	    --file=$(OUTPUT)/$(TARBASE)$(shell date +%Y%m%d-%H%M%S).tar.gz \
	    usr lib

.PHONY: stamp-%
stamp-%: $(MAKE_STAMPS_DIR)/%
	@:

.PHONY: clean
clean:
	rm -rf $(PROTO) $(MAKE_STAMPS_DIR)

.PHONY: clobber
clobber: clean
	rm -rf $(OUTPUT)
