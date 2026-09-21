# ClusterFuzzLite configuration for Tokenectomy
FROM gcr.io/oss-fuzz-base/base-builder-rust:v1@sha256:56810b48455ca1dae8e50b868f0cb1ce2c26ea7f2c6aa8c2e63c0a5d2eb34d40
COPY . $SRC/tokenectomy
WORKDIR $SRC/tokenectomy
COPY .clusterfuzzlite/build.sh $SRC/build.sh
