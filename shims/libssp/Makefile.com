
OBJECTS = shims.o
MAPFILE = mapfile-gen
LIB = libssp.so.0.0.0

CC = gcc
CFLAGS = -fno-builtin -w
LDFLAGS = -G -s -h libssp.so.0

COMMON = ../common
MAPFILE = $(COMMON)/mapfile.shim

%.o: $(COMMON)/%.c
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $<

$(LIB): $(OBJECTS) $(MAPFILE)
	$(LD) $(LDFLAGS) -M $(MAPFILE) -o $@ $(OBJECTS)


.PHONY: clean
clean:
	rm -f $(OBJECTS) $(LIB)

