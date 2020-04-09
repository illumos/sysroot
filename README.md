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

## sysroot for Rust

It is important to be deliberate with respect to what is included in the
sysroot archive.  The contents will affect the choice of target systems for
which binaries can be produced by the cross compiler.

In that spirit, we have selected the following illumos-gate commit:

```
commit de6af22ae73ba8d72672288621ff50b88f2cf5fd
Author:     Jason King <jason.brian.king@gmail.com>
AuthorDate: Thu Dec 13 10:43:17 2018 -0800
Commit:     Joshua M. Clulow <josh@sysmgr.org>
CommitDate: Thu Dec 13 10:43:17 2018 -0800

    9971 Make getrandom(2) a public interface
    Reviewed by: Dan McDonald <danmcd@joyent.com>
    Reviewed by: Mike Gerdts <mike.gerdts@joyent.com>
    Reviewed by: Peter Tribble <peter.tribble@gmail.com>
    Reviewed by: Robert Mustacchi <rm@joyent.com>
    Reviewed by: Andy Fiddaman <omnios@citrus-it.net>
    Reviewed by: Igor Kozhukhov <igor@dilos.org>
    Approved by: Joshua M. Clulow <josh@sysmgr.org>
```

This commit was available in:

* OpenIndiana in the [2019.04 ISO
  release](http://docs.openindiana.org/release-notes/2019.04-release-notes/),
  or via `pkg update` some time in December of that year.  Of note: packages
  from prior to 20190626 are sufficiently old at time of writing to have been
  garbage collected from the main IPS repository.

* SmartOS platform images starting with
  [20181220T002304Z](https://us-east.manta.joyent.com/Joyent_Dev/public/SmartOS/smartos.html#20181220T002304Z).

* OmniOS CE [releases](https://omniosce.org/schedule) starting with r151030
  (LTS, released 2019-05-06).

In addition to the illumos base, we are including the following additional
libraries that appear in `/usr/lib` on all of the above platforms:

* `/usr/lib/{,amd64}/libssp.so.0.0.0` (version `LIBSSP_1.0`)
* `/usr/lib/{,amd64}/libgcc_s.so.1` (version `GCC_4.8.0`)
