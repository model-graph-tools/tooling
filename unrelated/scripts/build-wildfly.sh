#!/usr/bin/env bash

# Builds the WildFly image to be analyzed
#
# Parameters
#   1. WildFly version >= 10


VERSION=$1
RELEASE=$VERSION.0.0.Final
REPO=quay.io/wildfly/wildfly
SUPPORTED_VERSIONS=10..35


# WildFly images
declare -A wildfly_images
wildfly_images["10"]="docker.io/jboss/wildfly:10.0.0.Final"
wildfly_images["11"]="docker.io/jboss/wildfly:11.0.0.Final"
wildfly_images["12"]="docker.io/jboss/wildfly:12.0.0.Final"
wildfly_images["13"]="docker.io/jboss/wildfly:13.0.0.Final"
wildfly_images["14"]="docker.io/jboss/wildfly:14.0.1.Final"
wildfly_images["15"]="docker.io/jboss/wildfly:15.0.1.Final"
wildfly_images["16"]="docker.io/jboss/wildfly:16.0.0.Final"
wildfly_images["17"]="docker.io/jboss/wildfly:17.0.1.Final"
wildfly_images["18"]="docker.io/jboss/wildfly:18.0.1.Final"
wildfly_images["19"]="docker.io/jboss/wildfly:19.0.0.Final"
wildfly_images["20"]="docker.io/jboss/wildfly:20.0.1.Final"
wildfly_images["21"]="docker.io/jboss/wildfly:21.0.2.Final"
wildfly_images["22"]="docker.io/jboss/wildfly:22.0.1.Final"
wildfly_images["23"]="docker.io/jboss/wildfly:23.0.2.Final"
wildfly_images["24"]="quay.io/wildfly/wildfly:24.0.0.Final"
wildfly_images["25"]="quay.io/wildfly/wildfly:25.0.1.Final"
wildfly_images["26"]="quay.io/wildfly/wildfly:26.0.1.Final"
wildfly_images["27"]="quay.io/wildfly/wildfly:27.0.1.Final-jdk19"
wildfly_images["28"]="quay.io/wildfly/wildfly:28.0.1.Final-jdk19"
wildfly_images["29"]="quay.io/wildfly/wildfly:29.0.1.Final-jdk20"
wildfly_images["30"]="quay.io/wildfly/wildfly:30.0.1.Final-jdk20"
wildfly_images["31"]="quay.io/wildfly/wildfly:31.0.1.Final-jdk20"
wildfly_images["32"]="quay.io/wildfly/wildfly:32.0.1.Final-jdk20"
wildfly_images["33"]="quay.io/wildfly/wildfly:33.0.2.Final-jdk21"
wildfly_images["34"]="quay.io/wildfly/wildfly:34.0.1.Final-jdk21"
wildfly_images["35"]="quay.io/wildfly/wildfly:35.0.0.Final-jdk21"

# Prerequisites
if [[ "$#" -ne 1 ]]; then
  echo "Illegal number of parameters. Please use $0 <wildfly-version>"
  exit 1
fi
if ! [[ $VERSION =~ ^[0-9]+$ ]] ; then
  echo "Illegal version. Must be numeric and >= 10."
  exit 1
fi
if [[ "$VERSION" -lt "10" ]]; then
  echo "Illegal version. Must be numeric and >= 10."
  exit 1
fi
# if [[ -n "${wildfly_images["$VERSION"]}" ]]; then
#   echo "Unsupported version. Supported versions: $SUPPORTED_VERSIONS"
#   exit 1
# fi


docker build \
  --build-arg WILDFLY_IMAGE=${wildfly_images["$VERSION"]} \
  --file src/main/docker/wildfly/Dockerfile \
  --tag quay.io/modelgraphtools/wildfly:$RELEASE \
  src/main/docker/wildfly
