# Tools for illumos Sysroot Generation

A sysroot archive contains artefacts such as shared libraries and C header
files.  The archive contents can be used for cross compilation of binaries for
an illumos system on a foreign operating system such as Linux.  They can also
be used, with some care, to cross compile binaries for an earlier version of
illumos on a more current version.  Compilers generally have some way of
building against a set of headers and libraries other than the native shipped
compilation environment; e.g., GCC provides the `--sysroot` option.

## Official Sysroot Archives

At various times, the illumos project will make available official archives
with sysroot contents that can be used by other projects.  They will be
uploaded [to the GitHub release
page](https://github.com/illumos/sysroot/releases) for this repository.

Release files will have names of the form:
`illumos-sysroot-$MACH-$DATE-$COMMIT-$VERSION.tar.gz`.  For example,
`illumos-sysroot-i386-20181213-de6af22ae73b-v1.tar.gz` would be artefacts for
x86 machines (32- and 64-bit) built from `illumos-gate` commit
`de6af22ae73ba8d72672288621ff50b88f2cf5fd` which integrated on 13th
December, 2018.  The version number (e.g., `v1`) reflects the revision of the
contents as determined by the build process in this repository; we may choose
to add additional files to the sysroot, without moving to a newer base commit,
if requested by a consumer.

## Producing Archives

To build a sysroot archive, you need to build *illumos-gate* such that you get
a fully populate IPS repository; i.e., `packages/$MACH/nightly-nd/repo.redist`.
If you happen to have an existing IPS repository with sufficiently complete
contents, you may be able to use `pkgrecv` to download packages into a local
repository tree and use that instead, but this has not been tested.

Note that in order to build an older version of *illumos-gate* on a newer build
host, sometimes we have to backport a small number of fixes to the build tools.
We may also need to use a somewhat unusual environment file for `nightly`.
Care in backporting must be taken, so as to preserve the accuracy of the
exposed API and ABI in the headers and libraries in the sysroot archive.  The
backport lives in a branch of *illumos-gate* named for the base version date;
e.g.,
[sysroot/20181213](https://github.com/illumos/illumos-gate/tree/sysroot/20181213).
The environment file lives in this repository under `env/`.

You'll need to install Rust (to build `mf2tar`) and a C compiler (to build the
shims).  Once you have those, and you have your illumos packages, making the
archive is (hopefully!) as simple as:

```
$ gmake archive \
    ILLUMOS_PKGREPO=/ws/oldgate/packages/i386/nightly-nd/repo.redist
...
gzip < output/illumos-sysroot-i386-custom-v20200411-224313.tar > output/illumos-sysroot-i386-custom-v20200411-224313.tar.gz
```

Note that by default, the archive will be named with a custom version string to
make it easy to see that it is not an official release.  Release maintainers
must override the `TARVERSION` make variable appropriately.

## Release Notes

### Sysroot Release 20181213 Version 1

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
  or via `pkg update` some time in December of 2018 if you had installed a
  prior release.  Of note: packages from prior to 20190626 are sufficiently
  old at time of writing to have been garbage collected from the main IPS
  repository.

* SmartOS platform images starting with
  [20181220T002304Z](https://us-east.manta.joyent.com/Joyent_Dev/public/SmartOS/smartos.html#20181220T002304Z).

* OmniOS CE [releases](https://omniosce.org/schedule) starting with r151030
  (LTS, released 2019-05-06).

In addition to the illumos base, we are including the following additional
libraries that appear in `/usr/lib` on all of the above platforms:

* `/usr/lib/{,amd64}/libssp.so.0.0.0` (version `LIBSSP_1.0`)
* `/usr/lib/{,amd64}/libgcc_s.so.1` (version `GCC_4.8.0`)

Note that these additional libraries are not usefully executable, but rather
are mere shim libraries that contain the same symbols and library versions as
we expect in the real thing.  This doesn't matter in practice, as the sysroot
is for cross compilation; the build machine must not execute program text for
the target machine.  These shim libraries are created through mapfiles and stub
code built from this repository.
