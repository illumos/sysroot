Witness the majesty:

```
$ gmake ILLUMOS_PROTO=/ws/rti/proto/root_i386-nd
mkdir -p proto
mkdir -p make_stamps; rm -f make_stamps/illumos
rsync -a /ws/rti/proto/root_i386-nd/usr/ "proto/usr/"
rsync -a /ws/rti/proto/root_i386-nd/lib/ "proto/lib/"
mkdir -p make_stamps; touch make_stamps/illumos
mkdir -p make_stamps; rm -f make_stamps/lift
/ws/sysroot/tool/scripts/lift_library.sh proto libssp.so.0.0.0 libssp.so.0 libssp.so
    lifting: /usr/lib/libssp.so.0.0.0
    lifting: /usr/lib/amd64/libssp.so.0.0.0
    linking: /usr/lib/libssp.so.0 -> libssp.so.0.0.0
    linking: /usr/lib/amd64/libssp.so.0 -> libssp.so.0.0.0
    linking: /usr/lib/libssp.so -> libssp.so.0.0.0
    linking: /usr/lib/amd64/libssp.so -> libssp.so.0.0.0
/ws/sysroot/tool/scripts/lift_library.sh proto libgcc_s.so.1 libgcc_s.so
    lifting: /usr/lib/libgcc_s.so.1
    lifting: /usr/lib/amd64/libgcc_s.so.1
    linking: /usr/lib/libgcc_s.so -> libgcc_s.so.1
    linking: /usr/lib/amd64/libgcc_s.so -> libgcc_s.so.1
mkdir -p make_stamps; touch make_stamps/lift
mkdir -p output
gtar -cz --directory=proto --owner=0 --group=0 \
    --file=output/sysroot-illumos-amd64-20200312-122033.tar.gz \
    usr lib

$ tar tvfz output/sysroot-illumos-amd64-20200312-122033.tar.gz | egrep 'libssp|libgcc_s'
-rwxrwxrwx   0/0       15 Mar 12 12:20 2020 usr/lib/libssp.so.0 symbolic link to libssp.so.0.0.0
-rwxrwxrwx   0/0       13 Mar 12 12:20 2020 usr/lib/libgcc_s.so symbolic link to libgcc_s.so.1
-rwxrwxrwx   0/0       15 Mar 12 12:20 2020 usr/lib/libssp.so symbolic link to libssp.so.0.0.0
-r-xr-xr-x   0/0   442956 Mar 12 12:20 2020 usr/lib/libgcc_s.so.1
-r-xr-xr-x   0/0    31684 Mar 12 12:20 2020 usr/lib/libssp.so.0.0.0
-rwxrwxrwx   0/0       15 Mar 12 12:20 2020 usr/lib/amd64/libssp.so symbolic link to libssp.so.0.0.0
-rwxrwxrwx   0/0       13 Mar 12 12:20 2020 usr/lib/amd64/libgcc_s.so symbolic link to libgcc_s.so.1
-rwxrwxrwx   0/0       15 Mar 12 12:20 2020 usr/lib/amd64/libssp.so.0 symbolic link to libssp.so.0.0.0
-r-xr-xr-x   0/0    43840 Mar 12 12:20 2020 usr/lib/amd64/libssp.so.0.0.0
-r-xr-xr-x   0/0   531776 Mar 12 12:20 2020 usr/lib/amd64/libgcc_s.so.1
```
