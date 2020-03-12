
SUBDIRS = amd64 i386


all: $(SUBDIRS)
clean: $(SUBDIRS)

$(SUBDIRS):
	$(MAKE) -C $@ $(MAKECMDGOALS)

.PHONY: $(SUBDIRS)
.PHONY: all clean
