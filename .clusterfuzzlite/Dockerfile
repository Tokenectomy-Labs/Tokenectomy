# ClusterFuzzLite configuration for Tokenectomy
FROM gcr.io/oss-fuzz-base/base-builder-rust:v1@sha256:28c118263e8455bfacf69723d01116817d0f81f44cae387d83bc4a8f6a7cd161
COPY . $SRC/tokenectomy
WORKDIR $SRC/tokenectomy
COPY .clusterfuzzlite/build.sh $SRC/build.sh
