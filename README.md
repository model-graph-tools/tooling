# Tooling

`mgt` is a command line tool to analyze the WildFly management model and create images for the various parts of the model graph project.

## Version

Central to `mgt` is the WildFly version you want to analyze. The version must be specified as `<major>[.<minor>]` (version identifier) with `major` being mandatory >= 10 and `minor` being optional >= 0 and <= 9.

The version identifier is used in the container names and published ports.

All supported versions are listed [here](https://github.com/hpehl/wildfly-container-versions?tab=readme-ov-file#supported-versions).

## Commands

### WildFly

### Database

### Analyze

The analyze command does the bulk of the work. For every version specified it executes the following steps:

1. Start a WildFly standalone instance from [quay.io/wado/wado-sa](https://quay.io/repository/wado/wado-sa) using the `standalone-full-ha.xml` configuration.
2. Start an empty Neo4J database from [docker.io/neo4j:community](https://hub.docker.com/layers/library/neo4j/community/images/sha256-509cfbdd9512e2fe608aef4aed6e675ce6bbe1f93cfe1588fb265181a7774fa7).  
3. Run the [`analyzer`](https://github.com/model-graph-tools/analyzer) using the WildFly and Neo4J containers.
4. Build a self-contained Neo4J image in [quay.io/modelgraphtools/wildfly-management-model](https://quay.io/repository/modelgraphtools/wildfly-management-model). 
5. Shutdown instances and cleanup resources. 

```shell
mgt analyze 34
mgt analyze 10,26.1,34
mgt analyze 20..29
mgt analyze 10,11,20..29,34

```

### Push

```shell
mgt push 34
mgt push 10,26.1,34
mgt push 20..29
mgt push 10,11,20..29,34

```

---

```shell
mgt single 34 wildfly build
mgt single 34 wildfly start
mgt single 34 neo4j start
mgt single 34 analyze
mgt single 34 neo4j stop
mgt single 34 wildfly stop
mgt single 34 neo4j build

mgt batch all
mgt batch 34
mgt batch 10,26.1,34
mgt batch 20..29
mgt batch 10,11,20..29,34
```

## Big Picture

The analyzer is part of the [model graph tools](https://model-graph-tools.github.io/) and creates the graph database
used by the [model](https://github.com/model-graph-tools/model) service.

Take a look at the [setup](https://github.com/model-graph-tools/setup) repository how to get started.

<img src="https://model-graph-tools.github.io/img/tools.svg" alt="Model Graph Tools" width="512" />
