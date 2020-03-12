
OBJECTS = shims.o
MAPFILE = mapfile-gen
LIB = libgcc_s.so.1

VERSION ?= 4_8_0

ifneq ($(shell echo $(VERSION) | grep '[3-7]_[0-9]_[0-9]'), $(VERSION))
    # TODO: validate against supported versions
    $(error "Version $(VERSION) invalid (expecting X_Y_X)")
endif

CFLAGS = -Wno-builtin-declaration-mismatch -D$(VERSTRING)
LDFLAGS = -G -s -h libgcc_s.so.1

COMMON = ../common
MAPFILE = $(COMMON)/mapfile.shim
MAPFILE_VERSION = mapfile.version

VERSTRING = VER_$(VERSION)

%.o: $(COMMON)/%.c
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $<

$(LIB): $(OBJECTS) $(MAPFILE)
	sh -c 'echo "\$$mapfile_version 2\n\$$add $(VERSTRING)" > $(MAPFILE_VERSION)'
	$(LD) $(LDFLAGS) -M $(MAPFILE_VERSION) -M $(MAPFILE) -o $@ $(OBJECTS)


.PHONY: clean
clean:
	rm -f $(OBJECTS) $(LIB) $(MAPFILE_VERSION)

